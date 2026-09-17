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

    def prepared(self, key="one", kind="repair", **runtime_changes):
        allocation = seed()[2]["bootstrap"]
        allocation["kinds"] = [kind]
        if kind == "complete_sprint":
            allocation["inputs"].update(receiving_action="fixture-receiver", receiving_commit="a" * 40,
                                        mandatory_gates=["fixture-gate"])
        allocation["resources"] = [key]
        allocation["inputs"]["worker"] = {
            "model": "fixture-model", "provider": "fixture", "effort": "high",
            "worktree": str(self.root), "policy": "--yolo", **runtime_changes}
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
        self.assertEqual([("one", "launch"), ("one", "prompt"), ("one", "start")], FakeWorker.events)

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
                    self.assertEqual([(key, name) for name in ("launch", "prompt", "start")],
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


if __name__ == "__main__":
    unittest.main()
