"""Local round-77 controls; synthetic owner/SSH responses are not admission."""
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
GUARDS = HERE.parent / "pf83-failclosed-73"
sys.path.insert(0, str(GUARDS))
import dispatch_guard
import preflight
import stage
from test_guards import Guards

PRIOR = HERE.parent / "pf83-handoff-70"
OBSERVATIONS = {}


class Accuracy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory(dir=HERE)
        cls.root = Path(cls.tmp.name)
        cls.harness = cls.root / "harness"
        cls.harness.mkdir()
        expected = json.loads((PRIOR / "packet-rebinding.json").read_text())["binding"]
        (cls.harness / "fixtures.py").write_text("CANDIDATE = " + repr(expected) + "\n")
        # Local APFS clone: no guest staging or package execution.
        subprocess.run(["/bin/cp", "-cpR", str(PRIOR / "artifacts/handoff-root"),
                        str(cls.root / "bundle")], check=True)
        subprocess.run(["/bin/cp", "-cpR", str(PRIOR / "artifacts/rebound"),
                        str(cls.root / "coordinator-packets")], check=True)
        cls.key = cls.root / "synthetic-key"
        cls.key.write_text("SYNTHETIC; NOT A PRIVATE KEY\n")
        cls.key.chmod(0o600)
        cls.checker = cls.root / "synthetic-owner-check"
        cls.checker.write_text("#!" + sys.executable + "\nimport json, sys\n"
                              "print(json.dumps(dict(check=sys.argv[2], status='passed', "
                              "evidence=['SYNTHETIC CONTROL ONLY'])))\n")
        cls.checker.chmod(0o700)

    @classmethod
    def tearDownClass(cls):
        # Clone is read-only by attestation; make only our disposable copy removable.
        for path in (cls.root / "bundle").rglob("*"):
            path.chmod(0o700 if path.is_dir() else 0o600)
        cls.tmp.cleanup()

    def args(self):
        return preflight.parser().parse_args([
            "--harness", str(self.harness), "--bundle", str(self.root / "bundle"),
            "--packets", str(self.root / "coordinator-packets"), "--receiving", str(self.root),
            "--key", str(self.key), "--owner-check", str(self.checker),
            "--owner-check-sha256", stage.sha(self.checker), "--contact-guest"])

    def run_preflight(self, args, identity=None):
        calls = []
        real_run = subprocess.run

        def transport(argv, **kwargs):
            if argv[0] != "/usr/bin/ssh":
                return real_run(argv, **kwargs)
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            self.assertEqual(argv[-1], stage.IDENTITY)
            calls.append(argv[-1])
            return subprocess.CompletedProcess(argv, 0, identity.encode())

        out = io.StringIO()
        with patch.object(preflight.subprocess, "run", side_effect=transport):
            with contextlib.redirect_stdout(out):
                code = preflight.run(args)
        return code, json.loads(out.getvalue()), calls

    def test_wrapper_preserves_real_process_exit_codes(self):
        rows = []
        for expected in (0, 7, 23):
            result = subprocess.run([
                sys.executable, "-B", str(GUARDS / "dispatch_guard.py"),
                "--harness", str(self.harness), "--bundle", str(self.root / "bundle"),
                "--packets", str(self.root / "coordinator-packets"), "--",
                sys.executable, "-I", "-B", "-c", "raise SystemExit(" + str(expected) + ")"],
                capture_output=True, text=True,
                env={"PATH": "/usr/bin:/bin", "HOME": str(self.root)})
            self.assertEqual(result.returncode, expected)
            self.assertEqual(result.stderr, "")
            self.assertEqual(json.loads(result.stdout)["checked_packets"], 288)
            rows.append(dict(child_exit=expected, wrapper_exit=result.returncode,
                             checked_packets=288, stderr=result.stderr))
        OBSERVATIONS["real_exit_code_demonstration"] = rows

    def test_missing_inputs_reports_every_precondition_without_contact(self):
        code, report, calls = self.run_preflight(preflight.parser().parse_args([]))
        self.assertEqual(code, 1)
        self.assertEqual(len(report["checks"]), 12)
        self.assertEqual(calls, [])
        self.assertFalse(report["passed"])
        OBSERVATIONS["missing_inputs"] = report

    def test_all_checks_with_synthetic_owner_and_ssh(self):
        good = 'Darwin arm64\n503\nagent\n26.2\n"IOPlatformUUID" = "' + stage.GUEST_UUID + '"\n'
        code, report, calls = self.run_preflight(self.args(), good)
        self.assertEqual(code, 0)
        self.assertEqual(len(calls), 1)
        self.assertTrue(all(row["status"] == "passed" for row in report["checks"]))
        OBSERVATIONS["synthetic_positive_preflight"] = report

    def test_wrong_uuid_fails_preflight(self):
        wrong = 'Darwin arm64\n503\nagent\n26.2\n"IOPlatformUUID" = "wrong"\n'
        code, report, calls = self.run_preflight(self.args(), wrong)
        self.assertEqual(code, 1)
        self.assertEqual(len(calls), 1)
        self.assertIn("UUID mismatch", report["checks"][-1]["detail"])
        OBSERVATIONS["synthetic_wrong_uuid"] = report

    def test_unmet_owner_checker_blocks_contact(self):
        args = self.args()
        args.owner_check_sha256 = "0" * 64
        code, report, calls = self.run_preflight(args)
        self.assertEqual(code, 1)
        self.assertEqual(calls, [])
        for row in report["checks"]:
            if row["name"] in preflight.OWNER_CHECKS:
                self.assertEqual(row["status"], "failed")
        OBSERVATIONS["wrong_owner_checker_digest"] = report


if __name__ == "__main__":
    suite = unittest.TestSuite([
        unittest.defaultTestLoader.loadTestsFromTestCase(Guards),
        unittest.defaultTestLoader.loadTestsFromTestCase(Accuracy)])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    OBSERVATIONS["summary"] = dict(run=result.testsRun, failures=len(result.failures),
                                    errors=len(result.errors), guest_contact=False,
                                    functional_cases_executed=0)
    with (HERE / "controls.json").open("x") as stream:
        json.dump(OBSERVATIONS, stream, indent=2, sort_keys=True)
        stream.write("\n")
    raise SystemExit(not result.wasSuccessful())
