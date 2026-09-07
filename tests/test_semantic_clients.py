import json
import unittest
from urllib.error import HTTPError
from unittest.mock import patch

from semantic_retrieval.clients import (
    OllamaEmbeddingProvider,
    QdrantIndexWriter,
    QdrantVectorStore,
    SemanticClientError,
)
from semantic_retrieval.indexer import IndexedPassage


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

    def test_writer_rebuilds_only_the_fixed_collection_with_deterministic_points(self):
        captured = []

        def fake_urlopen(request, timeout):
            captured.append((request.method, request.full_url, request.data))
            return FakeResponse({"status": "ok", "result": True})

        passages = (
            IndexedPassage("public-welcome", "PUBLIC", 0, "Bienvenue.", (0.2, -0.1)),
            IndexedPassage("public-welcome", "PUBLIC", 1, "Règles locales.", (0.4, 0.8)),
        )
        with patch("semantic_retrieval.clients.urlopen", side_effect=fake_urlopen):
            QdrantIndexWriter().replace(passages)

        self.assertEqual(
            [(method, url) for method, url, _ in captured],
            [
                ("DELETE", "http://127.0.0.1:6333/collections/lab_semantic_documents"),
                ("PUT", "http://127.0.0.1:6333/collections/lab_semantic_documents"),
                ("PUT", "http://127.0.0.1:6333/collections/lab_semantic_documents/points?wait=true"),
            ],
        )
        collection_body = json.loads(captured[1][2])
        point_body = json.loads(captured[2][2])
        self.assertEqual(collection_body, {"vectors": {"size": 2, "distance": "Cosine"}})
        self.assertEqual(point_body["points"][0]["payload"]["resource_id"], "public-welcome")
        self.assertEqual(point_body["points"][0]["payload"]["classification"], "PUBLIC")
        self.assertNotEqual(point_body["points"][0]["id"], point_body["points"][1]["id"])

    def test_writer_refuses_invalid_batch_before_any_qdrant_request(self):
        invalid = IndexedPassage("rh-onboarding", "SECRET", 0, "Interdit.", (0.1, 0.2))
        with patch("semantic_retrieval.clients.urlopen") as service:
            with self.assertRaises(SemanticClientError):
                QdrantIndexWriter().replace((invalid,))

        service.assert_not_called()

    def test_writer_cannot_target_another_collection(self):
        with self.assertRaises(SemanticClientError):
            QdrantIndexWriter(collection="other_collection")

    def test_writer_accepts_missing_collection_only_for_its_initial_delete(self):
        responses = iter(
            [
                HTTPError("http://127.0.0.1:6333/collections/lab_semantic_documents", 404, "", None, None),
                FakeResponse({"status": "ok", "result": True}),
                FakeResponse({"status": "ok", "result": True}),
            ]
        )

        def fake_urlopen(request, timeout):
            response = next(responses)
            if isinstance(response, HTTPError):
                raise response
            return response

        passage = IndexedPassage("public-welcome", "PUBLIC", 0, "Bienvenue.", (0.2, -0.1))
        with patch("semantic_retrieval.clients.urlopen", side_effect=fake_urlopen):
            QdrantIndexWriter().replace((passage,))


if __name__ == "__main__":
    unittest.main()
