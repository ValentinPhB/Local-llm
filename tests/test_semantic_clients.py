import json
import unittest
from unittest.mock import patch

from semantic_retrieval.clients import (
    OllamaEmbeddingProvider,
    QdrantVectorStore,
    SemanticClientError,
)


class FakeResponse:
    def __init__(self, body):
        self.body = body

    def __enter__(self):
        return self

    def __exit__(self, *args):
        return False

    def read(self):
        return json.dumps(self.body).encode("utf-8")


class SemanticClientTests(unittest.TestCase):
    def test_embedding_provider_uses_the_fixed_local_model_and_returns_vector(self):
        captured = {}

        def fake_urlopen(request, timeout):
            captured["url"] = request.full_url
            captured["body"] = json.loads(request.data)
            return FakeResponse({"embeddings": [[0.25, -0.5, 0.75]]})

        with patch("semantic_retrieval.clients.urlopen", side_effect=fake_urlopen):
            vector = OllamaEmbeddingProvider().embed("question fictive")

        self.assertEqual(vector, (0.25, -0.5, 0.75))
        self.assertEqual(captured["url"], "http://127.0.0.1:11434/api/embed")
        self.assertEqual(captured["body"], {"model": "embeddinggemma", "input": "question fictive"})

    def test_embedding_provider_rejects_non_finite_vector(self):
        with patch(
            "semantic_retrieval.clients.urlopen",
            return_value=FakeResponse({"embeddings": [[float("nan")]]}),
        ):
            with self.assertRaises(SemanticClientError):
                OllamaEmbeddingProvider().embed("question fictive")

    def test_qdrant_query_contains_only_oscar_authorized_resource_filter(self):
        captured = {}

        def fake_urlopen(request, timeout):
            captured["url"] = request.full_url
            captured["body"] = json.loads(request.data)
            return FakeResponse(
                {
                    "result": {
                        "points": [
                            {
                                "score": 0.9,
                                "payload": {
                                    "resource_id": "public-welcome",
                                    "classification": "PUBLIC",
                                    "text": "Bienvenue dans le laboratoire fictif.",
                                },
                            }
                        ]
                    }
                }
            )

        with patch("semantic_retrieval.clients.urlopen", side_effect=fake_urlopen):
            results = QdrantVectorStore().search([0.1, 0.2], ["public-welcome"])

        self.assertEqual(results[0].resource_id, "public-welcome")
        self.assertEqual(captured["url"], "http://127.0.0.1:6333/collections/lab_semantic_documents/points/query")
        self.assertEqual(
            captured["body"]["filter"],
            {"must": [{"key": "resource_id", "match": {"any": ["public-welcome"]}}]},
        )
        self.assertNotIn("rh-onboarding", json.dumps(captured["body"]))

    def test_qdrant_does_not_call_service_when_acl_resource_list_is_empty(self):
        with patch("semantic_retrieval.clients.urlopen") as service:
            results = QdrantVectorStore().search([0.1, 0.2], [])

        self.assertEqual(results, ())
        service.assert_not_called()

    def test_qdrant_rejects_an_unexpected_resource_even_if_service_returns_it(self):
        response = {
            "result": {
                "points": [
                    {
                        "score": 0.9,
                        "payload": {
                            "resource_id": "rh-onboarding",
                            "classification": "RH",
                            "text": "Contenu interdit",
                        },
                    }
                ]
            }
        }
        with patch("semantic_retrieval.clients.urlopen", return_value=FakeResponse(response)):
            with self.assertRaises(SemanticClientError):
                QdrantVectorStore().search([0.1, 0.2], ["public-welcome"])


if __name__ == "__main__":
    unittest.main()
