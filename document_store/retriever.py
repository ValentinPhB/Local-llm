"""Récupération lexicale bornée sur une liste de ressources déjà autorisées."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Sequence

from document_store.reader import read_policy_document


MAX_RESULTS = 3
MAX_EXCERPT_CHARS = 500
WORDS = re.compile(r"\w+", re.UNICODE)


@dataclass(frozen=True)
class RetrievalResult:
    resource_id: str
    classification: str
    excerpt: str


def _tokens(text: str) -> set[str]:
    return {word.lower() for word in WORDS.findall(text) if len(word) >= 2}


def _body(content: str) -> str:
    """Retire le front matter, qui est un contrat et non du texte à rechercher."""

    lines = content.splitlines()
    if lines and lines[0] == "---":
        for position, line in enumerate(lines[1:], start=1):
            if line == "---":
                return "\n".join(lines[position + 1 :]).strip()
    return content.strip()


def retrieve_documents(
    policy: Mapping[str, Any],
    authorized_resource_ids: Sequence[str],
    query: str,
    project_root: Path,
) -> tuple[RetrievalResult, ...]:
    """Classe seulement les documents dont l'ACL a déjà autorisé la lecture."""

    query_tokens = _tokens(query)
    if not query_tokens:
        return ()
    ranked: list[tuple[int, RetrievalResult]] = []
    for resource_id in authorized_resource_ids:
        document = read_policy_document(policy, resource_id, project_root)
        body = _body(document.content)
        score = len(query_tokens & _tokens(body))
        if score:
            excerpt = re.sub(r"\s+", " ", body)[:MAX_EXCERPT_CHARS]
            ranked.append((score, RetrievalResult(resource_id, document.classification, excerpt)))
    ranked.sort(key=lambda item: (-item[0], item[1].resource_id))
    return tuple(result for _, result in ranked[:MAX_RESULTS])
