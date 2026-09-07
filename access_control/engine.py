"""Moteur RBAC/ACL pur et fail-closed.

Ce module ne lit ni HTTP, ni session utilisateur, ni document. Il applique une
politique déjà chargée et retourne une décision déterministe.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping


@dataclass(frozen=True)
class AccessDecision:
    """Résultat exploitable par l'API, les tests et un futur audit."""

    allowed: bool
    reason: str
    matched_roles: tuple[str, ...] = ()


def _index_by_id(items: Any) -> dict[str, Mapping[str, Any]]:
    """Construit un index strict ; une politique ambiguë est invalide."""

    if not isinstance(items, list):
        raise ValueError("collection absente ou invalide")

    index: dict[str, Mapping[str, Any]] = {}
    for item in items:
        if not isinstance(item, Mapping) or not isinstance(item.get("id"), str):
            raise ValueError("élément de politique invalide")
        if item["id"] in index:
            raise ValueError("identifiant dupliqué")
        index[item["id"]] = item
    return index


def _roles(item: Mapping[str, Any], field: str) -> set[str]:
    values = item.get(field)
    if not isinstance(values, list) or not all(isinstance(role, str) for role in values):
        raise ValueError("rôles invalides")
    return set(values)


def decide_access(
    policy: Mapping[str, Any], identity_id: str, resource_id: str
) -> AccessDecision:
    """Autorise seulement une correspondance de rôle explicite.

    Toute erreur de format, identité inconnue, ressource inconnue ou absence de
    rôle commun produit un refus. Cette fonction ne dépend jamais du LLM.
    """

    if not isinstance(policy, Mapping) or policy.get("default_decision") != "deny":
        return AccessDecision(False, "invalid_policy")
    if not isinstance(identity_id, str) or not identity_id:
        return AccessDecision(False, "unknown_identity")
    if not isinstance(resource_id, str) or not resource_id:
        return AccessDecision(False, "unknown_resource")

    try:
        identities = _index_by_id(policy.get("identities"))
        resources = _index_by_id(policy.get("resources"))
        identity = identities.get(identity_id)
        if identity is None:
            return AccessDecision(False, "unknown_identity")
        resource = resources.get(resource_id)
        if resource is None:
            return AccessDecision(False, "unknown_resource")

        matched_roles = tuple(
            sorted(_roles(identity, "roles") & _roles(resource, "allowed_roles"))
        )
    except ValueError:
        return AccessDecision(False, "invalid_policy")

    if not matched_roles:
        return AccessDecision(False, "insufficient_role")
    return AccessDecision(True, "role_match", matched_roles)
