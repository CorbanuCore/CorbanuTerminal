"""Offline kernel evidence only; no providers, TMUX, Git, Slack or credentials."""
from contextlib import closing
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from coordinator import Coordinator, digest
import fable_launcher as f
import owner_daemon as owner
import owner_tmux as tmux
from test_coordinator import seed

ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "scripts/initiative_control/owner_daemon.py"

# Manager-recorded macOS 26.2, UID 503, Background/SSH-only transcript.
HEADLESS_GUI_STDERR = "Could not print domain: 125: Domain does not support specified action\n"
HEADLESS_USER_STDERR = 'Bad request.\nCould not find service "com.corbanu.absent" in domain for uid: 503\n'


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


class ArmingPreviewTests(unittest.TestCase):
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    child = OwnerDaemonTests.child
    sql = OwnerDaemonTests.sql
    tick = OwnerDaemonTests.tick

    def files(self):
        return {str(p.relative_to(self.root)): (p.read_bytes(), p.stat().st_mtime_ns,
                                               p.stat().st_mode)
                for p in self.root.rglob("*") if p.is_file()}

    def test_cli_preview_is_write_free_and_matches_arm(self):
        authority = self.root / "decision.json"
        f.write_json(authority, self.authority)
        before = self.files()
        result = self.child(None, "--arm", "--dry-run", "--config", str(self.config_path),
                            "--authority", str(authority))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        preview = json.loads(result.stdout)
        self.assertEqual("WOULD_ARM", preview["state"])
        self.assertEqual(before, self.files())
        with patch.object(f, "write_json", side_effect=AssertionError("persistent write")), \
                patch.object(owner, "remove_activation_intent", side_effect=AssertionError("unlink")):
            owner.arm_owner(self.config_path, self.authority, dry_run=True)
        owner.arm_owner(self.config_path, self.authority)
        with closing(sqlite3.connect(self.root / "owner.sqlite3")) as db:
            db.row_factory = sqlite3.Row
            self.assertEqual(preview["arm"]["after"], dict(db.execute("SELECT * FROM meta").fetchone()))
        self.assertEqual(preview["arm"]["activation_file"]["after"], owner.load(self.root / "activation.json"))
        self.assertEqual("ACTIVE", self.tick()["state"])
        effect = preview["first_admitted_tick"]
        self.assertEqual(effect["owner_rows"]["operations"]["op_id"],
                         self.sql("SELECT op_id FROM operations")[0][0])
        self.assertTrue(all(Path(path).exists() for path in effect["files"]))

    def test_preview_watchdog_matches_real_tick_including_manager_and_history(self):
        with self.c.mutation("fixture", {}) as (_, state):
            for index, status in enumerate(("dispatching", "dispatched", "running",
                                            "dispatch_uncertain", "returned")):
                state["actions"][status] = dict(id=status, status=status, deadline=1.0,
                    dispatch_epoch=2, workstream="delivery", sequence=index)
            state["manager"] = dict(id="manual-manager", deadline=1.0)
        before = self.files()
        preview = owner.arm_owner(self.config_path, self.authority, dry_run=True)["first_admitted_tick"]
        self.assertEqual(before, self.files())
        self.assertEqual("READY", preview["state"])
        self.assertIsNone(preview["fixture_event"])
        owner.arm_owner(self.config_path, self.authority)
        self.assertEqual("READY", self.tick()["state"])
        actual = self.c.snapshot()
        for change in preview["coordinator_changes"]:
            if change["path"][:3] == ["state", "1", "body"]:
                current = actual
                for key in change["path"][3:]:
                    current = current[key]
                self.assertEqual(change["after"], current)
        with self.c.connection() as db:
            for event in preview["watchdog_events"]:
                self.assertEqual(event, json.loads(db.execute(
                    "SELECT body FROM events WHERE id=?", (event["id"],)).fetchone()[0]))
        self.assertEqual(4, len(preview["watchdog_events"]))

    def test_preview_paused_and_held_skip_fixture_but_not_watchdog(self):
        for held in (False, True):
            with self.subTest(held=held):
                if held:
                    self.sql("INSERT INTO holds VALUES('fixture','global',NULL,'fixture',0,0,'x',NULL,NULL)")
                self.c.set_enabled(False, {"fixture": True})
                preview = owner.arm_owner(self.config_path, self.authority, dry_run=True)["first_admitted_tick"]
                self.assertEqual("HOLD" if held else "READY", preview["state"])
                self.assertEqual([], preview["files"])
                self.assertIsNone(preview["fixture_event"])
                self.assertEqual([], preview["coordinator_changes"])

    def test_preview_shares_authority_and_drift_refusals_without_writes(self):
        for changes, reason in (({"generation": 2}, "activation_generation_mismatch"),
                                ({"scope": "tmux-workers"}, "activation_scope_mismatch"),
                                ({"authority": ""}, "activation_authority_required")):
            before = self.files()
            for dry_run in (False, True):
                with self.subTest(dry_run=dry_run, reason=reason), self.assertRaisesRegex(f.LaunchError, reason):
                    owner.arm_owner(self.config_path, {**self.authority, **changes}, dry_run=dry_run)
            self.assertEqual(before, self.files())
        f.write_json(self.root / "activation-transaction.json", {"fixture": True})
        before = self.files()
        with self.assertRaisesRegex(f.LaunchError, "activation_recovery_required"):
            owner.arm_owner(self.config_path, self.authority, dry_run=True)
        self.assertEqual(before, self.files())

    def test_preview_refuses_unsettled_operation_and_coordinator_journal(self):
        self.sql("INSERT INTO operations(op_id,phase) VALUES('fixture','intent')")
        before = self.files()
        with self.assertRaisesRegex(f.LaunchError, "preview_requires_settled_operations"):
            owner.arm_owner(self.config_path, self.authority, dry_run=True)
        self.assertEqual(before, self.files())
        self.sql("DELETE FROM operations")
        f.write_file(self.root / "coordinator.sqlite3-journal", b"fixture")
        before = self.files()
        with self.assertRaisesRegex(f.LaunchError, "coordinator_recovery_required"):
            owner.arm_owner(self.config_path, self.authority, dry_run=True)
        self.assertEqual(before, self.files())

    def test_preview_never_acquires_owner_locks_even_during_backup(self):
        original = owner.preview_backup
        snapshots = []
        def check(source, destination):
            with owner.locked(self.root / "owner-daemon.lock"), \
                    owner.locked(self.root / "owner-admission.lock"):
                snapshots.append(True)
                return original(source, destination)
        before = self.files()
        with patch.object(owner, "preview_backup", side_effect=check):
            result = owner.arm_owner(self.config_path, self.authority, dry_run=True)
        self.assertEqual(2, len(snapshots))
        self.assertFalse(result["read_policy"]["owner_locks"])
        self.assertEqual(before, self.files())

    def test_backup_busy_or_elapsed_budget_refuses(self):
        with closing(sqlite3.connect(":memory:")) as destination:
            class Busy:
                def execute(self, sql):
                    pass
                def backup(self, dest, pages, progress, sleep):
                    progress(sqlite3.SQLITE_BUSY, 1, 2)
            with self.assertRaisesRegex(f.LaunchError, "preview_database_busy"):
                owner.preview_backup(Busy(), destination)
            with patch.object(owner.time, "monotonic", side_effect=[0, 1]):
                with self.assertRaisesRegex(f.LaunchError, "preview_read_budget_exceeded"):
                    owner.preview_backup(Busy(), destination)

    def test_preview_option_cannot_be_silently_ignored(self):
        for args in (("--dry-run",), ("--run", "--dry-run"),
                     ("--disarm", "--dry-run", "--config", str(self.config_path), "--generation", "0")):
            self.assertEqual(2, self.child(None, *args).returncode)


class ActivationVisibilityTests(unittest.TestCase):
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    child = OwnerDaemonTests.child
    tick = OwnerDaemonTests.tick

    def manual_claim(self):
        from test_coordinator import CoordinatorTests
        self.c.clock = lambda: 1000.0
        CoordinatorTests.prepared(self, "manual")
        return self.c.claim("manual")

    def test_status_lists_in_flight_and_overdue_with_exact_watchdog_effects(self):
        with self.c.mutation("fixture", {}) as (_, state):
            for index, status in enumerate(("prepared", "dispatching", "dispatched",
                                             "running", "dispatch_uncertain", "returned",
                                             "accepted", "failed", "cancelled")):
                state["actions"][status] = dict(id=status, status=status, deadline=999.0,
                                               stall_reported=status == "running")
            state["actions"]["boundary"] = dict(id="boundary", status="dispatching", deadline=1000.0)
            state["actions"]["future"] = dict(id="future", status="running", deadline=1001.0)
            for index, action in enumerate(state["actions"].values()):
                action.update(workstream="delivery", sequence=index)
            state["manager"] = dict(id="manual-manager", deadline=998.0)
        before = {p.name: p.read_bytes() for p in self.root.iterdir() if p.is_file()}
        with patch.object(owner.time, "time", return_value=1000.0):
            status = owner.activation_status(self.config_path)
        impact = status["coordinator"]
        self.assertEqual(1000.0, impact["observed_at"])
        self.assertEqual(self.c.snapshot()["revision"], impact["revision"])
        rows = {row["id"]: row for row in impact["in_flight"]}
        self.assertEqual({"dispatching", "dispatched", "running", "dispatch_uncertain",
                          "returned", "boundary", "future"}, set(rows))
        self.assertEqual(999.0, rows["dispatching"]["deadline"])
        self.assertEqual("dispatch_uncertain", rows["dispatching"]["watchdog_status"])
        self.assertEqual("dispatched", rows["dispatched"]["watchdog_status"])
        self.assertTrue(rows["dispatching"]["watchdog_will_report"])
        self.assertFalse(rows["running"]["watchdog_will_report"])
        self.assertFalse(rows["dispatch_uncertain"]["watchdog_will_report"])
        self.assertFalse(rows["returned"]["watchdog_will_report"])
        self.assertEqual({"dispatching", "dispatched", "running", "dispatch_uncertain", "returned"},
                         {row["id"] for row in impact["overdue"]})
        self.assertFalse(rows["boundary"]["overdue"])
        self.assertFalse(rows["future"]["overdue"])
        self.assertEqual(998.0, impact["manager"]["deadline"])
        self.assertTrue(impact["manager"]["watchdog_will_report"])
        self.assertIn("fixture-only", impact["warning"])
        self.assertIn("dispatch_uncertain", impact["warning"])
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.root.iterdir() if p.is_file()})

    def test_transport_coverage_matches_watchdog_for_default_hand_and_owner(self):
        with self.c.mutation("fixture", {}) as (_, state):
            state["dispatch_control"] = {"default": "hand"}
            for index, (key, side, status, deadline, reported) in enumerate((
                    ("owner-overdue", "owner", "dispatching", 999, False),
                    ("hand-overdue", "hand", "dispatching", 999, False),
                    ("default-hand", None, "running", 999, False),
                    ("owner-future", "owner", "running", 1001, False),
                    ("owner-reported", "owner", "running", 999, True),
                    ("owner-returned", "owner", "returned", 999, False),
                    ("owner-at-deadline", "owner", "dispatched", 1000, False),
                    ("hand-prepared", "hand", "prepared", 999, False),
                    ("hand-accepted", "hand", "accepted", 999, False),
                    ("hand-failed", "hand", "failed", 999, False),
                    ("hand-cancelled", "hand", "cancelled", 999, False),
                    ("hand-uncertain", "hand", "dispatch_uncertain", 999, False),
                    ("hand-returned", "hand", "returned", 999, False),
                    ("hand-reported", "hand", "running", 999, True),
                    ("hand-future", "hand", "running", 1001, False))):
                action = dict(id=key, status=status, deadline=deadline, dispatch_epoch=1,
                              stall_reported=reported, workstream="delivery", sequence=index)
                if side is not None:
                    action["dispatch_owner"] = side
                state["actions"][key] = action
            state["manager"] = dict(id="manager", deadline=999)
        # Status reads configured routing even when OFF; predictions explicitly
        # describe an admitted tick, not a promise that OFF executes anything.
        f.write_json(self.config_path, dict(self.config, transport={}))
        before = self.c.snapshot()
        with patch.object(owner.time, "time", return_value=1000.0):
            status = owner.activation_status(self.config_path)
        self.assertEqual("off", status["state"])
        self.assertEqual(before, self.c.snapshot())
        impact = status["coordinator"]
        coverage = impact["watchdog_coverage"]
        self.assertEqual(["default-hand", "hand-overdue"], coverage["excluded_actions"])
        self.assertEqual(["owner-overdue"], coverage["covered_actions"])
        self.assertIn("covered=1, excluded=2; manager=covered;", coverage["summary"])
        self.assertTrue(coverage["manager_covered"])
        self.assertFalse(coverage["future_default_covered"])
        self.assertIn("manager responsibility", coverage["summary"])
        self.assertIn("no scheduled hand watchdog", coverage["summary"])
        rows = {row["id"]: row for row in impact["in_flight"]}
        self.assertFalse(rows["hand-overdue"]["watchdog_will_report"])
        self.assertEqual("dispatching", rows["hand-overdue"]["watchdog_status"])
        self.c.clock = lambda: 1000.0
        events = self.c.watchdog(dispatcher="owner")
        self.assertEqual({"owner-overdue"}, {e["action"] for e in events if "action" in e})
        self.assertEqual({key for key, row in rows.items() if row["watchdog_will_report"]},
                         {e["action"] for e in events if "action" in e})
        self.assertTrue(self.c.snapshot()["manager"]["stall_reported"])
        self.assertEqual(before["actions"]["hand-overdue"],
                         self.c.snapshot()["actions"]["hand-overdue"])
        hand_events = self.c.watchdog(dispatcher="hand")
        self.assertEqual({"hand-overdue", "default-hand"}, {e["action"] for e in hand_events})

    def test_fixture_watchdog_covers_partitioned_hand_actions(self):
        with self.c.mutation("fixture", {}) as (_, state):
            state["dispatch_control"] = {"default": "hand"}
            state["actions"]["hand"] = dict(id="hand", status="dispatching", deadline=1,
                                            sequence=1, workstream="delivery", dispatch_epoch=1)
        impact = owner.activation_status(self.config_path)["coordinator"]
        self.assertEqual(["hand"], impact["watchdog_coverage"]["covered_actions"])
        self.assertTrue(impact["watchdog_coverage"]["future_default_covered"])
        self.assertTrue(impact["in_flight"][0]["watchdog_will_report"])
        self.assertEqual("hand", self.c.watchdog()[0]["action"])

    def test_status_discloses_unresolved_foreign_and_global_holds(self):
        with self.c.mutation("fixture", {}) as (_, state):
            state["dispatch_control"] = {"default": "hand"}
            state["actions"]["hand"] = dict(id="hand", status="prepared", sequence=1,
                                            workstream="delivery")
        f.write_json(self.config_path, dict(self.config, transport={}))
        with sqlite3.connect(self.root / "owner.sqlite3") as db:
            for op in (digest(["tmux", "hand", "claim"]), "global"):
                db.execute("INSERT INTO holds VALUES(?,?,?,?,?,?,?,?,?)",
                           (op, "operation", op, "fixture_hold", 1, 2, "evidence", None, None))
        result = owner.activation_status(self.config_path)
        self.assertTrue(result["complete"])
        rows = {row["op_id"]: row for row in result["unresolved_holds"]}
        foreign = rows[digest(["tmux", "hand", "claim"])]
        self.assertEqual("hand", foreign["action_id"])
        self.assertEqual("hand", foreign["dispatch_owner"])
        self.assertTrue(foreign["excluded_from_owner_lane"])
        self.assertEqual("resolved", foreign["ownership_status"])
        self.assertIsNone(rows["global"]["excluded_from_owner_lane"])
        self.assertEqual("unmapped", rows["global"]["ownership_status"])
        with patch.object(owner, "coordinator_activation_impact", side_effect=sqlite3.OperationalError):
            unavailable = owner.activation_status(self.config_path)
        self.assertFalse(unavailable["complete"])
        for row in unavailable["unresolved_holds"]:
            self.assertEqual(set(rows[row["op_id"]]), set(row))
            self.assertEqual("coordinator_unavailable", row["ownership_status"])
            self.assertIsNone(row["dispatch_owner"])
            self.assertIsNone(row["excluded_from_owner_lane"])

    def test_manager_coverage_and_execution_follow_shared_predicate(self):
        with self.c.mutation("fixture", {}) as (_, state):
            state["manager"] = dict(id="manager", deadline=999)
        self.c.clock = lambda: 1000.0
        original = Coordinator.watchdog_covers
        def exclude_manager(state, action, dispatcher=None):
            return action is not None and original(state, action, dispatcher)
        with patch.object(Coordinator, "watchdog_covers", side_effect=exclude_manager):
            with patch.object(owner.time, "time", return_value=1000.0):
                impact = owner.coordinator_activation_impact(self.root, "owner")
            self.assertFalse(impact["watchdog_coverage"]["manager_covered"])
            self.assertIn("manager=excluded", impact["watchdog_coverage"]["summary"])
            self.assertFalse(impact["manager"]["watchdog_will_report"])
            self.assertEqual([], self.c.watchdog(dispatcher="owner"))
            self.assertNotIn("stall_reported", self.c.snapshot()["manager"])
        self.assertEqual(["manager-stall:manager"],
                         [event["id"] for event in self.c.watchdog(dispatcher="owner")])

    def test_hold_row_shape_for_missing_and_resolved_action(self):
        with sqlite3.connect(self.root / "owner.sqlite3") as db:
            db.execute("INSERT INTO operations(op_id,action_id,phase) VALUES(?,?,?)",
                       ("missing-op", "gone", "intent"))
            db.execute("INSERT INTO holds VALUES(?,?,?,?,?,?,?,?,?)",
                       ("missing-op", "operation", "missing-op", "fixture_hold",
                        1, 2, "evidence", None, None))
        row = owner.activation_status(self.config_path)["unresolved_holds"][0]
        self.assertEqual("action_missing", row["ownership_status"])
        self.assertEqual("gone", row["action_id"])
        self.assertIsNone(row["excluded_from_owner_lane"])
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"]["gone"] = dict(id="gone", status="prepared",
                                            workstream="delivery", sequence=1)
        resolved = owner.activation_status(self.config_path)["unresolved_holds"][0]
        self.assertEqual(set(row), set(resolved))
        self.assertEqual("resolved", resolved["ownership_status"])
        self.assertFalse(resolved["excluded_from_owner_lane"])

    def test_empty_status_still_warns_about_future_shared_state_mutation(self):
        result = owner.activation_status(self.config_path)["coordinator"]
        self.assertEqual([], result["in_flight"])
        self.assertEqual([], result["overdue"])
        self.assertIsNone(result["manager"])
        self.assertIn("not read-only", result["warning"])
        self.assertIn("snapshot", result["warning"])

    def test_manual_overdue_claim_does_not_refuse_arm_and_paused_tick_matches_warning(self):
        claimed = self.manual_claim()
        self.c.set_enabled(False, {"fixture_only": True})
        before = self.c.snapshot()
        result = owner.arm_owner(self.config_path, self.authority)
        self.assertEqual("ARMED", result["state"])
        row = result["coordinator"]["overdue"][0]
        self.assertEqual(("manual", claimed["deadline"], "dispatch_uncertain"),
                         (row["id"], row["deadline"], row["watchdog_status"]))
        self.assertEqual(before, self.c.snapshot())
        self.tick()
        action = self.c.snapshot()["actions"]["manual"]
        self.assertEqual("dispatch_uncertain", action["status"])
        self.assertTrue(action["stall_reported"])

    def test_cli_status_discloses_manual_deadline_and_help_warns_before_arming(self):
        claimed = self.manual_claim()
        result = self.child(None, "--activation-status", "--config", str(self.config_path))
        self.assertEqual(0, result.returncode, result.stderr)
        row = json.loads(result.stdout)["coordinator"]["overdue"][0]
        self.assertEqual("manual", row["id"])
        self.assertEqual(claimed["deadline"], row["deadline"])
        help_text = " ".join(self.child(None, "--help").stdout.split())
        self.assertIn("not read-only", help_text)
        self.assertIn("--activation-status", help_text)

    def test_status_does_not_recover_coordinator_journal(self):
        journal = self.root / "coordinator.sqlite3-journal"
        f.write_file(journal, b"fixture hot-journal marker")
        before = {p.name: p.read_bytes() for p in self.root.iterdir() if p.is_file()}
        result = owner.activation_status(self.config_path)
        self.assertFalse(result["complete"])
        self.assertEqual("off", result["state"])
        self.assertIsNone(result["coordinator"])
        self.assertEqual("coordinator_recovery_required", result["unavailable"]["coordinator"]["reason"])
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.root.iterdir() if p.is_file()})

    def test_busy_coordinator_returns_partial_status_through_cli(self):
        with closing(sqlite3.connect(self.c.path)) as db:
            db.execute("BEGIN EXCLUSIVE")
            result = self.child(None, "--activation-status", "--config", str(self.config_path))
        self.assertEqual(0, result.returncode, result.stderr)
        value = json.loads(result.stdout)
        self.assertFalse(value["complete"])
        self.assertEqual("off", value["state"])
        self.assertEqual(0, value["generation"])
        self.assertIsNone(value["coordinator"])
        self.assertEqual("OperationalError", value["unavailable"]["coordinator"]["reason"])

    def test_busy_owner_still_reports_coordinator_and_unknown_owner(self):
        with owner.locked(self.root / "owner-daemon.lock"):
            result = owner.activation_status(self.config_path)
        self.assertFalse(result["complete"])
        self.assertIsNone(result["state"])
        self.assertIsNone(result["generation"])
        self.assertIsNone(result["stored"])
        self.assertIn("owner", result["unavailable"])
        self.assertEqual([], result["coordinator"]["in_flight"])

    def test_partial_status_cli_redacts_untrusted_error_details(self):
        import io
        from contextlib import redirect_stdout
        for component, operation in (("owner", "activation_store"),
                                     ("coordinator", "coordinator_activation_impact")):
            for error in (OSError("private-path-canary"), sqlite3.OperationalError("private-statement-canary"),
                          ValueError("private-value-canary"), TypeError("private-type-canary"),
                          KeyError("private-key-canary"), f.LaunchError("owner_busy"),
                          owner.Rejected("coordinator_recovery_required")):
                with self.subTest(component=component, error=type(error).__name__):
                    output = io.StringIO()
                    with patch.object(owner, operation, side_effect=error), redirect_stdout(output):
                        code = owner.main(["--activation-status", "--config", str(self.config_path)])
                    self.assertEqual(0, code)
                    value = json.loads(output.getvalue())
                    self.assertFalse(value["complete"])
                    expected = str(error) if isinstance(error, (f.LaunchError, owner.Rejected)) else type(error).__name__
                    self.assertEqual(dict(error=type(error).__name__, reason=expected),
                                     value["unavailable"][component])
                    self.assertNotIn("canary", output.getvalue())
                    self.assertIsNotNone(value["coordinator"] if component == "owner" else value["state"])

    def test_complete_status_is_explicit(self):
        result = owner.activation_status(self.config_path)
        self.assertTrue(result["complete"])
        self.assertEqual({}, result["unavailable"])


