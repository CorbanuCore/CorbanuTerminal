#!/usr/bin/env python3

import hashlib
import json
import os
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import collect_bazel_test_results as collector


SANDBOX_ENV = {
    name: os.environ[name]
    for name in (
        "CODEX_SANDBOX",
        "CODEX_SANDBOX_NETWORK_DISABLED",
        "CORBANU_TEST_NO_NATIVE_KEYRING",
    )
    if name in os.environ
}


class CollectBazelTestResultsTest(unittest.TestCase):
    invocation_id = "0b4068f6-1249-4d87-b832-110f7e61741b"
    checkout_sha = "55ffd0277cba12bbe90dfa687a086809d14c4c10"
    target = "//codex-rs/core:core_test"

    def setUp(self) -> None:
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.testlogs = self.root / "testlogs"
        self.testlogs.mkdir()
        self.bep = self.root / "current-invocation.jsonl"

    def log(
        self, text: str, relative: str = "core/test.log", *, framed: bool = True
    ) -> Path:
        if framed and not text.startswith("running "):
            statuses = [
                line.rsplit(" ... ", 1)[1].split(",", 1)[0]
                for line in text.splitlines()
                if line.startswith("test ") and " ... " in line
            ]
            passed, failed, ignored = [
                statuses.count(status) for status in ("ok", "FAILED", "ignored")
            ]
            outcome = "FAILED" if failed else "ok"
            text = (
                f"running {len(statuses)} tests\n{text}"
                f"test result: {outcome}. {passed} passed; {failed} failed; "
                f"{ignored} ignored; 0 measured; 0 filtered out; finished in 0.01s\n"
            )
        path = self.testlogs / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def result(
        self,
        log: Path | str | None,
        *,
        status: str = "PASSED",
        configuration: str = "windows-fastbuild",
        run: int = 1,
        shard: int = 1,
        attempt: int = 1,
    ) -> dict:
        outputs = []
        if log is not None:
            uri = log.as_uri() if isinstance(log, Path) else log
            outputs.append({"name": "test.log", "uri": uri})
        return {
            "id": {
                "testResult": {
                    "label": self.target,
                    "configuration": {"id": configuration},
                    "run": run,
                    "shard": shard,
                    "attempt": attempt,
                }
            },
            "testResult": {"status": status, "testActionOutput": outputs},
        }

    def write_bep(
        self,
        results: list[dict],
        *,
        invocation_id: str | None = None,
        finish: bool = True,
        started: bool = True,
    ) -> None:
        events = []
        if started:
            events.append(
                {
                    "id": {"started": {}},
                    "started": {"uuid": invocation_id or self.invocation_id},
                }
            )
        events.extend(results)
        if finish:
            events.append(
                {
                    "id": {"buildFinished": {}},
                    "finished": {"exitCode": {"name": "SUCCESS", "code": 0}},
                    "lastMessage": True,
                }
            )
        self.bep.write_text(
            "".join(json.dumps(event) + "\n" for event in events),
            encoding="utf-8",
        )

    def collect(self, workflow_outcome: str = "unknown") -> dict:
        return collector.collect(
            self.bep,
            self.testlogs,
            self.invocation_id,
            self.checkout_sha,
            workflow_outcome,
        )

    def assert_incomplete(self, report: dict) -> None:
        self.assertEqual(report["captureStatus"], "incomplete")
        self.assertTrue(report["issues"])
        for issue in report["issues"]:
            self.assertRegex(issue, r"^[a-z][a-z0-9_:-]*$")

    def assert_no_methods(self, report: dict) -> None:
        self.assertTrue(all(not item["methods"] for item in report["attempts"]))

    def assert_private_text_absent(self, report: dict, *private: str) -> None:
        encoded = json.dumps(report)
        for value in (str(self.root), *private):
            self.assertNotIn(value, encoded)

    def assert_limit_reported(self, report: dict) -> None:
        self.assert_incomplete(report)
        categories = list(report["issues"])
        for attempt in report["attempts"]:
            categories.append(attempt["logStatus"])
            categories.extend(attempt.get("issues", []))
        self.assertTrue(any("limit" in category for category in categories))

    def run_cli(
        self, output: Path, outcome: str = "failure", *, root_file: Path | None = None
    ) -> subprocess.CompletedProcess:
        root_argument = (
            ["--testlogs-root-file", str(root_file)]
            if root_file is not None
            else ["--testlogs-root", str(self.testlogs)]
        )
        return subprocess.run(
            [
                sys.executable,
                "-B",
                str(Path(collector.__file__).resolve()),
                "--bep",
                str(self.bep),
                *root_argument,
                "--invocation-id",
                self.invocation_id,
                "--checkout-sha",
                self.checkout_sha,
                "--workflow-outcome",
                outcome,
                "--output",
                str(output),
            ],
            cwd=self.root,
            env={"PATH": os.defpath, "GITHUB_SHA": "b" * 40, **SANDBOX_ENV},
            capture_output=True,
            text=True,
            check=False,
            timeout=10,
        )

    def test_named_success_failure_and_ignored_are_separate_statuses(self) -> None:
        text = (
            "running 3 tests\n"
            "test suite::handles_empty_input ... ok\n"
            "test suite::rejects_invalid_input ... FAILED\n"
            "test suite::needs_windows ... ignored\n"
            "test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured; "
            "0 filtered out; finished in 0.02s\n"
        )
        log = self.log(text)
        self.write_bep([self.result(log, status="FAILED")])

        report = self.collect("failure")

        self.assertEqual(report["schemaVersion"], 1)
        self.assertEqual(report["invocationId"], self.invocation_id)
        self.assertEqual(report["checkoutSha"], self.checkout_sha)
        self.assertEqual(report["workflowOutcome"], "failure")
        self.assertEqual(report["captureStatus"], "complete")
        self.assertEqual(report["issues"], [])
        self.assertEqual(len(report["attempts"]), 1)
        row = report["attempts"][0]
        self.assertEqual(row["target"], self.target)
        self.assertEqual(row["targetStatus"], "FAILED")
        self.assertEqual(row["logStatus"], "available")
        self.assertEqual(
            row["methods"],
            [
                {"name": "suite::handles_empty_input", "status": "ok"},
                {"name": "suite::rejects_invalid_input", "status": "FAILED"},
                {"name": "suite::needs_windows", "status": "ignored"},
            ],
        )
        self.assertEqual(
            row["logSha256"], hashlib.sha256(text.encode("utf-8")).hexdigest()
        )

    def test_unreferenced_stale_logs_are_not_collected(self) -> None:
        current = self.log("test suite::current ... ok\n")
        self.log("test suite::stale ... ok\n", "stale/test.log")
        self.write_bep([self.result(current)])

        report = self.collect()

        self.assertEqual(report["captureStatus"], "complete")
        self.assertEqual(
            report["attempts"][0]["methods"],
            [{"name": "suite::current", "status": "ok"}],
        )
        self.assertNotIn("suite::stale", json.dumps(report))

    def test_should_panic_row_keeps_canonical_name_and_status(self) -> None:
        # Observed in the first Windows native capture on PR1394.
        name = (
            "context_manager::history::tests::"
            "normalize_adds_missing_output_for_custom_tool_call_panics_in_debug"
        )
        for status in ("ok", "FAILED", "ignored"):
            with self.subTest(status=status):
                log = self.log(f"test {name} - should panic ... {status}\n")
                self.write_bep([self.result(log)])

                report = self.collect()

                self.assertEqual(report["captureStatus"], "complete")
                self.assertEqual(report["issues"], [])
                self.assertEqual(
                    report["attempts"][0]["methods"],
                    [{"name": name, "status": status}],
                )

    def test_unsupported_libtest_annotation_is_not_a_named_pass(self) -> None:
        for annotation in (
            " - should abort",
            " - should panic: private-panic-text",
            " - should panic - should panic",
        ):
            with self.subTest(annotation=annotation):
                log = self.log(f"test suite::panics{annotation} ... ok\n")
                self.write_bep([self.result(log)])

                report = self.collect()

                self.assert_incomplete(report)
                self.assert_no_methods(report)
                self.assertIn("unrecognized_named_row", report["attempts"][0]["issues"])
                self.assert_private_text_absent(report, "private-panic-text")

    def test_should_panic_annotation_does_not_hide_duplicate_name(self) -> None:
        log = self.log(
            "test suite::panics ... FAILED\ntest suite::panics - should panic ... ok\n"
        )
        self.write_bep([self.result(log)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assertIn("duplicate_named_row", report["attempts"][0]["issues"])
        self.assertEqual(
            report["attempts"][0]["methods"],
            [
                {"name": "suite::panics", "status": "FAILED"},
                {"name": "suite::panics", "status": "ok"},
            ],
        )

    def test_should_panic_row_still_requires_complete_framing(self) -> None:
        for opening, issue in (
            ("", "unframed_named_row"),
            ("running 2 tests\n", "incomplete_libtest_batch"),
        ):
            with self.subTest(opening=opening):
                log = self.log(
                    opening + "test suite::panics - should panic ... ok\n"
                    "test result: ok. 1 passed; 0 failed; 0 ignored; "
                    "0 measured; 0 filtered out; finished in 0.01s\n",
                    framed=False,
                )
                self.write_bep([self.result(log)])

                report = self.collect()

                self.assert_incomplete(report)
                self.assertIn(issue, report["attempts"][0]["issues"])

    def test_missing_reference_does_not_fall_back_to_existing_test_log(self) -> None:
        self.log("test suite::stale ... ok\n")
        self.write_bep([self.result(None)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assertEqual(report["attempts"][0]["targetStatus"], "PASSED")
        self.assertNotEqual(report["attempts"][0]["logStatus"], "available")

    def test_referenced_missing_file_keeps_target_status_without_named_pass(
        self,
    ) -> None:
        self.write_bep([self.result(self.testlogs / "missing/test.log")])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assertEqual(report["attempts"][0]["targetStatus"], "PASSED")

    def test_remote_log_uris_are_not_downloaded(self) -> None:
        for uri in (
            "bytestream://example.invalid/blobs/private-token/123",
            "https://example.invalid/private-token/test.log",
        ):
            with self.subTest(scheme=uri.split(":", 1)[0]):
                self.write_bep([self.result(uri)])

                report = self.collect()

                self.assert_incomplete(report)
                self.assert_no_methods(report)
                self.assert_private_text_absent(report, "private-token", uri)

    def test_path_outside_root_is_rejected(self) -> None:
        outside = self.root / "private-test.log"
        outside.write_text("test private::outside ... ok\n", encoding="utf-8")
        self.write_bep([self.result(outside)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "private::outside", outside.name)

    def test_parent_traversal_uri_cannot_escape_root(self) -> None:
        outside = self.root / "private-test.log"
        outside.write_text("test private::outside ... ok\n", encoding="utf-8")
        uri = self.testlogs.as_uri() + "/%2e%2e/private-test.log"
        self.write_bep([self.result(uri)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "private::outside", uri)

    def test_symlink_file_cannot_escape_root(self) -> None:
        outside = self.root / "private-test.log"
        outside.write_text("test private::outside ... ok\n", encoding="utf-8")
        linked = self.testlogs / "test.log"
        linked.symlink_to(outside)
        self.write_bep([self.result(linked)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "private::outside")

    def test_symlink_directory_cannot_escape_root(self) -> None:
        outside = self.root / "private"
        outside.mkdir()
        (outside / "test.log").write_text(
            "test private::outside ... ok\n", encoding="utf-8"
        )
        linked = self.testlogs / "linked"
        linked.symlink_to(outside, target_is_directory=True)
        self.write_bep([self.result(linked / "test.log")])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)

    def test_nonlocal_file_uri_is_not_treated_as_local(self) -> None:
        self.write_bep([self.result("file://private-host/share/test.log")])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "private-host")

    def test_missing_bep_is_explicitly_incomplete(self) -> None:
        report = self.collect()

        self.assert_incomplete(report)
        self.assertEqual(report["attempts"], [])
        self.assert_private_text_absent(report)

    def test_malformed_bep_does_not_echo_input(self) -> None:
        self.bep.write_text('{"private_token": "secret-not-json"', encoding="utf-8")

        report = self.collect()

        self.assert_incomplete(report)
        self.assertEqual(report["attempts"], [])
        self.assert_private_text_absent(report, "private_token", "secret-not-json")

    def test_truncated_bep_is_not_complete_even_with_a_named_success(self) -> None:
        log = self.log("test suite::ran ... ok\n")
        self.write_bep([self.result(log)], finish=False)

        report = self.collect()

        self.assert_incomplete(report)

    def test_missing_started_event_cannot_bind_named_rows(self) -> None:
        log = self.log("test private::unbound ... ok\n")
        self.write_bep([self.result(log)], started=False)

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)

    def test_invocation_mismatch_cannot_bind_named_rows(self) -> None:
        log = self.log("test private::wrong_invocation ... ok\n")
        self.write_bep(
            [self.result(log)], invocation_id="4c18f230-8d5a-4f67-b5aa-128e4bc3e95b"
        )

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assertNotIn("private::wrong_invocation", json.dumps(report))

    def test_finished_marker_without_build_finished_event_is_incomplete(self) -> None:
        log = self.log("test suite::ran ... ok\n")
        self.write_bep([self.result(log)], finish=False)
        with self.bep.open("a", encoding="utf-8") as stream:
            stream.write(
                json.dumps({"id": {"buildToolLogs": {}}, "lastMessage": True}) + "\n"
            )

        report = self.collect()

        self.assert_incomplete(report)

    def test_build_finished_without_last_message_is_incomplete(self) -> None:
        log = self.log("test suite::ran ... ok\n")
        self.write_bep([self.result(log)])
        events = [json.loads(line) for line in self.bep.read_text().splitlines()]
        events[-1].pop("lastMessage")
        self.bep.write_text("".join(json.dumps(event) + "\n" for event in events))

        report = self.collect()

        self.assert_incomplete(report)

    def test_cache_provenance_is_preserved_without_claiming_fresh_execution(
        self,
    ) -> None:
        local = self.result(self.log("test suite::local ... ok\n", "local/test.log"))
        local["testResult"]["cachedLocally"] = True
        local["testResult"]["executionInfo"] = {"cachedRemotely": False}
        remote = self.result(
            self.log("test suite::remote ... ok\n", "remote/test.log"), attempt=2
        )
        remote["testResult"]["cachedLocally"] = False
        remote["testResult"]["executionInfo"] = {"cachedRemotely": True}
        self.write_bep([local, remote])

        report = self.collect()

        rows = {row["attempt"]: row for row in report["attempts"]}
        self.assertEqual(rows[1]["cache"], {"local": True, "remote": False})
        self.assertEqual(rows[2]["cache"], {"local": False, "remote": True})
        for row in rows.values():
            self.assertEqual(row["provenance"], "cached")
            self.assertNotIn("freshExecution", row)
            for method in row["methods"]:
                self.assertEqual(set(method), {"name", "status"})

    def test_missing_execution_info_keeps_remote_cache_unknown(self) -> None:
        self.write_bep([self.result(self.log("test suite::ran ... ok\n"))])

        report = self.collect()

        self.assertEqual(
            report["attempts"][0]["cache"], {"local": False, "remote": None}
        )
        self.assertEqual(report["attempts"][0]["provenance"], "unknown")

    def test_execution_metadata_does_not_establish_fresh_execution(self) -> None:
        event = self.result(self.log("test suite::ran ... ok\n"))
        event["testResult"].update(
            {
                "cachedLocally": False,
                "executionInfo": {"strategy": "local", "cachedRemotely": False},
                "testAttemptStartMillisEpoch": "1792795680000",
                "testAttemptDurationMillis": 100,
            }
        )
        self.write_bep([event])

        report = self.collect()

        row = report["attempts"][0]
        self.assertEqual(row["cache"], {"local": False, "remote": False})
        self.assertEqual(row["provenance"], "unknown")
        self.assertEqual(
            row["executionEvidence"],
            {
                "strategy": "local",
                "testAttemptStartMillisEpoch": "1792795680000",
                "testAttemptDurationMillis": "100",
            },
        )

    def test_retry_configuration_run_and_shard_identity_are_not_collapsed(self) -> None:
        cases = [
            ("windows-fastbuild", 1, 1, 1, "FAILED", "FAILED"),
            ("windows-fastbuild", 1, 1, 2, "PASSED", "ok"),
            ("windows-fastbuild", 1, 2, 1, "PASSED", "ignored"),
            ("windows-fastbuild", 2, 1, 1, "PASSED", "ok"),
            ("windows-opt", 1, 1, 1, "PASSED", "ok"),
        ]
        events = []
        for index, (
            config,
            run,
            shard,
            attempt,
            target_status,
            method_status,
        ) in enumerate(cases):
            relative = (
                "core/test_attempts/attempt_1.log"
                if index == 0
                else f"case{index}/test.log"
            )
            log = self.log(f"test suite::same_name ... {method_status}\n", relative)
            events.append(
                self.result(
                    log,
                    configuration=config,
                    run=run,
                    shard=shard,
                    attempt=attempt,
                    status=target_status,
                )
            )
        self.write_bep(events)

        report = self.collect()

        self.assertEqual(len(report["attempts"]), len(cases))
        observed = {
            (
                row["configuration"],
                row["run"],
                row["shard"],
                row["attempt"],
                row["targetStatus"],
                row["methods"][0]["status"],
            )
            for row in report["attempts"]
        }
        self.assertEqual(observed, set(cases))

    def test_proto_default_attempt_coordinates_are_zero(self) -> None:
        event = self.result(self.log("test suite::ran ... ok\n"))
        for key in ("run", "shard", "attempt"):
            event["id"]["testResult"].pop(key)
        self.write_bep([event])

        report = self.collect()

        row = report["attempts"][0]
        self.assertEqual((row["run"], row["shard"], row["attempt"]), (0, 0, 0))

    def test_duplicate_conflicting_result_identity_is_explicitly_ambiguous(
        self,
    ) -> None:
        first = self.result(
            self.log("test suite::same_name ... FAILED\n", "first/test.log"),
            status="FAILED",
        )
        second = self.result(
            self.log("test suite::same_name ... ok\n", "second/test.log")
        )
        self.write_bep([first, second])

        report = self.collect()

        self.assert_incomplete(report)

    def test_duplicate_started_events_are_ambiguous(self) -> None:
        self.write_bep(
            [
                {"id": {"started": {}}, "started": {"uuid": self.invocation_id}},
                self.result(self.log("test suite::ran ... ok\n")),
            ]
        )

        report = self.collect()

        self.assert_incomplete(report)

    def test_conflicting_named_rows_are_not_silently_replaced_by_success(self) -> None:
        log = self.log(
            "test suite::same_name ... FAILED\ntest suite::same_name ... ok\n"
        )
        self.write_bep([self.result(log)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assertNotEqual(
            report["attempts"][0]["methods"],
            [{"name": "suite::same_name", "status": "ok"}],
        )

    def test_multiple_test_log_references_are_not_silently_selected(self) -> None:
        event = self.result(self.log("test suite::first ... ok\n", "first/test.log"))
        second = self.log("test suite::second ... FAILED\n", "second/test.log")
        event["testResult"]["testActionOutput"].append(
            {"name": "test.log", "uri": second.as_uri()}
        )
        self.write_bep([event])

        report = self.collect()

        self.assert_incomplete(report)

    def test_private_stdout_environment_and_diagnostics_are_excluded(self) -> None:
        log = self.log(
            "API_TOKEN=secret-fixture-token\n"
            "thread panicked at /private/operator/file.rs: private-panic-text\n"
            "test suite::public_case ... ok\n"
        )
        event = self.result(log)
        event["testResult"]["statusDetails"] = "private-status-detail"
        event["testResult"]["executionInfo"] = {
            "strategy": "private-strategy-text",
            "hostname": "private-host",
        }
        self.write_bep(
            [
                {
                    "id": {
                        "structuredCommandLine": {"commandLineLabel": "private-command"}
                    },
                    "structuredCommandLine": {"secret": "private-command-argument"},
                },
                event,
            ]
        )

        report = self.collect()

        self.assert_private_text_absent(
            report,
            "secret-fixture-token",
            "private-panic-text",
            "private-status-detail",
            "private-strategy-text",
            "private-host",
            "private-command",
            "API_TOKEN",
        )
        self.assertEqual(
            report["attempts"][0]["methods"],
            [{"name": "suite::public_case", "status": "ok"}],
        )

    def test_malformed_named_row_is_not_a_named_pass(self) -> None:
        self.write_bep(
            [self.result(self.log("test private email@example.invalid ... ok\n"))]
        )

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "email@example.invalid")

    def test_unfinished_named_row_is_incomplete(self) -> None:
        self.write_bep(
            [self.result(self.log("test suite::unfinished ... ", framed=False))]
        )

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)

    def test_invalid_event_coordinates_are_not_coerced_into_valid_identity(
        self,
    ) -> None:
        for invalid in (-1, True, "private-value", [], None):
            with self.subTest(invalid=invalid):
                event = self.result(self.log("test suite::ran ... ok\n"))
                event["id"]["testResult"]["attempt"] = invalid
                self.write_bep([event])

                report = self.collect()

                self.assert_incomplete(report)
                self.assert_no_methods(report)
                self.assert_private_text_absent(report, "private-value")

    def test_non_object_bep_event_is_incomplete_without_a_traceback(self) -> None:
        for event in (None, [], 123, "private-scalar-event"):
            with self.subTest(event=event):
                self.write_bep([event])

                report = self.collect()

                self.assert_incomplete(report)
                self.assert_private_text_absent(report, "private-scalar-event")

    def test_finished_build_without_test_results_is_not_named_evidence(self) -> None:
        self.write_bep([])

        report = self.collect()

        self.assert_incomplete(report)
        self.assertEqual(report["attempts"], [])

    def test_duplicate_json_keys_are_not_silently_overwritten(self) -> None:
        self.write_bep([self.result(self.log("test suite::ran ... ok\n"))])
        text = self.bep.read_text(encoding="utf-8")
        self.bep.write_text(
            text.replace(
                '"status": "PASSED"', '"status": "FAILED", "status": "PASSED"'
            ),
            encoding="utf-8",
        )

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)

    def test_non_utf8_log_has_no_invented_named_rows(self) -> None:
        log = self.log("", framed=False)
        log.write_bytes(b"\xff\xfeprivate-binary-data")
        self.write_bep([self.result(log)])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "private-binary-data")

    def test_ignored_reason_is_not_retained_as_output(self) -> None:
        self.write_bep(
            [
                self.result(
                    self.log(
                        "test suite::disabled ... ignored, private-operator-reason\n"
                    )
                )
            ]
        )

        report = self.collect()

        self.assertEqual(report["captureStatus"], "complete")
        self.assertEqual(
            report["attempts"][0]["methods"],
            [{"name": "suite::disabled", "status": "ignored"}],
        )
        self.assert_private_text_absent(report, "private-operator-reason")

    def test_bep_byte_limit_is_explicit(self) -> None:
        with self.bep.open("wb") as stream:
            stream.truncate(collector.MAX_BEP_BYTES + 1)

        report = self.collect()

        self.assert_limit_reported(report)
        self.assert_no_methods(report)

    def test_log_byte_limit_is_explicit(self) -> None:
        log = self.log("", framed=False)
        with log.open("wb") as stream:
            stream.truncate(collector.MAX_LOG_BYTES + 1)
        self.write_bep([self.result(log)])

        report = self.collect()

        self.assert_limit_reported(report)
        self.assert_no_methods(report)

    def test_event_count_limit_is_explicit(self) -> None:
        self.bep.write_text("{}\n" * (collector.MAX_EVENTS + 1), encoding="utf-8")

        report = self.collect()

        self.assert_limit_reported(report)
        self.assert_no_methods(report)

    def test_attempt_count_limit_is_explicit(self) -> None:
        self.write_bep(
            [
                self.result(None, attempt=index)
                for index in range(collector.MAX_ATTEMPTS + 1)
            ]
        )

        report = self.collect()

        self.assert_limit_reported(report)
        self.assertLessEqual(len(report["attempts"]), collector.MAX_ATTEMPTS)

    def test_method_count_limit_is_explicit(self) -> None:
        text = "".join(
            f"test suite::case_{index} ... ok\n"
            for index in range(collector.MAX_METHODS + 1)
        )
        self.write_bep([self.result(self.log(text))])

        report = self.collect()

        self.assert_limit_reported(report)
        self.assertLessEqual(
            len(report["attempts"][0]["methods"]), collector.MAX_METHODS
        )

    def test_excessive_method_name_is_rejected_without_echoing_it(self) -> None:
        name = "private_" + "x" * 3000
        self.write_bep([self.result(self.log(f"test {name} ... ok\n"))])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, name)

    def test_combined_log_byte_limit_is_explicit(self) -> None:
        events = []
        count = collector.MAX_TOTAL_LOG_BYTES // collector.MAX_LOG_BYTES + 1
        for index in range(count):
            log = self.log("test suite::ran ... ok\n", f"case{index}/test.log")
            with log.open("r+b") as stream:
                stream.truncate(collector.MAX_LOG_BYTES)
            events.append(self.result(log, attempt=index))
        self.write_bep(events)

        report = self.collect()

        self.assert_limit_reported(report)
        self.assertTrue(
            any(row["logStatus"] != "available" for row in report["attempts"])
        )

    def test_combined_method_limit_is_explicit(self) -> None:
        count = collector.MAX_METHODS // 2 + 1
        text = "".join(f"test suite::case_{index} ... ok\n" for index in range(count))
        self.write_bep(
            [
                self.result(self.log(text, f"case{attempt}/test.log"), attempt=attempt)
                for attempt in (1, 2)
            ]
        )

        report = self.collect()

        self.assert_limit_reported(report)
        self.assertLessEqual(
            sum(len(row["methods"]) for row in report["attempts"]),
            collector.MAX_METHODS,
        )

    def test_malformed_uri_is_incomplete_without_echoing_private_text(self) -> None:
        self.write_bep([self.result("file://[private-invalid-authority/test.log")])

        report = self.collect()

        self.assert_incomplete(report)
        self.assert_no_methods(report)
        self.assert_private_text_absent(report, "private-invalid-authority")

    def test_cli_uses_explicit_checkout_and_preserves_failed_workflow_outcome(
        self,
    ) -> None:
        self.write_bep([self.result(self.log("test suite::ran ... ok\n"))])
        output = self.root / "sanitized.json"
        unrelated_sha = "b" * 40
        process = self.run_cli(output)

        self.assertEqual(process.returncode, 0, process.stderr)
        report = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(report["checkoutSha"], self.checkout_sha)
        self.assertNotEqual(report["checkoutSha"], unrelated_sha)
        self.assertEqual(report["workflowOutcome"], "failure")
        self.assertEqual(report["captureStatus"], "complete")
        self.assert_private_text_absent(report)

    def test_cli_missing_evidence_writes_sanitized_incomplete_report(self) -> None:
        output = self.root / "sanitized.json"

        process = self.run_cli(output)

        self.assertEqual(process.returncode, 1)
        report = json.loads(output.read_text(encoding="utf-8"))
        self.assert_incomplete(report)
        self.assertEqual(report["workflowOutcome"], "failure")
        self.assert_private_text_absent(report)
        self.assertNotIn(str(self.root), process.stdout + process.stderr)

    def test_cli_does_not_overwrite_existing_report(self) -> None:
        self.write_bep([self.result(self.log("test suite::ran ... ok\n"))])
        output = self.root / "sanitized.json"
        original = b"original-report-preserved\n"
        output.write_bytes(original)

        process = self.run_cli(output)

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(output.read_bytes(), original)
        self.assertNotIn(str(self.root), process.stdout + process.stderr)

    def test_cli_reads_explicit_root_file(self) -> None:
        self.write_bep([self.result(self.log("test suite::ran ... ok\n"))])
        root_file = self.root / "testlogs-root.txt"
        root_file.write_text(str(self.testlogs) + "\n", encoding="utf-8")
        output = self.root / "sanitized.json"

        process = self.run_cli(output, root_file=root_file)

        self.assertEqual(process.returncode, 0, process.stderr)
        report = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(report["captureStatus"], "complete")
        self.assert_private_text_absent(report)

    def test_cli_root_lookup_failure_does_not_fall_back_to_workspace_logs(self) -> None:
        self.write_bep([self.result(self.log("test suite::stale ... ok\n"))])
        for index, contents in enumerate(("", "relative/root\n", "one\ntwo\n")):
            with self.subTest(contents=contents):
                root_file = self.root / "testlogs-root.txt"
                root_file.write_text(contents, encoding="utf-8")
                output = self.root / f"sanitized-{index}.json"

                process = self.run_cli(output, root_file=root_file)

                self.assertEqual(process.returncode, 1)
                report = json.loads(output.read_text(encoding="utf-8"))
                self.assert_incomplete(report)
                self.assert_no_methods(report)
                self.assert_private_text_absent(report)

    def test_workflow_preserves_native_runner_arguments_and_exit_status(self) -> None:
        workflow = (
            Path(__file__).parents[1] / "workflows/rust-ci-windows-native-tests.yml"
        ).read_text(encoding="utf-8")
        lines = workflow.splitlines()
        step = next(
            index
            for index, line in enumerate(lines)
            if line.strip() == "id: native_tests"
        )
        start = (
            next(
                index
                for index in range(step, len(lines))
                if lines[index].strip() == "run: |"
            )
            + 1
        )
        command = []
        for line in lines[start:]:
            if line and not line.startswith(" " * 10):
                break
            command.append(line[10:])
        runner = self.root / ".github/scripts/run-bazel-ci.sh"
        runner.parent.mkdir(parents=True)
        runner.write_text(
            '#!/bin/sh\nprintf "%s\\0" "$@" > "$ARGUMENT_CAPTURE"\nexit "$FAKE_TEST_EXIT"\n',
            encoding="utf-8",
        )
        runner.chmod(0o755)
        capture = self.root / "argv.bin"
        evidence_dir = str(self.root / "evidence with spaces")
        expected = [
            "--print-failed-action-summary",
            "--print-failed-test-logs",
            "--",
            "test",
            "--build_tests_only",
            "--flaky_test_attempts=//codex-rs/login:login-unit-tests@2",
            "--test_tag_filters=-argument-comment-lint",
            "--test_verbose_timeout_warnings",
            f"--build_metadata=COMMIT_SHA={self.checkout_sha}",
            "--build_metadata=TAG_windows_native_pr=true",
        ]
        targets = [
            "--",
            "//codex-rs/tui/...",
            "//codex-rs/core/...",
            "//codex-rs/state/...",
            "//codex-rs/login/...",
        ]
        for with_dir, with_invocation in (
            (False, False),
            (True, False),
            (False, True),
            (True, True),
        ):
            for exit_status in (0, 7):
                with self.subTest(
                    directory=with_dir,
                    invocation=with_invocation,
                    exit_status=exit_status,
                ):
                    process = subprocess.run(
                        [
                            "/bin/bash",
                            "--noprofile",
                            "--norc",
                            "-eo",
                            "pipefail",
                            "-c",
                            "\n".join(command),
                        ],
                        cwd=self.root,
                        env={
                            **SANDBOX_ENV,
                            "PATH": os.defpath,
                            "ARGUMENT_CAPTURE": str(capture),
                            "FAKE_TEST_EXIT": str(exit_status),
                            "GITHUB_SHA": self.checkout_sha,
                            "NAMED_EVIDENCE_DIR": evidence_dir if with_dir else "",
                            "NAMED_EVIDENCE_INVOCATION": self.invocation_id
                            if with_invocation
                            else "",
                        },
                        capture_output=True,
                        text=True,
                        check=False,
                        timeout=10,
                    )
                    self.assertEqual(process.returncode, exit_status, process.stderr)
                    arguments = capture.read_bytes().decode("utf-8").split("\0")[:-1]
                    additions = []
                    if with_dir and with_invocation:
                        additions = [
                            f"--build_event_json_file={evidence_dir}/events.json",
                            f"--invocation_id={self.invocation_id}",
                        ]
                    wrapper_additions = (
                        [
                            "--test-results-root-file",
                            f"{evidence_dir}/testlogs-root.txt",
                        ]
                        if with_dir and with_invocation
                        else []
                    )
                    self.assertEqual(
                        arguments,
                        expected[:2]
                        + wrapper_additions
                        + expected[2:]
                        + additions
                        + targets,
                    )

    def test_wrapper_root_lookup_preserves_test_exit_and_matching_configuration(
        self,
    ) -> None:
        source = Path(__file__).with_name("run-bazel-ci.sh")
        scripts = self.root / ".github/scripts"
        scripts.mkdir(parents=True)
        wrapper = scripts / source.name
        wrapper.write_bytes(source.read_bytes())
        fake_bazel = scripts / "run_bazel_with_buildbuddy.py"
        fake_bazel.write_text(
            f"#!{sys.executable}\n"
            "import json, os, sys\n"
            "from pathlib import Path\n"
            "with Path(os.environ['ARGUMENT_CAPTURE']).open('a') as stream:\n"
            "    stream.write(json.dumps(sys.argv[1:]) + '\\n')\n"
            "if 'info' in sys.argv:\n"
            "    print(os.environ['FAKE_TESTLOGS_ROOT'])\n"
            "    raise SystemExit(int(os.environ['FAKE_INFO_EXIT']))\n"
            "raise SystemExit(int(os.environ['FAKE_TEST_EXIT']))\n",
            encoding="utf-8",
        )
        fake_bazel.chmod(0o755)
        output_root = self.root / "bazel output"
        repo_contents = self.root / "repo contents"
        repository = self.root / "repository cache"
        capture = self.root / "bazel-arguments.jsonl"
        root_file = self.root / "testlogs-root.txt"
        for keyed in (False, True):
            for test_exit in (0, 7):
                for info_exit in (0, 9):
                    with self.subTest(
                        keyed=keyed, test_exit=test_exit, info_exit=info_exit
                    ):
                        capture.write_text("", encoding="utf-8")
                        env = {
                            **SANDBOX_ENV,
                            "PATH": os.defpath,
                            "RUNNER_OS": "Windows",
                            "CODEX_BAZEL_WINDOWS_PATH": "C:/synthetic-toolchain/bin",
                            "BAZEL_OUTPUT_USER_ROOT": str(output_root),
                            "BAZEL_REPO_CONTENTS_CACHE": str(repo_contents),
                            "BAZEL_REPOSITORY_CACHE": str(repository),
                            "ARGUMENT_CAPTURE": str(capture),
                            "FAKE_TEST_EXIT": str(test_exit),
                            "FAKE_INFO_EXIT": str(info_exit),
                            "FAKE_TESTLOGS_ROOT": str(self.testlogs),
                        }
                        if keyed:
                            env["BUILDBUDDY_API_KEY"] = "synthetic-fixture-key"
                        process = subprocess.run(
                            [
                                "/bin/bash",
                                str(wrapper),
                                "--windows-msvc-host-platform",
                                "--test-results-root-file",
                                str(root_file),
                                "--",
                                "test",
                                "--build_tests_only",
                                "--test_tag_filters=-argument-comment-lint",
                                "--",
                                "//codex-rs/core/...",
                            ],
                            cwd=self.root,
                            env=env,
                            capture_output=True,
                            text=True,
                            check=False,
                            timeout=10,
                        )
                        self.assertEqual(process.returncode, test_exit, process.stderr)
                        calls = [
                            json.loads(line)
                            for line in capture.read_text().splitlines()
                        ]
                        self.assertEqual(len(calls), 2)
                        test_args, info_args = calls
                        self.assertIn("test", test_args)
                        self.assertIn("--build_tests_only", test_args)
                        self.assertEqual(test_args[-2:], ["--", "//codex-rs/core/..."])
                        self.assertEqual(
                            info_args,
                            [
                                f"--output_user_root={output_root}",
                                "--noexperimental_remote_repo_contents_cache",
                                "info",
                                *(["--config=ci-windows"] if keyed else []),
                                "--host_platform=//:local_windows_msvc",
                                f"--repo_contents_cache={repo_contents}",
                                f"--repository_cache={repository}",
                                "bazel-testlogs",
                            ],
                        )
                        self.assertEqual(
                            root_file.read_text(encoding="utf-8"),
                            str(self.testlogs) + "\n" if info_exit == 0 else "",
                        )


if __name__ == "__main__":
    unittest.main()
