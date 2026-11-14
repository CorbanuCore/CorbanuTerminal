"""Exercise the real wrapper/collector with synthetic Bazel, without credentials."""

import json
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest


class NamedWorkflowTest(unittest.TestCase):
    def run_wrapper(self, codes, enabled=True, corrupt=False, blocked=False, optimized=False):
        script = Path(__file__).with_name("run-bazel-ci.sh").resolve()
        with TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            logs = root / "logs"
            logs.mkdir()
            (root / "logs-opt").mkdir()
            (root / "server").mkdir()
            evidence = root / "evidence"
            evidence.mkdir()
            if blocked:
                (evidence / "events-1.json").mkdir()
            fake = root / "bazel"
            fake.write_text("#!" + sys.executable + "\n" + """
import json, os, sys
from pathlib import Path
root = Path(os.environ["FIXTURE_ROOT"])
log_root = root / ("logs-opt" if "-c" in sys.argv and sys.argv[sys.argv.index("-c") + 1] == "opt" else "logs")
if "info" in sys.argv:
    (root / "server" / "jvm.out").write_text("new server")
    print(log_root)
    sys.exit(0)
counter = root / "calls"
attempt = int(counter.read_text()) + 1 if counter.exists() else 1
counter.write_text(str(attempt))
codes = json.loads(os.environ["FIXTURE_CODES"])
code = codes[min(attempt, len(codes)) - 1]
args = dict(arg[2:].split("=", 1) for arg in sys.argv[1:] if arg.startswith("--") and "=" in arg)
if "build_event_json_file" in args:
    log = log_root / "test.log"
    status = "FAILED" if code else "ok"
    log.write_text(f"running 1 test\\ntest attempt{attempt} ... {status}\\n"
                   f"test result: {status}. {int(code == 0)} passed; {int(code != 0)} failed; 0 ignored; 0 measured\\n")
    events = [
        {"started": {"uuid": args["invocation_id"]}},
        {"buildMetadata": {"metadata": {"CORBANU_CHECKOUT_SHA": args["build_metadata"].split("=", 1)[1]}}},
        {"id": {"testResult": {"label": "//codex-rs/core:unit-tests", "configuration": {"id": "test"}}},
         "testResult": {"status": "FAILED" if code else "PASSED",
                        "testActionOutput": [{"name": "test.log", "uri": log.as_uri() if "--nobuild_event_json_file_path_conversion" in sys.argv else "bytestream://fixture/log"}]}},
    ]
    if code != 37:
        events.append({"finished": {"exitCode": {"code": code}}, "lastMessage": True})
    payload = "".join(json.dumps(e) + "\\n" for e in events)
    if os.environ.get("CORRUPT"):
        payload = "private malformed BEP"
    Path(args["build_event_json_file"]).write_text(payload)
if code == 37:
    crash = root / "server" / "jvm.out"
    crash.write_text("original crash evidence")
    print(f"Server terminated abruptly (log file: '{crash}')")
sys.exit(code)
""", encoding="utf-8")
            fake.chmod(0o755)
            env = {
                "PATH": os.pathsep.join((str(Path(sys.executable).parent), "/usr/bin", "/bin")),
                "HOME": str(root), "CODEX_BAZEL_BIN": str(fake),
                "FIXTURE_ROOT": str(root), "FIXTURE_CODES": json.dumps(codes),
                "CORBANU_TEST_NO_NATIVE_KEYRING": "1",
            }
            if corrupt:
                env["CORRUPT"] = "1"
            args = ["bash", str(script)]
            if enabled:
                args += ["--named-evidence-dir", str(evidence)]
            args += ["--", "test"]
            if optimized:
                args += ["-c", "opt"]
            args += ["--", "//codex-rs/core/..."]
            result = subprocess.run(args, cwd=script.parents[2], env=env,
                                    capture_output=True, text=True, timeout=20)
            reports = [json.loads(path.read_text()) for path in sorted(evidence.glob("report-*.json"))]
            calls = int((root / "calls").read_text()) if (root / "calls").exists() else 0
            return result, reports, calls

    def test_success_and_failure_keep_original_exit_codes(self):
        for code in (0, 3):
            with self.subTest(code=code):
                result, reports, calls = self.run_wrapper([code])
                self.assertEqual(result.returncode, code, result.stderr)
                self.assertEqual(calls, 1)
                self.assertEqual(len(reports), 1)
                self.assertEqual(reports[0]["captureStatus"], "complete")
                self.assertEqual(reports[0]["workflowOutcome"], "success" if code == 0 else "failure")

    def test_crash_retry_captures_each_invocation_before_logs_change(self):
        result, reports, calls = self.run_wrapper([37, 0])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, 2)
        self.assertIn("original crash evidence", result.stderr)
        self.assertEqual(len(reports), 2)
        self.assertNotEqual(reports[0]["invocationId"], reports[1]["invocationId"])
        self.assertEqual(reports[0]["captureStatus"], "incomplete")
        self.assertEqual(reports[1]["captureStatus"], "complete")
        self.assertEqual(reports[0]["attempts"][0]["methods"], [{"name": "attempt1", "status": "FAILED"}])
        self.assertEqual(reports[1]["attempts"][0]["methods"], [{"name": "attempt2", "status": "ok"}])

    def test_second_crash_still_fails_and_preserves_both_reports(self):
        result, reports, calls = self.run_wrapper([37, 37])
        self.assertEqual(result.returncode, 37)
        self.assertEqual(calls, 2)
        self.assertEqual(len(reports), 2)
        self.assertTrue(all(r["captureStatus"] == "incomplete" for r in reports))

    def test_capture_failure_never_replaces_test_result_or_echoes_payload(self):
        result, reports, calls = self.run_wrapper([3], corrupt=True)
        self.assertEqual(result.returncode, 3)
        self.assertEqual(calls, 1)
        self.assertEqual(reports[0]["issues"], ["malformed_bep"])
        self.assertNotIn("private malformed BEP", result.stdout + result.stderr + json.dumps(reports))

    def test_unavailable_destination_does_not_prevent_tests(self):
        result, reports, calls = self.run_wrapper([3], blocked=True)
        self.assertEqual((result.returncode, reports, calls), (3, [], 1))
        self.assertIn("tests will still run", result.stderr)

    def test_caller_compilation_mode_selects_matching_log_root(self):
        result, reports, calls = self.run_wrapper([0], optimized=True)
        self.assertEqual((result.returncode, calls), (0, 1))
        self.assertEqual(reports[0]["captureStatus"], "complete")

    def test_disabled_capture_leaves_existing_wrapper_contract(self):
        result, reports, calls = self.run_wrapper([0], enabled=False)
        self.assertEqual((result.returncode, reports, calls), (0, [], 1))

    def test_workflow_uploads_reports_not_raw_bep_or_log_roots(self):
        workflow = (Path(__file__).parents[1] / "workflows/rust-ci-windows-native-tests.yml").read_text()
        capture = workflow.split("- name: Upload named test evidence", 1)[1].split("- name: Upload Bazel execution logs", 1)[0]
        self.assertIn("/report-*.json", capture)
        self.assertNotIn("events-", capture)
        self.assertNotIn("testlogs-root-", capture)
        self.assertIn("TARGETS: ${{ inputs.targets }}", workflow)
        self.assertIn('read -r -a bazel_targets <<< "$TARGETS"', workflow)


if __name__ == "__main__":
    unittest.main()