class ArmingTests(unittest.TestCase):
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    sql = OwnerDaemonTests.sql
    child = OwnerDaemonTests.child
    tick = OwnerDaemonTests.tick

    def state(self):
        return ((self.root / "owner.sqlite3").read_bytes(),
                {p.name: p.read_bytes() for p in self.root.glob("activation*")})

    def refuse(self, reason, **changes):
        before = self.state()
        with self.assertRaisesRegex(f.LaunchError, "^" + reason + "$"):
            owner.arm_owner(self.config_path, {**self.authority, **changes})
        self.assertEqual(before, self.state())
        self.assertEqual([], self.sql("SELECT * FROM boots"))

    def test_refuses_invalid_activation_keys(self):
        self.refuse("invalid_activation", extra=True)

    def test_refuses_invalid_decision_id(self):
        self.refuse("activation_decision_id_required", decision_id=[])

    def test_refuses_blank_authority(self):
        self.refuse("activation_authority_required", authority=" ")

    def test_refuses_invalid_revision(self):
        self.refuse("activation_revision_required", revision=True)

    def test_refuses_invalid_generation(self):
        self.refuse("activation_generation_required", generation=True)

    def test_refuses_wrong_scope_without_transport(self):
        self.refuse("activation_scope_mismatch", scope="tmux-workers")

    def transport(self):
        binary = str(Path("/bin/echo").resolve())
        self.config["transport"] = dict(kind="tmux", binary=binary,
                                       binary_sha256=f.file_digest(Path(binary)), tmux=binary,
                                       runs_dir=str(self.root),
                                       auth_link=str(self.root / "unused/auth.json"))
        f.write_json(self.config_path, self.config)
        self.sql("UPDATE meta SET config_digest=?", (digest(self.config),))
        self.authority.update(scope="tmux-workers", config_digest=digest(self.config))

    def test_refuses_wrong_scope_with_transport(self):
        self.transport()
        self.refuse("activation_scope_mismatch", scope="fixture-only")

    def test_refuses_wrong_config_digest(self):
        self.refuse("activation_config_mismatch", config_digest="0" * 64)

    def test_refuses_wrong_package_digest(self):
        self.refuse("activation_package_mismatch", package_digest="0" * 64)

    def test_refuses_stale_generation(self):
        owner.arm_owner(self.config_path, self.authority)
        owner.disarm_owner(self.config_path, 1)
        self.refuse("activation_generation_mismatch")

    def test_refuses_skipped_generation(self):
        self.refuse("activation_generation_mismatch", generation=2)

    def test_refuses_already_armed(self):
        owner.arm_owner(self.config_path, self.authority)
        self.refuse("owner_already_armed", generation=2)

    def test_refuses_state_drift(self):
        self.sql("UPDATE meta SET config_digest='wrong'")
        self.refuse("state_drift")

    def test_refuses_package_drift(self):
        f.write_json(self.config_path, {**self.config, "package_digest": "wrong"})
        self.refuse("package_drift")

    def test_refuses_disarm_stale_generation(self):
        before = self.state()
        with self.assertRaisesRegex(f.LaunchError, "^activation_generation_mismatch$"):
            owner.disarm_owner(self.config_path, 1)
        self.assertEqual(before, self.state())

    def test_refuses_disarm_boolean_generation(self):
        before = self.state()
        with self.assertRaisesRegex(f.LaunchError, "^activation_generation_required$"):
            owner.disarm_owner(self.config_path, True)
        self.assertEqual(before, self.state())

    def test_refuses_pending_activation(self):
        f.write_json(self.root / "activation-transaction.json", {"interrupted": True})
        self.refuse("activation_recovery_required")

    def test_refuses_owner_journal(self):
        f.write_file(self.root / "owner.sqlite3-journal", b"fixture")
        self.refuse("owner_recovery_required")

    def test_refuses_schema_drift(self):
        self.sql("CREATE TABLE unexpected (value TEXT)")
        self.sql("DROP TABLE health")
        self.refuse("schema_drift")

    def test_refuses_busy_owner_and_admission_locks(self):
        for name in ("owner-daemon.lock", "owner-admission.lock"):
            with self.subTest(name=name), owner.locked(self.root / name):
                before = self.state()
                with self.assertRaises(BlockingIOError):
                    owner.arm_owner(self.config_path, self.authority)
                self.assertEqual(before, self.state())

    def test_arm_tick_disarm_rearm_uses_real_entry_point(self):
        before = self.c.snapshot()
        result = owner.arm_owner(self.config_path, self.authority)
        self.assertEqual(dict(state="ARMED", generation=1,
                              activation_digest=digest(self.authority)),
                         {key: value for key, value in result.items() if key != "coordinator"})
        self.assertEqual([], result["coordinator"]["in_flight"])
        self.assertEqual(before, self.c.snapshot())
        self.assertEqual(self.authority, owner.load(self.root / "activation.json"))
        self.assertEqual("ACTIVE", self.tick()["state"])
        result = owner.disarm_owner(self.config_path, 1)
        self.assertEqual(dict(state="OFF", generation=2), result)
        with self.assertRaisesRegex(f.LaunchError, "owner_off"):
            self.tick()
        self.assertEqual(self.authority, owner.load(self.root / "activation.json"))
        owner.arm_owner(self.config_path, {**self.authority, "generation": 3})
        self.assertEqual("ACTIVE", self.tick()["state"])

    def test_transport_arm_does_not_launch_or_read_auth(self):
        self.transport()
        with self.assertRaisesRegex(f.LaunchError, "dispatch_handoff_required"):
            owner.arm_owner(self.config_path, self.authority)
        owner.handoff(self.config_path, dict(expected_revision=self.c.snapshot()["revision"],
                      assignments={}, evidence={"fixture": "both dispatchers quiescent"}))
        with patch.object(tmux, "TmuxAdapter", side_effect=AssertionError("no transport effects")):
            owner.arm_owner(self.config_path, self.authority)
        self.assertFalse((self.root / "unused").exists())
        self.assertEqual([], self.sql("SELECT * FROM processes"))
        self.assertEqual([], self.sql("SELECT * FROM boots"))

    def test_disarm_remains_available_after_package_and_config_drift(self):
        owner.arm_owner(self.config_path, self.authority)
        f.write_json(self.config_path, {**self.config, "package_digest": "wrong", "worktrees": []})
        owner.disarm_owner(self.config_path, 1)
        self.assertEqual([("off", 2)], self.sql("SELECT requested_mode,control_generation FROM meta"))

    def test_cli_arm_status_disarm_and_named_refusal(self):
        path = self.root / "decision.json"
        f.write_json(path, self.authority)
        result = self.child(None, "--arm", "--config", str(self.config_path),
                            "--authority", str(path))
        self.assertEqual(0, result.returncode, result.stderr)
        result = self.child(None, "--activation-status", "--config", str(self.config_path))
        self.assertEqual(0, result.returncode, result.stderr)
        state = json.loads(result.stdout)
        self.assertEqual(1, state["generation"])
        self.assertEqual(digest(self.config), state["config_digest"])
        result = self.child(None, "--arm", "--config", str(self.config_path),
                            "--authority", str(path))
        self.assertEqual(2, result.returncode)
        self.assertEqual("activation_generation_mismatch", json.loads(result.stdout)["reason"])
        result = self.child(None, "--disarm", "--config", str(self.config_path), "--generation", "1")
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("OFF", json.loads(result.stdout)["state"])

    def test_crash_each_arm_boundary_fences_admission_until_disarm(self):
        # Actual child exits: before/after each file publication and after SQL commit.
        for target in ("intent-before", "intent-after", "file-before", "file-after", "commit-after"):
            with self.subTest(target=target):
                code = """
import os, sys
from pathlib import Path
import owner_daemon as o
import fable_launcher as f
config = Path(sys.argv[1])
a = o.load(config.parent / "decision.json")
write, remove = f.write_json, o.remove_activation_intent
def crash_write(path, value):
    name = "intent" if path.name == "activation-transaction.json" else "file"
    if name + "-before" == TARGET: os._exit(73)
    write(path, value)
    if name + "-after" == TARGET: os._exit(73)
def crash_remove(root):
    if TARGET == "commit-after": os._exit(73)
    remove(root)
f.write_json, o.remove_activation_intent = crash_write, crash_remove
o.arm_owner(config, a)
"""
                generation = self.sql("SELECT control_generation FROM meta")[0][0]
                f.write_json(self.root / "decision.json", {**self.authority, "generation": generation + 1})
                result = self.child("TARGET = " + repr(target) + "\n" + code)
                self.assertEqual(73, result.returncode, result.stderr)
                with self.assertRaises(f.LaunchError):
                    self.tick()
                current = self.sql("SELECT control_generation FROM meta")[0][0]
                owner.disarm_owner(self.config_path, current)
                with self.assertRaisesRegex(f.LaunchError, "owner_off"):
                    self.tick()
                self.assertFalse((self.root / "activation-transaction.json").exists())

    def test_partial_file_write_recovery_preserves_off_and_consumes_generation(self):
        f.write_file(self.root / "activation-transaction.json.pending", b"partial")
        self.refuse("activation_recovery_required")
        owner.disarm_owner(self.config_path, 0)
        self.assertFalse((self.root / "activation-transaction.json.pending").exists())
        self.refuse("activation_generation_mismatch")
        owner.arm_owner(self.config_path, {**self.authority, "generation": 2})


class FakeWorker(tmux.Worker):
    """Durable harmless launcher seam; no tmux process, model or credential reads."""
    modes = {}
    events = []
    fail = None

    def effect(self, name):
        action = self.binding["action_id"]
        self.events.append((action, name))
        if self.fail == (action, name):
            raise OSError("synthetic effect failure")
        self.once("fake-" + name, {"effect": name})
        if self.modes.get(action) == "crash-" + name:
            raise SystemExit(73)
        return self.inspect() if name == "launch" else None

    def launch(self):
        return self.effect("launch")

    def prompt(self):
        return self.effect("prompt")

    def start(self):
        return self.effect("start")

    def close(self, timeout=3):
        self.effect("close")
        clean = self.modes.get(self.binding["action_id"]) != "close-unclean"
        return {"clean": clean, "forced": False, "timeout": timeout, "server_shutdown": None}

    def inspect(self, deadline=None):
        action = self.binding["action_id"]
        mode = self.modes.get(action)
        if mode == "inspect-error":
            raise ValueError("synthetic corrupt observation")
        ack = (self.run / "fake-prompt.json").exists() and mode != "missing-ack"
        submitted = (self.run / "fake-start.json").exists() and mode != "idle-start"
        return {"ready": mode != "not-ready", "identity_valid": mode != "invalid",
                "liveness": "crashed" if mode == "dead" else "alive",
                "process": {"pid": 12345, "pgid": 12345, "pane": "%0", "start": "synthetic"},
                "at": f.now(), "ack": ack, "ack_line": self.meta["ack"] if mode != "wrong-ack" else
                self.meta["ack"].replace(self.binding["allocation_digest"], "b" * 64),
                "submitted": submitted, "session_id": "fixture-session", "thread_id": "fixture-thread",
                "turn_id": "work-turn" if submitted else "ack-turn",
                "returned": "RETURN\nfixture work" if submitted and mode != "working" else None}


