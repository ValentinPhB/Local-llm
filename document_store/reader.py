"""Lecteur borné par la politique, sans recherche ni appel au LLM."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping


MAX_DOCUMENT_BYTES = 32_768


class DocumentError(ValueError):
    """Le document ou son contrat de politique est invalide ou indisponible."""


@dataclass(frozen=True)
class ControlledDocument:
    """Document dont le chemin et les métadonnées ont été validés."""

    resource_id: str
    classification: str
    content: str


def _resource(policy: Mapping[str, Any], resource_id: str) -> Mapping[str, Any]:
    resources = policy.get("resources")
    if not isinstance(resources, list):
        raise DocumentError("ressources invalides")
    matches = [item for item in resources if isinstance(item, Mapping) and item.get("id") == resource_id]
    if len(matches) != 1:
        raise DocumentError("ressource introuvable")
    return matches[0]


def _front_matter(content: str) -> dict[str, str]:
    lines = content.splitlines()
    if not lines or lines[0] != "---":
        raise DocumentError("métadonnées absentes")
    metadata: dict[str, str] = {}
    for line in lines[1:]:
        if line == "---":
            return metadata
        key, separator, value = line.partition(":")
        if not separator or not key or not value.strip() or key in metadata:
            raise DocumentError("métadonnées invalides")
        metadata[key] = value.strip()
    raise DocumentError("métadonnées non fermées")


def read_policy_document(
    policy: Mapping[str, Any], resource_id: str, project_root: Path
) -> ControlledDocument:
    """Lit le seul fichier déclaré par une ressource, après autorisation externe.

    L'appelant doit prendre la décision ACL *avant* d'appeler cette fonction.
    Aucun chemin fourni par le navigateur ne peut arriver ici.
    """

    resource = _resource(policy, resource_id)
    relative_path = resource.get("path")
    classification = resource.get("classification")
    if not isinstance(relative_path, str) or not isinstance(classification, str):
        raise DocumentError("contrat de ressource invalide")

    documents_root = (project_root / "demo-documents").resolve()
    try:
        candidate = (project_root / relative_path).resolve(strict=True)
        candidate.relative_to(documents_root)
        if not candidate.is_file() or candidate.stat().st_size > MAX_DOCUMENT_BYTES:
            raise DocumentError("fichier non autorisé")
        content = candidate.read_text(encoding="utf-8")
    except (OSError, ValueError) as error:
        raise DocumentError("lecture refusée") from error

    metadata = _front_matter(content)
    if metadata.get("id") != resource_id or metadata.get("classification") != classification:
        raise DocumentError("métadonnées incohérentes")
    return ControlledDocument(resource_id, classification, content)
