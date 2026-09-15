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
import owner_tmux as tmux
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

    def prepared(self, key="one", **runtime_changes):
        allocation = seed()[2]["bootstrap"]
        allocation["resources"] = [key]
        allocation["inputs"]["worker"] = {
            "model": "fixture-model", "provider": "fixture", "effort": "high",
            "worktree": str(self.root), "policy": "--yolo", **runtime_changes}
        self.c.put_allocation(key, allocation, False, self.c.snapshot()["revision"], {"fixture": True})
        self.c.event({"id": "prepare-" + key})
        packet = self.c.begin_manager()
        action = {"id": key, "kind": "repair", "workstream": "delivery", "sprint": "PF80",
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
        self.assertEqual([("one", "launch"), ("one", "prompt"), ("one", "start")], FakeWorker.events)
        self.assertEqual("returned", self.c.snapshot()["actions"]["one"]["status"])
        revision = self.c.snapshot()["revision"]
        self.assertEqual("returned", self.tick()["actions"]["one"])
        self.assertEqual(revision, self.c.snapshot()["revision"])
        self.assertEqual(3, len(FakeWorker.events))
        effects = dict(self.sql("SELECT effect,phase FROM operations"))
        self.assertEqual({"claim", "prepare", "launch", "prompt", "ack", "dispatched",
                          "acknowledged", "start", "working", "return_observed", "returned"}, set(effects))
        self.assertEqual({"applied"}, set(effects.values()))
        self.assertEqual([("work-turn",)], self.sql("SELECT active_turn_id FROM processes"))

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
        def crash(c, action):
            original(c, action)
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
        def crash(c, *args):
            original(c, *args)
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
            with self.assertRaisesRegex(f.LaunchError, "activation_authority_required"):
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

    def test_authority_rechecked_after_intent_before_launch(self):
        self.configure()
        self.prepared()
        original = owner.artifact
        launch = digest(["tmux", "one", "launch"])
        def pause(root, relative, value):
            result = original(root, relative, value)
            if relative == "runs/" + launch + "/request.json":
                self.c.set_enabled(False, {"fixture": True})
            return result
        with patch.object(owner, "artifact", side_effect=pause):
            self.assertEqual("HOLD", self.tick()["actions"]["one"])
        self.assertEqual([], FakeWorker.events)
        self.assertIn(("dispatch_paused_or_owned",), self.sql("SELECT reason_code FROM holds"))

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


if __name__ == "__main__":
    unittest.main()
