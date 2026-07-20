#!/usr/bin/env python3

from pathlib import Path
import sys
import tempfile
import textwrap
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from codex_package.cargo_lock import package_versions


class PackageVersionsTest(unittest.TestCase):
    def test_reads_matching_records_and_deduplicates_versions(self) -> None:
        cargo_lock = textwrap.dedent(
            """\
            version = 4

            [[package]]
            name = "other"
            version = "1.0.0"

            [[package]]
            name = "v8"
            version = "146.4.0"
            dependencies = ["other"]

            [[package]]
            name = "v8"
            version = "146.4.0"
            """
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            lock_path = Path(temp_dir) / "Cargo.lock"
            lock_path.write_text(cargo_lock, encoding="utf-8")
            self.assertEqual(["146.4.0"], package_versions(lock_path, "v8"))

    def test_returns_empty_list_when_package_is_absent(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            lock_path = Path(temp_dir) / "Cargo.lock"
            lock_path.write_text(
                '[[package]]\nname = "other"\nversion = "1.0.0"\n',
                encoding="utf-8",
            )
            self.assertEqual([], package_versions(lock_path, "v8"))


if __name__ == "__main__":
    unittest.main()
