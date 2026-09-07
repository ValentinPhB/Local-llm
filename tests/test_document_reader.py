import json
from pathlib import Path
import unittest

from document_store.reader import DocumentError, read_policy_document


ROOT = Path(__file__).parents[1]
POLICY_PATH = ROOT / "config" / "access-control" / "demo-policy.json"


class ControlledDocumentReaderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))

    def test_reads_only_the_policy_declared_document(self):
        document = read_policy_document(self.policy, "public-welcome", ROOT)
        self.assertEqual(document.resource_id, "public-welcome")
        self.assertEqual(document.classification, "PUBLIC")
        self.assertIn("Acme-Lab", document.content)

    def test_policy_path_outside_demo_documents_is_refused(self):
        invalid_policy = json.loads(json.dumps(self.policy))
        invalid_policy["resources"][0]["path"] = "AGENTS.md"
        with self.assertRaises(DocumentError):
            read_policy_document(invalid_policy, "public-welcome", ROOT)


if __name__ == "__main__":
    unittest.main()