class WorkerLifecycleTests(unittest.TestCase):
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    sql = OwnerDaemonTests.sql
    arm = OwnerDaemonTests.arm
    child = OwnerDaemonTests.child

    def configure(self):
        self.config["transport"] = {"kind": "tmux", "binary": str(Path("/bin/echo").resolve()),
                                    "binary_sha256": f.file_digest(Path("/bin/echo").resolve()),
                                    "tmux": str(Path("/bin/echo").resolve()), "runs_dir": str(self.root),
                                    "auth_link": str(self.root / "unused/auth.json")}
        f.write_json(self.config_path, self.config)
        self.sql("UPDATE meta SET config_digest=?", (digest(self.config),))
        self.authority.update(scope="tmux-workers", config_digest=digest(self.config))
        self.arm()
        FakeWorker.modes, FakeWorker.events, FakeWorker.fail = {}, [], None
        self.fake = patch.object(tmux, "Worker", FakeWorker)
        self.fake.start()
        self.addCleanup(self.fake.stop)

    def prepared(self, key="one", kind="repair", cycle_inputs=None, base_commit=None, **runtime_changes):
        allocation = seed()[2]["bootstrap"]
        allocation["kinds"] = [kind]
        if base_commit is not None:
            allocation["inputs"]["base_commit"] = base_commit
        if kind == "complete_sprint":
            allocation["inputs"].update(receiving_action="fixture-receiver", receiving_commit="a" * 40,
                                        mandatory_gates=["fixture-gate"])
        allocation["resources"] = [key]
        allocation["inputs"]["worker"] = {
            "model": "fixture-model", "provider": "fixture", "effort": "high",
            "worktree": str(self.root), "policy": "--yolo", **runtime_changes}
        if cycle_inputs is not None:
            allocation["inputs"] = cycle_inputs
        self.c.put_allocation(key, allocation, False, self.c.snapshot()["revision"], {"fixture": True})
        self.c.event({"id": "prepare-" + key})
        packet = self.c.begin_manager()
        action = {"id": key, "kind": kind, "workstream": "delivery", "sprint": "PF80",
                  "rationale": "fixture", "inputs": {"allocation": key, **allocation["inputs"]},
                  "expected_revision": packet["state_revision"], "timeout_seconds": 60}
        self.c.accept_decision(packet["manager_run"], {
            "state_revision": packet["state_revision"], "actions": [action]}, {"fixture": True})

    def tick(self):
        return owner.Kernel(self.config_path).tick()

    def test_full_lifecycle_one_tick_and_restart_are_idempotent(self):
        self.configure()
        self.prepared()
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual([("one", "launch"), ("one", "prompt"), ("one", "start"), ("one", "close")],
                         FakeWorker.events)
        self.assertEqual("returned", self.c.snapshot()["actions"]["one"]["status"])
        revision = self.c.snapshot()["revision"]
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(revision, self.c.snapshot()["revision"])
        self.assertEqual(4, len(FakeWorker.events))
        effects = dict(self.sql("SELECT effect,phase FROM operations"))
        self.assertEqual({"claim", "prepare", "launch", "prompt", "ack", "dispatched",
                          "acknowledged", "start", "working", "return_observed", "returned",
                          "close"}, set(effects))
        self.assertEqual({"applied"}, set(effects.values()))
        self.assertEqual([("work-turn",)], self.sql("SELECT active_turn_id FROM processes"))

    def test_cycle_bridge_is_frozen_through_claim_and_return(self):
        self.configure()
        flat = dict(base_commit="a" * 40, brief_file=str(self.root / "brief.json"),
                    brief_sha256="b" * 64, model="gpt-6-astra", reasoning_effort="high",
                    task="Read the frozen brief", worktree=str(self.root))
        frozen = tmux.freeze_worker_inputs(flat, provider="openai", policy="--yolo")
        self.prepared(cycle_inputs=frozen)
        before = self.c.snapshot()
        action = before["actions"]["one"]
        self.assertEqual({"allocation": "one", **frozen}, action["inputs"])
        self.assertEqual(digest(before["allocations"]["one"]), action["allocation_digest"])
        self.assertEqual("returned", self.tick()["actions"]["one"])
        current = self.c.snapshot()["actions"]["one"]
        self.assertEqual(action["inputs"], current["inputs"])
        self.assertEqual(action["allocation_digest"], current["allocation_digest"])
        self.assertEqual([("gpt-6-astra", "openai", "high", str(self.root))],
                         self.sql("SELECT model,provider,effort,worktree FROM processes"))
        self.assertEqual([("one", "launch"), ("one", "prompt"), ("one", "start"), ("one", "close")],
                         FakeWorker.events)

    def test_unbridged_or_conflicting_cycle_is_not_claimed(self):
        self.configure()
        flat = dict(model="gpt-6-astra", reasoning_effort="high", worktree=str(self.root))
        self.prepared("flat", cycle_inputs=flat)
        conflicting = tmux.freeze_worker_inputs(flat, provider="openai", policy="--yolo")
        conflicting["model"] = "another-model"
        self.prepared("conflict", cycle_inputs=conflicting)
        self.assertEqual({"flat": "HOLD", "conflict": "HOLD"}, self.tick()["actions"])
        for action in self.c.snapshot()["actions"].values():
            self.assertEqual("prepared", action["status"])
            self.assertIsNone(action.get("claim"))
        self.assertEqual([], FakeWorker.events)
        self.assertEqual([], self.sql("SELECT * FROM operations"))
        self.assertEqual({"recorded_worker_runtime_required", "conflicting_worker_runtime"},
                         {row[0] for row in self.sql("SELECT reason_code FROM holds")})

    def test_non_worker_actions_remain_unclaimed_across_healthy_ticks(self):
        self.configure()
        for kind in ("wait", "ask_human", "integrate", "complete_sprint",
                     "prepare_successor", "verify_integration", "pause", "cancel"):
            self.prepared(kind, kind=kind)
        untouched = self.c.snapshot()["actions"]
        self.prepared()
        for _ in range(2):
            result = self.tick()
            self.assertEqual(("ACTIVE", {"one": "returned"}), (result["state"], result["actions"]))
            current = self.c.snapshot()["actions"]
            self.assertEqual(untouched, {key: current[key] for key in untouched})
            self.assertEqual([("one",)], self.sql("SELECT DISTINCT action_id FROM operations"))
            self.assertEqual([], self.sql("SELECT * FROM holds"))
        self.assertEqual([("one", "launch"), ("one", "prompt"), ("one", "start"), ("one", "close")],
                         FakeWorker.events)

    def test_wrong_ack_is_hard_hold_and_never_retried(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "wrong-ack"
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertIn(("wrong_ack",), self.sql("SELECT reason_code FROM holds"))
        self.assertEqual("dispatching", self.c.snapshot()["actions"]["one"]["status"])
        FakeWorker.modes.clear()
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertNotIn(("one", "start"), FakeWorker.events)

    def held_wrong_ack(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "wrong-ack"
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        FakeWorker.modes.clear()
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        action = self.c.snapshot()["actions"]["one"]
        run = Path(self.sql("SELECT private_run_root FROM processes")[0][0])
        # The fake launcher records no identity; give it a dead one (synthetic start times).
        f.write_json(run / "process.json", dict(pid=12345, start="synthetic", server=12346,
                                                server_start="synthetic"))
        request = dict(action_id="one", claim=action["claim"], allocation_digest=action["allocation_digest"],
                       reason_codes=["wrong_ack", "operation_held"], evidence="worker stopped with /quit")
        return request, run

    def test_resolve_hold_requires_exact_reasons_claim_evidence_and_stopped_worker(self):
        request, run = self.held_wrong_ack()
        holds = self.sql("SELECT * FROM holds ORDER BY hold_id")
        refusals = [(dict(request, reason_codes=["wrong_ack"]), "hold_reasons_mismatch"),
                    (dict(request, claim="other"), "resolution_claim_mismatch"),
                    (dict(request, allocation_digest="0" * 64), "resolution_claim_mismatch"),
                    (dict(request, evidence="  "), "resolution_evidence_required"),
                    (dict(request, action_id="missing"), "no_unresolved_hold"),
                    (dict(request, extra=True), "invalid_resolution")]
        for value, reason in refusals:
            with self.subTest(reason=reason):
                with self.assertRaisesRegex(f.LaunchError, reason):
                    owner.resolve_hold(self.config_path, value)
        # A listening socket or a live recorded process is not a stopped worker.
        import socket
        listener = socket.socket(socket.AF_UNIX)
        listener.bind(str(run / "s"))
        listener.listen(1)
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_stopped_worker"):
            owner.resolve_hold(self.config_path, request)
        listener.close()  # Leaves a stale socket file: no listener, connection refused.
        self.assertEqual("stale", owner.socket_state(run / "s"))
        me = os.getpid()
        start = tmux.processes([me])[me][3]
        identity = owner.load(run / "process.json")
        f.write_json(run / "owned.json", {str(me): start})
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_stopped_worker"):
            owner.resolve_hold(self.config_path, request)
        (run / "owned.json").unlink()
        # Missing or mismatched identity evidence fails closed.
        (run / "process.json").unlink()
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_process_identity"):
            owner.resolve_hold(self.config_path, request)
        f.write_json(run / "process.json", dict(identity, server_start=None))
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_process_identity"):
            owner.resolve_hold(self.config_path, request)
        f.write_json(run / "process.json", dict(identity, pid=54321))
        with self.assertRaisesRegex(f.LaunchError, "resolution_process_identity_mismatch"):
            owner.resolve_hold(self.config_path, request)
        f.write_json(run / "process.json", identity)
        self.sql("UPDATE operations SET phase='applied' WHERE effect='launch'")
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_held_operations"):
            owner.resolve_hold(self.config_path, request)
        self.sql("UPDATE operations SET phase='held' WHERE effect='launch'")
        moved = run.with_name(run.name + "-moved")
        run.rename(moved)
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_run_record"):
            owner.resolve_hold(self.config_path, request)
        moved.rename(run)
        self.assertEqual(holds, self.sql("SELECT * FROM holds ORDER BY hold_id"))
        self.assertEqual("dispatching", self.c.snapshot()["actions"]["one"]["status"])
        self.assertEqual({"held"}, {r[0] for r in self.sql("SELECT phase FROM operations")})

    def test_resolve_hold_records_evidence_fails_claim_and_is_never_resumed(self):
        request, _ = self.held_wrong_ack()
        path = self.root / "resolution.json"
        f.write_json(path, request)
        result = self.child(None, "--resolve-hold", str(path), "--config", str(self.config_path))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        receipt = json.loads(result.stdout)
        self.assertEqual(("RESOLVED", "dispatching", "failed"), (
            receipt["state"], receipt["action_status_before"], receipt["action_status_after"]))
        document = owner.load(self.root / receipt["artifact"])
        self.assertEqual(receipt["resolution_evidence_digest"], digest(document))
        self.assertEqual(request, document["request"])
        rows = self.sql("SELECT reason_code,resolution_evidence_digest,resolved_at FROM holds")
        self.assertEqual({"wrong_ack", "operation_held"}, {r[0] for r in rows})
        self.assertTrue(all(r[1] == digest(document) and r[2] for r in rows))
        self.assertEqual({"resolved"}, {r[0] for r in self.sql("SELECT phase FROM operations")})
        action = self.c.snapshot()["actions"]["one"]
        self.assertEqual("failed", action["status"])
        self.assertIn("owner_failure", action)
        self.assertEqual([], owner.activation_status(self.config_path)["unresolved_holds"])
        events = list(FakeWorker.events)
        tick = self.tick()
        self.assertEqual(("ACTIVE", "resolved"), (tick["state"], tick["actions"]["one"]))
        self.assertEqual(events, FakeWorker.events)
        with self.assertRaisesRegex(f.LaunchError, "no_unresolved_hold"):
            owner.resolve_hold(self.config_path, request)
        # A recurrence opens a new row; the resolved row is kept unchanged.
        kernel = owner.Kernel(self.config_path)
        kernel.db = sqlite3.connect(self.root / "owner.sqlite3")
        kernel.db.row_factory = sqlite3.Row
        op_id = digest(["tmux", "one", "claim"])
        kernel.hold(op_id, "wrong_ack")
        kernel.db.close()
        after = self.sql("SELECT reason_code,resolution_evidence_digest,resolved_at FROM holds")
        self.assertEqual(3, len(after))
        self.assertTrue(set(rows) <= set(after))
        self.assertEqual([("wrong_ack",)], self.sql("SELECT reason_code FROM holds WHERE resolved_at IS NULL"))

    def test_off_reconfigure_refuses_held_and_accepts_resolved_action(self):
        request, _ = self.held_wrong_ack()
        replacement = self.root / "replacement.json"
        f.write_json(replacement, self.config)
        f.write_file(self.root / "installation.lock", b"")
        f.write_json(self.root / "installation.json",
                     dict(phase="uninstalled", label="com.corbanu.initiative-owner.test",
                          domain=f"user/{os.getuid()}", pins=dict(config=str(self.config_path))))
        owner.disarm_owner(self.config_path, 1)
        with patch.object(owner, "service", return_value=("absent", "")):
            with self.assertRaisesRegex(f.LaunchError, "reconfigure_requires_settled_operations"):
                owner.reconfigure_owner(self.config_path, replacement, self.root)
            owner.resolve_hold(self.config_path, request)
            self.assertEqual("OFF", owner.reconfigure_owner(self.config_path, replacement, self.root)["state"])

    def test_missing_ack_waits_then_holds_at_fixed_lease(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "missing-ack"
        self.assertEqual("awaiting_ack", self.tick()["actions"]["one"])
        deadline = self.c.snapshot()["actions"]["one"]["deadline"]
        with patch.object(owner.time, "time", return_value=deadline + 1):
            self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertIn(("missing_ack",), self.sql("SELECT reason_code FROM holds"))
        self.assertNotIn(("one", "start"), FakeWorker.events)
        self.assertEqual(1, FakeWorker.events.count(("one", "prompt")))

    def test_start_keys_without_working_turn_never_counts_as_started(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "idle-start"
        self.assertEqual("awaiting_working", self.tick()["actions"]["one"])
        self.assertEqual([], self.sql("SELECT * FROM operations WHERE effect='working'"))
        self.assertEqual("awaiting_working", self.tick()["actions"]["one"])
        deadline = self.c.snapshot()["actions"]["one"]["deadline"]
        with patch.object(owner.time, "time", return_value=deadline + 1):
            self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertIn(("start_not_working",), self.sql("SELECT reason_code FROM holds"))
        self.assertEqual(1, FakeWorker.events.count(("one", "start")))

    def test_expired_lease_with_surviving_pid_is_dead_and_never_relaunched(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "working"
        self.assertEqual("working", self.tick()["actions"]["one"])
        deadline = self.c.snapshot()["actions"]["one"]["deadline"]
        with patch.object(owner.time, "time", return_value=deadline + 1):
            self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual([(12345, "lease_expired")],
                         self.sql("SELECT actual_pid,terminal_status FROM processes"))
        self.assertIn(("lease_expired",), self.sql("SELECT reason_code FROM holds"))
        self.assertEqual(1, FakeWorker.events.count(("one", "launch")))

    def test_crash_between_each_transport_effect_and_receipt_holds_on_reload(self):
        self.configure()
        for effect in ("launch", "prompt", "start"):
            with self.subTest(effect=effect):
                self.prepared(effect)
                FakeWorker.modes[effect] = "crash-" + effect
                with self.assertRaises(SystemExit):
                    self.tick()
                FakeWorker.modes.clear()
                events = list(FakeWorker.events)
                self.assertEqual("HOLD", self.tick()["actions"][effect])
                self.assertEqual(events, FakeWorker.events)
        self.assertEqual(3, len(self.sql("SELECT * FROM holds WHERE reason_code='effect_uncertain'")))

    def test_one_failing_action_does_not_abort_another(self):
        self.configure()
        self.prepared("one")
        self.prepared("two")
        FakeWorker.fail = ("one", "launch")
        self.assertEqual({"one": "HOLD", "two": "returned"}, self.tick()["actions"])
        self.assertEqual("returned", self.c.snapshot()["actions"]["two"]["status"])
        self.assertNotIn(("one", "prompt"), FakeWorker.events)

    def test_launch_receipt_persistence_failure_never_repeats_effect(self):
        self.configure()
        self.prepared()
        original = owner.artifact
        launch = digest(["tmux", "one", "launch"])
        def fail(root, relative, value):
            if relative == "runs/" + launch + "/receipt.json":
                raise OSError("synthetic disk full")
            return original(root, relative, value)
        with patch.object(owner, "artifact", side_effect=fail):
            self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual([("one", "launch")], FakeWorker.events)

    def test_claim_commit_gap_holds_without_adopting_or_launching(self):
        self.configure()
        self.prepared()
        original = owner.ExistingCoordinator.claim
        def crash(c, action, **kwargs):
            original(c, action, **kwargs)
            raise SystemExit(75)
        with patch.object(owner.ExistingCoordinator, "claim", crash):
            with self.assertRaises(SystemExit):
                self.tick()
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual([], FakeWorker.events)

    def test_receipt_after_domain_commit_gap_holds_without_duplicate_mutation(self):
        self.configure()
        self.prepared()
        original = owner.ExistingCoordinator.returned
        def crash(c, *args, **kwargs):
            original(c, *args, **kwargs)
            raise SystemExit(75)
        with patch.object(owner.ExistingCoordinator, "returned", crash):
            with self.assertRaises(SystemExit):
                self.tick()
        revision = self.c.snapshot()["revision"]
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual(revision, self.c.snapshot()["revision"])
        self.assertEqual("returned", self.c.snapshot()["actions"]["one"]["status"])

    def test_paused_owned_wrong_policy_worktree_and_activation_refuse(self):
        self.configure()
        self.prepared()
        self.c.set_enabled(False, {"fixture": True})
        result = self.tick()
        self.assertEqual(("READY", "deferred"), (result["state"], result["actions"]["one"]))
        self.c.set_enabled(True, {"fixture": True})
        self.c.event({"id": "own-manager"})
        run = self.c.begin_manager()
        result = self.tick()
        self.assertEqual(("READY", "deferred"), (result["state"], result["actions"]["one"]))
        self.c.fail_manager(run["manager_run"], "fixture")
        for scope in ("fixture-only", "live"):
            self.authority["scope"] = scope
            self.arm()
            with self.assertRaisesRegex(f.LaunchError, "activation_scope_mismatch"):
                self.tick()
        self.authority["scope"] = "tmux-workers"
        self.arm()
        self.prepared("bad-policy", policy="approve-everything")
        self.prepared("bad-worktree", worktree="/")
        result = self.tick()["actions"]
        self.assertEqual("HOLD", result["bad-policy"])
        self.assertEqual("HOLD", result["bad-worktree"])
        self.assertNotIn(("bad-policy", "launch"), FakeWorker.events)
        self.assertNotIn(("bad-worktree", "launch"), FakeWorker.events)

    def test_return_then_close_through_journal_with_audit_record(self):
        self.configure()
        self.prepared()
        self.assertEqual("returned", self.tick()["actions"]["one"])
        rows = self.sql("SELECT effect,receipt_artifact FROM operations ORDER BY created_at, rowid")
        self.assertEqual(["returned", "close"], [row[0] for row in rows][-2:])
        receipt = owner.load(self.root / rows[-1][1])
        self.assertEqual({"clean": True, "forced": False, "timeout": owner.CLOSE_TIMEOUT,
                          "server_shutdown": None}, receipt["result"])
        self.assertEqual([("closed",)], self.sql("SELECT terminal_status FROM processes"))
        self.assertEqual("returned", self.c.snapshot()["actions"]["one"]["status"])
        # Idempotent: a later tick neither closes again nor changes the coordinator.
        revision = self.c.snapshot()["revision"]
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "close")))
        self.assertEqual(revision, self.c.snapshot()["revision"])

    def test_restart_between_return_and_close_closes_on_next_tick(self):
        self.configure()
        self.prepared()
        with patch.object(owner.Kernel, "close_worker", lambda *args: None):
            self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertNotIn(("one", "close"), FakeWorker.events)
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "close")))
        self.assertEqual([("applied",)], self.sql("SELECT phase FROM operations WHERE effect='close'"))
        self.assertEqual([], self.sql("SELECT * FROM holds"))

    def test_unclean_or_uncertain_close_holds_and_is_never_retried(self):
        self.configure()
        self.prepared("unclean")
        FakeWorker.modes["unclean"] = "close-unclean"
        self.assertEqual("HOLD", self.tick()["actions"]["unclean"])
        self.assertEqual([("worker_close_unclean",)], self.sql("SELECT reason_code FROM holds"))
        self.assertEqual([("close_unclean",)], self.sql("SELECT terminal_status FROM processes"))
        self.assertEqual("returned", self.c.snapshot()["actions"]["unclean"]["status"])
        self.prepared("crash")
        FakeWorker.fail = ("crash", "close")
        self.assertEqual("HOLD", self.tick()["actions"]["crash"])
        FakeWorker.fail = None
        self.assertEqual("HOLD", self.tick()["actions"]["crash"])
        self.assertEqual(1, FakeWorker.events.count(("crash", "close")))
        self.assertEqual(1, FakeWorker.events.count(("unclean", "close")))

    def returned_without_close(self, key="one"):
        self.prepared(key)
        with patch.object(owner.Kernel, "close_worker", lambda *args: None):
            self.assertEqual("returned", self.tick()["actions"][key])
        return Path(self.sql("SELECT private_run_root FROM processes WHERE op_id=?",
                             (digest(["tmux", key, "launch"]),))[0][0])

    def rearm(self):
        self.authority["generation"] += 1
        self.arm()

    def test_prior_generation_return_is_history_only_when_closed_or_proven_stopped(self):
        self.configure()
        self.prepared("closed")
        self.assertEqual("returned", self.tick()["actions"]["closed"])
        run = self.returned_without_close("stopped")
        # Dead synthetic identity: no recorded process alive and no socket.
        f.write_json(run / "process.json", dict(pid=12345, start="synthetic", server=12346,
                                                server_start="synthetic"))
        self.returned_without_close("unproven")
        self.rearm()
        self.assertEqual({"closed": "returned", "stopped": "returned", "unproven": "HOLD"},
                         self.tick()["actions"])
        self.assertEqual([("prior_activation",)], self.sql("SELECT reason_code FROM holds"))
        # History is never closed or re-sent anything under the new activation.
        self.assertEqual([("closed", "close")], [e for e in FakeWorker.events if e[1] == "close"])
        self.assertEqual([("closed",)], self.sql("SELECT action_id FROM operations WHERE effect='close'"))

    def dead_identity(self, run):
        f.write_json(run / "process.json", dict(pid=12345, start="synthetic", server=12346,
                                                server_start="synthetic"))

    def test_proven_stopped_worker_gets_proof_not_keys(self):
        self.configure()
        run = self.returned_without_close()
        self.dead_identity(run)  # e.g. a reboot between RETURN and close
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertNotIn(("one", "close"), FakeWorker.events)
        receipt = owner.load(self.root / self.sql(
            "SELECT receipt_artifact FROM operations WHERE effect='close'")[0][0])["result"]
        self.assertTrue(receipt["clean"])
        self.assertEqual([], receipt["already_stopped"][0]["alive"])
        self.assertEqual([("closed",)], self.sql("SELECT terminal_status FROM processes"))

    def test_unprovable_stop_falls_back_to_a_real_close(self):
        self.configure()
        self.dead_identity(self.returned_without_close())
        with patch.object(tmux, "processes", side_effect=subprocess.TimeoutExpired("ps", 3)):
            self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "close")))
        self.assertEqual([], self.sql("SELECT * FROM holds"))

    def test_post_return_holds_resolve_without_changing_the_verdict(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "close-unclean"
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        packet = self.c.begin_manager()
        self.c.accept_decision(packet["manager_run"], {
            "state_revision": packet["state_revision"], "actions": [],
            "verdicts": [{"action": "one", "accepted": True, "reason": "fixture ok"}]}, {"fixture": True})
        action = self.c.snapshot()["actions"]["one"]
        request = dict(action_id="one", claim=action["claim"], allocation_digest=action["allocation_digest"],
                       reason_codes=["worker_close_unclean"], evidence="survivor stopped by its owner")
        run = Path(self.sql("SELECT private_run_root FROM processes")[0][0])
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_process_identity"):
            owner.resolve_hold(self.config_path, request)
        self.dead_identity(run)
        result = owner.resolve_hold(self.config_path, request)
        self.assertEqual(("RESOLVED", "accepted", "accepted"),
                         (result["state"], result["action_status_before"], result["action_status_after"]))
        self.assertEqual(action, self.c.snapshot()["actions"]["one"])
        self.assertEqual({"resolved"}, {row[0] for row in self.sql("SELECT phase FROM operations")})
        result = self.tick()
        self.assertEqual(("ACTIVE", "resolved"), (result["state"], result["actions"]["one"]))
        # A prepared (never returned) action still cannot use the post-return path.
        self.prepared("two")
        FakeWorker.modes["two"] = "wrong-ack"
        self.assertEqual("HOLD", self.tick()["actions"]["two"])
        two = self.c.snapshot()["actions"]["two"]
        self.dead_identity(Path(self.sql("SELECT private_run_root FROM processes WHERE op_id=?",
                                         (digest(["tmux", "two", "launch"]),))[0][0]))
        result = owner.resolve_hold(self.config_path, dict(
            action_id="two", claim=two["claim"], allocation_digest=two["allocation_digest"],
            reason_codes=["wrong_ack"], evidence="stopped"))
        self.assertEqual("failed", result["action_status_after"])

    def test_prior_generation_uncertain_close_still_holds(self):
        self.configure()
        self.prepared()
        FakeWorker.fail = ("one", "close")
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        FakeWorker.fail = None
        self.sql("UPDATE operations SET phase='intent' WHERE effect='close'")
        self.sql("UPDATE operations SET phase='applied' WHERE effect<>'close'")
        self.sql("UPDATE holds SET resolved_at=1, resolution_evidence_digest='fixture'")
        self.rearm()
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual(("effect_uncertain",), self.sql(
            "SELECT reason_code FROM holds WHERE resolved_at IS NULL")[0])
        self.assertEqual(1, FakeWorker.events.count(("one", "close")))

    def test_close_waits_while_paused_or_manager_owned(self):
        self.configure()
        self.returned_without_close()
        self.c.set_enabled(False, {"fixture": True})
        self.assertEqual("deferred", self.tick()["actions"]["one"])
        self.c.set_enabled(True, {"fixture": True})
        self.c.event({"id": "manager-wakes"})
        packet = self.c.begin_manager()
        self.assertEqual("deferred", self.tick()["actions"]["one"])
        self.assertNotIn(("one", "close"), FakeWorker.events)
        # The pre-effect gate refused: no close intent, nothing journaled as uncertain.
        self.assertEqual([], self.sql("SELECT phase FROM operations WHERE effect='close'"))
        self.c.fail_manager(packet["manager_run"], "fixture release")
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "close")))
        self.assertEqual([], self.sql("SELECT * FROM holds"))

    def test_close_after_manager_verdict_and_refused_without_recorded_return(self):
        self.configure()
        self.returned_without_close()
        packet = self.c.begin_manager()
        self.c.accept_decision(packet["manager_run"], {
            "state_revision": packet["state_revision"], "actions": [],
            "verdicts": [{"action": "one", "accepted": True, "reason": "fixture ok"}]}, {"fixture": True})
        self.assertEqual("accepted", self.c.snapshot()["actions"]["one"]["status"])
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "close")))
        kernel = owner.Kernel(self.config_path)
        kernel.db = sqlite3.connect(self.root / "owner.sqlite3")
        kernel.db.row_factory = sqlite3.Row
        self.addCleanup(kernel.db.close)
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"]["one"].pop("result")
        with self.assertRaisesRegex(f.LaunchError, "close_requires_recorded_return"):
            kernel.close_gate(self.c.snapshot()["actions"]["one"])

    def test_prior_activation_holds_inflight_worker(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "working"
        self.tick()
        self.authority["generation"] = 2
        self.arm()
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertIn(("prior_activation",), self.sql("SELECT reason_code FROM holds"))
        self.assertEqual(3, len(FakeWorker.events))

    def test_startup_wait_is_resumable_without_duplicate_launch(self):
        self.configure()
        self.prepared()
        FakeWorker.modes["one"] = "not-ready"
        self.assertEqual("awaiting_ready", self.tick()["actions"]["one"])
        self.assertEqual([("one", "launch")], FakeWorker.events)
        FakeWorker.modes.clear()
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "launch")))

    def test_invalid_or_dead_worker_and_inspection_error_isolate_actions(self):
        self.configure()
        for mode in ("invalid", "dead", "inspect-error"):
            self.prepared(mode)
            FakeWorker.modes[mode] = mode
        self.prepared("good")
        outcomes = self.tick()["actions"]
        self.assertEqual("returned", outcomes.pop("good"))
        self.assertEqual({"invalid": "HOLD", "dead": "HOLD", "inspect-error": "HOLD"}, outcomes)
        self.assertEqual([], [event for event in FakeWorker.events
                              if event[0] != "good" and event[1] == "start"])

    def test_receipt_before_owner_commit_resumes_without_relaunch(self):
        self.configure()
        self.prepared()
        original = owner.artifact
        launch = digest(["tmux", "one", "launch"])
        def crash(root, relative, value):
            result = original(root, relative, value)
            if relative == "runs/" + launch + "/receipt.json":
                raise SystemExit(73)
            return result
        with patch.object(owner, "artifact", side_effect=crash):
            with self.assertRaises(SystemExit):
                self.tick()
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(1, FakeWorker.events.count(("one", "launch")))

    def test_tampered_receipt_request_binding_and_claim_hold(self):
        self.configure()
        for key, kind in (("receipt", "receipt"), ("request", "request"),
                          ("binding", "binding"), ("claim", "claim")):
            self.prepared(key)
            FakeWorker.modes[key] = "working"
            self.tick()
            if kind in {"receipt", "request"}:
                relative = self.sql("SELECT " + kind + "_artifact FROM operations "
                                    "WHERE action_id=? AND effect='launch'", (key,))[0][0]
                path = self.root / relative
                f.write_json(path, {**owner.load(path), "tampered": True})
            elif kind == "binding":
                run = self.sql("SELECT private_run_root FROM processes WHERE op_id=?",
                               (digest(["tmux", key, "launch"]),))[0][0]
                path = Path(run) / "worker.json"
                meta = owner.load(path)
                meta["binding"]["effort"] = "low"
                f.write_json(path, meta)
            else:
                with self.c.mutation("fixture-claim-change", {}) as (_, state):
                    state["actions"][key]["claim"] = "wrong"
            self.assertEqual("HOLD", self.tick()["actions"][key])

    def test_pre_effect_pause_or_manager_ownership_defers_and_resumes_without_relaunch(self):
        self.configure()
        original = owner.artifact
        for mode in ("pause", "owned"):
            for effect in ("launch", "prompt", "start"):
                with self.subTest(mode=mode, effect=effect):
                    key = mode + "-" + effect
                    self.prepared(key)
                    op_id = digest(["tmux", key, effect])
                    manager = []
                    def defer(root, relative, value):
                        result = original(root, relative, value)
                        if relative == "runs/" + op_id + "/request.json":
                            if mode == "pause":
                                self.c.set_enabled(False, {"fixture": True})
                            else:
                                self.c.event({"id": "own-" + key})
                                manager.append(self.c.begin_manager()["manager_run"])
                        return result
                    with patch.object(owner, "artifact", side_effect=defer):
                        result = self.tick()
                    self.assertEqual(("READY", "deferred"), (result["state"], result["actions"][key]))
                    events = [event for event in FakeWorker.events if event[0] == key]
                    self.assertEqual([(key, name) for name in ("launch", "prompt", "start")
                                      [:("launch", "prompt", "start").index(effect)]], events)
                    self.assertEqual([("deferred",)], self.sql(
                        "SELECT phase FROM operations WHERE op_id=?", (op_id,)))
                    self.assertEqual("deferred", self.tick()["actions"][key])
                    self.assertEqual(events, [e for e in FakeWorker.events if e[0] == key])
                    self.assertEqual([], self.sql("SELECT * FROM holds"))
                    if manager:
                        self.c.fail_manager(manager[0], "fixture release")
                    else:
                        self.c.set_enabled(True, {"fixture": True})
                    self.assertEqual("returned", self.tick()["actions"][key])
                    self.assertEqual([(key, name) for name in ("launch", "prompt", "start", "close")],
                                     [e for e in FakeWorker.events if e[0] == key])
                    self.assertEqual([("applied",)], self.sql(
                        "SELECT DISTINCT phase FROM operations WHERE action_id=?", (key,)))

    def test_deferred_retry_crash_restores_uncertain_intent_and_never_relaunches(self):
        self.configure()
        self.prepared()
        original = owner.Kernel.worker_gate
        def pause(kernel, action):
            if kernel.db.execute("SELECT 1 FROM operations WHERE effect='launch' "
                                 "AND phase='intent'").fetchone():
                self.c.set_enabled(False, {"fixture": True})
            return original(kernel, action)
        with patch.object(owner.Kernel, "worker_gate", pause):
            self.assertEqual("deferred", self.tick()["actions"]["one"])
        self.assertEqual([], FakeWorker.events)
        self.c.set_enabled(True, {"fixture": True})
        FakeWorker.modes["one"] = "crash-launch"
        with self.assertRaises(SystemExit):
            self.tick()
        self.assertEqual([("intent",)], self.sql("SELECT phase FROM operations WHERE effect='launch'"))
        FakeWorker.modes.clear()
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual([("one", "launch")], FakeWorker.events)
        self.assertIn(("effect_uncertain",), self.sql("SELECT reason_code FROM holds"))

    def test_unowned_claim_and_expired_paused_worker_hold(self):
        self.configure()
        self.prepared("unowned")
        self.c.claim("unowned")
        self.assertEqual("HOLD", self.tick()["actions"]["unowned"])
        self.prepared()
        FakeWorker.modes["one"] = "working"
        self.tick()
        self.c.set_enabled(False, {"fixture": True})
        deadline = self.c.snapshot()["actions"]["one"]["deadline"]
        with patch.object(owner.time, "time", return_value=deadline + 1):
            self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual([("lease_expired",)], self.sql("SELECT terminal_status FROM processes"))

    def test_real_process_exit_after_launch_intent_preserves_uncertainty(self):
        self.configure()
        self.prepared()
        result = self.child(
            "import os,sys,owner_daemon as o,owner_tmux as t\n"
            "from test_owner_daemon import FakeWorker\n"
            "def launch(self):\n self.once('fake-launch', {'created':True})\n os._exit(73)\n"
            "FakeWorker.launch=launch\n"
            "t.Worker=FakeWorker\n"
            "o.Kernel(sys.argv[1]).tick()")
        self.assertEqual(73, result.returncode, result.stderr)
        self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual(1, len(list(self.root.glob("w-*/fake-launch.json"))))
        self.assertEqual([], FakeWorker.events)


