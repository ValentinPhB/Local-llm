import json
import re
from pathlib import Path
import unittest


ROOT = Path(__file__).parents[1]
MARKDOWN_LINK = re.compile(r"(?<!!)\[[^]]+\]\(([^)#]+)(?:#[^)]*)?\)")


class RepositoryHygieneTests(unittest.TestCase):
    def test_all_versioned_json_configuration_is_valid(self):
        for path in sorted((ROOT / "config").rglob("*.json")):
            with self.subTest(path=path.relative_to(ROOT)):
                json.loads(path.read_text(encoding="utf-8"))

    def test_internal_markdown_links_target_existing_files(self):
        markdown_files = [ROOT / "README.md", ROOT / "STATUS.example.md", ROOT / "AGENTS.md"]
        markdown_files.extend((ROOT / "docs").glob("*.md"))
        for source in markdown_files:
            content = source.read_text(encoding="utf-8")
            for target in MARKDOWN_LINK.findall(content):
                if "://" in target or target.startswith("mailto:"):
                    continue
                with self.subTest(source=source.relative_to(ROOT), target=target):
                    self.assertTrue((source.parent / target).resolve().is_file())
