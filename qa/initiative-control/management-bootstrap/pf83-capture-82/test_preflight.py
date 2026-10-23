"""Local preflight controls. Every subprocess is intercepted; no staging or guest."""
import base64
import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-failclosed-73"))
import preflight
import stage

GOOD = ('Darwin arm64\n503\nagent\n26.2\n"IOPlatformUUID" = "'
        + stage.GUEST_UUID + '"\nsynthetic observation witness\n').encode()


class PreflightCapture(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=HERE)
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name)
        for name in ("bundle", "coordinator-packets", "harness"):
            (root / name).mkdir()
        self.key = root / "synthetic-key"
        self.key.write_text("synthetic, not a credential")
        self.key.chmod(0o600)
        checker = root / "synthetic-checker"
        checker.write_text("synthetic, never executed")
        checker.chmod(0o700)
        self.args = preflight.parser().parse_args([
            "--receiving", str(root), "--bundle", str(root / "bundle"),
            "--packets", str(root / "coordinator-packets"),
            "--harness", str(root / "harness"), "--key", str(self.key),
            "--owner-check", str(checker), "--owner-check-sha256", stage.sha(checker),
            "--contact-guest"])

    def run_check(self, stdout=GOOD, stderr=b"", code=0, timeout=False):
        def transport(argv, **kwargs):
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            if argv[0] == "SYNTHETIC_SSH":
                if timeout:
                    raise subprocess.TimeoutExpired(argv, 30, output=stdout, stderr=stderr)
                return subprocess.CompletedProcess(argv, code, stdout, stderr)
            self.assertEqual(Path(argv[0]).name, "synthetic-checker")
            return subprocess.CompletedProcess(argv, 0, json.dumps({
                "check": argv[2], "status": "passed", "evidence": ["synthetic only"]}), "")

        out = io.StringIO()
        with patch.object(preflight.subprocess, "run", side_effect=transport) as run, \
             patch.object(stage, "ssh_command", return_value=["SYNTHETIC_SSH"]), \
             patch.object(preflight.dispatch_guard, "verify", return_value={}) as verify, \
             patch.object(preflight.dispatch_guard, "check", return_value={}) as check, \
             patch.object(preflight.prepare_payload, "selected", return_value={}) as selected, \
             contextlib.redirect_stdout(out):
            result = preflight.run(self.args)
        return result, json.loads(out.getvalue()), run, (verify, check, selected)

    def test_exact_observation_retained_on_success(self):
        code, report, _, _ = self.run_check(stderr=b"synthetic stderr witness")
        self.assertEqual(code, 0)
        actual = report["guest_identity_observation"]
        self.assertEqual(base64.b64decode(actual["stdout_base64"]), GOOD)
        self.assertEqual(actual["stdout"], GOOD.decode())
        self.assertEqual(base64.b64decode(actual["stderr_base64"]), b"synthetic stderr witness")
        self.assertEqual(report["checks"][-1]["detail"], actual)

    def test_each_identity_difference_fails_and_preserves_observation(self):
        for before, after in [(b"Darwin", b"Linux"), (b"arm64", b"x86_64"),
                              (b"503", b"502"), (b"agent", b"root"),
                              (b"26.2", b"26.3"),
                              (stage.GUEST_UUID.encode(), b"wrong")]:
            with self.subTest(field=before):
                wrong = GOOD.replace(before, after)
                code, report, _, _ = self.run_check(stdout=wrong)
                self.assertEqual(code, 1)
                self.assertEqual(report["checks"][-1]["status"], "failed")
                actual = report["guest_identity_observation"]
                self.assertEqual(base64.b64decode(actual["stdout_base64"]), wrong)

    def test_invalid_utf8_fails_without_losing_bytes(self):
        code, report, _, _ = self.run_check(stdout=b"\xff")
        self.assertEqual(code, 1)
        self.assertEqual(base64.b64decode(
            report["guest_identity_observation"]["stdout_base64"]), b"\xff")

    def test_ssh_failure_preserves_output(self):
        code, report, _, _ = self.run_check(stdout=b"partial", stderr=b"rejected", code=255)
        self.assertEqual(code, 1)
        self.assertEqual(report["guest_identity_observation"]["returncode"], 255)
        self.assertEqual(base64.b64decode(
            report["guest_identity_observation"]["stderr_base64"]), b"rejected")

    def test_timeout_preserves_partial_output(self):
        code, report, _, _ = self.run_check(stdout=b"partial", timeout=True)
        self.assertEqual(code, 1)
        self.assertTrue(report["guest_identity_observation"]["timed_out"])
        self.assertEqual(base64.b64decode(
            report["guest_identity_observation"]["stdout_base64"]), b"partial")

    def test_missing_receiving_refuses_all_artifact_checks_and_contact(self):
        self.args = preflight.parser().parse_args([])
        code, report, run, artifact_checks = self.run_check()
        self.assertEqual(code, 1)
        self.assertEqual(len(report["checks"]), 12)
        run.assert_not_called()
        for check in artifact_checks:
            check.assert_not_called()
        self.assertFalse(report["guest_contact_attempted"])
        self.assertEqual(report["guest_identity_observation"], {})
        self.assertTrue(all(row["status"] == "failed" for row in report["checks"][:4]))

    def test_wrong_receiving_bundle_refuses_artifact_checks_and_contact(self):
        self.args.bundle = preflight.PRIOR / "artifacts/handoff-root"
        code, report, run, artifact_checks = self.run_check()
        self.assertEqual(code, 1)
        self.assertFalse(report["guest_contact_attempted"])
        for check in artifact_checks:
            check.assert_not_called()
        self.assertFalse(any(call.args[0][0] == "SYNTHETIC_SSH" for call in run.call_args_list))


if __name__ == "__main__":
    unittest.main(verbosity=2)
