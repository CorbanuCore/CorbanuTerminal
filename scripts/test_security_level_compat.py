import copy
import datetime
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_level_compat as compat

CONTROL_NOW = datetime.datetime(2026, 8, 30, 18, 0, tzinfo=datetime.timezone.utc)


class SecurityLevelCompatTests(unittest.TestCase):
    def test_prepare_validates_frozen_baseline_without_running_a_candidate(self):
        root = Path(__file__).resolve().parent.parent
        baseline = json.loads((root / compat.BASELINE_PATH).read_text())[
            "captured_from_commit"
        ]
        upstream = json.loads((root / compat.CONTROL_PATH).read_text())["identity"][
            "upstream_commit"
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = compat.prepare_compatibility(
                root,
                baseline,
                upstream,
                Path(directory) / "prepared",
            )
            report = json.loads(path.read_text())
            self.assertEqual(report["status"], "pending")
            self.assertIsNone(report["candidate"])
            self.assertEqual(report["immutable_probes_validated"], 5)
            self.assertEqual(report["surfaces"], 10)
            self.assertEqual(report["expanded_cases_validated"], 9)
            self.assertEqual(report["protected_cases_validated"], 4)
            self.assertEqual(report["expanded_surfaces"], 9)

    def test_prepare_rejects_rewritten_baseline_before_any_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / compat.BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text('{"schema_version":1}')
            with self.assertRaisesRegex(
                compat.CompatibilityError, "baseline bytes changed"
            ):
                compat.prepare_compatibility(root, "a" * 40, "b" * 40, root / "output")

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

    def test_expanded_extractor_requires_a_real_test_and_ignores_literal_braces(self):
        source = (
            '#[cfg_attr(\n    feature = "fixture",\n    ignore\n)]\n'
            "#[tokio::test]\nasync fn exact_case() {\n"
            '    let fixture = r##"{ not structure }"##; // }\n'
            "    /* { nested /* } */ comment } */\n"
            "    assert!(!fixture.is_empty());\n}\n"
        )
        extracted = compat.extract_expanded_test_source(source, "exact_case")
        self.assertIn('r##"{ not structure }"##', extracted)
        self.assertTrue(extracted.endswith("}\n"))
        with self.assertRaisesRegex(compat.CompatibilityError, "is not a test"):
            compat.extract_expanded_test_source("fn exact_case() {}\n", "exact_case")

    def test_expanded_case_requires_exactly_one_executed_test(self) -> None:
        broad_match = compat.CommandResult(
            command=["just", "test"],
            returncode=0,
            stdout="Summary [0.027s] 2 tests run: 2 passed, 10 skipped\n",
            stderr="",
        )
        case = {
            "id": "exact-case",
            "surface": "browser",
            "package": "codex-core",
            "test_filter": "exact_case",
        }
        with mock.patch.object(compat, "run_command", return_value=broad_match):
            [result] = compat.run_expanded_cases(Path("/repo"), [case], {})
        self.assertEqual(result["executed_tests"], 2)
        self.assertFalse(result["passed"])

    def test_protected_case_records_guarantee_without_a_surface(self) -> None:
        exact = compat.CommandResult(
            command=["just", "test"],
            returncode=0,
            stdout="Summary [0.027s] 1 test run: 1 passed, 10 skipped\n",
            stderr="",
        )
        case = {
            "id": "protected-case",
            "package": "codex-example",
            "test_filter": "exact_case",
            "guarantee": "explicit opt-in",
        }
        with mock.patch.object(compat, "run_command", return_value=exact):
            [result] = compat.run_expanded_cases(Path("/repo"), [case], {})
        self.assertEqual(result["guarantee"], "explicit opt-in")
        self.assertNotIn("surface", result)
        self.assertTrue(result["passed"])

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
                env=None,
            )

    def test_dirty_runtime_tree_fails_before_candidate_qualification(self) -> None:
        dirty = compat.CommandResult(
            command=["git", "status"],
            returncode=0,
            stdout=" M codex-rs/core/src/lib.rs\n",
            stderr="",
        )
        with mock.patch.object(compat, "run_command", return_value=dirty):
            with self.assertRaisesRegex(
                compat.CompatibilityError, "runtime tree is dirty"
            ):
                compat.require_clean_runtime_tree(Path("/repo"))

    def test_dirty_root_justfile_fails_runtime_qualification(self) -> None:
        dirty = compat.CommandResult(
            command=["git", "status"],
            returncode=0,
            stdout=" M justfile\n",
            stderr="",
        )
        with mock.patch.object(compat, "run_command", return_value=dirty) as run:
            with self.assertRaisesRegex(
                compat.CompatibilityError, "runtime tree is dirty"
            ):
                compat.require_clean_runtime_tree(Path("/repo"))
        self.assertEqual(run.call_args.args[0][-4:], list(compat.RUNTIME_INPUT_PATHS))

    def test_artifact_roots_inside_the_worktree_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            with self.assertRaisesRegex(compat.CompatibilityError, "outside"):
                compat.require_external_artifact_roots(root, root / "build")
            compat.require_external_artifact_roots(root, root.parent / "external-build")

    def test_stale_control_targets_are_pruned_but_current_targets_remain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "baseline-old").mkdir()
            (root / "upstream-current").mkdir()
            (root / "unrelated").mkdir()
            removed = compat.prune_stale_control_targets(
                root, keep_names=frozenset({"upstream-current"})
            )
            self.assertEqual(removed, ["baseline-old"])
            self.assertTrue((root / "upstream-current").is_dir())
            self.assertTrue((root / "unrelated").is_dir())

    def test_cleanup_attempts_every_control_worktree(self) -> None:
        with mock.patch.object(
            compat,
            "remove_detached_worktree",
            side_effect=[compat.CompatibilityError("first"), mock.DEFAULT],
        ) as remove:
            errors = compat.cleanup_control_worktrees(
                Path("/repo"), [Path("/one"), Path("/two")]
            )
        self.assertEqual(remove.call_count, 2)
        self.assertEqual(errors, ["first"])

    def test_command_timeout_is_a_contract_error(self) -> None:
        with mock.patch.object(
            compat.subprocess,
            "run",
            side_effect=compat.subprocess.TimeoutExpired(["slow"], 1),
        ):
            with self.assertRaisesRegex(compat.CompatibilityError, "timed out"):
                compat.run_command(["slow"], cwd=Path("/repo"), timeout_seconds=1)

    def test_write_report_replaces_complete_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            destination = compat.write_report(Path(directory), {"status": "passed"})
            self.assertEqual(
                json.loads(destination.read_text(encoding="utf-8")),
                {"status": "passed"},
            )

    def expanded_documents(self):
        root = Path(__file__).resolve().parent.parent
        control = json.loads((root / compat.CONTROL_PATH).read_text())
        ledger = json.loads((root / compat.DRIFT_LEDGER_PATH).read_text())
        return root, control, ledger

    def validate_documents(self, root, control, ledger, **kwargs):
        return compat.validate_expanded_control(
            control,
            ledger,
            root,
            control["identity"]["baseline_commit"],
            control["identity"]["upstream_commit"],
            compat.git_output(root, ["rev-parse", "HEAD"]),
            now=CONTROL_NOW,
            **kwargs,
        )

    def test_expanded_control_covers_every_required_surface(self) -> None:
        root, control, ledger = self.expanded_documents()
        cases = self.validate_documents(root, control, ledger)
        self.assertEqual(len(cases), 9)
        protected = compat.validate_protected_cases(
            control, root, compat.git_output(root, ["rev-parse", "HEAD"])
        )
        self.assertEqual(len(protected), 4)

    def test_expanded_control_rejects_a_missing_surface(self) -> None:
        root, control, ledger = self.expanded_documents()
        control["surfaces"].pop()
        with self.assertRaisesRegex(
            compat.CompatibilityError, "inventory is incomplete"
        ):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_rejects_mismatched_identities(self) -> None:
        root, control, ledger = self.expanded_documents()
        control["identity"]["baseline_manifest_sha256"] = "0" * 64
        with self.assertRaisesRegex(
            compat.CompatibilityError, "identities do not match"
        ):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_requires_distinct_pinned_rust_trees(self) -> None:
        root, control, ledger = self.expanded_documents()
        control["identity"]["upstream_codex_rs_tree"] = control["identity"][
            "baseline_codex_rs_tree"
        ]
        with self.assertRaisesRegex(compat.CompatibilityError, "distinct Rust trees"):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_rejects_candidate_derived_expectations(self) -> None:
        root, control, ledger = self.expanded_documents()
        candidate = compat.git_output(root, ["rev-parse", "HEAD"])
        control["identity"]["upstream_commit"] = candidate
        control["identity"]["expectations_constructed_from_commit"] = candidate
        with self.assertRaisesRegex(compat.CompatibilityError, "candidate-derived"):
            compat.validate_expanded_control(
                control,
                ledger,
                root,
                control["identity"]["baseline_commit"],
                candidate,
                candidate,
                now=CONTROL_NOW,
            )

    def test_expanded_control_rejects_stale_review(self) -> None:
        root, control, ledger = self.expanded_documents()
        with self.assertRaisesRegex(compat.CompatibilityError, "ledger is stale"):
            compat.validate_expanded_control(
                control,
                ledger,
                root,
                control["identity"]["baseline_commit"],
                control["identity"]["upstream_commit"],
                compat.git_output(root, ["rev-parse", "HEAD"]),
                now=datetime.datetime(2026, 10, 1, tzinfo=datetime.timezone.utc),
            )

    def test_expanded_control_rejects_future_review(self) -> None:
        root, control, ledger = self.expanded_documents()
        with self.assertRaisesRegex(compat.CompatibilityError, "in the future"):
            compat.validate_expanded_control(
                control,
                ledger,
                root,
                control["identity"]["baseline_commit"],
                control["identity"]["upstream_commit"],
                compat.git_output(root, ["rev-parse", "HEAD"]),
                now=datetime.datetime(2026, 8, 29, tzinfo=datetime.timezone.utc),
            )

    def test_expanded_control_rejects_unbounded_ledger_age(self) -> None:
        root, control, ledger = self.expanded_documents()
        ledger["max_age_days"] = compat.MAX_LEDGER_AGE_DAYS + 1
        with self.assertRaisesRegex(compat.CompatibilityError, "between 1 and 30"):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_requires_exact_test_filter(self) -> None:
        root, control, ledger = self.expanded_documents()
        control["cases"][0]["test_filter"] = "authorization_header"
        with self.assertRaisesRegex(compat.CompatibilityError, "exact function"):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_rejects_unobserved_drift(self) -> None:
        root, control, ledger = self.expanded_documents()
        ledger["entries"].append(
            {
                "identity": "candidate",
                "case_id": control["cases"][0]["id"],
                "disposition": "accepted-intentional",
                "upstream_source_sha256": control["cases"][0]["upstream_source_sha256"],
                "observed_source_sha256": "0" * 64,
                "rationale": "fixture",
            }
        )
        with self.assertRaisesRegex(compat.CompatibilityError, "unobserved entries"):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_rejects_duplicate_drift_entries(self) -> None:
        root, control, ledger = self.expanded_documents()
        entry = {
            "identity": "candidate",
            "case_id": control["cases"][0]["id"],
            "disposition": "accepted-intentional",
            "upstream_source_sha256": control["cases"][0]["upstream_source_sha256"],
            "observed_source_sha256": "1" * 64,
            "rationale": "fixture",
        }
        ledger["entries"] = [entry, copy.deepcopy(entry)]
        with self.assertRaisesRegex(compat.CompatibilityError, "duplicate"):
            self.validate_documents(root, control, ledger)

    def test_expanded_control_rejects_unaccepted_drift_entries(self) -> None:
        root, control, ledger = self.expanded_documents()
        ledger["entries"] = [
            {
                "identity": "candidate",
                "case_id": control["cases"][0]["id"],
                "disposition": "rejected",
            }
        ]
        with self.assertRaisesRegex(compat.CompatibilityError, "unaccepted drift"):
            self.validate_documents(root, control, ledger)

    def synthetic_candidate_drift(self, root, control, candidate_digest):
        candidate = compat.git_output(root, ["rev-parse", "HEAD"])
        target = control["cases"][0]
        original = compat.expanded_source_digest

        def digest(repo_root, commit, path, function):
            if function == target["function"] and commit == candidate:
                return candidate_digest
            return original(repo_root, commit, path, function)

        return mock.patch.object(compat, "expanded_source_digest", side_effect=digest)

    def test_expanded_control_rejects_unknown_observed_drift(self) -> None:
        root, control, ledger = self.expanded_documents()
        with self.synthetic_candidate_drift(root, control, "1" * 64):
            with self.assertRaisesRegex(
                compat.CompatibilityError, "unknown candidate drift"
            ):
                self.validate_documents(root, control, ledger)

    def test_expanded_control_rejects_stale_observed_drift_entry(self) -> None:
        root, control, ledger = self.expanded_documents()
        case = control["cases"][0]
        ledger["entries"] = [
            {
                "identity": "candidate",
                "case_id": case["id"],
                "disposition": "accepted-intentional",
                "upstream_source_sha256": case["upstream_source_sha256"],
                "observed_source_sha256": "2" * 64,
                "rationale": "fixture",
            }
        ]
        with self.synthetic_candidate_drift(root, control, "1" * 64):
            with self.assertRaisesRegex(
                compat.CompatibilityError, "stale candidate drift"
            ):
                self.validate_documents(root, control, ledger)

    def test_expanded_control_accepts_exact_reviewed_observed_drift(self) -> None:
        root, control, ledger = self.expanded_documents()
        case = control["cases"][0]
        ledger["entries"] = [
            {
                "identity": "candidate",
                "case_id": case["id"],
                "disposition": "accepted-intentional",
                "upstream_source_sha256": case["upstream_source_sha256"],
                "observed_source_sha256": "1" * 64,
                "rationale": "fixture exercises exact intentional drift",
            }
        ]
        with self.synthetic_candidate_drift(root, control, "1" * 64):
            self.assertEqual(len(self.validate_documents(root, control, ledger)), 9)

    def test_protected_control_rejects_candidate_drift(self) -> None:
        root, control, _ = self.expanded_documents()
        candidate = compat.git_output(root, ["rev-parse", "HEAD"])
        target = control["protected_cases"][0]
        original = compat.expanded_source_digest

        def digest(repo_root, commit, path, function):
            if function == target["function"] and commit == candidate:
                return "3" * 64
            return original(repo_root, commit, path, function)

        with mock.patch.object(compat, "expanded_source_digest", side_effect=digest):
            with self.assertRaisesRegex(
                compat.CompatibilityError, "protected candidate drift"
            ):
                compat.validate_protected_cases(control, root, candidate)

    def test_every_expanded_case_failure_fails_acceptance(self) -> None:
        _, control, _ = self.expanded_documents()
        passing = [{"id": case["id"], "passed": True} for case in control["cases"]]
        self.assertTrue(compat.expanded_results_pass(passing, passing, passing))
        for index, case in enumerate(control["cases"]):
            with self.subTest(case=case["id"]):
                failing = copy.deepcopy(passing)
                failing[index]["passed"] = False
                self.assertFalse(
                    compat.expanded_results_pass(passing, passing, failing)
                )

        protected = [
            {"id": case["id"], "passed": True} for case in control["protected_cases"]
        ]
        self.assertTrue(
            compat.expanded_results_pass(passing, passing, passing, protected)
        )
        for index, case in enumerate(control["protected_cases"]):
            with self.subTest(protected_case=case["id"]):
                failing = copy.deepcopy(protected)
                failing[index]["passed"] = False
                self.assertFalse(
                    compat.expanded_results_pass(passing, passing, passing, failing)
                )

    def test_controlled_environment_drops_ambient_secret_values(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with mock.patch.dict(os.environ, {"PF_SECRET_FIXTURE": "do-not-copy"}):
                environment = compat.controlled_environment(
                    root / "cache", root / "target", root / "tmp"
                )
            self.assertNotIn("PF_SECRET_FIXTURE", environment)


if __name__ == "__main__":
    unittest.main()
