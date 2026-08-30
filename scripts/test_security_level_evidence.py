import json
import re
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

import security_level_evidence as evidence

ROOT = Path(__file__).resolve().parent.parent
CANDIDATE = {"source_commit": "a" * 40, "binary_sha256": "b" * 64, "platform": "macos"}


class EvidenceTests(unittest.TestCase):
    def test_cli_preparation_and_default_qualification_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            baseline = evidence.load_json(
                ROOT / "qa/security-levels/permissive-baseline-v1.json"
            )
            compatibility_control = evidence.load_json(
                ROOT / "qa/security-levels/compatibility/upstream-control-v2.json"
            )
            commands = [
                [
                    "security-level-compat",
                    "--prepare",
                    "--baseline",
                    baseline["captured_from_commit"],
                    "--upstream",
                    compatibility_control["identity"]["upstream_commit"],
                    "--output",
                    str(output / "compat"),
                ],
                [
                    "security-level-adversarial",
                    "--prepare",
                    "--output",
                    str(output / "attacks"),
                ],
                [
                    "security-level-standards-check",
                    "--template",
                    str(output / "crosswalk.json"),
                ],
                [
                    "security-level-standards-check",
                    "--check-plan",
                    "--manifest",
                    str(output / "crosswalk.json"),
                ],
            ]
            for command in commands:
                result = subprocess.run(
                    [sys.executable, str(ROOT / "scripts" / command[0]), *command[1:]],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn("pending", result.stdout.lower())
            for entry in (
                "security-level-adversarial",
                "security-level-standards-check",
            ):
                args = (
                    ["--output", str(output / "bad")]
                    if entry.endswith("adversarial")
                    else ["--manifest", str(output / "crosswalk.json")]
                )
                result = subprocess.run(
                    [sys.executable, str(ROOT / "scripts" / entry), *args],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(result.returncode, 2)
                self.assertNotIn("Traceback", result.stderr)

    def test_catalog_matches_frozen_contracts_and_native_paths(self):
        catalog = evidence.load_catalog(ROOT)
        self.assertEqual(len(catalog["ingresses"]), 10)
        self.assertEqual(len(catalog["sinks"]), 17)
        self.assertEqual(len(catalog["controls"]), 16)

    def test_pf27_test_selectors_exist_at_the_pinned_contract_commit(self):
        pins = evidence.validate_pins(ROOT)
        source_manifest = ROOT / "qa/security-levels/sprints/PF-27-S01/code-sha256.txt"
        for line in source_manifest.read_text().splitlines():
            expected, path = line.split(maxsplit=1)
            # Consumers may later modify these files. Check the historical contract,
            # not their current implementation, and never label this a test run.
            data = subprocess.check_output(
                ["git", "show", f"{pins['contract_commit']}:{path}"], cwd=ROOT
            )
            self.assertEqual(evidence.digest(data), expected)
        for fixture in evidence.load_json(ROOT / evidence.ADAPTERS)["fixtures"]:
            for selector in fixture["contract_tests"]:
                package, *modules, name = selector.split("::")
                directory = ROOT / "codex-rs" / package.removeprefix("codex-") / "src"
                result = subprocess.run(
                    [
                        "git",
                        "grep",
                        "-l",
                        "-E",
                        f"fn {re.escape(name)}[ (]",
                        pins["contract_commit"],
                        "--",
                        str(directory.relative_to(ROOT)),
                    ],
                    cwd=ROOT,
                    capture_output=True,
                )
                self.assertEqual(result.returncode, 0, selector)

    def test_reject_duplicate_json_keys_and_non_objects(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "data.json"
            for raw in ['{"status":"failed","status":"passed"}', "[]"]:
                path.write_text(raw)
                with self.assertRaises(evidence.EvidenceError):
                    evidence.load_json(path)

    def test_artifact_paths_and_digests_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "data").write_bytes(b"proof")
            reference = {"path": "data", "sha256": evidence.digest(b"proof")}
            self.assertEqual(evidence.checked_artifact(root, reference), b"proof")
            for path in ["../data", "/tmp/data", "C:\\data", "missing"]:
                with self.subTest(path=path), self.assertRaises(evidence.EvidenceError):
                    evidence.local_path(root, path)
            (root / "data").write_bytes(b"changed")
            with self.assertRaises(evidence.EvidenceError):
                evidence.checked_artifact(root, reference)

    def test_source_symlink_cannot_escape(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "inside").mkdir()
            (root / "outside").write_bytes(b"data")
            try:
                (root / "inside" / "link").symlink_to(root / "outside")
            except OSError:
                self.skipTest("host cannot create symlinks")
            with self.assertRaises(evidence.EvidenceError):
                evidence.local_path(root / "inside", "link")

    def test_evidence_identity_and_freshness(self):
        run = {
            "schema_version": 1,
            "phase": "qualification",
            "candidate": CANDIDATE,
            "catalog_sha256": "c" * 64,
            "run_id": "d" * 32,
            "recorded_at": datetime.now(timezone.utc)
            .isoformat()
            .replace("+00:00", "Z"),
        }
        evidence.validate_run(run, CANDIDATE, "c" * 64, "2026-01-01T00:00:00Z")
        mutations = {
            "phase": "fixture-self-test",
            "catalog_sha256": "e" * 64,
            "run_id": "",
            "recorded_at": "2020-01-01T00:00:00Z",
            "candidate": {**CANDIDATE, "source_commit": "f" * 40},
        }
        for key, value in mutations.items():
            with self.subTest(key=key), self.assertRaises(evidence.EvidenceError):
                evidence.validate_run(
                    {**run, key: value}, CANDIDATE, "c" * 64, "2026-01-01T00:00:00Z"
                )

    def test_exclusive_evidence_write_preserves_previous_run(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            evidence.write_json(path, {"status": "failed"})
            with self.assertRaises(FileExistsError):
                evidence.write_json(path, {"status": "passed"})
            self.assertEqual(json.loads(path.read_text())["status"], "failed")


if __name__ == "__main__":
    unittest.main()
