"""Packaging-boundary regression tests; synthetic bytes, no binary execution."""
from dataclasses import replace
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[5] / "scripts"))
import build_codex_package as package
from codex_package.cargo import SourceBuildOutputs


class PackageGuardTests(unittest.TestCase):
    def test_clean_source_outputs_are_preserved(self):
        with tempfile.TemporaryDirectory() as root:
            clean = Path(root) / "clean"
            clean.write_bytes(b"ordinary executable fixture")
            outputs = SourceBuildOutputs(clean, clean, {"extra": clean}, clean, clean, clean)
            with patch.object(package, "build_source_binaries", return_value=outputs):
                self.assertIs(package.distribution_source_binaries(), outputs)

    def test_every_source_output_rejects_feature_bytes(self):
        with tempfile.TemporaryDirectory() as root:
            clean, marked = Path(root) / "clean", Path(root) / "marked"
            clean.write_bytes(b"ordinary executable fixture")
            marked.write_bytes(b"prefix\0CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION\0suffix")
            outputs = SourceBuildOutputs(clean, clean, {}, clean, clean, clean)
            for field in vars(outputs):
                with self.subTest(field=field):
                    value = {"extra": marked} if field == "extra_bins" else marked
                    with patch.object(package, "build_source_binaries",
                                      return_value=replace(outputs, **{field: value})):
                        with self.assertRaisesRegex(SystemExit, "Refusing distribution package"):
                            package.distribution_source_binaries()

    def test_cli_rejects_archive_and_directory_before_assembly(self):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            marked = root / "corbanu"
            marked.write_bytes(b"CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION")
            marked.chmod(0o755)
            for archive_args in [[], ["--archive-output", str(root / "package.tar.gz")]]:
                with self.subTest(archive_args=archive_args):
                    argv = ["build_codex_package.py", "--target", "aarch64-apple-darwin",
                            "--variant", "corbanu", "--cargo-profile", "dev-small",
                            "--entrypoint-bin", str(marked), "--code-mode-host-bin", str(marked),
                            "--extra-bin", f"corbanu-acp={marked}",
                            "--extra-bin", f"corbanu-walletd={marked}",
                            "--package-dir", str(root / "package"), *archive_args]
                    with patch.object(sys, "argv", argv), patch.object(
                        package.cli, "build_source_binaries", package.cli.build_source_binaries
                    ), patch.object(package.cli, "build_package_dir") as assemble:
                        with self.assertRaisesRegex(SystemExit, "Refusing distribution package"):
                            package.main()
                        assemble.assert_not_called()
                    self.assertFalse((root / "package").exists())
                    self.assertFalse((root / "package.tar.gz").exists())


if __name__ == "__main__":
    unittest.main()
