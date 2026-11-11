from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location(
    "release_notes", HERE / "release_notes.py"
)
assert SPEC and SPEC.loader
release_notes = importlib.util.module_from_spec(SPEC)
sys.modules["release_notes"] = release_notes
SPEC.loader.exec_module(release_notes)


class ReleaseNotesTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.notes = self.root / "qa/release/unreleased/notes"
        self.notes.mkdir(parents=True)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def write(self, name: str, text: str) -> None:
        (self.notes / name).write_text(text, encoding="utf-8")

    def run_cli(self, *args: str) -> int:
        return release_notes.main(["--root", str(self.root), *args])

    def test_fragments_sharing_a_heading_join_one_section_in_name_order(self) -> None:
        self.write("0000-01-a.md", "## Agents\n\n- **One** first.\n  More.\n")
        self.write("0000-02-b.md", "## Other\n\nIntro paragraph.\n")
        self.write("1796-c.md", "## Agents\n\n- **Two** second.\n")
        fragments, errors = release_notes.load(self.root)
        self.assertEqual(errors, [])
        self.assertEqual(
            release_notes.assemble(fragments),
            "## Agents\n\n- **One** first.\n  More.\n- **Two** second.\n"
            "\n## Other\n\nIntro paragraph.\n",
        )

    def test_check_rejects_the_shared_file_and_malformed_fragments(self) -> None:
        (self.root / "qa/release/unreleased/RELEASE_NOTES.md").write_text("x\n")
        self.write("Bad Name.md", "## H\n\n- x\n")
        self.write("no-heading.md", "- x\n")
        self.write("empty.md", "## H\n")
        self.write("two.md", "## A\n\n- x\n\n## B\n\n- y\n")
        self.write("title.md", "## A\n\n# Title\n")
        self.write("space.md", "## A\n\n- x \n")
        self.write("fence.md", "## A\n\n- x\n```\n## B\n")
        _, errors = release_notes.load(self.root)
        text = "\n".join(errors)
        for needle in [
            "RELEASE_NOTES.md must not exist",
            "Bad Name.md: name",
            "no-heading.md: must start",
            "empty.md: has a heading but no entries",
            "two.md:5: one '## ' section",
            "title.md:3: '# ' titles",
            "space.md:3: trailing whitespace",
            "fence.md: a ``` code block is never closed",
        ]:
            self.assertIn(needle, text)
        self.assertEqual(self.run_cli("check"), 1)

    def test_assemble_writes_the_version_file_and_removes_fragments(self) -> None:
        fenced = "## A\n\n- x (#1796)\n\n````shell\n# a comment\n```\n## not a heading\n````\n"
        self.write("1-a.md", fenced)
        self.assertEqual(self.run_cli("check"), 0)
        self.assertEqual(self.run_cli("assemble", "--version", "0.1.49"), 0)
        target = self.root / "qa/release/0.1.49/RELEASE_NOTES.md"
        self.assertEqual(target.read_text(), f"# Corbanu Terminal 0.1.49\n\n{fenced}")
        self.assertEqual(list(self.notes.iterdir()), [])
        # A second assembly appends after the notes already there.
        self.write("2-b.md", "## B\n\n- y\n")
        self.assertEqual(self.run_cli("assemble", "--version", "0.1.49"), 0)
        self.assertEqual(
            target.read_text(),
            f"# Corbanu Terminal 0.1.49\n\n{fenced}\n## B\n\n- y\n",
        )
        # A section the file already has is not appended twice.
        self.write("3-b.md", "## B\n\n- z\n")
        self.assertEqual(self.run_cli("assemble", "--version", "0.1.49"), 1)
        self.assertTrue((self.notes / "3-b.md").exists())

    def test_assemble_refuses_a_bad_version_or_fragment_and_keeps_fragments(
        self,
    ) -> None:
        self.write("1-a.md", "## A\n\n- x\n")
        for version in ["../x", "0.1.49\n", "0.1.49-rc1"]:
            self.assertEqual(self.run_cli("assemble", "--version", version), 1)
        self.write("2-bad.md", "- no heading\n")
        self.assertEqual(self.run_cli("assemble", "--version", "0.1.49"), 1)
        self.assertTrue((self.notes / "1-a.md").exists())
        self.assertFalse((self.root / "qa/release/0.1.49").exists())

    def test_a_paragraph_after_a_list_keeps_a_blank_line(self) -> None:
        self.write("1-a.md", "## A\n\n- x\n")
        self.write("2-a.md", "## A\n\nA paragraph.\n")
        fragments, _ = release_notes.load(self.root)
        self.assertEqual(
            release_notes.assemble(fragments), "## A\n\n- x\n\nA paragraph.\n"
        )

    def test_line_numbers_count_leading_blank_lines(self) -> None:
        self.write("a.md", "\n\n## A\n\n- x\n## B\n")
        _, errors = release_notes.load(self.root)
        self.assertIn("a.md:6: one '## ' section", "\n".join(errors))


if __name__ == "__main__":
    unittest.main()
