import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_credential_canary as canary


class SecurityCredentialCanaryTests(unittest.TestCase):
    def test_real_subprocess_canary_after_limit_is_rejected(self) -> None:
        for stream in ("stdout", "stderr"):
            with self.subTest(stream=stream):
                source = (
                    f"import sys; sys.{stream}.write('x' * {canary.MAX_CAPTURE_BYTES} "
                    "+ '\\nsk-synthetic-subprocess-canary\\n')"
                )
                with self.assertRaisesRegex(
                    canary.QualificationError, "credential-shaped material"
                ):
                    canary.run_command(
                        [sys.executable, "-c", source], cwd=Path.cwd(), env={}
                    )

    def test_run_command_scans_both_streams_before_capture_limit(self) -> None:
        probe = canary.PROBES[0]
        passing = "".join(f"test {name} ... ok\n" for name in probe.expected_tests)
        passing += f"test result: ok. {len(probe.expected_tests)} passed;\n"
        for stream in ("stdout", "stderr"):
            for offset in (-32, 0, 32):
                with self.subTest(stream=stream, offset=offset):
                    output = passing + "x" * (
                        canary.MAX_CAPTURE_BYTES + offset - len(passing)
                    )
                    output += "\nsk-synthetic-overflow-canary\n"
                    completed = subprocess.CompletedProcess(
                        ["fixture"],
                        0,
                        **{
                            stream: output,
                            ("stderr" if stream == "stdout" else "stdout"): "",
                        },
                    )
                    with mock.patch.object(
                        canary.subprocess, "run", return_value=completed
                    ):
                        with self.assertRaisesRegex(
                            canary.QualificationError, "credential-shaped material"
                        ):
                            canary.run_command(["fixture"], cwd=Path.cwd(), env={})

    def test_run_command_capture_limit_is_utf8_bytes_and_fails_closed(self) -> None:
        for stream in ("stdout", "stderr"):
            for output in (
                "x" * canary.MAX_CAPTURE_BYTES,
                "é" * (canary.MAX_CAPTURE_BYTES // 2),
            ):
                with self.subTest(stream=stream, characters=len(output)):
                    streams = {"stdout": "", "stderr": "", stream: output}
                    with mock.patch.object(
                        canary.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess(
                            ["fixture"], 0, **streams
                        ),
                    ):
                        result = canary.run_command(["fixture"], cwd=Path.cwd(), env={})
                        self.assertEqual(getattr(result, stream), output)
                    streams[stream] += "x"
                    with mock.patch.object(
                        canary.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess(
                            ["fixture"], 0, **streams
                        ),
                    ):
                        with self.assertRaisesRegex(
                            canary.QualificationError,
                            f"command {stream} exceeds the capture limit: "
                            f"{canary.MAX_CAPTURE_BYTES + 1} bytes > "
                            f"{canary.MAX_CAPTURE_BYTES} bytes",
                        ):
                            canary.run_command(["fixture"], cwd=Path.cwd(), env={})

    def test_run_command_timeout_does_not_expose_partial_output(self) -> None:
        marker = "sk-synthetic-timeout-canary"
        with mock.patch.object(
            canary.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(["fixture"], 1, output=marker),
        ):
            with self.assertRaisesRegex(
                canary.QualificationError, "capture is incomplete"
            ) as caught:
                canary.run_command(["fixture"], cwd=Path.cwd(), env={})
        self.assertNotIn(marker, str(caught.exception))

    def test_sanitized_environment_removes_credential_material(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.dict(
                os.environ,
                {
                    "PATH": "/bin",
                    "OPENAI_API_KEY": "credential-value",
                    "GITHUB_TOKEN": "credential-value",
                    "SERVICE_PASSWORD": "credential-value",
                    "SAFE_SETTING": "kept",
                },
                clear=True,
            ):
                result = canary.sanitized_environment(Path(directory))
        self.assertEqual(result["PATH"], "/bin")
        self.assertEqual(result["SAFE_SETTING"], "kept")
        self.assertEqual(result["TMPDIR"], directory)
        self.assertNotIn("OPENAI_API_KEY", result)
        self.assertNotIn("GITHUB_TOKEN", result)
        self.assertNotIn("SERVICE_PASSWORD", result)

    def test_secret_shaped_output_fails_closed(self) -> None:
        with self.assertRaisesRegex(
            canary.QualificationError, "credential-shaped material"
        ):
            canary.assert_secret_free(
                "Authorization: Bearer sk-escaped-credential-value", "stdout"
            )

    def test_validate_probe_output_requires_each_named_test(self) -> None:
        probe = canary.Probe(
            probe_id="fixture",
            package="fixture",
            cargo_args=("--lib", "credential"),
            expected_tests=("first_case", "second_case"),
            source_paths=("fixture.rs",),
            covers=("surface",),
        )
        passed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=(
                "test module::first_case ... ok\n"
                "test module::second_case ... ok\n"
                "test result: ok. 2 passed; 0 failed; 0 ignored\n"
            ),
            stderr="",
        )
        self.assertEqual(canary.validate_probe_output(probe, passed), 2)

        missing = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=(
                "test module::first_case ... ok\n"
                "test result: ok. 1 passed; 0 failed; 0 ignored\n"
            ),
            stderr="",
        )
        with self.assertRaisesRegex(
            canary.QualificationError, "did not execute expected tests"
        ):
            canary.validate_probe_output(probe, missing)

    def test_failed_probe_reports_assertion_expected_and_observed(self) -> None:
        probe = canary.Probe(
            probe_id="fixture",
            package="fixture",
            cargo_args=("--lib", "credential"),
            expected_tests=("first_case", "second_case"),
            source_paths=("fixture.rs",),
            covers=("surface",),
        )
        failed = canary.CommandResult(
            command=["cargo", "test", "-p", "fixture"],
            returncode=101,
            stdout=(
                "test module::first_case ... ok\n"
                "thread 'module::second_case' (4242) panicked at src/fixture.rs:12:5:\n"
                "assertion `left == right` failed\n"
                "  left: Some(Brokered)\n"
                " right: Some(Unavailable)\n"
                "note: run with `RUST_BACKTRACE=1` to display a backtrace\n"
                "test module::second_case ... FAILED\n"
                "test result: FAILED. 1 passed; 1 failed; 0 ignored\n"
            ),
            stderr="error: test failed, to rerun pass `--lib`\n",
        )
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(canary.QualificationError) as caught:
                canary.validate_probe_output(probe, failed)
            message = str(caught.exception)
            self.assertIn("probe fixture failed", message)
            self.assertIn("exited with 101", message)
            self.assertIn("failed tests: module::second_case", message)
            self.assertIn("module::second_case at src/fixture.rs:12:5", message)
            self.assertIn("left: Some(Brokered)", message)
            self.assertIn("right: Some(Unavailable)", message)
            self.assertIn("expected tests not reported ok: second_case", message)
            diagnostics = caught.exception.diagnostics
            self.assertEqual(diagnostics["failed_tests"], ["module::second_case"])
            self.assertEqual(
                diagnostics["panics"][0]["location"], "src/fixture.rs:12:5"
            )
            report = canary.write_report(
                Path(directory), diagnostics, canary.FAILURE_REPORT_NAME
            )
            self.assertEqual(json.loads(report.read_text())["probe"], "fixture")

    def test_failure_diagnostics_redact_token_shaped_values(self) -> None:
        probe = canary.PROBES[0]
        token = "ghp_" + "Ab1" * 12
        failed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=101,
            stdout=(
                "thread 't' panicked at src/x.rs:1:1:\n"
                f'  left: "Bearer {token}"\n'
                f' right: "x-api-key: {token}"\n'
                "test t ... FAILED\n"
            ),
            stderr="",
        )
        message, diagnostics = canary.describe_probe_failure(probe, failed, [])
        self.assertNotIn(token, message)
        self.assertNotIn(token, json.dumps(diagnostics))
        self.assertIn("Bearer ***", message)

    def test_crashed_probe_reports_process_error_without_test_verdict(self) -> None:
        probe = canary.PROBES[0]
        crashed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=101,
            stdout="running 3 tests\n",
            stderr=(
                "error: test failed, to rerun pass `--lib`\n"
                "Caused by:\n  process didn't exit successfully: `deps/x` "
                "(signal: 11, SIGSEGV: invalid memory reference)\n"
            ),
        )
        message, diagnostics = canary.describe_probe_failure(probe, crashed, [])
        self.assertIn("no test failure was reported", message)
        self.assertIn("SIGSEGV", message)
        self.assertTrue(diagnostics["process_errors"])

    def test_pretty_assertions_diff_is_reported_after_its_blank_line(self) -> None:
        probe = canary.PROBES[0]
        failed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=101,
            stdout=(
                "running 2 tests\n"
                "thread 'm::t' panicked at src/x.rs:7:9:\n"
                "assertion failed: `(left == right)`\n"
                "\n"
                "\x1b[1mDiff\x1b[0m \x1b[31m< left\x1b[0m / \x1b[32mright >\x1b[0m :\n"
                "\x1b[31m<Some(Brokered)\x1b[0m\n"
                "\x1b[32m>Some(IsolatedBrokerUnavailable)\x1b[0m\n"
                "\n"
                "note: run with `RUST_BACKTRACE=1` environment variable\n"
                "test m::t ... FAILED\n"
            ),
            stderr="",
        )
        message, diagnostics = canary.describe_probe_failure(probe, failed, [])
        self.assertIn("<Some(Brokered)", message)
        self.assertIn(">Some(IsolatedBrokerUnavailable)", message)
        self.assertNotIn("\x1b", message)
        self.assertEqual(diagnostics["tests_started"], 2)
        self.assertEqual(diagnostics["tests_reported"], 1)

    def test_secret_scan_sees_through_colour_codes_and_underscore_prefixes(
        self,
    ) -> None:
        for value in (
            "sk-\x1b[31mAbCdEfGh12345678\x1b[0m",
            "ghp_AbCdEfGh12345678",
            "github_pat_AbCdEfGh12345678",
        ):
            with self.subTest(value=value):
                with self.assertRaisesRegex(
                    canary.QualificationError, "credential-shaped material"
                ) as caught:
                    canary.assert_secret_free(value, "stdout")
                self.assertNotIn("AbCdEfGh", str(caught.exception))

    def test_redaction_keeps_names_and_paths_readable(self) -> None:
        self.assertEqual(
            canary.redact('{"api_key": "AbCdEf123", "token":"x9"}'),
            '{"api_key": *** "token":***',
        )
        readable = (
            "credential_broker::isolated::tests::"
            "pf_27_s04_pf_27_s01_externally_terminated_broker_is_detected_and_cleaned_up "
            "/var/tmp/corbanu-credential-canary-k2j3h4x9/cbk-a1b2c3/d.sock"
        )
        self.assertEqual(canary.redact(readable), readable)
        self.assertEqual(canary.redact("key " + "a1" * 20), "key ***")

    def test_timeout_reports_a_secret_free_output_tail(self) -> None:
        with mock.patch.object(
            canary.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(
                ["cargo"], 900, output=b"running 3 tests\ntest a ... ok\n"
            ),
        ):
            with self.assertRaisesRegex(
                canary.QualificationError, r"(?s)last output:.*test a \.\.\. ok"
            ):
                canary.run_command(["cargo"], cwd=Path.cwd(), env={})

    def test_timeout_names_the_command(self) -> None:
        with mock.patch.object(
            canary.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(["cargo"], 900),
        ):
            with self.assertRaisesRegex(
                canary.QualificationError, r"after 900s.*cargo test -p fixture"
            ):
                canary.run_command(
                    ["cargo", "test", "-p", "fixture"], cwd=Path.cwd(), env={}
                )

    def test_parse_canary_result_requires_exact_surface_and_use_counts(self) -> None:
        payload = {
            "canary_sha256": "a" * 64,
            "outgoing_request_count": 1,
            "raw_secret_observations": 1,
            "scanned_surfaces": sorted(canary.REQUIRED_CANARY_SURFACES),
        }
        result = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=f"{canary.CANARY_SENTINEL}{json.dumps(payload)}\n",
            stderr="",
        )
        self.assertEqual(canary.parse_canary_result([result]), payload)

        payload["outgoing_request_count"] = 2
        result = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=f"{canary.CANARY_SENTINEL}{json.dumps(payload)}\n",
            stderr="",
        )
        with self.assertRaisesRegex(
            canary.QualificationError, "exactly one outgoing request"
        ):
            canary.parse_canary_result([result])

    def test_build_candidate_rejects_non_workspace_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            arbitrary = root / "arbitrary"
            arbitrary.write_text("fixture", encoding="utf-8")
            with self.assertRaisesRegex(
                canary.QualificationError, "must be the workspace binary"
            ):
                canary.build_candidate(root, arbitrary, {})

    def test_write_report_is_atomic_and_secret_free(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            destination = canary.write_report(
                Path(directory), {"status": "passed", "digest": "a" * 64}
            )
            self.assertEqual(
                json.loads(destination.read_text(encoding="utf-8"))["status"],
                "passed",
            )
            with self.assertRaisesRegex(
                canary.QualificationError, "credential-shaped material"
            ):
                canary.write_report(
                    Path(directory),
                    {"status": "failed", "output": "Bearer sk-leaked-test-secret"},
                )

    def test_qualification_identifies_final_artifact_after_probes(self) -> None:
        for final_build_fails in (False, True):
            with self.subTest(final_build_fails=final_build_fails):
                calls = mock.Mock()
                first = canary.CommandResult(["initial-build"], 0, "", "")
                final = canary.CommandResult(["final-build"], 0, "", "")
                calls.build.side_effect = [
                    first,
                    canary.QualificationError("final build failed")
                    if final_build_fails
                    else final,
                ]
                calls.identity.side_effect = [
                    ({"sha256": "initial"}, first),
                    ({"sha256": "final"}, final),
                ]
                calls.probe.return_value = first
                calls.validate.return_value = 1
                calls.sources.return_value = []
                calls.canary.return_value = {}
                calls.report.return_value = Path("report.json")
                with (
                    mock.patch.multiple(
                        canary,
                        PROBES=(canary.PROBES[0],),
                        build_candidate=calls.build,
                        candidate_identity=calls.identity,
                        run_command=calls.probe,
                        validate_probe_output=calls.validate,
                        source_evidence=calls.sources,
                        parse_canary_result=calls.canary,
                        write_report=calls.report,
                    ),
                    mock.patch.object(canary, "git_output", side_effect=["a" * 40, ""]),
                    mock.patch.object(canary, "sanitized_environment", return_value={}),
                ):
                    arguments = (Path.cwd(), Path("corbanu"), Path("evidence"))
                    if final_build_fails:
                        with self.assertRaisesRegex(
                            canary.QualificationError, "final build failed"
                        ):
                            canary.run_qualification(*arguments)
                        calls.report.assert_not_called()
                    else:
                        self.assertEqual(
                            canary.run_qualification(*arguments),
                            (True, Path("report.json")),
                        )
                        report = calls.report.call_args.args[1]
                        self.assertEqual(report["candidate"], {"sha256": "final"})
                        self.assertEqual(
                            report["candidate_build_command"], final.as_json()
                        )
                        self.assertEqual(
                            report["candidate_pre_probe_build_command"], first.as_json()
                        )
                expected = [
                    "build",
                    "identity",
                    "probe",
                    "validate",
                    "sources",
                    "canary",
                    "build",
                ]
                if not final_build_fails:
                    expected += ["identity", "report"]
                self.assertEqual([call[0] for call in calls.mock_calls], expected)


if __name__ == "__main__":
    unittest.main()
