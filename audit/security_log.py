"""Événements d'audit minimaux, sans contenu utilisateur ni secret.

Ce module ne connaît ni HTTP, ni document, ni Ollama. Il ne reçoit donc aucun
prompt, réponse, cookie ou jeton. L'API lui fournira seulement les métadonnées
utiles à la traçabilité lors de l'étape de raccordement suivante.
"""

from __future__ import annotations

from datetime import datetime, timezone
import json
from typing import Callable, TextIO


AUDIT_EVENT = "access_decision"
ALLOWED_OUTCOMES = frozenset({"allowed", "denied", "error"})


class AuditEventError(ValueError):
    """Un événement d'audit ne respecte pas le contrat minimal."""


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


class SecurityAuditLog:
    """Écrit une ligne JSON par décision d'accès dans un flux fourni.

    Le flux (fichier local ou autre destination approuvée) est volontairement
    injecté par l'appelant. Cela garde ce composant testable et évite de créer
    une persistance implicite avant que sa destination et sa rétention soient
    définies.
    """

    def __init__(self, output: TextIO, now: Callable[[], datetime] = _utc_now) -> None:
        self._output = output
        self._now = now

    def record_access_decision(
        self,
        *,
        route: str,
        outcome: str,
        identity_id: str | None = None,
        resource_id: str | None = None,
    ) -> None:
        if outcome not in ALLOWED_OUTCOMES:
            raise AuditEventError("invalid audit outcome")
        if not route.startswith("/"):
            raise AuditEventError("invalid audit route")
        for field_name, value in (("identity_id", identity_id), ("resource_id", resource_id)):
            if value is not None and (not value or len(value) > 80):
                raise AuditEventError(f"invalid audit {field_name}")

        timestamp = self._now()
        if timestamp.tzinfo is None:
            raise AuditEventError("audit timestamp must include a timezone")
        event = {
            "timestamp": timestamp.astimezone(timezone.utc).isoformat().replace("+00:00", "Z"),
            "event": AUDIT_EVENT,
            "route": route,
            "outcome": outcome,
            "identity_id": identity_id,
            "resource_id": resource_id,
        }
        self._output.write(json.dumps(event, separators=(",", ":"), sort_keys=True) + "\n")
        self._output.flush()
