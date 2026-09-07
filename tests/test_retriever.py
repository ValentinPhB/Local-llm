import json
from pathlib import Path
import unittest

from document_store.retriever import retrieve_documents


ROOT = Path(__file__).parents[1]
POLICY_PATH = ROOT / "config" / "access-control" / "demo-policy.json"


class RetrieverTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))

    def test_returns_bounded_lexical_match(self):
        results = retrieve_documents(self.policy, ["public-welcome"], "organisation Acme-Lab", ROOT)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].resource_id, "public-welcome")
        self.assertLessEqual(len(results[0].excerpt), 500)

    def test_no_match_returns_empty_result(self):
        self.assertEqual(retrieve_documents(self.policy, ["public-welcome"], "zyxwvu", ROOT), ())
