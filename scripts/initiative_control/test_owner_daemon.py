"""Offline kernel evidence only; no providers, TMUX, Git, Slack or credentials."""
from contextlib import closing
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from coordinator import Coordinator, digest
import fable_launcher as f
import owner_daemon as owner
from test_coordinator import seed

ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "scripts/initiative_control/owner_daemon.py"


class OwnerDaemonTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="owner-fixture-", dir=Path(tempfile.gettempdir()).resolve())
        self.root = Path(self.tmp.name) / "state"
        self.c = Coordinator(self.root)
        self.c.initialize(*seed())
        self.c.set_enabled(True, {"fixture_only": True})
        self.config_path = self.root / "config.json"
        self.config = {"coordinator": str(self.root), "worktrees": [str(self.root)],
                       "package_digest": owner.package_digest(), "manager_enabled": True}
        f.write_json(self.config_path, self.config)
        owner.setup(self.config_path)
        self.authority = {"decision_id": "fixture-activation", "revision": 1,
                          "authority": "offline test fixture, not human activation",
                          "scope": "fixture-only", "generation": 1,
                          "config_digest": digest(self.config),
                          "package_digest": owner.package_digest()}

    def tearDown(self):
        self.tmp.cleanup()

    def sql(self, statement, args=()):
        with closing(sqlite3.connect(self.root / "owner.sqlite3")) as db, db:
            return db.execute(statement, args).fetchall()

    def arm(self):
        f.write_json(self.root / "activation.json", self.authority)
        self.sql("UPDATE meta SET requested_mode='armed',control_generation=?,"
                 "activation_decision_id=?,activation_revision=?,activation_digest=?",
                 (self.authority["generation"], self.authority["decision_id"],
                  self.authority["revision"], digest(self.authority)))

    def tick(self):
        return owner.Kernel(self.config_path).tick(owner.FixedTestAdapter())

    def child(self, code=None, *args):
        env = {"PATH": f.SAFE_PATH, "PYTHONDONTWRITEBYTECODE": "1",
               **{key: str(self.root) for key in ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")},
               "PYTHONPATH": str(CLI.parent) + os.pathsep + os.environ.get("PYTHONPATH", "")}
        argv = [sys.executable, "-B"]
        argv += ["-c", code, str(self.config_path)] if code else [str(CLI), *args]
        return subprocess.run(argv, env=env, capture_output=True, text=True, timeout=20)

    def test_setup_is_off_and_cli_default_help_import_are_read_only(self):
        before = {p.name: p.read_bytes() for p in self.root.iterdir() if p.is_file()}
        self.assertEqual("off", self.sql("SELECT requested_mode FROM meta")[0][0])
        for args in ((), ("--help",)):
            self.assertEqual(0, self.child(None, *args).returncode)
        self.assertEqual(0, self.child("import owner_daemon").returncode)
        after = {p.name: p.read_bytes() for p in self.root.iterdir() if p.is_file()}
        self.assertEqual(before, after)
        with self.assertRaisesRegex(f.LaunchError, "owner_off"):
            self.tick()
        with self.assertRaisesRegex(f.LaunchError, "owner_state_exists"):
            owner.setup(self.config_path)

    def test_cli_defaults_to_fixed_fixture_without_dynamic_import_seam(self):
        self.arm()
        result = self.child(None, "--run", "--config", str(self.config_path))
        self.assertEqual(0, result.returncode)
        self.assertTrue(json.loads(result.stdout)["fixture_only"])
        self.assertEqual(1, len(self.sql("SELECT * FROM boots")))
        with self.assertRaisesRegex(f.LaunchError, "live_adapter_unavailable"):
            owner.Kernel(self.config_path).tick(lambda request: {})
        self.assertEqual(2, self.child(None, "--run").returncode)

    def test_activation_missing_authority_wrong_scope_and_binding_refused(self):
        self.sql("UPDATE meta SET requested_mode='armed'")
        with self.assertRaises(FileNotFoundError):
            self.tick()
        for key, value in (("authority", ""), ("scope", "live"), ("revision", 0),
                           ("generation", True), ("package_digest", "wrong")):
            with self.subTest(key=key):
                old = self.authority[key]
                self.authority[key] = value
                self.arm()
                with self.assertRaises(f.LaunchError):
                    self.tick()
                self.authority[key] = old
        self.arm()
        self.sql("UPDATE meta SET activation_digest='wrong'")
        with self.assertRaisesRegex(f.LaunchError, "activation_mismatch"):
            self.tick()
        self.assertEqual([], self.sql("SELECT * FROM boots"))

    def test_missing_original_state_never_initializes(self):
        for name in ("owner.sqlite3", "coordinator.sqlite3"):
            with self.subTest(name=name):
                path = self.root / name
                saved = path.with_suffix(".saved")
                path.rename(saved)
                try:
                    with self.assertRaises(FileNotFoundError):
                        self.tick()
                    self.assertFalse(path.exists())
                finally:
                    saved.rename(path)

    def test_config_package_worktree_and_schema_drift_refused(self):
        self.arm()
        for key, value in (("package_digest", "wrong"), ("worktrees", ["/missing/owner-fixture"]),
                           ("manager_enabled", "yes")):
            with self.subTest(key=key):
                f.write_json(self.config_path, {**self.config, key: value})
                with self.assertRaises(f.LaunchError):
                    self.tick()
        f.write_json(self.config_path, self.config)
        self.sql("ALTER TABLE operations ADD COLUMN unexpected TEXT")
        with self.assertRaisesRegex(f.LaunchError, "schema_drift"):
            self.tick()
        self.assertEqual([], self.sql("SELECT * FROM boots"))

    def test_mode_link_and_journal_refusals(self):
        self.arm()
        for name in ("config.json", "owner.sqlite3", "owner-daemon.lock", "activation.json"):
            with self.subTest(name=name):
                path = self.root / name
                path.chmod(0o644)
                with self.assertRaises(f.LaunchError):
                    self.tick()
                path.chmod(0o600)
                link = self.root / "hardlink"
                os.link(path, link)
                with self.assertRaises(f.LaunchError):
                    self.tick()
                link.unlink()
                saved = self.root / "saved"
                path.rename(saved)
                path.symlink_to(saved)
                with self.assertRaises(f.LaunchError):
                    self.tick()
                path.unlink()
                saved.rename(path)
        journal = self.root / "owner.sqlite3-journal"
        journal.write_bytes(b"unrecovered")
        with self.assertRaisesRegex(f.LaunchError, "owner_recovery_required"):
            self.tick()
        self.assertEqual(b"unrecovered", journal.read_bytes())

    def test_lifetime_and_admission_locks_refuse_second_owner(self):
        self.arm()
        for name in ("owner-daemon.lock", "owner-admission.lock"):
            with self.subTest(name=name), owner.locked(self.root / name):
                result = self.child("import sys,owner_daemon as o; "
                                    "o.Kernel(sys.argv[1]).tick(o.FixedTestAdapter())")
                self.assertNotEqual(0, result.returncode)
                self.assertEqual([], self.sql("SELECT * FROM operations"))
        self.assertEqual("ACTIVE", self.tick()["state"])

    def test_one_effect_restarts_do_not_change_semantic_revision(self):
        self.arm()
        with patch.object(owner.FixedTestAdapter, "observe", side_effect=owner.fixture_receipt) as observe:
            first = self.tick()
        self.assertEqual(1, observe.call_count)
        self.assertEqual(["RECOVERING", "READY", "ACTIVE"], first["states"])
        revision = self.c.snapshot()["revision"]
        for _ in range(3):
            self.assertEqual("ACTIVE", self.tick()["state"])
        self.assertEqual(revision, self.c.snapshot()["revision"])
        self.assertEqual([("applied",)], self.sql("SELECT phase FROM operations"))
        self.assertEqual([("applied",)], self.sql("SELECT phase FROM deliveries"))
        self.assertEqual(4, len(self.sql("SELECT * FROM boots WHERE stopped_at IS NOT NULL")))
        self.assertEqual(1, len(self.sql("SELECT * FROM observations")))
        for path in self.root.rglob("*"):
            if path.is_file():
                self.assertEqual(0o600, path.stat().st_mode & 0o777)

    def test_paused_and_owned_coordinator_do_not_admit_effect(self):
        self.arm()
        self.c.set_enabled(False, {"fixture_only": True})
        self.assertEqual("READY", self.tick()["state"])
        self.assertEqual([], self.sql("SELECT * FROM operations"))
        self.c.set_enabled(True, {"fixture_only": True})
        self.c.event({"id": "manager-fixture"})
        self.c.begin_manager()
        self.assertEqual("owned", self.tick()["manager"])
        self.assertEqual([], self.sql("SELECT * FROM operations"))

    def test_manager_enabled_is_deferred_and_never_inferred(self):
        self.arm()
        self.c.event({"id": "meaningful-fixture"})
        self.assertEqual("deferred", self.tick()["manager"])
        self.assertIsNone(self.c.snapshot()["manager"])
        self.assertEqual({}, self.c.snapshot()["actions"])

    def test_watchdog_once_per_epoch_preserves_flagged_actions(self):
        self.arm()
        self.c.event({"id": "prepare-fixture"})
        packet = self.c.begin_manager()
        action = {"id": "fixture-action", "kind": "repair", "workstream": "delivery",
                  "sprint": "PF80", "rationale": "fixture", "inputs": {
                      "allocation": "bootstrap", "base": "a" * 40},
                  "expected_revision": packet["state_revision"], "timeout_seconds": 60}
        self.c.accept_decision(packet["manager_run"], {
            "state_revision": packet["state_revision"], "actions": [action]}, {"fixture_only": True})
        self.c.clock = lambda: 1
        self.c.claim("fixture-action")
        self.tick()
        action = self.c.snapshot()["actions"]["fixture-action"]
        self.assertEqual("dispatch_uncertain", action["status"])
        self.assertEqual(0, action["dispatch_epoch"])
        revision = self.c.snapshot()["revision"]
        self.tick()
        self.assertEqual(revision, self.c.snapshot()["revision"])
        self.assertEqual([("ok",)], self.sql("SELECT status FROM health WHERE component='watchdog'"))

    def test_real_process_exit_after_intent_holds_without_retry(self):
        self.arm()
        result = self.child("import os,sys,owner_daemon as o; "
                            "o.FixedTestAdapter.observe=lambda *a: os._exit(73); "
                            "o.Kernel(sys.argv[1]).tick(o.FixedTestAdapter())")
        self.assertEqual(73, result.returncode)
        self.assertEqual([("intent",)], self.sql("SELECT phase FROM operations"))
        self.assertEqual("HOLD", self.tick()["state"])
        self.assertEqual("HOLD", self.tick()["state"])
        self.assertEqual([("effect_uncertain",)], self.sql("SELECT reason_code FROM holds"))
        self.assertEqual([], self.sql("SELECT * FROM deliveries"))

    def test_real_exit_after_receipt_replays_without_effect(self):
        self.arm()
        result = self.child("import os,sys,owner_daemon as o; "
                            "o.Kernel.replay=lambda *a: os._exit(74); "
                            "o.Kernel(sys.argv[1]).tick(o.FixedTestAdapter())")
        self.assertEqual(74, result.returncode)
        self.assertEqual([("intent",)], self.sql("SELECT phase FROM operations"))
        self.assertEqual("ACTIVE", self.tick()["state"])
        self.assertEqual([("applied",)], self.sql("SELECT phase FROM operations"))
        self.assertEqual(1, len(self.sql("SELECT * FROM observations")))

    def test_real_exit_after_domain_commit_deduplicates_replay(self):
        self.arm()
        result = self.child("import os,sys,owner_daemon as o\n"
                            "original=o.ExistingCoordinator.event\n"
                            "def crash(self,event):\n original(self,event)\n os._exit(75)\n"
                            "o.ExistingCoordinator.event=crash\n"
                            "o.Kernel(sys.argv[1]).tick(o.FixedTestAdapter())")
        self.assertEqual(75, result.returncode)
        revision = self.c.snapshot()["revision"]
        self.assertEqual([("observed",)], self.sql("SELECT phase FROM operations"))
        self.tick()
        self.assertEqual(revision, self.c.snapshot()["revision"])

    def test_receipt_tamper_and_prior_activation_do_not_apply(self):
        self.arm()
        with patch.object(owner.Kernel, "replay", side_effect=SystemExit):
            with self.assertRaises(SystemExit):
                self.tick()
        receipt = next(self.root.glob("runs/*/receipt.json"))
        original = owner.load(receipt)
        f.write_json(receipt, {**original, "fixture_only": False})
        with self.assertRaisesRegex(f.LaunchError, "invalid_fixture_receipt"):
            self.tick()
        f.write_json(receipt, original)
        self.authority["generation"] = 2
        self.arm()
        self.assertEqual("HOLD", self.tick()["state"])
        self.assertEqual([("prior_activation",)], self.sql("SELECT reason_code FROM holds"))

    def test_disk_failure_after_intent_never_retries_adapter(self):
        self.arm()
        original = owner.artifact

        def disk_full(root, relative, value):
            if relative.endswith("/receipt.json"):
                raise OSError("fixture disk full")
            return original(root, relative, value)

        with patch.object(owner, "artifact", side_effect=disk_full):
            with self.assertRaises(OSError):
                self.tick()
        with patch.object(owner.FixedTestAdapter, "observe", side_effect=AssertionError("retry")):
            self.assertEqual("HOLD", self.tick()["state"])

    def test_receipt_replay_waits_while_manager_owns_revision(self):
        self.arm()
        with patch.object(owner.Kernel, "replay", side_effect=SystemExit):
            with self.assertRaises(SystemExit):
                self.tick()
        self.c.event({"id": "pending-manager"})
        self.c.begin_manager()
        revision = self.c.snapshot()["revision"]
        self.assertEqual("owned", self.tick()["manager"])
        self.assertEqual(revision, self.c.snapshot()["revision"])
        self.assertEqual([("intent",)], self.sql("SELECT phase FROM operations"))

    def test_admission_rechecks_config_and_drain_remains_off(self):
        self.arm()
        kernel = owner.Kernel(self.config_path)
        f.write_json(self.config_path, {**self.config, "manager_enabled": False})
        with self.assertRaisesRegex(f.LaunchError, "config_drift"):
            kernel.tick(owner.FixedTestAdapter())
        f.write_json(self.config_path, self.config)
        self.sql("UPDATE meta SET requested_mode='draining'")
        with self.assertRaisesRegex(f.LaunchError, "owner_off"):
            self.tick()
        self.assertEqual([], self.sql("SELECT * FROM operations"))


if __name__ == "__main__":
    unittest.main()
