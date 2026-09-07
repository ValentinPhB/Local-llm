"""Événements d'audit minimaux, sans contenu utilisateur ni secret.

Ce module ne connaît ni HTTP, ni document, ni Ollama. Il ne reçoit donc aucun
prompt, réponse, cookie ou jeton. L'API lui fournira seulement les métadonnées
utiles à la traçabilité lors de l'étape de raccordement suivante.
"""

from __future__ import annotations

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import stat
from threading import Lock
from typing import Callable, TextIO


AUDIT_EVENT = "access_decision"
ALLOWED_OUTCOMES = frozenset({"allowed", "denied", "error"})
DEFAULT_AUDIT_LOG_PATH = Path(".local") / "audit" / "access-decisions.jsonl"
DEFAULT_MAX_AUDIT_BYTES = 1_048_576


class AuditEventError(ValueError):
    """Un événement d'audit ne respecte pas le contrat minimal."""


class AuditStorageError(OSError):
    """La destination locale d'audit ne respecte pas les protections attendues."""


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


class RotatingAuditFile:
    """Flux local borné, privé et sans suivi Git pour les événements d'audit.

    Le fichier courant est plafonné ; lorsqu'il est plein, il devient l'unique
    sauvegarde ``.1`` et un nouveau fichier est créé. La classe n'accepte pas
    les liens symboliques ni les fichiers spéciaux afin de ne pas écrire dans
    une destination inattendue.
    """

    def __init__(self, path: Path, max_bytes: int = DEFAULT_MAX_AUDIT_BYTES) -> None:
        if max_bytes <= 0:
            raise ValueError("audit log size limit must be positive")
        self.path = path
        self.backup_path = path.with_name(f"{path.stem}.1{path.suffix}")
        self.max_bytes = max_bytes
        self._lock = Lock()

    @staticmethod
    def _validate_regular_path(path: Path) -> None:
        if not path.exists():
            return
        details = path.lstat()
        if stat.S_ISLNK(details.st_mode) or not stat.S_ISREG(details.st_mode):
            raise AuditStorageError("audit path must be a regular local file")

    def _prepare_directory(self) -> None:
        self.path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        os.chmod(self.path.parent, 0o700)

    def write(self, value: str) -> int:
        encoded = value.encode("utf-8")
        if len(encoded) > self.max_bytes:
            raise AuditStorageError("audit event exceeds log size limit")
        with self._lock:
            self._prepare_directory()
            self._validate_regular_path(self.path)
            if self.path.exists() and self.path.stat().st_size + len(encoded) > self.max_bytes:
                self._validate_regular_path(self.backup_path)
                os.replace(self.path, self.backup_path)
            try:
                with self.path.open("ab") as output:
                    os.chmod(self.path, 0o600)
                    output.write(encoded)
            except OSError as error:
                raise AuditStorageError("unable to write local audit log") from error
        return len(value)

    def flush(self) -> None:
        """Compatibilité avec le flux attendu par ``SecurityAuditLog``."""


def local_audit_log(
    project_root: Path,
    now: Callable[[], datetime] = _utc_now,
    max_bytes: int = DEFAULT_MAX_AUDIT_BYTES,
) -> SecurityAuditLog:
    """Construit le journal local borné à l'emplacement approuvé du projet."""

    return SecurityAuditLog(
        RotatingAuditFile(project_root / DEFAULT_AUDIT_LOG_PATH, max_bytes=max_bytes),
        now=now,
    )
