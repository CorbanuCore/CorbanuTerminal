#!/usr/bin/env python3
"""Distribution guard discriminator; reads built artifacts, never executes them."""

import argparse
from dataclasses import replace
import mmap
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))

import build_codex_package as package
from codex_package.cargo import SourceBuildOutputs


MARKER = b"CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION"


def require_feature_marker(artifact):
    with artifact.open("rb") as binary:
        if artifact.stat().st_size:
            with mmap.mmap(binary.fileno(), 0, access=mmap.ACCESS_READ) as data:
                if data.find(MARKER) != -1:
                    return
    raise AssertionError(f"Feature artifact is missing the required marker: {artifact}")


class PackageGuardTests(unittest.TestCase):
    feature_artifact = None

    def test_feature_artifact_rejected_before_directory_or_archive(self):
        self.assertIsNotNone(self.feature_artifact, "--feature-artifact is required")
        require_feature_marker(self.feature_artifact)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for archive_args in [[], ["--archive-output", str(root / "package.tar.gz")]]:
                with self.subTest(archive=bool(archive_args)):
                    argv = [
                        "build_codex_package.py", "--target", "aarch64-apple-darwin",
                        "--variant", "corbanu", "--cargo-profile", "dev-small",
                        "--entrypoint-bin", str(self.feature_artifact),
                        "--code-mode-host-bin", str(self.feature_artifact),
                        "--extra-bin", f"corbanu-acp={self.feature_artifact}",
                        "--extra-bin", f"corbanu-walletd={self.feature_artifact}",
                        "--package-dir", str(root / "package"), *archive_args,
                    ]
                    with patch.object(sys, "argv", argv), patch.object(
                        package.cli, "build_source_binaries", package.cli.build_source_binaries
                    ), patch.object(
                        package.cli, "resolve_rg_bin",
                        side_effect=AssertionError("Guard failed before resource resolution"),
                    ):
                        with self.assertRaisesRegex(
                            SystemExit, "Refusing distribution package: developer-accounting enabled"
                        ) as rejection:
                            package.main()
                    print(str(rejection.exception), flush=True)
                    self.assertFalse((root / "package").exists())
                    self.assertFalse((root / "package.tar.gz").exists())

    def test_missing_feature_marker_is_a_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "missing-marker"
            for content in [b"", b"feature artifact with emitter removed"]:
                artifact.write_bytes(content)
                with self.assertRaisesRegex(AssertionError, "missing the required marker"):
                    require_feature_marker(artifact)

    def test_every_source_output_rejects_marker(self):
        with tempfile.TemporaryDirectory() as directory:
            clean, marked = Path(directory) / "clean", Path(directory) / "marked"
            clean.write_bytes(b"ordinary executable fixture")
            marked.write_bytes(b"prefix\0" + MARKER + b"\0suffix")
            outputs = SourceBuildOutputs(clean, clean, {}, clean, clean, clean)
            for field in vars(outputs):
                with self.subTest(field=field):
                    value = {"extra": marked} if field == "extra_bins" else marked
                    with patch.object(
                        package, "build_source_binaries",
                        return_value=replace(outputs, **{field: value}),
                    ):
                        with self.assertRaisesRegex(SystemExit, "Refusing distribution package"):
                            package.distribution_source_binaries()

    def test_unmarked_outputs_are_allowed(self):
        with tempfile.TemporaryDirectory() as directory:
            clean = Path(directory) / "clean"
            clean.write_bytes(b"ordinary executable fixture")
            outputs = SourceBuildOutputs(clean, clean, {"extra": clean}, clean, clean, clean)
            with patch.object(package, "build_source_binaries", return_value=outputs):
                self.assertIs(package.distribution_source_binaries(), outputs)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--feature-artifact", type=Path, required=True)
    args = parser.parse_args()
    PackageGuardTests.feature_artifact = args.feature_artifact.resolve()
    unittest.main(argv=[sys.argv[0]], verbosity=2)
