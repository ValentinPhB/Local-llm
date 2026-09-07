from datetime import datetime, timezone
from io import StringIO
import json
import unittest

from audit.security_log import AuditEventError, SecurityAuditLog


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


if __name__ == "__main__":
    unittest.main()
