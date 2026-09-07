import json
from pathlib import Path
import unittest

from semantic_retrieval.indexer import (
    MAX_CHUNK_CHARS,
    ControlledIndexer,
    IndexingError,
    chunk_document,
)


ROOT = Path(__file__).parents[1]
POLICY_PATH = ROOT / "config" / "access-control" / "demo-policy.json"


class FakeEmbeddingProvider:
    def __init__(self):
        self.texts = []

    def embed(self, text):
        self.texts.append(text)
        return (float(len(text)), 1.0)


class FakeWriter:
    def __init__(self):
        self.calls = []

    def replace(self, passages):
        self.calls.append(tuple(passages))


class ControlledIndexerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))

    def test_chunks_keep_sentence_boundaries_and_small_overlap(self):
        sentence = "Une phrase de test contient assez de mots pour rester lisible."
        chunks = chunk_document(" ".join([sentence] * 30))

        self.assertGreater(len(chunks), 1)
        self.assertTrue(all(len(chunk) <= MAX_CHUNK_CHARS for chunk in chunks))
        self.assertTrue(all(chunk.endswith(".") for chunk in chunks))
        self.assertTrue(any(chunks[index - 1].split(".")[-2] in chunks[index] for index in range(1, len(chunks))))

    def test_indexes_only_policy_declared_documents_and_preserves_metadata(self):
        provider = FakeEmbeddingProvider()
        writer = FakeWriter()
        indexer = ControlledIndexer(self.policy, ROOT, provider, writer)

        passages = indexer.index(["public-welcome"])

        self.assertEqual(len(writer.calls), 1)
        self.assertEqual(writer.calls[0], passages)
        self.assertTrue(passages)
        self.assertTrue(all(item.resource_id == "public-welcome" for item in passages))
        self.assertTrue(all(item.classification == "PUBLIC" for item in passages))
        self.assertTrue(all(not item.text.startswith("---") for item in passages))
        self.assertEqual(provider.texts, [item.text for item in passages])

    def test_refuses_resource_outside_policy_before_embedding_or_write(self):
        provider = FakeEmbeddingProvider()
        writer = FakeWriter()
        indexer = ControlledIndexer(self.policy, ROOT, provider, writer)

        with self.assertRaises(IndexingError):
            indexer.index(["../../AGENTS.md"])

        self.assertEqual(provider.texts, [])
        self.assertEqual(writer.calls, [])

    def test_refuses_invalid_acl_policy_before_embedding_or_write(self):
        invalid_policy = json.loads(json.dumps(self.policy))
        invalid_policy["resources"][0]["allowed_roles"] = "lab_reader"
        provider = FakeEmbeddingProvider()
        writer = FakeWriter()

        with self.assertRaises(IndexingError):
            ControlledIndexer(invalid_policy, ROOT, provider, writer)

        self.assertEqual(provider.texts, [])
        self.assertEqual(writer.calls, [])

    def test_does_not_write_a_partial_lot_if_embedding_dimensions_change(self):
        class InconsistentProvider:
            def __init__(self):
                self.calls = 0

            def embed(self, text):
                self.calls += 1
                return (1.0,) if self.calls == 1 else (1.0, 2.0)

        writer = FakeWriter()
        indexer = ControlledIndexer(self.policy, ROOT, InconsistentProvider(), writer)

        with self.assertRaises(IndexingError):
            indexer.index(["public-welcome", "public-model-guidelines"])

        self.assertEqual(writer.calls, [])


if __name__ == "__main__":
    unittest.main()
