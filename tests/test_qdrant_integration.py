"""Intégration Qdrant éphémère, activée exclusivement par la CI.

Ce test n'indexe aucun document du dépôt : il ne manipule que deux passages
fictifs dans un conteneur de service détruit à la fin du job GitHub Actions.
"""

from __future__ import annotations

import os
import time
import unittest
from urllib.error import URLError
from urllib.request import urlopen

from semantic_retrieval.clients import QDRANT_URL, QdrantIndexWriter, QdrantVectorStore
from semantic_retrieval.indexer import IndexedPassage


RUN_INTEGRATION = os.environ.get("RUN_QDRANT_INTEGRATION") == "1"


@unittest.skipUnless(RUN_INTEGRATION, "Qdrant integration runs only in the CI service")
class QdrantIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        for _ in range(40):
            try:
                with urlopen(f"{QDRANT_URL}/healthz", timeout=1) as response:  # noqa: S310 -- fixed loopback URL
                    if response.read():
                        return
            except (URLError, TimeoutError):
                time.sleep(0.25)
        raise RuntimeError("Qdrant CI service did not become ready")

    def test_writer_and_acl_filter_work_against_real_ephemeral_qdrant(self):
        public = IndexedPassage(
            "public-welcome",
            "PUBLIC",
            0,
            "Bienvenue dans le laboratoire fictif.",
            (0.2, -0.1, 0.8),
        )
        rh = IndexedPassage(
            "rh-onboarding",
            "RH",
            0,
            "Checklist RH fictive d'intégration.",
            (0.2, -0.1, 0.8),
        )

        QdrantIndexWriter().replace((public, rh))
        results = QdrantVectorStore().search((0.2, -0.1, 0.8), ["public-welcome"])

        self.assertEqual([result.resource_id for result in results], ["public-welcome"])
        self.assertEqual(results[0].classification, "PUBLIC")
        self.assertNotIn("RH", results[0].excerpt)
