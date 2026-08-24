import tempfile
import unittest
from pathlib import Path

from scripts.check_portable_skills import compare_skill_trees


class PortableSkillMirrorTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.source = root / ".codex" / "skills"
        self.portable = root / ".agents" / "skills"
        self.source.mkdir(parents=True)
        self.portable.mkdir(parents=True)

    def tearDown(self):
        self.temporary.cleanup()

    def write_pair(self, relative: str, content: str = "same\n"):
        source = self.source / relative
        portable = self.portable / relative
        source.parent.mkdir(parents=True, exist_ok=True)
        portable.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(content, encoding="utf-8")
        portable.write_text(content, encoding="utf-8")
        return Path(relative), source, portable

    def test_matching_tree_passes(self):
        relative, _, _ = self.write_pair("example/SKILL.md")
        self.assertEqual(
            compare_skill_trees(self.source, self.portable, [relative]),
            [],
        )

    def test_missing_extra_and_changed_files_fail(self):
        relative, _, portable = self.write_pair("example/SKILL.md")
        portable.write_text("changed\n", encoding="utf-8")
        extra = self.portable / "extra/SKILL.md"
        extra.parent.mkdir(parents=True)
        extra.write_text("extra\n", encoding="utf-8")
        missing = Path("missing/SKILL.md")
        self.assertEqual(
            compare_skill_trees(
                self.source,
                self.portable,
                [relative, missing],
            ),
            [
                "missing portable skill file: missing/SKILL.md",
                "unexpected portable skill file: extra/SKILL.md",
                "content differs: example/SKILL.md",
            ],
        )

    def test_executable_mode_must_match(self):
        relative, source, _ = self.write_pair("example/scripts/run.py")
        source.chmod(0o755)
        self.assertEqual(
            compare_skill_trees(self.source, self.portable, [relative]),
            ["executable mode differs: example/scripts/run.py"],
        )


if __name__ == "__main__":
    unittest.main()
