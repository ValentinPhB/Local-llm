import json
from pathlib import Path
import unittest

from access_control.engine import decide_access, decide_access_for_roles, policy_is_valid, roles_from_groups


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

    def test_directory_groups_map_to_application_roles(self):
        roles = roles_from_groups(self.policy, ("LAB_READERS", "HR"))
        self.assertEqual(roles, ("lab_reader", "rh_reader"))
        decision = decide_access_for_roles(self.policy, roles, "rh-demo")
        self.assertTrue(decision.allowed)

    def test_invalid_group_mapping_fails_closed(self):
        invalid_policy = dict(self.policy)
        invalid_policy["group_role_mappings"] = [
            {"group": "HR", "role": "rh_reader"},
            {"group": "HR", "role": "it_reader"},
        ]
        self.assertIsNone(roles_from_groups(invalid_policy, ("HR",)))

    def test_missing_resources_make_policy_invalid(self):
        invalid_policy = dict(self.policy)
        invalid_policy.pop("resources")
        self.assertFalse(policy_is_valid(invalid_policy))


if __name__ == "__main__":
    unittest.main()
