"""Simulation locale d'un fournisseur d'identité émettant des JWT HS256.

Ce module ne constitue pas un système d'authentification de production : la
sélection d'une identité est volontairement libre dans le laboratoire. Il
permet en revanche d'exercer le contrat utile à une intégration OIDC future :
un jeton signé, limité dans le temps, apporte un sujet et des groupes que
l'API vérifie avant de prendre une décision RBAC/ACL.
"""

from __future__ import annotations

import base64
import hashlib
import hmac
import json
import secrets
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping


class TokenError(ValueError):
    """Jeton ou annuaire de démonstration invalide."""


@dataclass(frozen=True)
class DemoIdentity:
    """Identité fictive proposée par l'annuaire de démonstration."""

    identity_id: str
    display_name: str
    groups: tuple[str, ...]


@dataclass(frozen=True)
class DemoDirectory:
    """Contrat minimal comparable aux informations d'un fournisseur OIDC."""

    issuer: str
    audience: str
    token_ttl_seconds: int
    identities: Mapping[str, DemoIdentity]


@dataclass(frozen=True)
class VerifiedIdentity:
    """Informations fiables extraites d'un jeton après vérification."""

    identity_id: str
    groups: tuple[str, ...]


def _b64encode(value: bytes) -> str:
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")


def _b64decode(value: str) -> bytes:
    if not isinstance(value, str) or not value:
        raise TokenError("segment JWT invalide")
    try:
        return base64.urlsafe_b64decode(value + "=" * (-len(value) % 4))
    except (ValueError, UnicodeEncodeError) as error:
        raise TokenError("encodage JWT invalide") from error


def _json_segment(value: str) -> Mapping[str, Any]:
    try:
        decoded = json.loads(_b64decode(value))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise TokenError("JSON JWT invalide") from error
    if not isinstance(decoded, Mapping):
        raise TokenError("objet JWT invalide")
    return decoded


def _string_list(value: Any, field: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value or not all(
        isinstance(item, str) and item for item in value
    ):
        raise TokenError(f"{field} invalide")
    if len(set(value)) != len(value):
        raise TokenError(f"{field} dupliqué")
    return tuple(value)


def load_demo_directory(path: Path) -> DemoDirectory:
    """Charge un annuaire fictif strict ; toute ambiguïté arrête le démarrage."""

    try:
        content = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise TokenError("annuaire de démonstration illisible") from error

    if not isinstance(content, Mapping):
        raise TokenError("annuaire de démonstration invalide")
    issuer = content.get("issuer")
    audience = content.get("audience")
    ttl = content.get("token_ttl_seconds")
    entries = content.get("identities")
    if not isinstance(issuer, str) or not issuer:
        raise TokenError("issuer invalide")
    if not isinstance(audience, str) or not audience:
        raise TokenError("audience invalide")
    if not isinstance(ttl, int) or isinstance(ttl, bool) or not 60 <= ttl <= 3600:
        raise TokenError("durée de jeton invalide")
    if not isinstance(entries, list) or not entries:
        raise TokenError("identités invalides")

    identities: dict[str, DemoIdentity] = {}
    for entry in entries:
        if not isinstance(entry, Mapping):
            raise TokenError("identité invalide")
        identity_id = entry.get("id")
        display_name = entry.get("display_name")
        if not isinstance(identity_id, str) or not identity_id:
            raise TokenError("identifiant invalide")
        if identity_id in identities:
            raise TokenError("identifiant dupliqué")
        if not isinstance(display_name, str) or not display_name:
            raise TokenError("nom d'affichage invalide")
        identities[identity_id] = DemoIdentity(
            identity_id=identity_id,
            display_name=display_name,
            groups=_string_list(entry.get("groups"), "groups"),
        )

    return DemoDirectory(issuer, audience, ttl, identities)


def new_signing_key() -> bytes:
    """Crée une clé éphémère : elle n'est jamais écrite sur disque."""

    return secrets.token_bytes(32)


def issue_demo_token(
    directory: DemoDirectory, identity_id: str, signing_key: bytes, now: int | None = None
) -> str:
    """Émet un JWT court pour une identité présente dans l'annuaire fictif."""

    identity = directory.identities.get(identity_id)
    if identity is None:
        raise TokenError("identité inconnue")
    issued_at = int(time.time()) if now is None else now
    if not isinstance(issued_at, int):
        raise TokenError("horodatage invalide")
    header = _b64encode(json.dumps({"alg": "HS256", "typ": "JWT"}, separators=(",", ":")).encode())
    payload = _b64encode(
        json.dumps(
            {
                "iss": directory.issuer,
                "aud": directory.audience,
                "sub": identity.identity_id,
                "groups": list(identity.groups),
                "iat": issued_at,
                "exp": issued_at + directory.token_ttl_seconds,
            },
            separators=(",", ":"),
        ).encode()
    )
    signed = f"{header}.{payload}".encode("ascii")
    signature = hmac.new(signing_key, signed, hashlib.sha256).digest()
    return f"{header}.{payload}.{_b64encode(signature)}"


def verify_demo_token(
    token: str,
    directory: DemoDirectory,
    signing_key: bytes,
    now: int | None = None,
) -> VerifiedIdentity:
    """Vérifie signature, contrat et expiration avant toute autorisation."""

    parts = token.split(".") if isinstance(token, str) else []
    if len(parts) != 3:
        raise TokenError("structure JWT invalide")
    header, payload, signature = parts
    decoded_header = _json_segment(header)
    if decoded_header != {"alg": "HS256", "typ": "JWT"}:
        raise TokenError("en-tête JWT refusé")
    expected_signature = hmac.new(
        signing_key, f"{header}.{payload}".encode("ascii"), hashlib.sha256
    ).digest()
    try:
        presented_signature = _b64decode(signature)
    except TokenError:
        raise
    if not hmac.compare_digest(expected_signature, presented_signature):
        raise TokenError("signature JWT invalide")

    claims = _json_segment(payload)
    if claims.get("iss") != directory.issuer or claims.get("aud") != directory.audience:
        raise TokenError("émetteur ou audience refusé")
    identity_id = claims.get("sub")
    if not isinstance(identity_id, str) or identity_id not in directory.identities:
        raise TokenError("sujet JWT refusé")
    groups = _string_list(claims.get("groups"), "groups")
    if groups != directory.identities[identity_id].groups:
        raise TokenError("groupes JWT refusés")
    expiration = claims.get("exp")
    issued_at = claims.get("iat")
    if (
        not isinstance(expiration, int)
        or isinstance(expiration, bool)
        or not isinstance(issued_at, int)
        or isinstance(issued_at, bool)
        or expiration - issued_at != directory.token_ttl_seconds
    ):
        raise TokenError("dates JWT refusées")
    current_time = int(time.time()) if now is None else now
    if not isinstance(current_time, int) or expiration <= current_time:
        raise TokenError("jeton expiré")
    return VerifiedIdentity(identity_id=identity_id, groups=groups)