def linked_worktree(path, head="d" * 40):
    """Shape of `git worktree add`: a .git file naming a gitdir whose HEAD is the
    detached commit (disposable) or a branch ref (an integration checkout)."""
    Path(path).mkdir(mode=0o700, exist_ok=True)
    gitdir = Path(str(path) + ".gitdir")
    gitdir.mkdir(mode=0o700, exist_ok=True)
    (gitdir / "HEAD").write_text(head + "\n")
    (Path(path) / ".git").write_text("gitdir: " + str(gitdir) + "\n")
    return path


class HandoffTests(unittest.TestCase):
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    sql = OwnerDaemonTests.sql
    arm = OwnerDaemonTests.arm
    child = OwnerDaemonTests.child
    tick = WorkerLifecycleTests.tick

    def prepared(self, key="one", kind="repair", cycle_inputs=None, base_commit="d" * 40, **changes):
        # Default routing needs the frozen base commit the disposable worktree is at.
        return WorkerLifecycleTests.prepared(self, key, kind, cycle_inputs, base_commit, **changes)

    def configure(self):
        # Default routing adopts only disposable worktrees; the fixture's is one.
        linked_worktree(self.root)
        WorkerLifecycleTests.configure(self)

    def test_default_route_adopts_only_disposable_worktrees(self):
        branch = linked_worktree(self.root / "integration", "ref: refs/heads/integrate/fixture")
        plain = self.root / "plain"
        plain.mkdir()
        main = self.root / "main-checkout"
        (main / ".git").mkdir(parents=True)
        moved = linked_worktree(self.root / "rebasing", "e" * 40)  # detached, other commit
        self.config["worktrees"] += [str(branch), str(plain), str(main), str(moved)]
        self.configure()
        self.prepared("first")
        self.transfer({"first": "hand"})
        for key, path in (("branch", branch), ("plain", plain), ("main", main), ("moved", moved),
                          ("disposable", self.root)):
            self.prepared(key, worktree=str(path))
        self.prepared("unbased", worktree=str(self.root), base_commit=None)
        self.assertEqual({"disposable": "returned"}, self.tick()["actions"])
        current = self.c.snapshot()
        for key in ("branch", "plain", "main", "moved", "unbased"):
            # Aimed at a branch checkout (such as the main integration worktree), a
            # plain directory or a main checkout: it stays on the hand lane, unheld.
            self.assertNotIn("dispatch_owner", current["actions"][key], key)
            self.assertEqual("prepared", current["actions"][key]["status"])
        self.assertEqual("owner", current["actions"]["disposable"]["dispatch_owner"])
        self.assertEqual([], self.sql("SELECT * FROM holds"))
        self.assertTrue(owner.disposable_worktree(self.root))
        self.assertFalse(owner.disposable_worktree(branch))
        self.assertFalse(owner.disposable_worktree(plain))
        self.assertFalse(owner.disposable_worktree(main))
        self.assertFalse(owner.disposable_worktree(moved, "d" * 40))
        (branch.parent / "integration.gitdir" / "HEAD").write_text("not-a-commit\n")
        self.assertFalse(owner.disposable_worktree(branch))

    def test_default_routed_worktree_is_rechecked_at_claim(self):
        self.configure()
        self.prepared("first")
        self.transfer({"first": "hand"})
        self.prepared("later")
        real = owner.Kernel.worker_action
        def moved(kernel, adapter, action):
            # Between routing and claim the worktree is switched to a branch.
            (Path(str(self.root) + ".gitdir") / "HEAD").write_text("ref: refs/heads/topic\n")
            return real(kernel, adapter, action)
        with patch.object(owner.Kernel, "worker_action", moved):
            self.assertEqual({"later": "HOLD"}, self.tick()["actions"])
        self.assertEqual("owner", self.c.snapshot()["actions"]["later"]["dispatch_owner"])
        self.assertEqual("prepared", self.c.snapshot()["actions"]["later"]["status"])
        self.assertEqual([("worktree_not_disposable",)], self.sql("SELECT reason_code FROM holds"))
        self.assertEqual([], FakeWorker.events)

    def transfer(self, choices, revision=None):
        state = self.c.snapshot()
        request = dict(expected_revision=state["revision"] if revision is None else revision,
                       evidence={"fixture": "cooperating dispatchers; no raw key senders"},
                       assignments={key: dict(
                           **{field: state["actions"][key].get(field)
                              for field in ("claim", "status", "allocation_digest")},
                           **{"from": self.c.dispatch_owner(state, state["actions"][key]), "to": side})
                           for key, side in choices.items()})
        return owner.handoff(self.config_path, request)

    def test_partition_is_atomic_and_routes_in_scope_new_actions_to_owner(self):
        self.configure()
        self.prepared("owned")
        self.prepared("manual")
        before = self.c.snapshot()
        with self.assertRaisesRegex(owner.Rejected, "partition every"):
            self.transfer({"owned": "owner"})
        self.assertEqual(before, self.c.snapshot())
        with self.assertRaisesRegex(owner.Rejected, "stale handoff revision"):
            self.transfer({"owned": "owner", "manual": "hand"}, revision=0)
        receipt = self.transfer({"owned": "owner", "manual": "hand"})
        self.assertEqual(before["revision"] + 1, receipt["control"]["revision"])
        self.prepared("later")
        self.prepared("outside", kind="integrate")
        with self.assertRaisesRegex(owner.Rejected, "wrong dispatch owner"):
            self.c.claim("owned")
        # Unassigned (default-hand) actions read as hand until an armed owner tick
        # routes the in-scope manager-prepared worker action, with an audit record.
        self.assertEqual("hand", self.c.dispatch_owner(self.c.snapshot(),
                                                       self.c.snapshot()["actions"]["later"]))
        self.assertEqual({"owned": "returned", "later": "returned"}, self.tick()["actions"])
        self.assertEqual("prepared", self.c.snapshot()["actions"]["manual"]["status"])
        self.assertEqual("prepared", self.c.snapshot()["actions"]["outside"]["status"])
        control = self.c.snapshot()["dispatch_control"]
        evidence = self.c.read_evidence(control["evidence"]["evidence_digest"])
        self.assertEqual(("armed_scope_default_route", ["later"], 1),
                         (evidence["kind"], evidence["actions"], evidence["generation"]))
        self.assertEqual({"manual": "returned"},
                         owner.Kernel(self.config_path, dispatcher="hand").tick()["actions"])
        self.assertEqual([], self.sql("SELECT * FROM holds"))
        observed = owner.activation_status(self.config_path)["coordinator"]["ownership"]
        self.assertEqual("owner", observed["owned"]["owner"])
        self.assertEqual("hand", observed["manual"]["owner"])
        self.assertEqual("owner", observed["later"]["owner"])
        self.assertEqual("hand", observed["outside"]["owner"])

    def test_default_route_keeps_out_of_scope_and_hand_work_hand(self):
        self.configure()
        other = self.root / "other-worktree"
        other.mkdir()
        self.prepared("first")
        self.transfer({"first": "hand"})
        self.prepared("unconfigured", worktree=str(other))
        self.prepared("claimed")
        self.c.claim("claimed")
        self.prepared("unbridged", cycle_inputs=dict(model="m", reasoning_effort="high",
                                                     worktree=str(self.root)))
        self.prepared("waiting", kind="wait")
        before = self.c.snapshot()
        # The hand dispatcher never routes.
        owner.Kernel(self.config_path, dispatcher="hand").tick()
        self.assertNotIn("dispatch_owner", self.c.snapshot()["actions"]["unconfigured"])
        # A paused coordinator routes nothing either.
        self.c.set_enabled(False, {"fixture": True})
        self.tick()
        self.assertNotIn("dispatch_owner", self.c.snapshot()["actions"]["unconfigured"])
        self.c.set_enabled(True, {"fixture": True})
        self.tick()
        current = self.c.snapshot()
        for key in ("unconfigured", "claimed", "unbridged", "waiting"):
            self.assertEqual("hand", self.c.dispatch_owner(current, current["actions"][key]), key)
            self.assertNotIn("dispatch_owner", current["actions"][key])
        self.assertEqual("hand", current["actions"]["first"]["dispatch_owner"])
        self.assertEqual(before["dispatch_control"], current["dispatch_control"])

    def test_default_route_never_adopts_hand_touched_work(self):
        self.configure()
        self.prepared("first")
        self.transfer({"first": "hand"})
        self.prepared("touched")
        self.prepared("unowned_run")
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"]["unowned_run"].pop("manager_run")
        real = owner.Kernel.worker_gate
        def refuse(kernel, action):
            if action["id"] == "touched":
                raise f.LaunchError("fixture_gate_refusal")
            return real(kernel, action)
        with patch.object(owner.Kernel, "worker_gate", refuse):
            result = owner.Kernel(self.config_path, dispatcher="hand").tick()
        self.assertEqual("HOLD", result["actions"]["touched"])
        self.assertEqual("prepared", self.c.snapshot()["actions"]["touched"]["status"])
        result = self.tick()
        self.assertEqual(("ACTIVE", {}), (result["state"], result["actions"]))
        current = self.c.snapshot()["actions"]
        self.assertNotIn("dispatch_owner", current["touched"])
        self.assertNotIn("dispatch_owner", current["unowned_run"])
        self.assertEqual([("fixture_gate_refusal",)], self.sql("SELECT reason_code FROM holds"))

    def test_default_route_admits_only_what_the_claim_gate_would_admit(self):
        self.configure()
        self.prepared("first")
        self.transfer({"first": "hand"})
        self.prepared("a")
        self.prepared("b")
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"]["b"]["resources"] = ["a"]  # same resource as a
        FakeWorker.modes["a"] = "working"
        result = self.tick()
        self.assertEqual(("ACTIVE", {"a": "working"}), (result["state"], result["actions"]))
        self.assertNotIn("dispatch_owner", self.c.snapshot()["actions"]["b"])
        FakeWorker.modes.clear()
        self.assertEqual({"a": "returned"}, self.tick()["actions"])
        # Returned is not terminal: b still contends, so it stays hand and unheld.
        self.assertNotIn("dispatch_owner", self.c.snapshot()["actions"]["b"])
        packet = self.c.begin_manager()
        self.c.accept_decision(packet["manager_run"], {
            "state_revision": packet["state_revision"], "actions": [],
            "verdicts": [{"action": "a", "accepted": True, "reason": "fixture ok"}]}, {"fixture": True})
        self.assertEqual({"a": "returned", "b": "returned"}, self.tick()["actions"])
        self.assertEqual([], self.sql("SELECT * FROM holds"))
        control = self.c.snapshot()["dispatch_control"]
        evidence = self.c.read_evidence(control["evidence"]["evidence_digest"])
        chain = self.c.read_evidence(evidence["previous_dispatch_control"]["evidence"]["evidence_digest"])
        self.assertEqual(["a"], chain["actions"])

    def test_default_route_skips_a_concurrent_revision_without_effect(self):
        self.configure()
        self.prepared("first")
        self.transfer({"first": "hand"})
        self.prepared("later")
        real = owner.ExistingCoordinator._handoff
        def stale(coordinator, assignments, expected_revision, evidence):
            return real(coordinator, assignments, expected_revision - 1, evidence)
        with patch.object(owner.ExistingCoordinator, "_handoff", stale):
            self.assertEqual({}, self.tick()["actions"])
        self.assertNotIn("dispatch_owner", self.c.snapshot()["actions"]["later"])
        self.assertEqual({"later": "returned"}, self.tick()["actions"])
        self.assertEqual("owner", self.c.snapshot()["actions"]["later"]["dispatch_owner"])

    def test_legacy_hand_claim_is_not_launched_held_or_watchdog_mutated(self):
        self.configure()
        self.prepared("manual")
        claim = self.c.claim("manual")
        self.transfer({"manual": "hand"})
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"]["manual"]["deadline"] = 1
        before = self.c.snapshot()["actions"]["manual"]
        self.assertEqual({}, self.tick()["actions"])
        self.assertEqual(before, self.c.snapshot()["actions"]["manual"])
        self.assertEqual([], self.sql("SELECT * FROM holds"))
        self.assertEqual([], FakeWorker.events)
        with self.assertRaisesRegex(f.LaunchError, "handoff_requires_receipted_claim"):
            self.transfer({"manual": "owner"})
        self.assertEqual(claim["claim"], self.c.snapshot()["actions"]["manual"]["claim"])

    def test_active_claim_round_trip_reuses_journal_without_duplicate_keys(self):
        self.configure()
        self.prepared("work")
        self.transfer({"work": "owner"})
        FakeWorker.modes["work"] = "missing-ack"
        self.assertEqual("awaiting_ack", self.tick()["actions"]["work"])
        claim = self.c.snapshot()["actions"]["work"]["claim"]
        self.transfer({"work": "hand"})
        self.assertEqual({}, self.tick()["actions"])
        self.assertEqual("awaiting_ack",
                         owner.Kernel(self.config_path, dispatcher="hand").tick()["actions"]["work"])
        self.transfer({"work": "owner"})
        FakeWorker.modes.clear()
        self.assertEqual("returned", self.tick()["actions"]["work"])
        self.assertEqual(claim, self.c.snapshot()["actions"]["work"]["claim"])
        self.assertEqual([("work", "launch"), ("work", "prompt"), ("work", "start"), ("work", "close")],
                         FakeWorker.events)

    def test_cutover_refuses_while_delivery_lock_held_and_preserves_claim(self):
        self.configure()
        self.prepared()
        before = self.c.snapshot()
        with owner.locked(self.root / "owner-daemon.lock"):
            with self.assertRaises(BlockingIOError):
                self.transfer({"one": "owner"})
        self.assertEqual(before, self.c.snapshot())

    def test_hand_hold_does_not_hold_owner_lane(self):
        self.configure()
        self.prepared("manual")
        self.prepared("owned")
        self.transfer({"manual": "hand", "owned": "owner"})
        FakeWorker.modes["manual"] = "wrong-ack"
        self.assertEqual("HOLD", owner.Kernel(self.config_path, dispatcher="hand").tick()["state"])
        result = self.tick()
        self.assertEqual(("ACTIVE", {"owned": "returned"}), (result["state"], result["actions"]))
        self.assertEqual([("wrong_ack",)], self.sql("SELECT reason_code FROM holds"))
        self.assertEqual("manual", result["unresolved_holds"][0]["action_id"])
        status = owner.activation_status(self.config_path)
        self.assertEqual("wrong_ack", status["unresolved_holds"][0]["reason_code"])
        self.assertTrue(status["unresolved_holds"][0]["excluded_from_owner_lane"])
        action = self.c.snapshot()["actions"]["manual"]
        with self.assertRaisesRegex(f.LaunchError, "resolution_requires_owner_action"):
            owner.resolve_hold(self.config_path, dict(
                action_id="manual", claim=action["claim"], allocation_digest=action["allocation_digest"],
                reason_codes=["wrong_ack"], evidence="hand-owned"))
        self.assertEqual([("wrong_ack",)], self.sql("SELECT reason_code FROM holds WHERE resolved_at IS NULL"))

    def test_hand_cannot_ack_or_return_owner_claim(self):
        self.configure()
        self.prepared()
        self.transfer({"one": "owner"})
        claim = self.c.claim("one", dispatcher="owner")
        with self.assertRaisesRegex(owner.Rejected, "wrong dispatch owner"):
            self.c.dispatched("one", claim["claim"], "agent", {"fixture": True})
        self.c.dispatched("one", claim["claim"], "agent", {"fixture": True}, dispatcher="owner")
        with self.assertRaisesRegex(owner.Rejected, "wrong dispatch owner"):
            self.c.acknowledge("one", "agent", claim["allocation_digest"], {"fixture": True})
        self.c.acknowledge("one", "agent", claim["allocation_digest"], {"fixture": True}, dispatcher="owner")
        with self.assertRaisesRegex(owner.Rejected, "wrong dispatch owner"):
            self.c.returned("one", "agent", {"fixture": True})
        with self.assertRaisesRegex(owner.Rejected, "wrong dispatch owner"):
            self.c.reconcile_dispatch("one", {"fixture": True})

    def test_reconfigure_requires_off_and_preserves_old_history(self):
        owner.arm_owner(self.config_path, self.authority)
        owner.Kernel(self.config_path).tick()
        replacement = self.root / "replacement.json"
        config = dict(self.config, manager_enabled=False)
        f.write_json(replacement, config)
        f.write_file(self.root / "installation.lock", b"")
        f.write_json(self.root / "installation.json",
                     dict(phase="uninstalled", label="com.corbanu.initiative-owner.test",
                          domain=f"user/{os.getuid()}", pins=dict(config=str(self.config_path))))
        with patch.object(owner, "service", return_value=("absent", "")):
            with self.assertRaisesRegex(f.LaunchError, "reconfigure_requires_off"):
                owner.reconfigure_owner(self.config_path, replacement, self.root)
            owner.disarm_owner(self.config_path, 1)
            before = self.sql("SELECT * FROM operations")
            result = owner.reconfigure_owner(self.config_path, replacement, self.root)
        self.assertEqual(2, result["generation"])
        self.assertEqual(before, self.sql("SELECT * FROM operations"))
        self.assertEqual(config, owner.load(self.config_path))
        with self.assertRaisesRegex(f.LaunchError, "owner_off"):
            owner.Kernel(self.config_path).tick()


