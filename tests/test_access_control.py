import json
from pathlib import Path
import unittest

from access_control.engine import decide_access


POLICY_PATH = Path(__file__).parents[1] / "config" / "access-control" / "demo-policy.json"


class AccessControlTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy = json.loads(POLICY_PATH.read_text())

    def test_expected_access_matrix(self):
        expected = {
            ("alice", "public-demo"): True,
            ("alice", "rh-demo"): True,
            ("alice", "it-demo"): False,
            ("bob", "public-demo"): True,
            ("bob", "rh-demo"): False,
            ("bob", "it-demo"): True,
            ("charlie", "public-demo"): True,
            ("charlie", "rh-demo"): False,
            ("charlie", "it-demo"): False,
        }

        for (identity_id, resource_id), allowed in expected.items():
            with self.subTest(identity=identity_id, resource=resource_id):
                decision = decide_access(self.policy, identity_id, resource_id)
                self.assertEqual(decision.allowed, allowed)

    def test_unknown_identity_is_denied(self):
        decision = decide_access(self.policy, "unknown", "public-demo")
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.reason, "unknown_identity")

    def test_unknown_resource_is_denied(self):
        decision = decide_access(self.policy, "alice", "unknown")
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.reason, "unknown_resource")

    def test_invalid_policy_is_denied(self):
        invalid_policy = {"default_decision": "allow", "identities": [], "resources": []}
        decision = decide_access(invalid_policy, "alice", "public-demo")
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.reason, "invalid_policy")

    def test_role_match_identifies_the_role_used(self):
        decision = decide_access(self.policy, "alice", "rh-demo")
        self.assertTrue(decision.allowed)
        self.assertEqual(decision.reason, "role_match")
        self.assertEqual(decision.matched_roles, ("rh_reader",))


if __name__ == "__main__":
    unittest.main()
