import base64
import json
from pathlib import Path
import unittest

from identity.demo_sso import (
    TokenError,
    issue_demo_token,
    load_demo_directory,
    new_signing_key,
    verify_demo_token,
)


DIRECTORY_PATH = Path(__file__).parents[1] / "config" / "demo-idp" / "directory.json"


class DemoSSOTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = load_demo_directory(DIRECTORY_PATH)
        cls.signing_key = new_signing_key()

    def test_signed_token_yields_directory_identity_and_groups(self):
        token = issue_demo_token(self.directory, "alice", self.signing_key, now=1_000)
        identity = verify_demo_token(token, self.directory, self.signing_key, now=1_001)
        self.assertEqual(identity.identity_id, "alice")
        self.assertEqual(identity.groups, ("LAB_READERS", "HR"))

    def test_modified_token_is_rejected(self):
        token = issue_demo_token(self.directory, "alice", self.signing_key, now=1_000)
        header, payload, signature = token.split(".")
        signature_bytes = base64.urlsafe_b64decode(signature + "=" * (-len(signature) % 4))
        altered_signature = base64.urlsafe_b64encode(
            bytes([signature_bytes[0] ^ 1]) + signature_bytes[1:]
        ).rstrip(b"=").decode("ascii")
        altered = f"{header}.{payload}.{altered_signature}"
        with self.assertRaises(TokenError):
            verify_demo_token(altered, self.directory, self.signing_key, now=1_001)

    def test_expired_token_is_rejected(self):
        token = issue_demo_token(self.directory, "bob", self.signing_key, now=1_000)
        with self.assertRaises(TokenError):
            verify_demo_token(token, self.directory, self.signing_key, now=1_900)

    def test_unknown_identity_cannot_receive_a_token(self):
        with self.assertRaises(TokenError):
            issue_demo_token(self.directory, "mallory", self.signing_key)


if __name__ == "__main__":
    unittest.main()
