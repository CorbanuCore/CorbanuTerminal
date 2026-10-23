import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

from coordinator import Rejected
from integration import Integrator, git, write_record


class IntegrationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        git(self.repo, "init", "-b", "integration")
        git(self.repo, "config", "user.name", "Integration test")
        git(self.repo, "config", "user.email", "fixture@example.invalid")
        git(self.repo, "config", "commit.gpgSign", "false")
        (self.repo / "value.txt").write_text("initial\n")
        git(self.repo, "add", "value.txt")
        git(self.repo, "commit", "-m", "base")
        self.base = git(self.repo, "rev-parse", "HEAD")
        git(self.repo, "checkout", "-b", "worker")
        (self.repo / "value.txt").write_text("worker\n")
        git(self.repo, "commit", "-am", "worker")
        self.source = git(self.repo, "rev-parse", "HEAD")
        git(self.repo, "checkout", "integration")
        self.runner = Integrator(self.repo, self.root / "receipts")
        self.assignment = {"id": "merge-1", "branch": "integration", "base": self.base,
                           "source": self.source, "scope": ["value.txt"],
                           "approval": {"review": "fixture", "functional": "internal-only fixture"},
                           "tests": [{"argv": [sys.executable, "-c",
                                              "from pathlib import Path; assert Path('value.txt').read_text() == 'worker\\n'"],
                                      "timeout_seconds": 5}]}

    def tearDown(self):
        self.tmp.cleanup()

    def test_real_merge_receiving_check_and_duplicate_prevention(self):
        receipt = self.runner.merge(self.assignment)
        self.assertEqual("verified", receipt["status"])
        self.assertEqual(0, receipt["tests"][0]["exit_code"])
        self.assertNotEqual(self.base, receipt["receiving_commit"])
        self.assertEqual(2, len(git(self.repo, "rev-list", "--parents", "-n", "1", "HEAD").split()) - 1)
        self.assertFalse(self.runner.marker.exists())
        self.assertEqual("verified", json.loads((self.root / "receipts/merge-1.json").read_text())["status"])
        with self.assertRaisesRegex(Rejected, "already attempted"):
            self.runner.merge(self.assignment)

    def test_other_writer_denied_even_with_separate_receipts(self):
        second = Integrator(self.repo, self.root / "second-receipts")
        with self.runner.lock():
            with self.assertRaisesRegex(Rejected, "writer already active"):
                second.merge(self.assignment)

    def test_dirty_or_stale_or_wrong_branch_rejected_before_mutation(self):
        (self.repo / "untracked.txt").write_text("user data")
        with self.assertRaisesRegex(Rejected, "dirty"):
            self.runner.merge(self.assignment)
        self.assertFalse(self.runner.marker.exists())
        self.assignment["base"] = self.source
        with self.assertRaisesRegex(Rejected, "stale"):
            self.runner.merge(self.assignment)
        self.assignment["branch"] = "wrong"
        with self.assertRaisesRegex(Rejected, "wrong receiving"):
            self.runner.merge(self.assignment)

    def test_scope_escape_rejected(self):
        self.assignment["scope"] = ["different.txt"]
        with self.assertRaisesRegex(Rejected, "write scope"):
            self.runner.merge(self.assignment)
        self.assertEqual(self.base, git(self.repo, "rev-parse", "HEAD"))

    def test_actual_conflict_preserved_not_reset_or_retried(self):
        (self.repo / "value.txt").write_text("receiving change\n")
        git(self.repo, "commit", "-am", "receiving divergence")
        self.assignment["base"] = git(self.repo, "rev-parse", "HEAD")
        receipt = self.runner.merge(self.assignment)
        self.assertEqual("merge_failed", receipt["status"])
        self.assertTrue(self.runner.marker.exists())
        self.assertIn("UU value.txt", git(self.repo, "status", "--porcelain"))
        with self.assertRaisesRegex(Rejected, "unreconciled"):
            self.runner.merge({**self.assignment, "id": "merge-2"})
        with self.assertRaisesRegex(Rejected, "resolve checkout"):
            self.runner.reconcile("merge-1", self.assignment["base"], {"reason": "not resolved"})

    def test_receiving_failure_keeps_gate_until_owner_reconciliation(self):
        self.assignment["tests"] = [{"argv": [sys.executable, "-c", "raise SystemExit(7)"], "timeout_seconds": 5}]
        receipt = self.runner.merge(self.assignment)
        self.assertEqual("verification_failed", receipt["status"])
        self.assertEqual(7, receipt["tests"][0]["exit_code"])
        self.runner.reconcile("merge-1", receipt["receiving_commit"], {"disposition": "failed test retained; assign repair next"})
        self.assertFalse(self.runner.marker.exists())
        self.assertEqual("verification_failed", json.loads((self.root / "receipts/merge-1.json").read_text())["status"])

    def test_crash_marker_survives_new_instance(self):
        write_record(self.runner.marker, {"assignment": self.assignment, "status": "started"})
        reopened = Integrator(self.repo, self.root / "receipts")
        with self.assertRaisesRegex(Rejected, "unreconciled"):
            reopened.merge(self.assignment)
        reopened.reconcile("merge-1", self.base, {"observed": "no merge started; base and clean status inspected"})
        self.assertEqual("reconciled_not_accepted", json.loads((self.root / "receipts/merge-1.reconciled.json").read_text())["status"])

    def test_timeout_terminates_owned_command(self):
        self.assignment["tests"] = [{"argv": [sys.executable, "-c", "import time; time.sleep(20)"], "timeout_seconds": 0.05}]
        receipt = self.runner.merge(self.assignment)
        self.assertEqual("verification_failed", receipt["status"])
        self.assertTrue(receipt["tests"][0]["timed_out"])
        self.assertLess(receipt["tests"][0]["elapsed_seconds"], 5)

    def test_test_mutating_receiving_tree_cannot_pass(self):
        self.assignment["tests"] = [{"argv": [sys.executable, "-c", "from pathlib import Path; Path('value.txt').write_text('mutation')"], "timeout_seconds": 5}]
        self.assertEqual("verification_failed", self.runner.merge(self.assignment)["status"])

    def test_out_of_scope_rename_into_scope_is_denied(self):
        git(self.repo, "checkout", "worker")
        git(self.repo, "mv", "value.txt", "allowed.txt")
        git(self.repo, "commit", "-m", "rename outside allocation")
        self.assignment["source"] = git(self.repo, "rev-parse", "HEAD")
        self.assignment["scope"] = ["allowed.txt"]
        git(self.repo, "checkout", "integration")
        with self.assertRaisesRegex(Rejected, "write scope"):
            self.runner.merge(self.assignment)
        self.assertTrue((self.repo / "value.txt").exists())

    def test_normal_exit_stops_lingering_test_child(self):
        ready, escaped = self.root / "child-started", self.root / "child-escaped"
        child = f"from pathlib import Path; import time; Path({str(ready)!r}).touch(); time.sleep(0.4); Path({str(escaped)!r}).touch()"
        parent = (f"import subprocess,sys,time; from pathlib import Path; "
                  f"subprocess.Popen([sys.executable,'-c',{child!r}]); "
                  f"\nwhile not Path({str(ready)!r}).exists(): time.sleep(0.005)")
        self.assignment["tests"] = [{"argv": [sys.executable, "-c", parent], "timeout_seconds": 5}]
        self.assertEqual("verified", self.runner.merge(self.assignment)["status"])
        self.assertTrue(ready.exists())
        time.sleep(0.5)
        self.assertFalse(escaped.exists())

    def test_invalid_receiving_check_rejected_before_merge(self):
        for check in [{"argv": "not argv", "timeout_seconds": 5},
                      {"argv": [sys.executable], "timeout_seconds": 7200},
                      {"argv": [sys.executable]}]:
            with self.subTest(check=check):
                self.assignment["tests"] = [check]
                with self.assertRaises(Rejected):
                    self.runner.merge(self.assignment)
                self.assertEqual(self.base, git(self.repo, "rev-parse", "HEAD"))
                self.assertFalse(self.runner.marker.exists())

    def test_reconciliation_from_another_worktree_is_denied(self):
        write_record(self.runner.marker, {"assignment": self.assignment, "status": "started"})
        other = self.root / "other"
        git(self.repo, "worktree", "add", "-b", "other", str(other), self.base)
        runner = Integrator(other, self.root / "other-receipts")
        with self.assertRaisesRegex(Rejected, "exact receiving branch"):
            runner.reconcile("merge-1", self.base, {"inspection": "wrong checkout"})
        self.assertTrue(self.runner.marker.exists())


if __name__ == "__main__":
    unittest.main()