class RecurrenceTests(unittest.TestCase):
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    sql = OwnerDaemonTests.sql
    arm = OwnerDaemonTests.arm

    def installation(self):
        import shutil
        from argparse import Namespace
        runtime = Path(self.tmp.name) / "runtime"
        runtime.mkdir(mode=0o700)
        for source in CLI.parent.glob("*.py"):
            if not source.name.startswith("test_"):
                target = runtime / source.name
                shutil.copyfile(source, target)
                target.chmod(0o600)
        schedule = Path(self.tmp.name) / "schedule"
        schedule.mkdir(mode=0o700)
        python = Path(sys.executable).resolve()
        return Namespace(owner="install", root=schedule, label="com.corbanu.initiative-owner.test-unit",
                         python=python, python_sha256=f.file_digest(python), runtime=runtime,
                         config=self.config_path, interval=2, publish_state=self.root, confirm_live=False)

    def install(self, args, loaded=None):
        import activate
        loaded = {} if loaded is None else loaded
        calls = []
        real_run = subprocess.run

        def run(argv, **kwargs):
            if argv[0] != "/bin/launchctl":
                return real_run(argv, **kwargs)
            calls.append(argv[1])
            if argv[1] == "bootstrap":
                loaded[argv[2]] = Path(argv[3])
            elif argv[1] == "bootout":
                loaded.pop(argv[2].rsplit("/", 1)[0], None)
            return subprocess.CompletedProcess(argv, 0)

        def service(label, domain=None):
            domain = domain or f"gui/{os.getuid()}"
            return ("present", f"path = {loaded[domain]}\n") if domain in loaded else ("absent", "")

        return patch.object(owner, "service", side_effect=service), patch.object(subprocess, "run", side_effect=run), calls

    def test_real_arming_requires_separate_schedule_hold_recovery(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            self.assertEqual("owner_run_refused", owner.scheduled_tick(args.root)["reason"])
            owner.arm_owner(self.config_path, self.authority)
            self.assertEqual("owner_run_refused", owner.scheduled_tick(args.root)["reason"])
            self.assertEqual([], self.sql("SELECT * FROM boots"))
            owner.scheduled_tick(args.root, recover="disposable arming verified")
            self.assertEqual("ACTIVE", owner.scheduled_tick(args.root)["state"])
            owner.disarm_owner(self.config_path, 1)
            result = owner.scheduled_tick(args.root)
            self.assertEqual("owner_run_refused", result["reason"])
            self.assertEqual("owner_off", result["refusal"])
            self.assertEqual("owner_off", owner.load(args.root / "tick.json")["refusal"])

    def test_explicit_off_only_schedule_repin_preserves_tick_history(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            owner.scheduled_tick(args.root)
            tick = owner.load(args.root / "tick.json")
            args.repin = True
            with self.assertRaisesRegex(f.LaunchError, "repin_requires_uninstalled"):
                activate.owner_activation(args)
            args.owner = "uninstall"
            activate.owner_activation(args)
            replacement = self.root / "replacement.json"
            f.write_json(replacement, dict(self.config, manager_enabled=False))
            owner.reconfigure_owner(self.config_path, replacement, args.root)
            args.owner = "install"
            args.repin = False
            with self.assertRaisesRegex(f.LaunchError, "installation_conflict"):
                activate.owner_activation(args)
            args.repin = True
            activate.owner_activation(args)
            self.assertEqual(tick, owner.load(args.root / "tick.json"))
            self.assertEqual("installed", owner.load(args.root / "installation.json")["phase"])

    def test_reconfigure_refuses_installed_missing_wrong_and_loaded_schedule(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        replacement = self.root / "replacement.json"
        f.write_json(replacement, dict(self.config, manager_enabled=False))
        before_config = self.config_path.read_bytes()
        before_meta = self.sql("SELECT * FROM meta")
        with service, command:
            with self.assertRaisesRegex(f.LaunchError, "reconfigure_requires_schedule"):
                owner.reconfigure_owner(self.config_path, replacement)
            activate.owner_activation(args)
            with self.assertRaisesRegex(f.LaunchError, "reconfigure_requires_uninstalled"):
                owner.reconfigure_owner(self.config_path, replacement, args.root)
            args.owner = "uninstall"
            activate.owner_activation(args)
            for presence in ("present", "unknown"):
                with patch.object(owner, "service", return_value=(presence, "")):
                    with self.assertRaisesRegex(f.LaunchError, "reconfigure_requires_absent_service"):
                        owner.reconfigure_owner(self.config_path, replacement, args.root)
            receipt = owner.load(args.root / "installation.json")
            receipt["pins"]["config"] = str(replacement)
            f.write_json(args.root / "installation.json", receipt)
            with self.assertRaisesRegex(f.LaunchError, "reconfigure_schedule_config_mismatch"):
                owner.reconfigure_owner(self.config_path, replacement, args.root)
        self.assertEqual(before_config, self.config_path.read_bytes())
        self.assertEqual(before_meta, self.sql("SELECT * FROM meta"))

    def test_busy_hand_dispatch_does_not_latch_or_count_as_admitted_success(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            owner.arm_owner(self.config_path, self.authority)
            with owner.locked(self.root / "owner-daemon.lock"):
                self.assertEqual("BUSY", owner.scheduled_tick(args.root)["state"])
            tick = owner.load(args.root / "tick.json")
            self.assertIsNone(tick["last_success"])
            self.assertIsNone(tick["hold"])
            self.assertEqual(1, tick["skipped"])
            self.assertEqual([], self.sql("SELECT * FROM boots"))
            self.assertEqual("ACTIVE", owner.scheduled_tick(args.root)["state"])

    def test_install_twice_and_uninstall_preserve_latch_and_remove_service(self):
        import activate
        import plistlib
        args = self.installation()
        service, command, calls = self.install(args)
        with service, command:
            activate.owner_activation(args)
            raw = (args.root / "installation.json").read_bytes()
            activate.owner_activation(args)
            self.assertEqual(raw, (args.root / "installation.json").read_bytes())
            self.assertEqual(["bootstrap", "kickstart"], calls)
            job = plistlib.loads((args.root / "owner.plist").read_bytes())
            self.assertNotRegex(repr(job), r"@(PINNED_PYTHON|PRIVATE_RUNTIME|PRIVATE_CONFIG|PRIVATE_LOGS)@")
            self.assertEqual(str(args.python), job["ProgramArguments"][0])
            self.assertIn("-S", job["ProgramArguments"])
            self.assertEqual(2, job["StartInterval"])
            self.assertLess(job["ThrottleInterval"], job["StartInterval"])
            self.assertEqual(30, job["ExitTimeOut"])
            self.assertEqual("HOLD", owner.scheduled_tick(args.root)["state"])
            args.owner = "uninstall"
            activate.owner_activation(args)
            activate.owner_activation(args)
            self.assertEqual(1, calls.count("bootout"))
            self.assertFalse((args.root / "owner.plist").exists())
            self.assertEqual("uninstalled", owner.load(args.root / "installation.json")["phase"])
            self.assertEqual("owner_run_refused", owner.load(args.root / "tick.json")["hold"])

    def test_installation_domain_controls_install_observation_and_uninstall(self):
        import activate
        import plistlib
        args = self.installation()
        for kind in ("gui", "user"):
            with self.subTest(domain=kind):
                args.domain, args.owner = kind, "install"
                args.root = Path(self.tmp.name) / kind
                args.root.mkdir(mode=0o700)
                domain = f"{kind}/{os.getuid()}"
                service, command, _ = self.install(args)
                with service as observed, command as commands:
                    activate.owner_activation(args)
                    receipt = owner.load(args.root / "installation.json")
                    self.assertEqual(domain, receipt["domain"])
                    job = plistlib.loads((args.root / "owner.plist").read_bytes())
                    self.assertEqual("Background" if kind == "user" else None,
                                     job.get("LimitLoadToSessionType"))
                    self.assertIn(["/bin/launchctl", "bootstrap", domain,
                                   str(args.root / "owner.plist")],
                                  [call.args[0] for call in commands.call_args_list])
                    owner.observe_schedule(args.root)
                    observed.assert_called_with(args.label, domain)
                    output = (f"path = {args.root / 'owner.plist'}\n\tstate = running\n"
                              f"\tpid = {os.getpid()}\n\timmediate reason = interval\n")
                    with patch.object(owner, "service", return_value=("present", output)) as probe:
                        self.assertEqual("interval", owner.firing_source(args.root, receipt))
                        probe.assert_called_once_with(args.label, domain)
                    args.domain = "user" if kind == "gui" else "gui"
                    with self.assertRaisesRegex(f.LaunchError, "installation_domain_conflict"):
                        activate.owner_activation(args)
                    args.owner = "uninstall"
                    activate.owner_activation(args)
                    self.assertIn(["/bin/launchctl", "bootout", f"{domain}/{args.label}"],
                                  [call.args[0] for call in commands.call_args_list])
                    observed.assert_called_with(args.label, domain)

    def test_fresh_install_refuses_same_label_in_either_domain_before_writes(self):
        import activate
        args = self.installation()
        for selected in ("gui", "user"):
            args.domain = selected
            for occupied in ("gui", "user"):
                domain = f"{occupied}/{os.getuid()}"
                loaded = {domain: Path(self.tmp.name) / "other.plist"}
                service, command, calls = self.install(args, loaded)
                with self.subTest(selected=selected, occupied=occupied), service, command:
                    with self.assertRaisesRegex(f.LaunchError, f"service_conflict: {domain}/{args.label}"):
                        activate.owner_activation(args)
                    self.assertEqual([], calls)
                    self.assertEqual(["installation.lock"], [p.name for p in args.root.iterdir()])

    def test_reinstall_refuses_new_conflict_without_changing_receipt(self):
        import activate
        args = self.installation()
        loaded = {}
        service, command, calls = self.install(args, loaded)
        with service, command:
            activate.owner_activation(args)
            before = (args.root / "installation.json").read_bytes()
            other = f"user/{os.getuid()}"
            loaded[other] = args.root / "owner.plist"
            with self.assertRaisesRegex(f.LaunchError, f"service_conflict: {other}/{args.label}"):
                activate.owner_activation(args)
            self.assertEqual(before, (args.root / "installation.json").read_bytes())
            self.assertEqual(["bootstrap", "kickstart"], calls)

    def test_uninstall_removes_owned_cross_domain_instances_including_orphan(self):
        import activate
        args = self.installation()
        for selected in ("gui", "user"):
            for orphan in (False, True):
                with self.subTest(selected=selected, orphan=orphan):
                    args.domain, args.owner = selected, "install"
                    args.root = Path(self.tmp.name) / f"{selected}-{orphan}"
                    args.root.mkdir(mode=0o700)
                    loaded = {}
                    service, command, calls = self.install(args, loaded)
                    with service, command:
                        activate.owner_activation(args)
                        if orphan:
                            loaded.clear()
                        other = "user" if selected == "gui" else "gui"
                        loaded[f"{other}/{os.getuid()}"] = args.root / "owner.plist"
                        args.owner = "uninstall"
                        activate.owner_activation(args)
                        activate.owner_activation(args)
                        self.assertEqual({}, loaded)
                        self.assertEqual(1 if orphan else 2, calls.count("bootout"))
                        self.assertFalse((args.root / "owner.plist").exists())
                        self.assertEqual("uninstalled", owner.load(args.root / "installation.json")["phase"])

    def test_uninstall_stale_publication_refuses_before_bootout(self):
        import activate
        args = self.installation()
        service, command, calls = self.install(args)
        with service, command:
            activate.owner_activation(args)
            self.arm()
            pending = self.root / "owner-recurrence.json.pending"
            f.write_file(pending, b"interrupted publication evidence")
            paths = [args.root / "installation.json", args.root / "owner.plist",
                     self.root / "owner.sqlite3", pending]
            before = {str(p): p.read_bytes() for p in paths}
            calls.clear()
            args.owner = "uninstall"
            with self.assertRaisesRegex(f.LaunchError, "stale_publication_pending"):
                activate.owner_activation(args)
            self.assertEqual([], calls)
            self.assertEqual(before, {str(p): p.read_bytes() for p in paths})
            self.assertEqual("armed", owner.activation_status(self.config_path)["state"])

    def test_uninstall_refuses_foreign_cross_domain_job_before_any_bootout(self):
        import activate
        args = self.installation()
        loaded = {}
        service, command, calls = self.install(args, loaded)
        with service, command:
            activate.owner_activation(args)
            before = (args.root / "installation.json").read_bytes()
            other = f"user/{os.getuid()}"
            loaded[other] = Path(self.tmp.name) / "other.plist"
            args.owner = "uninstall"
            with self.assertRaisesRegex(f.LaunchError, f"unowned_service: {other}/{args.label}"):
                activate.owner_activation(args)
            self.assertEqual(before, (args.root / "installation.json").read_bytes())
            self.assertTrue((args.root / "owner.plist").exists())
            self.assertEqual(2, len(loaded))
            self.assertNotIn("bootout", calls)

    def test_unavailable_cross_domain_observation_blocks_install_and_uninstall(self):
        import activate
        args = self.installation()
        service, command, calls = self.install(args)
        with service as observed, command:
            normal = observed.side_effect
            def unavailable(label, domain=None):
                if domain == f"user/{os.getuid()}":
                    raise subprocess.TimeoutExpired("launchctl", 5)
                return normal(label, domain)
            observed.side_effect = unavailable
            with self.assertRaisesRegex(f.LaunchError, f"service_observation_unavailable: user/{os.getuid()}"):
                activate.owner_activation(args)
            self.assertEqual([], calls)
            observed.side_effect = normal
            activate.owner_activation(args)
            before = (args.root / "installation.json").read_bytes()
            args.owner = "uninstall"
            observed.side_effect = unavailable
            with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                activate.owner_activation(args)
            self.assertEqual(before, (args.root / "installation.json").read_bytes())
            self.assertNotIn("bootout", calls)

    def test_missing_sibling_domain_allows_install_reinstall_and_uninstall(self):
        import activate
        args = self.installation()
        real_service = owner.service
        for kind in ("gui", "user"):
            with self.subTest(domain=kind):
                args.domain, args.owner = kind, "install"
                args.root = Path(self.tmp.name) / kind
                args.root.mkdir(mode=0o700)
                sibling = f"{'user' if kind == 'gui' else 'gui'}/{os.getuid()}"
                descriptor = "uid" if kind == "gui" else "user gui"
                diagnostic = f"Could not find domain for {descriptor}: {os.getuid()}"
                service, command, calls = self.install(args)
                with service as observed, command:
                    normal = observed.side_effect
                    def missing(label, domain=None):
                        if domain == sibling:
                            result = subprocess.CompletedProcess([], 112, "", f"Bad request.\n{diagnostic}\n")
                            with patch.object(subprocess, "run", return_value=result):
                                return real_service(label, domain)
                        return normal(label, domain)
                    observed.side_effect = missing
                    for action in ("install", "install", "uninstall"):
                        args.owner = action
                        activate.owner_activation(args)
                        receipt = owner.load(args.root / "installation.json")
                        self.assertEqual(dict(domain=sibling, state="domain_absent", reason=diagnostic),
                                         receipt["sibling_observation"])
                    self.assertEqual("uninstalled", receipt["phase"])
                    self.assertEqual(["bootstrap", "kickstart", "bootout"], calls)
                    self.assertFalse((args.root / "owner.plist").exists())

    def test_headless_recorded_transcript_distinguishes_gui_domain_and_user_service(self):
        label = "com.corbanu.initiative-owner.test-headless"
        with patch.object(os, "getuid", return_value=503), patch.object(
                subprocess, "run", return_value=subprocess.CompletedProcess(
                    [], 125, "", HEADLESS_GUI_STDERR)) as run:
            self.assertEqual(("domain_absent", HEADLESS_GUI_STDERR.rstrip("\n")),
                             owner.service(label, "gui/503"))
            self.assertEqual(["/bin/launchctl", "print", f"gui/503/{label}"],
                             run.call_args.args[0])
            with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                owner.service(label, "user/503")
        # The recorded probe used a non-owner label. Keep its bytes verbatim,
        # reject a mismatched label, then substitute only the queried label.
        with patch.object(os, "getuid", return_value=503), patch.object(
                subprocess, "run", return_value=subprocess.CompletedProcess(
                    [], 113, "", HEADLESS_USER_STDERR)) as run:
            with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                owner.service(label, "user/503")
            run.return_value.stderr = HEADLESS_USER_STDERR.replace("com.corbanu.absent", label)
            self.assertEqual(("absent", ""), owner.service(label, "user/503"))

    def test_headless_gui_sibling_allows_install_reinstall_and_uninstall(self):
        import activate
        args = self.installation()
        args.domain = "user"
        sibling = f"gui/{os.getuid()}"
        real_service = owner.service
        service, command, calls = self.install(args)
        with service as observed, command:
            normal = observed.side_effect
            def headless(label, domain=None):
                if domain == sibling:
                    result = subprocess.CompletedProcess([], 125, "", HEADLESS_GUI_STDERR)
                    with patch.object(subprocess, "run", return_value=result):
                        return real_service(label, domain)
                return normal(label, domain)
            observed.side_effect = headless
            for action in ("install", "install", "uninstall"):
                args.owner = action
                activate.owner_activation(args)
                receipt = owner.load(args.root / "installation.json")
                self.assertEqual(dict(domain=sibling, state="domain_absent",
                                      reason=HEADLESS_GUI_STDERR.rstrip("\n")),
                                 receipt["sibling_observation"])
            self.assertEqual("uninstalled", receipt["phase"])
            self.assertEqual(["bootstrap", "kickstart", "bootout"], calls)
            self.assertFalse((args.root / "owner.plist").exists())

    def test_selected_domain_vanishes_after_bootout_preserves_installation(self):
        import activate
        args = self.installation()
        service, command, calls = self.install(args)
        with service as observed, command:
            activate.owner_activation(args)
            before = (args.root / "installation.json").read_bytes()
            normal = observed.side_effect
            def vanished(label, domain=None):
                if domain == f"gui/{os.getuid()}" and "bootout" in calls:
                    return "domain_absent", HEADLESS_GUI_STDERR.rstrip("\n")
                return normal(label, domain)
            observed.side_effect = vanished
            args.owner = "uninstall"
            with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                activate.owner_activation(args)
            self.assertEqual(before, (args.root / "installation.json").read_bytes())
            self.assertTrue((args.root / "owner.plist").exists())
            self.assertEqual(["bootstrap", "kickstart", "bootout"], calls)

    def test_undecodable_service_output_refuses_install_and_uninstall(self):
        import activate
        args = self.installation()
        real_service = owner.service
        service, command, calls = self.install(args)
        with service as observed, command:
            normal = observed.side_effect
            def undecodable(label, domain=None):
                with patch.object(subprocess, "run", side_effect=UnicodeDecodeError(
                        "utf-8", bytes([255]), 0, 1, "invalid start byte")):
                    return real_service(label, domain)
            observed.side_effect = undecodable
            with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                activate.owner_activation(args)
            self.assertEqual([], calls)
            self.assertEqual(["installation.lock"], [p.name for p in args.root.iterdir()])
            observed.side_effect = normal
            activate.owner_activation(args)
            before = (args.root / "installation.json").read_bytes()
            args.owner = "uninstall"
            observed.side_effect = undecodable
            with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                activate.owner_activation(args)
            self.assertEqual(before, (args.root / "installation.json").read_bytes())
            self.assertTrue((args.root / "owner.plist").exists())
            self.assertNotIn("bootout", calls)

    def test_missing_selected_domain_refuses_before_install_writes(self):
        import activate
        args = self.installation()
        for kind in ("gui", "user"):
            args.domain = kind
            with self.subTest(domain=kind), patch.object(
                    owner, "service", return_value=("domain_absent", "missing")):
                with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                    activate.owner_activation(args)
                self.assertEqual(["installation.lock"], [p.name for p in args.root.iterdir()])

    def test_service_distinguishes_domain_absence_from_unreadable_domain(self):
        label = "com.corbanu.initiative-owner.test-domain"
        for kind, descriptor in (("gui", "user gui"), ("user", "uid")):
            domain = f"{kind}/{os.getuid()}"
            missing = f"Could not find domain for {descriptor}: {os.getuid()}"
            for code, stderr, expected in (
                    (112, f"Bad request.\n{missing}\n", ("domain_absent", missing)),
                    (113, f'Could not find service "{label}" in domain\n', ("absent", "")),
                    (1, "Could not print domain: 1: Operation not permitted\n", None),
                    (112, f"{missing}0\n", None),
                    (1, missing, None),
                    (112, "Bad request.\n", None),
                    (125, HEADLESS_GUI_STDERR,
                     ("domain_absent", HEADLESS_GUI_STDERR.rstrip("\n")) if kind == "gui" else None),
                    (125, HEADLESS_GUI_STDERR.replace("action", "action denied"), None),
                    (1, HEADLESS_GUI_STDERR, None),
                    (125, "Could not print domain: 125: Permission denied\n", None)):
                with self.subTest(domain=kind, code=code, stderr=stderr), patch.object(
                        subprocess, "run", return_value=subprocess.CompletedProcess([], code, "", stderr)):
                    if expected:
                        self.assertEqual(expected, owner.service(label, domain))
                    else:
                        with self.assertRaisesRegex(f.LaunchError, "service_observation_unavailable"):
                            owner.service(label, domain)

    def test_missing_domain_preserves_unknown_schedule_observation(self):
        args = self.installation()
        with patch.object(owner, "service", return_value=("domain_absent", "missing")):
            observed = owner.observe_schedule(args.root, args.label)
        self.assertEqual("unknown", observed["service"])
        self.assertEqual("observation-unavailable", observed["reason"])

    def test_invalid_label_is_not_remapped_to_observation_failure(self):
        import activate
        args = self.installation()
        args.label = "not-an-owner-label"
        with patch.object(subprocess, "run") as run:
            with self.assertRaisesRegex(f.LaunchError, "^invalid_label$"):
                activate.owner_activation(args)
            for label in (None, 42, "com.corbanu.initiative-owner/bad"):
                with self.assertRaisesRegex(f.LaunchError, "^invalid_label$"):
                    owner.service(label)
            run.assert_not_called()

    def test_uninstall_requires_verified_absence_in_other_domain_after_bootout(self):
        import activate
        args = self.installation()
        loaded = {}
        service, command, calls = self.install(args, loaded)
        with service as observed, command:
            activate.owner_activation(args)
            other = f"user/{os.getuid()}"
            loaded[other] = args.root / "owner.plist"
            normal = observed.side_effect
            def surviving(label, domain=None):
                if domain == other:
                    return "present", f"path = {args.root / 'owner.plist'}\n"
                return normal(label, domain)
            observed.side_effect = surviving
            args.owner = "uninstall"
            with self.assertRaisesRegex(f.LaunchError, f"service_still_present: {other}/{args.label}"):
                activate.owner_activation(args)
            self.assertEqual("installed", owner.load(args.root / "installation.json")["phase"])
            self.assertTrue((args.root / "owner.plist").exists())
            self.assertEqual(2, calls.count("bootout"))

    def test_legacy_gui_receipt_remains_observable_and_reinstallable(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service as observed, command:
            activate.owner_activation(args)
            receipt = owner.load(args.root / "installation.json")
            del receipt["domain"]
            f.write_json(args.root / "installation.json", receipt)
            activate.owner_activation(args)
            owner.observe_schedule(args.root)
            observed.assert_called_with(args.label, f"gui/{os.getuid()}")

    def test_service_targets_only_current_user_gui_or_user_domain(self):
        label = "com.corbanu.initiative-owner.test-domain"
        with patch.object(subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "ok")) as run:
            for kind in ("gui", "user"):
                domain = f"{kind}/{os.getuid()}"
                self.assertEqual(("present", "ok"), owner.service(label, domain))
                self.assertEqual(["/bin/launchctl", "print", f"{domain}/{label}"], run.call_args.args[0])
            run.reset_mock()
            for domain in ("system", f"user/{os.getuid() + 1}", "", None):
                with self.subTest(domain=domain), self.assertRaisesRegex(f.LaunchError, "invalid_domain"):
                    owner.installation_domain({"domain": domain})
            run.assert_not_called()

    def test_hold_probes_never_enter_kernel_until_explicit_recovery(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
        owner.scheduled_tick(args.root)
        before = owner.load(args.root / "tick.json")
        with patch.object(owner, "Kernel", side_effect=AssertionError("kernel entered")):
            self.assertEqual("HOLD", owner.scheduled_tick(args.root)["state"])
            self.assertEqual("HOLD", owner.scheduled_tick(args.root)["state"])
        after = owner.load(args.root / "tick.json")
        self.assertEqual(before["first_refusal"], after["first_refusal"])
        self.assertEqual(2, after["skipped"])
        self.arm()
        self.assertEqual("RECOVERED", owner.scheduled_tick(args.root, "fixture authority repaired")["state"])
        self.assertEqual("ACTIVE", owner.scheduled_tick(args.root)["state"])
        self.assertEqual(1, len(list((args.root / "recovery").glob("*.json"))))
        self.assertEqual(1, len(self.sql("SELECT * FROM boots")))

    def test_overlapping_and_interrupted_ticks_do_not_enter_kernel(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
        with owner.locked(args.root / "tick.lock"), self.assertRaises(BlockingIOError):
            owner.scheduled_tick(args.root)
        status = owner.load(args.root / "tick.json")
        status.update(started_at=1, completed_at=None)
        f.write_json(args.root / "tick.json", status)
        with patch.object(owner, "Kernel", side_effect=AssertionError("kernel entered")):
            self.assertEqual("interrupted_tick", owner.scheduled_tick(args.root)["reason"])

    def test_runtime_drift_latches_and_per_action_hold_does_not(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
        with patch.object(owner.Kernel, "tick", return_value={"state": "HOLD", "actions": {"one": "HOLD"}}):
            owner.scheduled_tick(args.root)
            owner.scheduled_tick(args.root)
        self.assertIsNone(owner.load(args.root / "tick.json")["hold"])
        (args.runtime / "owner_daemon.py").write_text("# changed\n")
        with patch.object(owner, "Kernel", side_effect=AssertionError("kernel entered")):
            self.assertEqual("owner_run_refused", owner.scheduled_tick(args.root)["reason"])

    def test_unsafe_runtime_python_and_conflicting_install_refused(self):
        import activate
        args = self.installation()
        service, command, calls = self.install(args)
        with service, command:
            args.python_sha256 = "0" * 64
            with self.assertRaisesRegex(f.LaunchError, "python_pin_mismatch"):
                activate.owner_activation(args)
            args.python_sha256 = f.file_digest(args.python)
            args.runtime.chmod(0o755)
            with self.assertRaises(f.LaunchError):
                activate.owner_activation(args)
            args.runtime.chmod(0o700)
            source = args.runtime / "owner_daemon.py"
            source.chmod(0o644)
            with self.assertRaises(f.LaunchError):
                activate.owner_activation(args)
            source.chmod(0o600)
            activate.owner_activation(args)
            args.interval = 3
            with self.assertRaisesRegex(f.LaunchError, "installation_conflict"):
                activate.owner_activation(args)
        self.assertEqual(["bootstrap", "kickstart"], calls)

    def test_bare_install_and_implicit_live_reinstall_have_no_effect(self):
        import activate
        args = self.installation()
        with patch.object(owner, "service", side_effect=AssertionError("service touched")):
            for label, reason in ((None, "explicit_label_required"),
                                  ("com.corbanu.initiative-owner", "live_confirmation_required")):
                args.label = label
                with self.assertRaisesRegex(f.LaunchError, reason):
                    activate.owner_activation(args)
                self.assertEqual([], list(args.root.iterdir()))
        args.confirm_live = True
        args.interval = 30
        service, command, calls = self.install(args)
        with service, command:
            activate.owner_activation(args)
            args.label = None
            with self.assertRaisesRegex(f.LaunchError, "explicit_label_required"):
                activate.owner_activation(args)
        self.assertEqual(["bootstrap", "kickstart"], calls)

    def test_transient_timeout_is_counted_published_and_retried_without_recovery(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            with patch.object(owner.Kernel, "tick", side_effect=subprocess.TimeoutExpired("ps", 2)):
                self.assertEqual(2, owner.main(["--run", "--schedule", str(args.root)]))
                self.assertEqual(2, owner.main(["--run", "--schedule", str(args.root)]))
            status = owner.load(args.root / "tick.json")
            self.assertIsNone(status["hold"])
            self.assertEqual((2, 2, "TimeoutExpired"), (status["errors"],
                             status["consecutive_errors"], status["last_error"]))
            published = owner.load(self.root / "owner-recurrence.json")
            self.assertEqual(2, published["errors"])
            self.arm()
            self.assertEqual(0, owner.main(["--run", "--schedule", str(args.root)]))
            status = owner.load(args.root / "tick.json")
            self.assertEqual((2, 0, None), (status["errors"],
                             status["consecutive_errors"], status["last_error"]))
            self.assertIsNone(status["previous_success"])
            owner.scheduled_tick(args.root)
            self.assertIsNone(owner.load(args.root / "tick.json")["previous_success"])

    def test_coordinator_refusal_latches_instead_of_retrying(self):
        import activate
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
        with patch.object(owner.Kernel, "tick", side_effect=owner.Rejected("state refused")) as tick:
            self.assertEqual("owner_run_refused", owner.scheduled_tick(args.root)["reason"])
            self.assertEqual("owner_run_refused", owner.scheduled_tick(args.root)["reason"])
            self.assertEqual(1, tick.call_count)

    def test_uninstalled_receipt_publishes_verified_absence(self):
        import activate
        import decision_feed
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            args.owner = "uninstall"
            activate.owner_activation(args)
            value = owner.load(self.root / "owner-recurrence.json")
            self.assertFalse(value["installed"])
            self.assertEqual("never-installed", decision_feed.owner_health(value, f.now())["state"])

    def test_only_two_attributed_interval_runs_prove_recurrence(self):
        import activate
        import decision_feed
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
        receipt = owner.load(args.root / "installation.json")
        base = f"\tpath = {args.root / 'owner.plist'}\n\tstate = running\n"
        at = __import__("time").time()
        self.arm()
        # Manual process cannot borrow an interval job's PID, and a pending
        # interval is not the immediate cause of a kickstart.
        for pid, reason, expected in (
                (os.getpid() + 1, "interval", "manual"),
                (os.getpid(), "non-ipc demand", "other"),
                (os.getpid(), "interval", "interval"),
                (os.getpid(), "new-unknown-reason", "other")):
            output = base + f"\tpid = {pid}\n\timmediate reason = {reason}\n\tpended nondemand spawn = interval\n"
            with self.subTest(expected=expected), patch.object(owner, "service", return_value=("present", output)):
                self.assertEqual(expected, owner.firing_source(args.root, receipt))
                for when in (at - 2, at):
                    with patch.object(owner.time, "time", return_value=when):
                        owner.scheduled_tick(args.root)
                status = owner.load(args.root / "tick.json")
                health = decision_feed.owner_health(owner.observe_schedule(args.root), f.now())
                self.assertEqual("recurring-at-observation" if expected == "interval" else "stalled", health["state"])
                self.assertEqual(at - 2 if expected == "interval" else None, status["previous_success"])
        with patch.object(owner, "service", side_effect=TimeoutError):
            self.assertEqual("unknown", owner.firing_source(args.root, receipt))
        with patch.object(owner, "service", return_value=("present", base + f"\tpid = {os.getpid()}\n\timmediate reason = interval\n")):
            receipt["plist_sha256"] = "0" * 64
            self.assertEqual("unknown", owner.firing_source(args.root, receipt))

    def test_uninstalled_refusal_remains_in_record_and_rendering(self):
        import activate
        import decision_feed
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            owner.scheduled_tick(args.root)
            before = owner.load(args.root / "tick.json")
            args.owner = "uninstall"
            activate.owner_activation(args)
            self.assertEqual(before, owner.load(args.root / "tick.json"))
            value = owner.load(self.root / "owner-recurrence.json")
            health = decision_feed.owner_health(value, f.now())
            self.assertEqual(("never-installed", "owner_run_refused"), (health["state"], health["reason"]))

    def test_publisher_failure_preserves_tick_outcome_and_projects_separate_history(self):
        import activate
        import decision_feed
        from contextlib import redirect_stdout
        from io import StringIO
        args = self.installation()
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
            self.arm()
            for refused in (False, True):
                failure = f.LaunchError("fixture refusal") if refused else None
                with patch.object(owner.Kernel, "tick", side_effect=failure, return_value={"state": "ACTIVE"}):
                    with patch.object(owner, "publish_schedule", side_effect=OSError("private detail")), redirect_stdout(StringIO()) as stdout:
                        self.assertEqual(2 if refused else 3, owner.main(["--run", "--schedule", str(args.root)]))
                result = json.loads(stdout.getvalue())
                self.assertEqual("HOLD" if refused else "ACTIVE", result["state"])
                self.assertEqual("owner_run_refused" if refused else None, result.get("reason"))
                self.assertEqual("OSError", result["publication_error"])
                self.assertTrue(result["publication_recorded"])
                self.assertNotIn("private detail", stdout.getvalue())
                status = owner.load(args.root / "tick.json")
                self.assertEqual("owner_run_refused" if refused else None, status["hold"])
                self.assertIsNotNone(status["last_success"])
                owner.publish_schedule(args.root)
                value = owner.load(self.root / "owner-recurrence.json")
                self.assertEqual(2 if refused else 1, value["publication_errors"])
                page = decision_feed.render(dict(schema=1, status="missing", feed=None, owner_recurrence=value), f.now(), [], {})
                self.assertIn("Publication failures:", page)
                self.assertIn("OSError", page)

    def test_publisher_and_local_recording_failure_do_not_relabel_success(self):
        from contextlib import redirect_stdout
        from io import StringIO
        with patch.object(owner, "scheduled_tick", return_value={"state": "ACTIVE"}), \
             patch.object(owner, "publish_schedule", side_effect=OSError), \
             patch.object(owner, "locked", side_effect=OSError), redirect_stdout(StringIO()) as stdout:
            self.assertEqual(3, owner.main(["--run", "--schedule", "/missing-fixture"]))
        result = json.loads(stdout.getvalue())
        self.assertEqual("ACTIVE", result["state"])
        self.assertNotIn("reason", result)
        self.assertFalse(result["publication_recorded"])

    def test_service_absence_is_verified_and_observation_failure_stays_unknown(self):
        root = Path(self.tmp.name)
        with patch.object(owner, "service", return_value=("absent", "")):
            value = owner.observe_schedule(root)
            self.assertEqual("absent", value["service"])
            self.assertFalse(value["installed"])
            (root / "installation.json").symlink_to(root / "missing-receipt")
            self.assertEqual("unknown", owner.observe_schedule(root)["service"])
            (root / "installation.json").unlink()
            f.write_json(root / "tick.json", {})
            self.assertEqual("unknown", owner.observe_schedule(root)["service"])
        with patch.object(owner, "service", side_effect=TimeoutError):
            value = owner.observe_schedule(root)
            self.assertEqual("unknown", value["service"])
            self.assertEqual("observation-unavailable", value["reason"])



CRASH_CHILD = """
import os, sys
from pathlib import Path
import fable_launcher as f
import manager_cycle
import owner_daemon as owner

def crash(**kwargs):
    import uuid
    c = owner.ExistingCoordinator(kwargs["state"])
    run = str(uuid.uuid4())
    launch = Path(kwargs["runs_dir"]) / ("m-" + run) / "launches" / "f-crash"
    launch.mkdir(mode=0o700, parents=True)
    c.begin_manager(priority=True, run_id=run)
    f.write_json(launch / "process.json", {"pid": os.getpid(), "pgid": os.getpgrp(),
                                           "started": f.processes()[os.getpid()][3]})
    os._exit(9)  # mid-cycle crash: no result, no completed tick

manager_cycle.run_cycle = crash
owner.scheduled_tick(Path(sys.argv[1]))
"""


class ManagerLaneTests(unittest.TestCase):
    """Automatic manager cycles: triggers, rate limits, audit, fail-closed holds."""
    setUp = OwnerDaemonTests.setUp
    tearDown = OwnerDaemonTests.tearDown
    sql = OwnerDaemonTests.sql
    arm = OwnerDaemonTests.arm
    installation = RecurrenceTests.installation
    install = RecurrenceTests.install

    def manager(self, arm=True, **changes):
        runs, vault = Path(self.tmp.name) / "mruns", Path(self.tmp.name) / "vault"
        for path in (runs, vault):
            path.mkdir(mode=0o700, exist_ok=True)
        binary = Path("/bin/echo").resolve()
        self.config["manager_cycle"] = dict(
            dict(binary=str(binary), binary_sha256=f.file_digest(binary), auth_vault_home=str(vault),
                 runs_dir=str(runs), timeout_seconds=300, min_interval_seconds=120, daily_cap=3), **changes)
        f.write_json(self.config_path, self.config)
        self.sql("UPDATE meta SET config_digest=?", (digest(self.config),))
        self.authority.update(config_digest=digest(self.config))
        if arm:
            self.arm()
        # The seed's never-used allocation would be an idle-wake candidate; keep it
        # as a template and retire it so each test controls its own triggers.
        self.template = dict(self.c.snapshot()["allocations"]["bootstrap"])
        with self.c.mutation("fixture", {}) as (_, state):
            state["allocations"]["bootstrap"] = dict(self.template, inputs={"consumed": True})
        self.schedule = Path(self.tmp.name) / "mschedule"
        self.schedule.mkdir(mode=0o700, exist_ok=True)
        self.clock, self.calls = 1_000_000.0, []
        self.runs = runs

    def lane(self, runner=None):
        return owner.manager_lane(self.config_path, self.schedule, clock=lambda: self.clock,
                                  runner=runner or self.accepting)

    def accepting(self, **kwargs):
        self.calls.append(kwargs)
        c = owner.ExistingCoordinator(kwargs["state"])
        packet = c.begin_manager(priority=True)
        c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": [],
                                                  "no_action_reason": "fixture"}, {"fixture": True})
        return {"status": "accepted", "manager_run": packet["manager_run"], "prepared_actions": [],
                "verdicts": [], "no_action_reason": "fixture"}

    def failing(self, phase, launcher="dead"):
        """run_cycle's shape: the cycle directory exists before the claim; a launch
        leaves launches/f-*/process.json (a live or a finished process)."""
        def runner(**kwargs):
            import uuid
            self.calls.append(kwargs)
            assert callable(kwargs["acceptance_gate"])
            run = str(uuid.uuid4())
            cycle = Path(kwargs["runs_dir"]) / ("m-" + run)
            cycle.mkdir(mode=0o700)
            owner.ExistingCoordinator(kwargs["state"]).begin_manager(priority=True, run_id=run)
            if phase != "briefing":
                self.launch_record(cycle, launcher)
            return {"status": "owner_hold", "phase": phase, "reason": "invalid_json",
                    "manager_run": run, "prepared_actions": []}
        return runner

    def launch_record(self, cycle, launcher):
        launch = cycle / "launches" / "f-fixture1"
        launch.mkdir(mode=0o700, parents=True, exist_ok=True)
        if launcher == "live":
            identity = {"pid": os.getpid(), "pgid": os.getpgrp(), "started": f.processes()[os.getpid()][3]}
        else:
            done = subprocess.run(["/usr/bin/true"])
            identity = {"pid": 2 ** 22 + 7, "pgid": 2 ** 22 + 7, "started": "finished process " + str(done.returncode)}
        if launcher != "none":
            f.write_json(launch / "process.json", identity)
        return launch

    def log(self):
        path = self.schedule / owner.MANAGER_LOG
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def returned(self, key, accepted=None):
        allocation = dict(self.template, resources=[key], kinds=["repair"])
        self.c.put_allocation(key, allocation, False, self.c.snapshot()["revision"], {"fixture": True})
        packet = self.c.begin_manager()
        self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": [{
            "id": key, "kind": "repair", "workstream": "delivery", "sprint": "PF80", "rationale": "fixture",
            "inputs": {"allocation": key, **allocation["inputs"]}, "expected_revision": packet["state_revision"],
            "timeout_seconds": 60}]}, {"fixture": True})
        claim = self.c.claim(key)
        self.c.dispatched(key, claim["claim"], "agent-" + key, {"native": True})
        self.c.acknowledge(key, "agent-" + key, claim["allocation_digest"], {"ack": True})
        self.c.returned(key, "agent-" + key, {"summary": "RETURN fixture"})
        if accepted is not None:
            self.c.verify(key, {"owner": "fixture"}, accepted)

    def test_configuration_validates_manager_settings(self):
        self.manager()
        owner.configuration(self.config_path)
        good = self.config["manager_cycle"]
        for change in ({"daily_cap": 0}, {"daily_cap": 289}, {"min_interval_seconds": 59},
                       {"timeout_seconds": 30}, {"timeout_seconds": True}, {"runs_dir": "relative"},
                       {"binary": str(self.root / "absent")}, {"binary_sha256": "short"}, {"extra": 1}):
            with self.subTest(change=change):
                f.write_json(self.config_path, dict(self.config, manager_cycle=dict(good, **change)))
                with self.assertRaises((f.LaunchError, OSError)):
                    owner.configuration(self.config_path)

    def test_off_without_settings_or_while_disarmed(self):
        self.schedule = Path(self.tmp.name) / "mschedule"
        self.schedule.mkdir(mode=0o700)
        self.arm()
        self.c.event({"id": "work"})
        self.calls, self.clock = [], 1.0
        self.assertEqual("OFF", self.lane()["state"])
        owner.disarm_owner(self.config_path, 1)
        self.manager(arm=False)
        self.c.event({"id": "more-work"})
        self.assertEqual(("OFF", "owner_off"), tuple(self.lane()[k] for k in ("state", "reason_off")))
        self.assertEqual([], self.calls)
        self.assertEqual([], self.log())

    def test_paused_owned_and_idle_start_nothing(self):
        self.manager()
        self.assertEqual("IDLE", self.lane()["state"])
        self.assertEqual([], self.log())
        self.c.event({"id": "work"})
        self.c.set_enabled(False, {"fixture": True})
        self.assertEqual("PAUSED", self.lane()["state"])
        self.c.set_enabled(True, {"fixture": True})
        run = self.c.begin_manager()["manager_run"]
        self.assertEqual("OWNED", self.lane()["state"])
        self.c.fail_manager(run, "fixture")
        self.assertEqual([], self.calls)
        self.assertEqual("CYCLE", self.lane()["state"])

    def test_pending_event_starts_one_audited_cycle_and_respects_the_interval(self):
        self.manager()
        self.c.event({"id": "work"})
        result = self.lane()
        self.assertEqual(("CYCLE", "accepted"), (result["state"], result["status"]))
        self.assertEqual(1, len(self.calls))
        call = self.calls[0]
        self.assertEqual((self.root, 300), (call["state"], call["timeout"]))
        context = owner.load(call["owner_context"])
        self.assertEqual(["pending_events"], context["context"]["triggers"])
        self.assertIn("no person started", context["context"]["source"])
        self.assertEqual(["cycle_started", "cycle_finished"], [r["event"] for r in self.log()])
        self.assertEqual("fixture", self.log()[1]["no_action_reason"])
        self.c.event({"id": "more"})
        self.clock += 119
        self.assertEqual("WAITING", self.lane()["state"])
        self.clock += 1
        self.assertEqual("CYCLE", self.lane()["state"])
        status = owner.load(self.schedule / owner.MANAGER_STATE)
        self.assertEqual((2, 2), (status["day_count"], status["cycles"]))

    def test_daily_cap_is_recorded_once_and_resets_next_day(self):
        self.manager()
        for index in range(3):
            self.c.event({"id": "work-%d" % index})
            self.assertEqual("CYCLE", self.lane()["state"])
            self.clock += 120
        self.c.event({"id": "over-cap"})
        for _ in range(3):
            self.assertEqual("CAPPED", self.lane()["state"])
            self.clock += 120
        self.assertEqual(1, [r["event"] for r in self.log()].count("daily_cap_reached"))
        self.assertEqual(3, len(self.calls))
        self.clock += 86400
        self.assertEqual("CYCLE", self.lane()["state"])

    def test_returned_result_awaiting_a_verdict_wakes_exactly_once(self):
        self.manager()
        self.returned("work")
        self.assertEqual("CYCLE", self.lane()["state"])  # its returned: event
        self.clock += 120
        # The manager consumed the event without a verdict; the owner wakes it once.
        result = self.lane()
        self.assertEqual("CYCLE", result["state"])
        context = owner.load(self.calls[-1]["owner_context"])["context"]
        self.assertEqual((["awaiting_verdict"], ["work"]), (context["triggers"], context["awaiting_verdict"]))
        self.clock += 120
        self.assertEqual("IDLE", self.lane()["state"])
        self.assertEqual(2, len(self.calls))
        self.assertEqual(1, [r["event"] for r in self.log()].count("wake"))
        # Still unanswered after the repeat window: one reminder, then quiet again.
        self.clock += owner.WAKE_REPEAT_SECONDS
        self.assertEqual("CYCLE", self.lane()["state"])
        self.clock += 120
        self.assertEqual("IDLE", self.lane()["state"])
        self.assertEqual(2, [r["event"] for r in self.log()].count("wake"))

    def test_unused_allocation_with_nothing_queued_wakes_once(self):
        self.manager()
        allocation = dict(self.template, resources=["fresh"])
        self.c.put_allocation("fresh", allocation, False, self.c.snapshot()["revision"], {"fixture": True})
        self.assertEqual("CYCLE", self.lane()["state"])  # its owner_allocation event
        self.clock += 120
        self.assertEqual("CYCLE", self.lane()["state"])
        self.assertEqual(["no_prepared_work"],
                         owner.load(self.calls[-1]["owner_context"])["context"]["triggers"])
        self.clock += 120
        self.assertEqual("IDLE", self.lane()["state"])

    def test_finished_failed_work_with_nothing_queued_wakes_once(self):
        self.manager()
        self.returned("work", accepted=False)
        self.assertEqual("CYCLE", self.lane()["state"])
        self.clock += 120
        self.assertEqual("CYCLE", self.lane()["state"])
        self.assertEqual(["no_prepared_work"],
                         owner.load(self.calls[-1]["owner_context"])["context"]["triggers"])
        self.clock += 120
        self.assertEqual("IDLE", self.lane()["state"])
        # Queued work suppresses the wake entirely.
        state = self.c.snapshot()
        self.assertEqual([], owner.idle_allocations(dict(state, actions=dict(
            state["actions"], queued=dict(state["actions"]["work"], id="queued", status="prepared")))))

    def test_accepted_work_is_compacted_by_the_lane_without_waking(self):
        self.manager()
        self.returned("work", accepted=True)
        self.clock = time.time()
        self.assertEqual("CYCLE", self.lane()["state"])
        self.clock += 120
        self.assertEqual("IDLE", self.lane()["state"])
        self.assertNotIn("consumed", self.c.snapshot()["allocations"]["work"]["inputs"])  # grace
        self.clock += owner.manager_cycle.COMPACT_ACCEPTED_AFTER
        self.assertEqual("IDLE", self.lane()["state"])
        self.assertIs(True, self.c.snapshot()["allocations"]["work"]["inputs"]["consumed"])
        self.assertIn("compacted", [r["event"] for r in self.log()])

    def test_failed_cycle_holds_and_releases_the_claim_only_with_launchers_proven_stopped(self):
        self.manager()
        for phase, launcher in (("briefing", None), ("validation", "dead")):
            self.c.event({"id": "work-" + phase})
            result = self.lane(self.failing(phase, launcher))
            self.assertEqual(("HOLD", "manager_cycle_failed", "invalid_json"),
                             (result["state"], result["reason"], result["refusal"]))
            self.assertIsNone(self.c.snapshot()["manager"])
            with self.c.connection() as db:
                reason = json.loads(db.execute("SELECT body FROM events WHERE id=?", (
                    "manager-failed:" + result["released_manager_run"],)).fetchone()[0])["reason"]
            self.assertIn("owner_auto_cycle_failed", reason)
            self.clock += 120
        result = self.lane(self.failing("launch", "live"))
        self.assertEqual("HOLD", result["state"])
        self.assertIsNone(result["released_manager_run"])
        self.assertIsNotNone(self.c.snapshot()["manager"])
        self.assertEqual("manager_launcher_running", self.log()[-1]["release_refusal"])

    def test_missing_launcher_record_is_not_proof_of_a_stopped_launcher(self):
        self.manager()
        self.c.event({"id": "work"})
        result = self.lane(self.failing("launch", "none"))
        run = self.c.snapshot()["manager"]
        self.assertIsNone(run)  # no record, no process naming it, socket absent: stopped
        self.c.event({"id": "more"})
        self.clock += 120
        holder = []
        def runner(**kwargs):
            result = self.failing("launch", "none")(**kwargs)
            launch = Path(kwargs["runs_dir"]) / ("m-" + result["manager_run"]) / "launches" / "f-fixture1"
            holder.append(subprocess.Popen(["/bin/sh", "-c", "sleep 30; : " + str(launch)]))
            time.sleep(0.2)
            return result
        try:
            result = self.lane(runner)
        finally:
            for process in holder:
                process.kill()
                process.wait()
        self.assertEqual("HOLD", result["state"])
        self.assertIsNotNone(self.c.snapshot()["manager"])
        self.assertEqual("manager_launcher_running", self.log()[-1]["release_refusal"])

    def test_disarm_during_a_cycle_refuses_acceptance(self):
        self.manager()
        self.c.event({"id": "work"})
        gates = []
        def runner(**kwargs):
            gates.append(kwargs["acceptance_gate"])
            owner.disarm_owner(self.config_path, 1)
            with self.assertRaisesRegex(f.LaunchError, "owner_off"):
                kwargs["acceptance_gate"]()
            return self.failing("acceptance", "dead")(**kwargs)
        self.assertEqual("HOLD", self.lane(runner)["state"])
        self.assertEqual(1, len(gates))

    def scheduled(self):
        import activate
        args = self.installation()
        args.lane = "manager"
        service, command, _ = self.install(args)
        with service, command:
            activate.owner_activation(args)
        receipt = owner.load(args.root / "installation.json")
        self.assertEqual(("manager", "gui/%d" % os.getuid()), (receipt["lane"], receipt["domain"]))
        self.schedule = args.root
        return args.root

    def test_manager_lane_install_requires_gui_and_a_dedicated_label(self):
        import activate
        self.manager()
        args = self.installation()
        args.lane, args.domain = "manager", "user"
        service, command, _ = self.install(args)
        with service, command, self.assertRaisesRegex(f.LaunchError, "manager_lane_requires_gui_label"):
            activate.owner_activation(args)

    def test_scheduled_failure_latches_until_evidenced_recovery(self):
        self.manager()
        root = self.scheduled()
        self.c.event({"id": "work"})
        with patch.object(owner.manager_cycle, "run_cycle", self.failing("launch", "live")):
            self.assertEqual("HOLD", owner.scheduled_tick(root)["state"])
            self.c.event({"id": "more-work"})
            self.assertEqual(("HOLD", "manager_cycle_failed"),
                             tuple(owner.scheduled_tick(root)[k] for k in ("state", "reason")))
        self.assertEqual(1, len(self.calls))
        run = self.c.snapshot()["manager"]["id"]
        # The launcher recorded for the failed cycle is still alive: recovery refuses.
        with self.assertRaisesRegex(f.LaunchError, "manager_launcher_running"):
            owner.scheduled_tick(root, recover="fixture: launcher stopped")
        self.assertEqual("manager_cycle_failed", owner.load(root / "tick.json")["hold"])
        self.assertEqual(run, self.c.snapshot()["manager"]["id"])
        self.launch_record(self.runs / ("m-" + run), "dead")
        self.assertEqual("RECOVERED", owner.scheduled_tick(root, recover="fixture: launcher stopped")["state"])
        self.assertIsNone(self.c.snapshot()["manager"])
        self.assertEqual("recovered", self.log()[-1]["event"])
        self.assertEqual(run, self.log()[-1]["reconciliation"]["manager_run"])
        status = owner.load(root / owner.MANAGER_STATE)
        f.write_json(root / owner.MANAGER_STATE, dict(status, last_started_at=0))
        with patch.object(owner.manager_cycle, "run_cycle", self.accepting):
            result = owner.scheduled_tick(root)
        self.assertEqual(("CYCLE", "accepted"), (result["state"], result["status"]))

    def test_crash_mid_cycle_holds_after_restart_and_recovers_with_evidence(self):
        self.manager()
        root = self.scheduled()
        self.c.event({"id": "work"})
        env = {"PATH": f.SAFE_PATH, "PYTHONDONTWRITEBYTECODE": "1", "HOME": str(self.root),
               "PYTHONPATH": str(CLI.parent)}
        crashed = subprocess.run([sys.executable, "-B", "-c", CRASH_CHILD, str(root)], env=env,
                                 capture_output=True, text=True, timeout=60)
        self.assertEqual(9, crashed.returncode, crashed.stderr)
        tick = owner.load(root / "tick.json")
        self.assertIsNotNone(tick["started_at"])
        self.assertIsNone(tick["completed_at"])
        run = self.c.snapshot()["manager"]["id"]
        # Restart: the next scheduled pass sees the interrupted cycle and holds,
        # without calling the manager again.
        with patch.object(owner.manager_cycle, "run_cycle", self.accepting):
            for _ in range(2):
                self.assertEqual(("HOLD", "interrupted_tick"),
                                 tuple(owner.scheduled_tick(root)[k] for k in ("state", "reason")))
        self.assertEqual([], self.calls)
        self.assertEqual(["cycle_started", "hold"], [r["event"] for r in self.log()])
        self.assertEqual("interrupted_tick", self.log()[-1]["reason"])
        result = owner.scheduled_tick(root, recover="fixture: crashed manager cycle; launcher pid gone")
        self.assertEqual("RECOVERED", result["state"])
        self.assertIsNone(self.c.snapshot()["manager"])
        reconciliation = self.log()[-1]["reconciliation"]
        self.assertEqual((run, "absent"), (reconciliation["manager_run"], reconciliation["launches"][0]["socket"]))
        status = owner.load(root / owner.MANAGER_STATE)
        f.write_json(root / owner.MANAGER_STATE, dict(status, last_started_at=0))
        with patch.object(owner.manager_cycle, "run_cycle", self.accepting):
            result = owner.scheduled_tick(root)
        self.assertEqual(("CYCLE", "accepted"), (result["state"], result["status"]))
        # The released run's failure event was part of that cycle's batch.
        with self.c.connection() as db:
            self.assertIsNotNone(db.execute("SELECT consumed FROM events WHERE id=?",
                                            ("manager-failed:" + run,)).fetchone()[0])

    def test_recover_without_a_hold_never_touches_the_coordinator(self):
        self.manager()
        root = self.scheduled()
        self.c.event({"id": "work"})
        run = self.c.begin_manager()["manager_run"]  # e.g. a manual cycle in progress
        self.assertEqual("RECOVERED", owner.scheduled_tick(root, recover="fixture: no hold")["state"])
        self.assertEqual(run, self.c.snapshot()["manager"]["id"])

    def test_lanes_publish_separately_and_keep_their_lane(self):
        import activate
        self.manager()
        root = self.scheduled()
        owner.publish_schedule(root)
        self.assertTrue((self.root / "manager-recurrence.json").exists())
        self.assertFalse((self.root / "owner-recurrence.json").exists())
        args = self.installation.__func__  # noqa: F841 (documentation)
        receipt = owner.load(root / "installation.json")
        f.write_json(root / "installation.json", dict(receipt, phase="uninstalled"))
        from argparse import Namespace
        again = Namespace(owner="install", root=root, label=receipt["label"], python=Path(receipt["pins"]["python"]),
                          python_sha256=receipt["pins"]["python_sha256"], runtime=Path(receipt["pins"]["runtime"]),
                          config=self.config_path, interval=2, publish_state=self.root, confirm_live=False,
                          lane="owner")
        service, command, _ = self.install(again)
        with service, command, self.assertRaisesRegex(f.LaunchError, "installation_lane_conflict"):
            activate.owner_activation(again)

    def test_binary_pin_mismatch_latches(self):
        self.manager(binary_sha256="0" * 64)
        root = self.scheduled()
        self.c.event({"id": "work"})
        result = owner.scheduled_tick(root)
        self.assertEqual(("HOLD", "owner_run_refused", "manager_binary_pin_mismatch"),
                         (result["state"], result["reason"], result["refusal"]))
        self.assertEqual("owner_run_refused", owner.load(root / "tick.json")["hold"])
        self.assertEqual([], self.calls)


if __name__ == "__main__":
    unittest.main()
