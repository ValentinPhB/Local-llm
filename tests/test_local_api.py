import http.client
from io import StringIO
import json
import threading
import unittest
from unittest.mock import patch

from audit.security_log import AuditStorageError, SecurityAuditLog
from ui.server import (
    ACCESS_CHECK_AUDIT_ROUTE,
    DOCUMENT_AUDIT_ROUTE,
    RETRIEVAL_AUDIT_ROUTE,
    SESSION_COOKIE_NAME,
    create_server,
)


class FakeOllamaResponse:
    def __enter__(self):
        return self

    def __exit__(self, *args):
        return False

    def read(self):
        return b'{"message":{"content":"RAG-OK"}}'


class ThinkingOllamaResponse(FakeOllamaResponse):
    def read(self):
        return b'{"message":{"content":"raisonnement interne</think>REPONSE-SURE"}}'


class LocalAPITests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.audit_output = StringIO()
        cls.audit_log = SecurityAuditLog(cls.audit_output)
        cls.server = create_server(port=0, audit_log=cls.audit_log)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.host, cls.port = cls.server.server_address

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()

    def request(self, method, path, body=None, headers=None):
        connection = http.client.HTTPConnection(self.host, self.port, timeout=2)
        connection.request(method, path, body=body, headers=headers or {})
        response = connection.getresponse()
        payload = json.loads(response.read())
        headers = dict(response.getheaders())
        connection.close()
        return response.status, payload, headers

    def start_demo_session(self, identity_id):
        body = json.dumps({"identity_id": identity_id})
        status, payload, headers = self.request(
            "POST", "/api/demo-session", body, {"Content-Type": "application/json"}
        )
        self.assertEqual(status, 200)
        self.assertTrue(payload["authenticated"])
        cookie = headers["Set-Cookie"].split(";", 1)[0]
        self.assertTrue(cookie.startswith(f"{SESSION_COOKIE_NAME}="))
        return {"Cookie": cookie}

    def test_chat_without_session_is_rejected_before_ollama(self):
        status, payload, _ = self.request(
            "POST",
            "/api/chat",
            json.dumps({"message": "Bonjour", "identity_id": "alice"}),
            {"Content-Type": "application/json"},
        )
        self.assertEqual(status, 401)
        self.assertIn("Session", payload["error"])

    def test_chat_removes_thinking_trace_from_fake_ollama(self):
        headers = self.start_demo_session("alice")
        with patch("ui.server.urlopen", return_value=ThinkingOllamaResponse()):
            status, payload, _ = self.request(
                "POST", "/api/chat", json.dumps({"message": "Bonjour"}),
                {"Content-Type": "application/json", **headers},
            )
        self.assertEqual(status, 200)
        self.assertEqual(payload["content"], "REPONSE-SURE")

    def test_session_is_signed_and_used_for_access_decision(self):
        headers = self.start_demo_session("alice")
        status, payload, _ = self.request("GET", "/api/session", headers=headers)
        self.assertEqual(status, 200)
        self.assertEqual(payload["identity"]["id"], "alice")

        status, payload, _ = self.request("GET", "/api/access-check?resource_id=rh-onboarding", headers=headers)
        self.assertEqual(status, 200)
        self.assertTrue(payload["allowed"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["route"], ACCESS_CHECK_AUDIT_ROUTE)
        self.assertEqual(event["outcome"], "allowed")
        self.assertEqual(event["identity_id"], "alice")
        self.assertEqual(event["resource_id"], "rh-onboarding")

        status, payload, _ = self.request("GET", "/api/access-check?resource_id=it-workstation", headers=headers)
        self.assertEqual(status, 403)
        self.assertFalse(payload["allowed"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["route"], ACCESS_CHECK_AUDIT_ROUTE)
        self.assertEqual(event["outcome"], "denied")
        self.assertEqual(event["resource_id"], "it-workstation")

    def test_access_check_without_session_is_audited_as_anonymous_denial(self):
        status, payload, _ = self.request("GET", "/api/access-check?resource_id=public-welcome")

        self.assertEqual(status, 401)
        self.assertIn("Session", payload["error"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["route"], ACCESS_CHECK_AUDIT_ROUTE)
        self.assertEqual(event["outcome"], "denied")
        self.assertIsNone(event["identity_id"])
        self.assertIsNone(event["resource_id"])

    def test_access_check_is_not_disclosed_when_audit_storage_is_unavailable(self):
        headers = self.start_demo_session("alice")
        with patch.object(
            self.audit_log,
            "record_access_decision",
            side_effect=AuditStorageError("unavailable"),
        ):
            status, payload, _ = self.request(
                "GET", "/api/access-check?resource_id=rh-onboarding", headers=headers
            )

        self.assertEqual(status, 503)
        self.assertIn("Journal", payload["error"])

    def test_modified_cookie_is_rejected(self):
        headers = self.start_demo_session("bob")
        cookie = headers["Cookie"]
        altered = f"{cookie[:-1]}{'A' if cookie[-1] != 'A' else 'B'}"
        status, payload, _ = self.request("GET", "/api/session", headers={"Cookie": altered})
        self.assertEqual(status, 401)
        self.assertIn("Session", payload["error"])

    def test_oscar_can_only_access_public_welcome(self):
        headers = self.start_demo_session("oscar")
        status, payload, _ = self.request(
            "GET", "/api/access-check?resource_id=public-welcome", headers=headers
        )
        self.assertEqual(status, 200)
        self.assertTrue(payload["allowed"])
        status, payload, _ = self.request(
            "GET", "/api/access-check?resource_id=public-glossary", headers=headers
        )
        self.assertEqual(status, 403)
        self.assertFalse(payload["allowed"])

    def test_authorized_document_is_returned_only_after_access_check(self):
        headers = self.start_demo_session("oscar")
        status, payload, _ = self.request(
            "GET", "/api/documents/public-welcome", headers=headers
        )
        self.assertEqual(status, 200)
        self.assertEqual(payload["resource_id"], "public-welcome")
        self.assertEqual(payload["classification"], "PUBLIC")
        self.assertIn("Acme-Lab", payload["content"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["route"], DOCUMENT_AUDIT_ROUTE)
        self.assertEqual(event["outcome"], "allowed")
        self.assertEqual(event["identity_id"], "oscar")
        self.assertEqual(event["resource_id"], "public-welcome")

    def test_denied_document_is_not_read(self):
        headers = self.start_demo_session("oscar")
        with patch("ui.server.read_policy_document") as reader:
            status, payload, _ = self.request(
                "GET", "/api/documents/public-glossary", headers=headers
            )
        self.assertEqual(status, 403)
        self.assertIn("refusé", payload["error"])
        reader.assert_not_called()
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["outcome"], "denied")
        self.assertEqual(event["identity_id"], "oscar")
        self.assertEqual(event["resource_id"], "public-glossary")

    def test_document_without_session_is_audited_as_anonymous_denial(self):
        status, payload, _ = self.request("GET", "/api/documents/public-welcome")

        self.assertEqual(status, 401)
        self.assertIn("Session", payload["error"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["outcome"], "denied")
        self.assertIsNone(event["identity_id"])
        self.assertIsNone(event["resource_id"])

    def test_document_is_not_read_when_audit_storage_is_unavailable(self):
        headers = self.start_demo_session("oscar")
        with patch.object(
            self.audit_log,
            "record_access_decision",
            side_effect=AuditStorageError("unavailable"),
        ), patch("ui.server.read_policy_document") as reader:
            status, payload, _ = self.request(
                "GET", "/api/documents/public-welcome", headers=headers
            )

        self.assertEqual(status, 503)
        self.assertIn("Journal", payload["error"])
        reader.assert_not_called()

    def test_document_path_like_identifier_is_rejected(self):
        headers = self.start_demo_session("alice")
        status, payload, _ = self.request(
            "GET", "/api/documents/%2E%2E%2FAGENTS.md", headers=headers
        )
        self.assertEqual(status, 400)
        self.assertIn("invalide", payload["error"])

    def test_retrieval_reads_only_oscar_authorized_resource(self):
        headers = self.start_demo_session("oscar")
        status, payload, _ = self.request(
            "POST", "/api/retrieve", json.dumps({"query": "organisation Acme-Lab"}),
            {"Content-Type": "application/json", **headers},
        )
        self.assertEqual(status, 200)
        self.assertEqual([result["resource_id"] for result in payload["results"]], ["public-welcome"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["route"], RETRIEVAL_AUDIT_ROUTE)
        self.assertEqual(event["outcome"], "allowed")
        self.assertEqual(event["identity_id"], "oscar")
        self.assertIsNone(event["resource_id"])
        self.assertNotIn("organisation", json.dumps(event, ensure_ascii=False))
        self.assertNotIn("Acme-Lab", json.dumps(event, ensure_ascii=False))

    def test_retrieval_returns_no_rh_result_for_bob(self):
        headers = self.start_demo_session("bob")
        status, payload, _ = self.request(
            "POST", "/api/retrieve", json.dumps({"query": "intégration checklist"}),
            {"Content-Type": "application/json", **headers},
        )
        self.assertEqual(status, 200)
        self.assertEqual(payload["results"], [])

    def test_retrieval_without_session_is_audited_as_anonymous_denial(self):
        status, payload, _ = self.request(
            "POST",
            "/api/retrieve",
            json.dumps({"query": "donnée privée"}),
            {"Content-Type": "application/json"},
        )

        self.assertEqual(status, 401)
        self.assertIn("Session", payload["error"])
        event = json.loads(self.audit_output.getvalue().splitlines()[-1])
        self.assertEqual(event["route"], RETRIEVAL_AUDIT_ROUTE)
        self.assertEqual(event["outcome"], "denied")
        self.assertIsNone(event["identity_id"])
        self.assertIsNone(event["resource_id"])
        self.assertNotIn("donnée privée", json.dumps(event, ensure_ascii=False))

    def test_retrieval_does_not_read_documents_when_audit_storage_is_unavailable(self):
        headers = self.start_demo_session("oscar")
        with patch.object(
            self.audit_log,
            "record_access_decision",
            side_effect=AuditStorageError("unavailable"),
        ), patch("ui.server.retrieve_documents") as retriever:
            status, payload, _ = self.request(
                "POST",
                "/api/retrieve",
                json.dumps({"query": "organisation Acme-Lab"}),
                {"Content-Type": "application/json", **headers},
            )

        self.assertEqual(status, 503)
        self.assertIn("Journal", payload["error"])
        retriever.assert_not_called()

    def test_rag_chat_sends_only_oscar_authorized_source_to_ollama(self):
        headers = self.start_demo_session("oscar")
        captured = {}

        def fake_urlopen(request, timeout):
            captured["payload"] = json.loads(request.data)
            return FakeOllamaResponse()

        with patch("ui.server.urlopen", side_effect=fake_urlopen):
            status, payload, _ = self.request(
                "POST",
                "/api/rag-chat",
                json.dumps({"message": "organisation Acme-Lab", "context": "RH interdit"}),
                {"Content-Type": "application/json", **headers},
            )
        self.assertEqual(status, 200)
        self.assertEqual(payload["content"], "RAG-OK")
        self.assertEqual(payload["sources"], ["public-welcome"])
        sent = json.dumps(captured["payload"], ensure_ascii=False)
        self.assertIn("public-welcome", sent)
        self.assertNotIn("RH interdit", sent)
        self.assertNotIn("rh-onboarding", sent)

    def test_server_is_bound_to_loopback(self):
        self.assertEqual(self.host, "127.0.0.1")


if __name__ == "__main__":
    unittest.main()
