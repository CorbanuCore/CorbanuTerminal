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

    def test_failed_probe_names_the_assertion_with_redacted_values(self) -> None:
        probe = canary.Probe(
            probe_id="fixture",
            package="fixture",
            cargo_args=("--lib", "credential"),
            expected_tests=("first_case", "second_case"),
            source_paths=("fixture.rs",),
            covers=("surface",),
        )
        failed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=101,
            stdout=(
                "test module::first_case ... ok\n"
                "test module::second_case ... FAILED\n"
                "test module::third_case ... FAILED\n"
                "failures:\n    module::second_case\n"
                "test result: FAILED. 1 passed; 1 failed; 0 ignored\n"
            ),
            stderr=(
                "thread 'module::second_case' (4242) panicked at "
                "net/src/fixture_tests.rs:822:5:\n"
                "assertion failed: `(left == right)`\n\n"
                "\x1b[1mDiff\x1b[0m \x1b[31m< left\x1b[0m / \x1b[32mright >\x1b[0m :\n"
                "\x1b[31m<None\x1b[0m\n"
                '\x1b[32m>Some("ghp_pf27SyntheticCanary000000000000")\x1b[0m\n'
                "observed key c2VjcmV0LWtleS1tYXRlcmlhbC0xMjM0NTY3OA==\n"
                "test fixture state: api_key=plainlowercasevalue, token: abc\n"
                "while running pf_27_s04_slowly_exiting_broker_is_detected\n"
                "note: run with `RUST_BACKTRACE=1` to display a backtrace\n"
                "thread 'module::third_case' (7) panicked at "
                "C:\\a\\deps\\lib.rs:3:9:\r\n"
                "\x1b]8;;https://example.invalid\x07explicit panic\r\n"
            ),
        )
        with self.assertRaises(canary.QualificationError) as raised:
            canary.validate_probe_output(probe, failed)
        message = str(raised.exception)
        self.assertIn(
            "probe fixture failed: cargo exited 101; failed tests: 2", message
        )
        self.assertIn("FAILED module::second_case", message)
        self.assertIn("panicked at net/src/fixture_tests.rs:822:5", message)
        self.assertIn("| assertion failed: `(left == right)`", message)
        self.assertIn("| <None", message)
        self.assertIn('| >Some("[REDACTED]")', message)
        self.assertIn("| observed key [REDACTED]", message)
        self.assertIn(
            "| test fixture state: api_key=[REDACTED], token: [REDACTED]", message
        )
        self.assertIn(
            "| while running pf_27_s04_slowly_exiting_broker_is_detected", message
        )
        self.assertIn(
            "FAILED module::third_case\n    panicked at C:\\a\\deps\\lib.rs:3:9",
            message,
        )
        self.assertIn("| explicit panic", message)
        self.assertNotIn("plainlowercasevalue", message)
        self.assertNotIn("Synthetic", message)
        self.assertNotIn("c2VjcmV0", message)
        self.assertNotIn("\x1b", message)
        self.assertNotIn("RUST_BACKTRACE", message)

    def test_failed_probe_without_test_failures_reports_output_tail(self) -> None:
        probe = canary.Probe(
            probe_id="fixture",
            package="fixture",
            cargo_args=("--lib", "credential"),
            expected_tests=("first_case",),
            source_paths=("fixture.rs",),
            covers=("surface",),
        )
        crashed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=101,
            stdout="",
            stderr="error[E0425]: cannot find value `x` in this scope\n",
        )
        with self.assertRaisesRegex(
            canary.QualificationError,
            r"failed tests: 0\n  no failed test reported; output tail:\n"
            r"    \| error\[E0425\]: cannot find value `x`",
        ):
            canary.validate_probe_output(probe, crashed)

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
