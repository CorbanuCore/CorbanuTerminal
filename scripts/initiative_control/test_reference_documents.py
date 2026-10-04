import hashlib
from pathlib import Path
import tempfile
import unittest

import control
import export


class ReferenceDocumentTests(unittest.TestCase):
    def test_default_and_invalid_inventories(self):
        self.assertEqual(control.reference_documents({}), [])
        for paths in (None, "qa/a.md", ["qa/a.md"] * 101,
                      ["/qa/a.md"], ["qa/../a.md"], ["qa//a.md"],
                      ["qa/.private/a.md"], ["qa/a.log"], ["docs/other/a.md"],
                      ["https://example.com/a.md"], [1]):
            with self.subTest(paths=paths), self.assertRaises(ValueError):
                control.reference_documents({"reference_documents": paths})

    def test_export_collect_and_link_use_same_explicit_inventory(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "scripts/initiative_control").mkdir(parents=True)
            selected = ["docs/research/accounting/allocation.md", "qa/proof.md"]
            texts = ["# Allocation\n[Proof](../../../qa/proof.md)",
                     "# Proof\n[Private](../private.md)"]
            for relative, text in zip(selected, texts):
                path = repo / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            (repo / "private.md").write_text("Do not publish")
            config = {"human_tests": [], "reference_documents": selected}
            paths = export.source_paths(repo, config)
            self.assertTrue(all(repo / p in paths for p in selected))
            self.assertNotIn(repo / "private.md", paths)
            hashes = {p: hashlib.sha256(t.encode()).hexdigest()
                      for p, t in zip(selected, texts)}
            docs = control.collect_references(repo, config, hashes)
            self.assertEqual(docs, dict(zip(selected, texts)))
            rendered = control.safe_markdown(texts[0], selected[0], docs)
            self.assertIn(control.route(selected[1]), rendered)
            self.assertIn('#unpublished', control.safe_markdown(texts[1], selected[1], docs))
            with self.assertRaisesRegex(ValueError, "uncollected"):
                control.collect_references(repo, config, {})

    def test_missing_sensitive_and_symlink_sources_fail_before_export(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "scripts/initiative_control").mkdir(parents=True)
            (repo / "qa").mkdir()
            path = repo / "qa/proof.md"
            config = {"human_tests": [], "reference_documents": ["qa/proof.md"]}
            with self.assertRaises(FileNotFoundError):
                export.source_paths(repo, config)
            path.write_text("password=synthetic-canary")
            with self.assertRaises(ValueError):
                export.source_paths(repo, config)
            path.unlink()
            target = repo / "qa/other.md"
            target.write_text("# Safe")
            path.symlink_to(target)
            with self.assertRaises(ValueError):
                export.source_paths(repo, config)


if __name__ == "__main__":
    unittest.main()
