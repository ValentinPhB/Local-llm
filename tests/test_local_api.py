import http.client
import json
import threading
import unittest

from ui.server import SESSION_COOKIE_NAME, create_server


class LocalAPITests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = create_server(port=0)
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

    def test_session_is_signed_and_used_for_access_decision(self):
        headers = self.start_demo_session("alice")
        status, payload, _ = self.request("GET", "/api/session", headers=headers)
        self.assertEqual(status, 200)
        self.assertEqual(payload["identity"]["id"], "alice")

        status, payload, _ = self.request("GET", "/api/access-check?resource_id=rh-onboarding", headers=headers)
        self.assertEqual(status, 200)
        self.assertTrue(payload["allowed"])

        status, payload, _ = self.request("GET", "/api/access-check?resource_id=it-workstation", headers=headers)
        self.assertEqual(status, 403)
        self.assertFalse(payload["allowed"])

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


if __name__ == "__main__":
    unittest.main()
