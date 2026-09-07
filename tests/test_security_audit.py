from datetime import datetime, timezone
from io import StringIO
import json
from pathlib import Path
import stat
import tempfile
import unittest

from audit.security_log import (
    AuditEventError,
    AuditStorageError,
    RotatingAuditFile,
    SecurityAuditLog,
    local_audit_log,
)


class SecurityAuditLogTests(unittest.TestCase):
    def test_records_only_the_minimal_access_metadata_as_json_line(self):
        output = StringIO()
        log = SecurityAuditLog(
            output,
            now=lambda: datetime(2026, 9, 7, 12, 30, tzinfo=timezone.utc),
        )

        log.record_access_decision(
            route="/api/documents/public-welcome",
            outcome="allowed",
            identity_id="oscar",
            resource_id="public-welcome",
        )

        event = json.loads(output.getvalue())
        self.assertEqual(
            event,
            {
                "timestamp": "2026-09-07T12:30:00Z",
                "event": "access_decision",
                "route": "/api/documents/public-welcome",
                "outcome": "allowed",
                "identity_id": "oscar",
                "resource_id": "public-welcome",
            },
        )
        self.assertNotIn("message", event)
        self.assertNotIn("content", event)
        self.assertNotIn("token", event)

    def test_supports_anonymous_denial_without_identity_or_resource(self):
        output = StringIO()
        log = SecurityAuditLog(
            output,
            now=lambda: datetime(2026, 9, 7, tzinfo=timezone.utc),
        )

        log.record_access_decision(route="/api/session", outcome="denied")

        event = json.loads(output.getvalue())
        self.assertIsNone(event["identity_id"])
        self.assertIsNone(event["resource_id"])
        self.assertEqual(event["outcome"], "denied")

    def test_rejects_invalid_outcome_and_route(self):
        log = SecurityAuditLog(StringIO())

        with self.assertRaises(AuditEventError):
            log.record_access_decision(route="api/session", outcome="allowed")
        with self.assertRaises(AuditEventError):
            log.record_access_decision(route="/api/session", outcome="ignored")

    def test_local_audit_file_is_private_and_outside_git_tracking(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = local_audit_log(root, now=lambda: datetime(2026, 9, 7, tzinfo=timezone.utc))

            log.record_access_decision(route="/api/session", outcome="denied")

            path = root / ".local" / "audit" / "access-decisions.jsonl"
            self.assertTrue(path.is_file())
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
            event = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(event["outcome"], "denied")

    def test_local_audit_file_rotates_to_one_backup_when_size_limit_is_reached(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "access-decisions.jsonl"
            output = RotatingAuditFile(path, max_bytes=200)
            log = SecurityAuditLog(
                output,
                now=lambda: datetime(2026, 9, 7, tzinfo=timezone.utc),
            )

            log.record_access_decision(route="/api/one", outcome="allowed")
            first_event = path.read_text(encoding="utf-8")
            log.record_access_decision(route="/api/two", outcome="denied")

            self.assertEqual(output.backup_path.read_text(encoding="utf-8"), first_event)
            current_event = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(current_event["route"], "/api/two")

    def test_local_audit_file_refuses_a_symbolic_link(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target"
            target.write_text("do not overwrite", encoding="utf-8")
            path = root / "access-decisions.jsonl"
            path.symlink_to(target)

            with self.assertRaises(AuditStorageError):
                RotatingAuditFile(path).write("event\n")


if __name__ == "__main__":
    unittest.main()
