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


def _resource(policy: Mapping[str, Any], resource_id: str) -> Mapping[str, Any] | None:
    resources = _index_by_id(policy.get("resources"))
    return resources.get(resource_id)


def roles_from_groups(policy: Mapping[str, Any], groups: Any) -> tuple[str, ...] | None:
    """Traduit des groupes d'identité fiables en rôles applicatifs.

    ``None`` signifie que la politique ou les groupes sont invalides : l'appelant
    doit alors refuser l'accès. Un groupe sans correspondance est inoffensif et
    ne fournit aucun rôle.
    """

    if not isinstance(policy, Mapping) or policy.get("default_decision") != "deny":
        return None
    if not isinstance(groups, (tuple, list)) or not all(
        isinstance(group, str) and group for group in groups
    ):
        return None
    mappings = policy.get("group_role_mappings")
    if not isinstance(mappings, list):
        return None
    translated: dict[str, str] = {}
    try:
        for mapping in mappings:
            if not isinstance(mapping, Mapping):
                raise ValueError
            group = mapping.get("group")
            role = mapping.get("role")
            if not isinstance(group, str) or not group or not isinstance(role, str) or not role:
                raise ValueError
            if group in translated:
                raise ValueError
            translated[group] = role
    except ValueError:
        return None
    return tuple(sorted({translated[group] for group in groups if group in translated}))


def policy_is_valid(policy: Any) -> bool:
    """Valide le contrat complet au démarrage, sans jamais autoriser d'accès."""

    if not isinstance(policy, Mapping) or policy.get("default_decision") != "deny":
        return False
    if roles_from_groups(policy, ()) is None:
        return False
    try:
        identities = _index_by_id(policy.get("identities"))
        resources = _index_by_id(policy.get("resources"))
        for identity in identities.values():
            _roles(identity, "roles")
        for resource in resources.values():
            _roles(resource, "allowed_roles")
    except ValueError:
        return False
    return True


def decide_access_for_roles(
    policy: Mapping[str, Any], roles: Any, resource_id: str
) -> AccessDecision:
    """Applique une ACL à des rôles déjà issus d'une identité vérifiée."""

    if not policy_is_valid(policy):
        return AccessDecision(False, "invalid_policy")
    if not isinstance(resource_id, str) or not resource_id:
        return AccessDecision(False, "unknown_resource")
    if not isinstance(roles, (tuple, list)) or not all(isinstance(role, str) for role in roles):
        return AccessDecision(False, "invalid_policy")
    try:
        resource = _resource(policy, resource_id)
        if resource is None:
            return AccessDecision(False, "unknown_resource")
        matched_roles = tuple(sorted(set(roles) & _roles(resource, "allowed_roles")))
    except ValueError:
        return AccessDecision(False, "invalid_policy")
    if not matched_roles:
        return AccessDecision(False, "insufficient_role")
    return AccessDecision(True, "role_match", matched_roles)


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
        identity = identities.get(identity_id)
        if identity is None:
            return AccessDecision(False, "unknown_identity")
        roles = _roles(identity, "roles")
    except ValueError:
        return AccessDecision(False, "invalid_policy")
    return decide_access_for_roles(policy, tuple(roles), resource_id)
