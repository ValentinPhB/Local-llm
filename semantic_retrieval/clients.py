"""Clients locaux, isolés et testables pour embeddings et recherche vectorielle.

Ces adaptateurs ne sont pas encore reliés à une route HTTP ni à l'indexeur. Ils
ne lisent aucun document et ne prennent aucune décision ACL : l'appelant doit
leur fournir uniquement les resource_id déjà autorisés.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import math
import re
from typing import Any, Sequence
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlsplit
from urllib.request import Request, urlopen


OLLAMA_EMBED_URL = "http://127.0.0.1:11434/api/embed"
QDRANT_URL = "http://127.0.0.1:6333"
QDRANT_COLLECTION = "lab_semantic_documents"
MAX_RESULTS = 3
MAX_EXCERPT_CHARS = 500
RESOURCE_ID = re.compile(r"[a-z0-9-]{1,80}")


class SemanticClientError(ValueError):
    """La réponse locale ou le contrat de recherche sémantique est invalide."""


class SemanticServiceUnavailable(OSError):
    """Ollama embeddings ou Qdrant local est indisponible."""


def _require_loopback_url(url: str, expected_port: int) -> str:
    parsed = urlsplit(url)
    if (
        parsed.scheme != "http"
        or parsed.hostname != "127.0.0.1"
        or parsed.port != expected_port
        or parsed.username
        or parsed.password
        or parsed.query
        or parsed.fragment
    ):
        raise SemanticClientError("semantic service must use its fixed loopback URL")
    return url.rstrip("/")


def _vector(value: Any) -> tuple[float, ...]:
    if not isinstance(value, list) or not value:
        raise SemanticClientError("embedding vector is invalid")
    vector: list[float] = []
    for coordinate in value:
        if not isinstance(coordinate, (int, float)) or isinstance(coordinate, bool):
            raise SemanticClientError("embedding vector is invalid")
        coordinate = float(coordinate)
        if not math.isfinite(coordinate):
            raise SemanticClientError("embedding vector is invalid")
        vector.append(coordinate)
    return tuple(vector)


class OllamaEmbeddingProvider:
    """Appelle le seul endpoint local d'embeddings, avec modèle imposé."""

    def __init__(self, model: str = "embeddinggemma", endpoint: str = OLLAMA_EMBED_URL) -> None:
        if not isinstance(model, str) or not model:
            raise SemanticClientError("embedding model is invalid")
        self.model = model
        self.endpoint = _require_loopback_url(endpoint, expected_port=11434)

    def embed(self, text: str) -> tuple[float, ...]:
        if not isinstance(text, str) or not (text := text.strip()) or len(text) > 8_000:
            raise SemanticClientError("embedding input is invalid")
        body = json.dumps({"model": self.model, "input": text}).encode("utf-8")
        request = Request(
            self.endpoint,
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urlopen(request, timeout=30) as response:  # noqa: S310 -- fixed loopback URL
                payload = json.loads(response.read())
        except (HTTPError, URLError, TimeoutError, json.JSONDecodeError) as error:
            raise SemanticServiceUnavailable("local embedding service is unavailable") from error
        try:
            embeddings = payload["embeddings"]
            if not isinstance(embeddings, list) or len(embeddings) != 1:
                raise SemanticClientError("embedding response is invalid")
            return _vector(embeddings[0])
        except (KeyError, TypeError) as error:
            raise SemanticClientError("embedding response is invalid") from error


@dataclass(frozen=True)
class SemanticMatch:
    """Passage Qdrant déjà filtré par l'ACL, prêt pour la couche supérieure."""

    resource_id: str
    classification: str
    excerpt: str
    score: float


class QdrantVectorStore:
    """Recherche Qdrant avec un filtre ACL obligatoire, sans écriture d'index."""

    def __init__(
        self,
        base_url: str = QDRANT_URL,
        collection: str = QDRANT_COLLECTION,
    ) -> None:
        if not isinstance(collection, str) or not re.fullmatch(r"[a-z0-9_-]{1,80}", collection):
            raise SemanticClientError("Qdrant collection is invalid")
        self.base_url = _require_loopback_url(base_url, expected_port=6333)
        self.collection = collection

    def search(
        self,
        query_vector: Sequence[float],
        allowed_resource_ids: Sequence[str],
        limit: int = MAX_RESULTS,
    ) -> tuple[SemanticMatch, ...]:
        vector = _vector(list(query_vector))
        if not isinstance(limit, int) or not 1 <= limit <= MAX_RESULTS:
            raise SemanticClientError("semantic result limit is invalid")
        allowed = tuple(dict.fromkeys(allowed_resource_ids))
        if not allowed:
            # Une liste ACL vide ne doit jamais devenir une recherche non filtrée.
            return ()
        if not all(isinstance(item, str) and RESOURCE_ID.fullmatch(item) for item in allowed):
            raise SemanticClientError("authorized resource list is invalid")

        body = {
            "query": vector,
            "filter": {"must": [{"key": "resource_id", "match": {"any": allowed}}]},
            "limit": limit,
            "with_payload": True,
            "with_vector": False,
        }
        request = Request(
            f"{self.base_url}/collections/{quote(self.collection, safe='')}/points/query",
            data=json.dumps(body).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urlopen(request, timeout=5) as response:  # noqa: S310 -- fixed loopback URL
                response_body = json.loads(response.read())
        except (HTTPError, URLError, TimeoutError, json.JSONDecodeError) as error:
            raise SemanticServiceUnavailable("local vector store is unavailable") from error
        try:
            points = response_body["result"]["points"]
            if not isinstance(points, list) or len(points) > limit:
                raise SemanticClientError("Qdrant response is invalid")
            matches = tuple(self._match(point, allowed) for point in points)
        except (KeyError, TypeError) as error:
            raise SemanticClientError("Qdrant response is invalid") from error
        return matches

    @staticmethod
    def _match(point: Any, allowed: Sequence[str]) -> SemanticMatch:
        if not isinstance(point, dict):
            raise SemanticClientError("Qdrant response is invalid")
        payload = point.get("payload")
        resource_id = payload.get("resource_id") if isinstance(payload, dict) else None
        classification = payload.get("classification") if isinstance(payload, dict) else None
        text = payload.get("text") if isinstance(payload, dict) else None
        score = point.get("score")
        if (
            not isinstance(resource_id, str)
            or resource_id not in allowed
            or not isinstance(classification, str)
            or not isinstance(text, str)
            or not isinstance(score, (int, float))
            or isinstance(score, bool)
            or not math.isfinite(float(score))
        ):
            raise SemanticClientError("Qdrant response is invalid")
        return SemanticMatch(resource_id, classification, text[:MAX_EXCERPT_CHARS], float(score))
