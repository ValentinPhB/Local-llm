"""Indexeur déterministe des seuls documents déclarés par la politique.

Cette étape ne connaît ni HTTP ni Qdrant réel : les dépendances d'embedding et
d'écriture sont injectées. Cela permet de vérifier le contrat de sécurité avant
de connecter une écriture persistante.
"""

from __future__ import annotations

from dataclasses import dataclass
import math
import re
from pathlib import Path
from typing import Any, Mapping, Protocol, Sequence

from access_control.engine import policy_is_valid
from document_store.reader import DocumentError, read_policy_document


MAX_CHUNK_CHARS = 700
MAX_OVERLAP_CHARS = 160
_PARAGRAPH_BREAK = re.compile(r"\n\s*\n+")
_SENTENCE_BREAK = re.compile(r"(?<=[.!?])\s+")


class IndexingError(ValueError):
    """Le contrat d'indexation ou un document contrôlé est invalide."""


class EmbeddingProvider(Protocol):
    """Dépendance injectée : un texte contrôlé devient un vecteur."""

    def embed(self, text: str) -> Sequence[float]: ...


@dataclass(frozen=True)
class IndexedPassage:
    """Un futur point Qdrant, sans détail de transport ni écriture réelle."""

    resource_id: str
    classification: str
    chunk_id: int
    text: str
    vector: tuple[float, ...]


class VectorIndexWriter(Protocol):
    """Dépendance injectée qui recevra le lot validé une seule fois."""

    def replace(self, passages: Sequence[IndexedPassage]) -> None: ...


def _body(content: str) -> str:
    """Écarte le front matter contrôlé : il ne doit pas influencer la recherche."""

    lines = content.splitlines()
    if not lines or lines[0] != "---":
        raise IndexingError("document metadata is invalid")
    for position, line in enumerate(lines[1:], start=1):
        if line == "---":
            body = "\n".join(lines[position + 1 :]).strip()
            if body:
                return body
            break
    raise IndexingError("document body is empty")


def _parts(text: str) -> tuple[str, ...]:
    """Produit des unités phrase/paragraphes ; force seulement les cas extrêmes."""

    paragraphs = (part.strip() for part in _PARAGRAPH_BREAK.split(text.strip()))
    parts: list[str] = []
    for paragraph in paragraphs:
        if not paragraph:
            continue
        sentences = (sentence.strip() for sentence in _SENTENCE_BREAK.split(paragraph))
        for sentence in sentences:
            if not sentence:
                continue
            while len(sentence) > MAX_CHUNK_CHARS:
                boundary = sentence.rfind(" ", 0, MAX_CHUNK_CHARS + 1)
                if boundary <= 0:
                    boundary = MAX_CHUNK_CHARS
                parts.append(sentence[:boundary].strip())
                sentence = sentence[boundary:].strip()
            if sentence:
                parts.append(sentence)
    return tuple(parts)


def chunk_document(content: str) -> tuple[str, ...]:
    """Découpe à des frontières logiques, avec une phrase de recouvrement bornée."""

    if not isinstance(content, str) or not content.strip():
        raise IndexingError("document content is invalid")
    chunks: list[str] = []
    current: list[str] = []
    for part in _parts(content):
        candidate = " ".join((*current, part))
        if current and len(candidate) > MAX_CHUNK_CHARS:
            completed = " ".join(current)
            chunks.append(completed)
            overlap = current[-1] if len(current[-1]) <= MAX_OVERLAP_CHARS else ""
            current = [overlap] if overlap else []
        current.append(part)
    if current:
        chunks.append(" ".join(current))
    if not chunks or any(not chunk or len(chunk) > MAX_CHUNK_CHARS for chunk in chunks):
        raise IndexingError("document chunks are invalid")
    return tuple(chunks)


def _vector(value: Sequence[float]) -> tuple[float, ...]:
    if not isinstance(value, Sequence) or isinstance(value, (str, bytes)) or not value:
        raise IndexingError("embedding vector is invalid")
    vector: list[float] = []
    for coordinate in value:
        if not isinstance(coordinate, (int, float)) or isinstance(coordinate, bool):
            raise IndexingError("embedding vector is invalid")
        number = float(coordinate)
        if not math.isfinite(number):
            raise IndexingError("embedding vector is invalid")
        vector.append(number)
    return tuple(vector)


class ControlledIndexer:
    """Prépare des passages déclarés par politique, puis remet un lot au writer.

    ``resource_ids`` est une liste de ressources de la politique, jamais une
    liste de chemins. L'ACL utilisateur reste appliquée lors de la recherche,
    pas au moment de constituer l'index administratif local.
    """

    def __init__(
        self,
        policy: Mapping[str, Any],
        project_root: Path,
        embedding_provider: EmbeddingProvider,
        writer: VectorIndexWriter,
    ) -> None:
        if not policy_is_valid(policy):
            raise IndexingError("access policy is invalid")
        if not isinstance(project_root, Path):
            raise IndexingError("project root is invalid")
        self.policy = policy
        self.project_root = project_root
        self.embedding_provider = embedding_provider
        self.writer = writer

    def index(self, resource_ids: Sequence[str]) -> tuple[IndexedPassage, ...]:
        """Valide tout le lot avant son unique remise au writer injecté."""

        if (
            not isinstance(resource_ids, Sequence)
            or isinstance(resource_ids, (str, bytes))
            or not resource_ids
            or not all(isinstance(resource_id, str) and resource_id for resource_id in resource_ids)
            or len(set(resource_ids)) != len(resource_ids)
        ):
            raise IndexingError("resource list is invalid")

        passages: list[IndexedPassage] = []
        vector_size: int | None = None
        for resource_id in resource_ids:
            try:
                document = read_policy_document(self.policy, resource_id, self.project_root)
            except DocumentError as error:
                raise IndexingError("policy document cannot be indexed") from error
            for chunk_id, text in enumerate(chunk_document(_body(document.content))):
                vector = _vector(self.embedding_provider.embed(text))
                if vector_size is None:
                    vector_size = len(vector)
                elif len(vector) != vector_size:
                    raise IndexingError("embedding dimensions are inconsistent")
                passages.append(
                    IndexedPassage(
                        resource_id=document.resource_id,
                        classification=document.classification,
                        chunk_id=chunk_id,
                        text=text,
                        vector=vector,
                    )
                )
        if not passages:
            raise IndexingError("no passages to index")
        self.writer.replace(tuple(passages))
        return tuple(passages)
