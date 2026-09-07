import json
from collections import Counter
from pathlib import Path
import unittest


ROOT = Path(__file__).parents[1]
POLICY_PATH = ROOT / "config" / "access-control" / "demo-policy.json"


def front_matter(path):
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "---":
        raise ValueError("front matter absent")
    metadata = {}
    for line in lines[1:]:
        if line == "---":
            return metadata
        key, separator, value = line.partition(":")
        if not separator or not key or not value.strip():
            raise ValueError("front matter invalide")
        metadata[key] = value.strip()
    raise ValueError("front matter non fermé")


class DemoDocumentsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
        cls.resources = cls.policy["resources"]

    def test_exact_distribution(self):
        self.assertEqual(len(self.resources), 15)
        self.assertEqual(
            Counter(resource["classification"] for resource in self.resources),
            {"PUBLIC": 9, "RH": 3, "IT": 3},
        )

    def test_every_acl_resource_is_a_real_classified_document(self):
        paths = set()
        for resource in self.resources:
            with self.subTest(resource=resource["id"]):
                relative_path = resource.get("path")
                self.assertIsInstance(relative_path, str)
                self.assertNotIn(relative_path, paths)
                paths.add(relative_path)
                document_path = ROOT / relative_path
                self.assertTrue(document_path.is_file())
                metadata = front_matter(document_path)
                self.assertEqual(metadata.get("id"), resource["id"])
                self.assertEqual(metadata.get("classification"), resource["classification"])
                self.assertTrue(metadata.get("owner", "").startswith("demo-"))


if __name__ == "__main__":
    unittest.main()
