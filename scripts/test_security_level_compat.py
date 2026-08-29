import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_level_compat as compat


class SecurityLevelCompatTests(unittest.TestCase):
    def test_prepare_validates_frozen_baseline_without_running_a_candidate(self):
        root = Path(__file__).resolve().parent.parent
        baseline = json.loads((root / compat.BASELINE_PATH).read_text())[
            "captured_from_commit"
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = compat.prepare_compatibility(
                root, baseline, Path(directory) / "prepared"
            )
            report = json.loads(path.read_text())
            self.assertEqual(report["status"], "pending")
            self.assertIsNone(report["candidate"])
            self.assertEqual(report["immutable_probes_validated"], 5)
            self.assertEqual(report["surfaces"], 10)

    def test_prepare_rejects_rewritten_baseline_before_any_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / compat.BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text('{"schema_version":1}')
            with self.assertRaisesRegex(
                compat.CompatibilityError, "baseline bytes changed"
            ):
                compat.prepare_compatibility(root, "a" * 40, root / "output")

    def test_extract_test_source_stops_at_next_test(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "tests.rs"
            source.write_text(
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n\n"
                "#[tokio::test]\nasync fn adjacent() {\n    assert!(false);\n}\n",
                encoding="utf-8",
            )
            self.assertEqual(
                compat.extract_test_source(source, "frozen_probe"),
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n",
            )

    def test_extract_test_source_includes_attached_attributes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "tests.rs"
            source.write_text(
                '#[ignore = "frozen probe must execute"]\n'
                '#[cfg(not(target_os = "none"))]\n'
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n",
                encoding="utf-8",
            )
            self.assertEqual(
                compat.extract_test_source(source, "frozen_probe"),
                '#[ignore = "frozen probe must execute"]\n'
                '#[cfg(not(target_os = "none"))]\n'
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n",
            )

    def test_executed_test_count_requires_a_nextest_summary(self) -> None:
        result = compat.CommandResult(
            command=["just", "test"],
            returncode=0,
            stdout="Summary [0.027s] 1 test run: 1 passed, 10 skipped\n",
            stderr="",
        )
        self.assertEqual(compat.executed_test_count(result), 1)
        self.assertEqual(
            compat.executed_test_count(
                compat.CommandResult(
                    command=["just", "test"],
                    returncode=0,
                    stdout="Starting 0 tests\n",
                    stderr="",
                )
            ),
            0,
        )

    def test_manifest_requires_immutable_probe_digest_and_full_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "probe.rs"
            source.write_text(
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n",
                encoding="utf-8",
            )
            probe = {
                "id": "frozen-probe",
                "package": "codex-example",
                "test_filter": "frozen_probe",
                "source": "probe.rs",
                "function": "frozen_probe",
                "source_sha256": compat.sha256_bytes(
                    compat.extract_test_source(source, "frozen_probe").encode("utf-8")
                ),
                "covers": ["surface"],
            }
            manifest = {
                "schema_version": 1,
                "captured_from_commit": "a" * 40,
                "composition_contract": {
                    "rule": "final_allow = existing_allow && security_layer_allow",
                    "permissive_security_layer_allow": True,
                },
                "surfaces": [{"id": "surface"}],
                "probes": [probe],
            }
            self.assertEqual(
                compat.validate_manifest(manifest, root, "a" * 40), [probe]
            )

            source.write_text(
                "#[test]\nfn frozen_probe() {\n    assert!(false);\n}\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                compat.CompatibilityError, "probe source drift"
            ):
                compat.validate_manifest(manifest, root, "a" * 40)

    @unittest.skipIf(os.name == "nt", "executable fixture uses a POSIX shebang")
    def test_candidate_identity_records_binary_hash_and_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            candidate = root / "candidate"
            candidate.write_text(
                '#!/bin/sh\n[ "$1" = "--version" ] && echo \'corbanu 1.2.3\'\n',
                encoding="utf-8",
            )
            candidate.chmod(0o755)
            identity, result = compat.candidate_identity(candidate, root)
            self.assertEqual(result.returncode, 0)
            self.assertEqual(identity["version"], "corbanu 1.2.3")
            self.assertEqual(identity["sha256"], compat.sha256_file(candidate))

    def test_build_workspace_candidate_rejects_an_arbitrary_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(
                compat.CompatibilityError, "must be the workspace binary"
            ):
                compat.build_workspace_candidate(root, root / "unrelated-corbanu")

    def test_build_workspace_candidate_builds_the_canonical_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = compat.workspace_candidate_path(root)
            build_result = compat.CommandResult(
                command=["cargo", "build"], returncode=0, stdout="", stderr=""
            )
            with mock.patch.object(
                compat, "run_command", return_value=build_result
            ) as run:
                self.assertEqual(
                    compat.build_workspace_candidate(root, expected), build_result
                )
            run.assert_called_once_with(
                [
                    "cargo",
                    "build",
                    "--target-dir",
                    str(root / "codex-rs" / "target"),
                    "-p",
                    "codex-cli",
                    "--bin",
                    "corbanu",
                ],
                cwd=root / "codex-rs",
            )

    def test_write_report_replaces_complete_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            destination = compat.write_report(Path(directory), {"status": "passed"})
            self.assertEqual(
                json.loads(destination.read_text(encoding="utf-8")),
                {"status": "passed"},
            )


if __name__ == "__main__":
    unittest.main()
