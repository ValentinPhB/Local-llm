from pathlib import Path
import unittest


UI_PATH = Path(__file__).parents[1] / "ui" / "index.html"


class UIContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.page = UI_PATH.read_text(encoding="utf-8")

    def test_ui_exposes_all_demo_identities(self):
        for identity in ("alice", "bob", "charlie", "oscar"):
            with self.subTest(identity=identity):
                self.assertIn(f'data-identity="{identity}"', self.page)

    def test_ui_exposes_separate_simple_and_rag_chat_actions(self):
        self.assertIn("/api/chat", self.page)
        self.assertIn("/api/rag-chat", self.page)
        self.assertIn("Envoyer avec sources autorisées", self.page)
