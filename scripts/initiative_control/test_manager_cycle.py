"""Offline driver tests: real SQLite and launcher-shaped artifacts; no inference."""

import contextlib
import io
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import uuid

from coordinator import Coordinator, encoded
import fable_launcher as f
import manager_cycle as m
from test_coordinator import seed


def fixture_launcher(args):
    """Shapes from run_launcher/evidence/Tui.stop, not fabricated production proof."""
    run = Path(tempfile.mkdtemp(prefix="f-", dir=args.runs_dir))
    for directory in ("packet", "home/sessions", "logs"):
        (run / directory).mkdir(mode=0o700, parents=True)
    raw = f.read_file(args.briefing, f.BRIEF_LIMIT, private=True)
    brief = f.strict_json(raw)
    revision = brief["state_revision"]
    decision = {"state_revision": revision, "actions": [{
        "id": "proposal-" + str(revision), "kind": "repair", "workstream": "delivery",
        "sprint": "PF80", "rationale": "Qualify the allocated bootstrap operation.",
        "inputs": {"allocation": "bootstrap", **brief["allocations"]["bootstrap"]["inputs"]},
        "timeout_seconds": 60, "expected_revision": revision}]}
    final = json.dumps(decision)
    sid, tid = str(uuid.uuid4()), str(uuid.uuid4())
    def record(kind, **payload):
        return {"type": kind, "payload": payload}
    records = [
        record("session_meta", id=tid, session_id=sid, cwd=str(run / "packet"),
               model_provider=f.PROVIDER, source="cli"),
        record("event_msg", type="task_started", turn_id="turn-1"),
        record("turn_context", turn_id="turn-1", cwd=str(run / "packet"), model=f.MODEL,
               model_provider=f.PROVIDER, effort=f.EFFORT, approval_policy="never",
               sandbox_policy={"type": "read-only"}),
        record("event_msg", type="model_response_completed", turn_id="turn-1", model=f.MODEL,
               model_provider_id=f.PROVIDER, response_id="response-1", finish_reason="end_turn"),
        record("event_msg", type="task_complete", turn_id="turn-1", last_agent_message=final)]
    rollout = run / "home/sessions/rollout.jsonl"
    f.write_file(rollout, "".join(json.dumps(r) + "\n" for r in records))
    f.write_file(run / "final.txt", final)
    f.write_file(run / "packet/briefing.md", f.INSTRUCTIONS + "\n\n" + json.dumps(
        brief, ensure_ascii=True, separators=(",", ":")) + "\n")
    f.write_file(run / "launcher.py", Path(f.__file__).read_bytes())
    f.write_file(run / "launch.sh", "#!/bin/sh\n# Offline fixture only; never executed.\n", 0o700)
    started = f.now()
    manifest = {"run_id": run.name, "started_at": started, "briefing_sha256": f.digest(raw),
                "briefing": brief, "binary": str(args.binary), "binary_version": "fixture 0",
                "binary_sha256": f.file_digest(args.binary), "argv": f.command(args.binary, run),
                "model": f.MODEL, "provider": f.PROVIDER, "effort": f.EFFORT}
    for field, name in (("packet_sha256", "packet/briefing.md"), ("launcher_sha256", "launcher.py"),
                        ("launch_sh_sha256", "launch.sh")):
        manifest[field] = f.file_digest(run / name)
    f.write_json(run / "manifest.json", manifest)
    f.write_json(run / "launch.json", {**{key: manifest[key] for key in ("binary", "binary_sha256", "argv")},
                                       "auth_file": str(args.auth_file), "auth_sha256": f.digest(b"fixture-only")})
    receipt = {"run_id": run.name, "status": "completed", "started_at": started,
               "model": f.MODEL, "provider": f.PROVIDER, "effort": f.EFFORT,
               "session_id": sid, "thread_id": tid, "turn_id": "turn-1", "response_id": "response-1",
               "decision": decision, "tmux_socket": str(run / "tmux.sock"), "tmux_session": "manager",
               "auth_stage": 8, "auth_source": "managed_subscription_token", "artifacts": {
                   "run_dir": str(run), "receipt": str(run / "receipt.json"),
                   "manifest": str(run / "manifest.json"), "private_logs": str(run / "logs"),
                   "final": str(run / "final.txt"), "private_rollout": str(rollout)}}
    f.write_json(run / "candidate.json", {**receipt, "status": "pending_shutdown"})
    f.write_json(run / "process.json", {"pid": 4242, "pgid": 4242, "started": "fixture process identity"})
    f.write_file(run / "stop", "stop\n")
    receipt.update(shutdown={"clean": True, "forced": False, "session_gone": True,
                             "errors": [], "owned_pids": [4242]}, finished_at=f.now())
    f.write_json(run / "receipt.json", receipt)
    return receipt


def rewrite_decision(receipt, change):
    change(receipt["decision"])
    run = Path(receipt["artifacts"]["run_dir"])
    final = json.dumps(receipt["decision"])
    candidate = m.load_json(run / "candidate.json")
    candidate["decision"] = receipt["decision"]
    f.write_json(run / "candidate.json", candidate)
    f.write_file(run / "final.txt", final)
    path, records = f.rollout(run)
    records[-1]["payload"]["last_agent_message"] = final
    f.write_file(path, "".join(json.dumps(r) + "\n" for r in records))


def invocation(root, launcher=fixture_launcher, **options):
    return m.run_cycle(state=root / "state", runs_dir=root / "runs", binary=root / "binary",
                       auth_file=root / "ABSENT-auth", owner_context=root / "context.json",
                       launcher=launcher, **options)


def hot_journal_child(directory):
    db = sqlite3.connect(directory / "coordinator.sqlite3", isolation_level=None)
    db.execute("PRAGMA journal_mode=DELETE")
    db.execute("PRAGMA synchronous=FULL")
    db.execute("PRAGMA cache_size=1")
    db.execute("BEGIN IMMEDIATE")
    db.execute("UPDATE state SET body='uncommitted crash fixture'")
    # Force dirty pages to disk with a synced rollback journal before abrupt exit.
    db.executemany("INSERT INTO evidence VALUES (?, ?)",
                   [(f"crash-{i}", "x" * 4096) for i in range(128)])
    os._exit(77)


def crash_child(root, boundary):
    original = f.write_json
    def write(path, value):
        original(path, value)
        if path.name == boundary:
            os._exit(71)
    def launch(args):
        if boundary == "during-launch":
            os._exit(71)
        return fixture_launcher(args)
    accept = Coordinator.accept_decision
    def accepted(self, *args):
        accept(self, *args)
        os._exit(71)
    with patch.object(f, "write_json", write):
        if boundary == "after-commit":
            with patch.object(Coordinator, "accept_decision", accepted):
                invocation(root, launch)
        else:
            invocation(root, launch)


class CycleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve())
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "runs").mkdir(mode=0o700)
        f.write_file(self.root / "binary", "offline fixture; never execute", 0o700)
        f.write_json(self.root / "context.json", {"observed_at": "2026-09-13T00:00:00Z",
                     "context": {"authority": "bounded fixture only", "unresolved": ["gate still open"]}})
        self.c = Coordinator(self.root / "state")
        self.c.initialize(*seed())
        self.c.set_enabled(True, {"authority": "fixture"})
        self.c.event({"id": "event-1", "text": "original reply and result evidence"})
        self.calls = 0

    def launch(self, args):
        self.calls += 1
        return fixture_launcher(args)

    def cycle(self, launcher=None):
        with patch.object(f, "auth_token", side_effect=AssertionError("no credentials")):
            return invocation(self.root, launcher or self.launch)

    def pending(self):
        with self.c.connection() as db:
            return db.execute("SELECT id FROM events WHERE consumed IS NULL AND meaningful=1").fetchall()

    def assert_hold(self, result):
        self.assertEqual("owner_hold", result["status"], result)
        self.assertIsNotNone(self.c.snapshot()["manager"])
        self.assertTrue(self.pending())
        self.assertEqual({}, self.c.snapshot()["actions"])

    def test_success_prepares_without_dispatch_and_preserves_full_brief(self):
        result = self.cycle()
        self.assertEqual("accepted", result["status"], result)
        self.assertEqual(1, self.calls)
        self.assertEqual("prepared", result["prepared_actions"][0]["status"])
        self.assertNotIn("claim", result["prepared_actions"][0])
        self.assertIsNone(self.c.snapshot()["manager"])
        self.assertFalse(self.pending())
        cycle = Path(result["artifacts"])
        brief = m.load_json(cycle / "briefing.json")
        self.assertEqual(3, len(brief["workstreams"]))
        self.assertEqual(3, len(brief["last_three_actions"]))
        self.assertIn("historical", brief["seed_metadata_status"])
        self.assertIn("under 300 UTF-8 bytes", brief["directive"])
        self.assertIn("1000-byte", brief["directive"])
        self.assertEqual([], brief["evidence_omissions"])
        self.assertEqual("original reply and result evidence", next(iter(brief["original_evidence"].values()))["text"])
        self.assertEqual("empty", self.cycle()["status"])
        self.assertEqual(1, self.calls)
        self.assertEqual(0o600, (cycle / "briefing.json").stat().st_mode & 0o777)

    def test_paused_empty_owned_do_not_read_inputs_or_mutate(self):
        for mode in ("paused", "owned", "empty"):
            with self.subTest(mode=mode):
                if mode == "paused":
                    self.c.set_enabled(False, {"pause": "owner"})
                elif mode == "owned":
                    self.c.set_enabled(True, {"resume": "fixture"})
                    self.c.begin_manager()
                else:
                    self.c.fail_manager(self.c.snapshot()["manager"]["id"], "fixture reconciliation")
                    with self.c.connection() as db:
                        db.execute("UPDATE events SET meaningful=0")
                before = self.c.path.read_bytes()
                with patch.object(m, "briefing", side_effect=AssertionError), patch.object(f, "run_launcher", side_effect=AssertionError):
                    result = m.run_cycle(state=self.c.directory, runs_dir=None, binary=None,
                                         auth_file=None, owner_context=None)
                self.assertEqual(mode, result["status"])
                self.assertEqual(before, self.c.path.read_bytes())
                self.assertEqual([], list((self.root / "runs").iterdir()))

    def test_off_help_import_and_missing_state_never_initialize(self):
        for argv in ([], ["--state", str(self.root / "absent")], ["--help"]):
            with contextlib.redirect_stdout(io.StringIO()), patch.object(m, "ExistingCoordinator", side_effect=AssertionError):
                if "--help" in argv:
                    with self.assertRaises(SystemExit) as exc:
                        m.main(argv)
                    self.assertEqual(0, exc.exception.code)
                else:
                    self.assertEqual(0, m.main(argv))
        result = m.run_cycle(state=self.root / "absent", runs_dir=None, binary=None, auth_file=None, owner_context=None)
        self.assertEqual("owner_hold", result["status"])
        self.assertFalse((self.root / "absent").exists())
        proc = subprocess.run([sys.executable, "-B", "-c", "import manager_cycle"],
                              cwd=Path(__file__).parent, capture_output=True, text=True)
        self.assertEqual((0, "", ""), (proc.returncode, proc.stdout, proc.stderr))

    def test_nested_evidence_and_last_three_actions(self):
        with self.c.connection() as db:
            reference = self.c._reference(db, {"original": "z" * 800})
        self.c.event({"id": "nested", "result": reference})
        for index in range(4):
            with self.c.mutation("fixture", {}) as (_, state):
                state["actions"][str(index)] = {"id": str(index), "workstream": "delivery",
                    "status": "accepted", "sequence": [index, 0], "result": reference}
        result = self.cycle()
        self.assertEqual("accepted", result["status"], result)
        brief = m.load_json(Path(result["artifacts"]) / "briefing.json")
        self.assertEqual(["1", "2", "3"], [a["id"] for a in brief["last_three_actions"]["delivery"]])
        self.assertEqual("z" * 800, brief["original_evidence"][reference["evidence_digest"]]["original"])
        self.assertNotIn("preview", brief["actions"]["3"]["result"])
        claim = m.load_json(Path(result["artifacts"]) / "claim.json")
        for key, action in brief["actions"].items():
            restored = dict(action)
            ref = restored.get("result")
            if ref:
                restored["result"] = {**ref, "preview": encoded(brief["original_evidence"][ref["evidence_digest"]])[:400]}
            self.assertEqual(claim["actions"][key], restored)
        for event, original in zip(brief["events"], claim["events"]):
            self.assertEqual(original, {**event, "preview": encoded(brief["original_evidence"][event["evidence_digest"]])[:400]})

    def test_frozen_inputs_that_resemble_references_are_not_rewritten(self):
        with self.c.connection() as db:
            ref = self.c._reference(db, {"original": "full body"})
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"]["prior"] = {"id": "prior", "workstream": "delivery", "status": "accepted",
                                          "sequence": [0, 0], "inputs": {"data": ref},
                                          "result": {**ref, "extra": "retain unknown shape"}}
        result = self.cycle()
        self.assertEqual("accepted", result["status"], result)
        brief = m.load_json(Path(result["artifacts"]) / "briefing.json")
        self.assertEqual({"data": ref}, brief["actions"]["prior"]["inputs"])
        self.assertEqual({**ref, "extra": "retain unknown shape"}, brief["actions"]["prior"]["result"])

    def test_large_actions_are_losslessly_indexed_without_duplicate_records(self):
        expected = {}
        for index in range(3):
            action = {"id": "prior-" + str(index), "workstream": "delivery",
                      "status": "accepted", "sequence": [index, 0],
                      "full_frozen_input": str(index) * 11000}
            expected[action["id"]] = action
        with self.c.mutation("fixture", {}) as (_, state):
            state["actions"].update(expected)
        result = self.cycle()
        self.assertEqual("accepted", result["status"], result)
        cycle = Path(result["artifacts"])
        claim = m.load_json(cycle / "claim.json")
        self.assertGreater(len(encoded(claim, limit=240000).encode()), f.BRIEF_LIMIT)
        brief = m.load_json(cycle / "briefing.json")
        self.assertLessEqual(len(encoded(brief).encode()), f.BRIEF_LIMIT)
        refs = brief["last_three_actions"]["delivery"]
        self.assertEqual([{"id": key} for key in expected], refs)
        self.assertEqual(list(expected.values()), [brief["actions"][r["id"]] for r in refs])
        self.assertEqual([], brief["evidence_omissions"])
        self.assertIn("lossless references", brief["directive"])

    def test_mismatched_last_action_reference_holds_before_launch(self):
        packet = self.c.begin_manager()
        packet["last_three_actions"]["delivery"] = [{"id": "missing", "detail": "must not disappear"}]
        with self.assertRaisesRegex(f.LaunchError, "last_action_reference_mismatch"):
            m.briefing(self.c, packet, self.root / "context.json")
        self.assertEqual(0, self.calls)

    def test_missing_evidence_holds_before_launch(self):
        self.c.event({"id": "missing", "result": {"evidence_digest": "a" * 64}})
        result = self.cycle()
        self.assert_hold(result)
        self.assertEqual("missing_original_evidence", result["reason"])
        self.assertEqual(0, self.calls)

    def test_oversized_original_holds_before_launch(self):
        self.c.event({"id": "large", "text": "é" * 20000})
        result = self.cycle()
        self.assert_hold(result)
        self.assertEqual("briefing_size_hold", result["reason"])
        self.assertEqual(0, self.calls)
        self.assertTrue((Path(result["artifacts"]) / "claim.json").exists())

    def test_context_must_be_private_bounded_and_dated(self):
        path = self.root / "context.json"
        for value in ({"observed_at": "not a date", "context": {"reason": "fixture"}},
                      {"observed_at": f.now(), "context": {"text": "x" * 8192}}):
            f.write_json(path, value)
            self.assert_hold(self.cycle())
            self.c.fail_manager(self.c.snapshot()["manager"]["id"], "no launcher started")
        path.chmod(0o644)
        self.assert_hold(self.cycle())
        self.assertEqual(0, self.calls)

    def test_stale_revision_is_transactionally_rejected(self):
        def launch(args):
            receipt = self.launch(args)
            self.c.event({"id": "new-input"})
            return receipt
        self.assert_hold(self.cycle(launch))
        self.assertEqual(1, self.calls)

    def test_pause_during_launch_retains_ownership(self):
        def launch(args):
            receipt = self.launch(args)
            self.c.set_enabled(False, {"pause": "during inference"})
            return receipt
        self.assert_hold(self.cycle(launch))

    def test_elapsed_deadline_never_clears_ownership(self):
        def launch(args):
            receipt = self.launch(args)
            with self.c.connection() as db:
                state = self.c.snapshot()
                state["manager"]["deadline"] = 0
                db.execute("UPDATE state SET body=?", (encoded(state),))
            return receipt
        self.assert_hold(self.cycle(launch))
        self.assertEqual("owned", self.cycle()["status"])
        self.assertEqual(1, self.calls)

    def timed_cycle(self, timeout, prep, overhead=17):
        tick = [m.time.time()]
        start = tick[0]
        original_brief, original_validate = m.briefing, m.validate
        def brief(*args):
            tick[0] += prep
            return original_brief(*args)
        def launch(args):
            run = self.c.snapshot()["manager"]
            self.assertAlmostEqual(start + timeout, run["deadline"])
            self.assertAlmostEqual(timeout - prep - 30, args.timeout)
            self.assertTrue(1 <= args.timeout <= 3570)
            tick[0] += 5  # Launcher setup/version probe before its own timer.
            tick[0] += args.timeout - 0.01  # Completion near its real window end.
            receipt = self.launch(args)
            tick[0] += overhead - 9  # Shutdown after completion.
            return receipt
        def validate(*args):
            tick[0] += 4
            return original_validate(*args)
        with patch.object(m, "briefing", brief), patch.object(m, "validate", validate):
            result = invocation(self.root, launch, timeout=timeout, clock=lambda: tick[0])
        return result, tick[0]

    def test_late_valid_completion_with_real_core_clock_and_overhead(self):
        for timeout, prep in ((31, 0), (31.5, 0.25), (300, 12), (3600, 12)):
            with self.subTest(timeout=timeout):
                self.c.event({"id": "timed-" + str(timeout)})
                result, accepted_at = self.timed_cycle(timeout, prep)
                self.assertEqual("accepted", result["status"], result)
                with self.c.connection() as db:
                    at = db.execute("SELECT at FROM audit WHERE operation='manager_decision' "
                                    "ORDER BY seq DESC LIMIT 1").fetchone()[0]
                self.assertEqual(accepted_at, at)
        self.assertEqual(4, self.calls)

    def test_overhead_exhaustion_uses_core_deadline_without_releasing_claim(self):
        result, _ = self.timed_cycle(300, 12, overhead=31)
        self.assert_hold(result)
        self.assertEqual(("acceptance", "core_rejected"), (result["phase"], result["reason"]))
        self.assertEqual("owned", self.cycle()["status"])
        self.assertEqual(1, self.calls)

    def test_preparation_exhaustion_holds_before_launcher(self):
        result, _ = self.timed_cycle(31, 0.01)
        self.assert_hold(result)
        self.assertEqual("insufficient_launch_budget", result["reason"])
        self.assertEqual(0, self.calls)

    def test_invalid_total_budgets_do_not_claim(self):
        before = self.c.path.read_bytes()
        for timeout in (True, None, "300", -1, 0, 1, 30, 30.999, 3600.01, 10**400,
                        float("nan"), float("inf"), -float("inf")):
            with self.subTest(timeout=timeout):
                result = invocation(self.root, self.launch, timeout=timeout)
                self.assertEqual("invalid_timeout", result["reason"])
                self.assertIsNone(result["manager_run"])
                self.assertEqual(before, self.c.path.read_bytes())
        self.assertEqual(0, self.calls)

    def test_sqlite_diagnostics_are_fixed_and_do_not_expose_exception_text(self):
        for code, reason in ((776, "sqlite_recovery_required"), (5, "sqlite_unavailable"),
                             (9999, "sqlite_unavailable"), (None, "sqlite_unavailable")):
            error = sqlite3.OperationalError("private SQL/event/path must not escape")
            error.sqlite_errorcode = code
            error.sqlite_errorname = "private unexpected error name"
            with patch.object(m.ExistingCoordinator, "readiness", side_effect=error):
                result = self.cycle()
            self.assertEqual(reason, result["reason"])
            self.assertNotIn("private", json.dumps(result))
            self.assertIsNone(result["artifacts"])

    def test_hot_journal_holds_readonly_until_explicit_owner_recovery(self):
        for mode in ("paused", "empty", "owned", "ready"):
            with self.subTest(mode=mode):
                c = Coordinator(self.root / mode)
                c.initialize(*seed())
                if mode != "paused":
                    c.set_enabled(True, {"authority": "fixture"})
                if mode != "empty":
                    c.event({"id": "pending-crash-event"})
                if mode == "owned":
                    c.begin_manager()
                with c.connection() as db:
                    original = list(db.iterdump())
                proc = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()),
                                       "--hot-journal", str(c.directory)],
                                      capture_output=True, text=True, timeout=10)
                self.assertEqual(77, proc.returncode, proc.stderr)
                journal = c.path.with_name(c.path.name + "-journal")
                self.assertGreater(journal.stat().st_size, 512)
                self.assertNotEqual(b"\0" * 8, journal.read_bytes()[:8])
                before = {p.name: p.read_bytes() for p in c.directory.iterdir()}
                for _ in range(2):
                    result = m.run_cycle(state=c.directory, runs_dir=None, binary=None,
                                         auth_file=None, owner_context=None, launcher=self.launch)
                    self.assertEqual(("owner_hold", "preflight", "sqlite_recovery_required"),
                                     (result["status"], result["phase"], result["reason"]))
                    self.assertIsNone(result["manager_run"])
                    self.assertIsNone(result["artifacts"])
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(0, m.main(["--state", str(c.directory)]))
                self.assertEqual(before, {p.name: p.read_bytes() for p in c.directory.iterdir()})
                # Child is reaped: explicit owner opens existing supported rw Coordinator.
                recovered = Coordinator(c.directory)
                with recovered.connection() as db:
                    self.assertEqual(original, list(db.iterdump()))
                self.assertFalse(journal.exists())
                self.assertEqual(mode, m.ExistingCoordinator(c.directory).readiness())
        self.assertEqual(0, self.calls)

    def test_launcher_exception_retained_without_retry(self):
        def launch(args):
            self.calls += 1
            raise RuntimeError("private event text must not appear")
        result = self.cycle(launch)
        self.assert_hold(result)
        self.assertNotIn("private event", json.dumps(result))
        self.assertEqual("owned", self.cycle()["status"])
        self.assertEqual(1, self.calls)

    def test_frozen_inputs_and_hard_rationale_limit(self):
        for field, value in (("inputs", {"allocation": "bootstrap", "base": "b" * 40}),
                             ("rationale", "é" * 501)):
            with self.subTest(field=field):
                def launch(args):
                    receipt = self.launch(args)
                    rewrite_decision(receipt, lambda d: d["actions"][0].update({field: value}))
                    f.write_json(Path(receipt["artifacts"]["receipt"]), receipt)
                    return receipt
                self.assert_hold(self.cycle(launch))
                self.c.fail_manager(self.c.snapshot()["manager"]["id"], "offline fixture reset after returned launcher")

    def test_short_rationale_is_directive_not_new_hard_limit(self):
        def launch(args):
            receipt = self.launch(args)
            rewrite_decision(receipt, lambda d: d["actions"][0].update(rationale="x" * 1000))
            f.write_json(Path(receipt["artifacts"]["receipt"]), receipt)
            return receipt
        self.assertEqual("accepted", self.cycle(launch)["status"])

    def test_artifact_and_identity_rejections(self):
        mutations = [
            ("receipt.json", "model", "wrong"), ("manifest.json", "provider", "wrong"),
            ("candidate.json", "effort", "low"), ("receipt.json", "session_id", str(uuid.uuid4())),
            ("receipt.json", "turn_id", "wrong"), ("receipt.json", "response_id", "wrong"),
            ("receipt.json", "shutdown", {"clean": True}), ("receipt.json", "shutdown.clean", False),
            ("receipt.json", "shutdown.session_gone", False), ("process.json", "pid", 9999),
            ("receipt.json", "status", "timeout"), ("manifest.json", "binary_sha256", "a" * 64),
            ("launch.json", "binary", "wrong"),
            ("manifest.json", "briefing_sha256", "a" * 64), ("manifest.json", "briefing.manager_run", "wrong"),
            ("manifest.json", "argv", []), ("manifest.json", "launcher_sha256", "a" * 64),
            ("receipt.json", "started_at", "2000-01-01T00:00:00Z"),
            ("candidate.json", "decision.state_revision", 0), ("receipt.json", "artifacts.run_dir", str(self.root)),
        ]
        for filename, key, value in mutations:
            with self.subTest(file=filename, key=key):
                def launch(args):
                    receipt = self.launch(args)
                    path = Path(receipt["artifacts"]["run_dir"]) / filename
                    obj = m.load_json(path)
                    target = obj
                    parts = key.split(".")
                    for part in parts[:-1]:
                        target = target[part]
                    target[parts[-1]] = value
                    f.write_json(path, obj)
                    return obj if filename == "receipt.json" else receipt
                self.assert_hold(self.cycle(launch))
                self.c.fail_manager(self.c.snapshot()["manager"]["id"], "offline fixture reset")

    def test_full_rollout_completion_required(self):
        def launch(args):
            receipt = self.launch(args)
            path, records = f.rollout(Path(receipt["artifacts"]["run_dir"]))
            f.write_file(path, "".join(json.dumps(r) + "\n" for r in records[:-1]))
            return receipt
        self.assert_hold(self.cycle(launch))

    def test_cli_run_uses_default_launcher_and_returns_prepared_actions(self):
        output = io.StringIO()
        argv = ["--run"]
        for key, value in (("state", "state"), ("runs-dir", "runs"), ("binary", "binary"),
                           ("auth-file", "ABSENT-auth"), ("owner-context", "context.json")):
            argv.extend(["--" + key, str(self.root / value)])
        with contextlib.redirect_stdout(output), patch.object(f, "run_launcher", self.launch):
            self.assertEqual(0, m.main(argv))
        self.assertEqual("prepared", json.loads(output.getvalue())["prepared_actions"][0]["status"])
        self.assertEqual(1, self.calls)

    def test_uninitialized_existing_store_and_missing_database_are_not_created(self):
        empty = Coordinator(self.root / "uninitialized")
        before = empty.path.read_bytes()
        for directory in (empty.directory, self.root / "runs"):
            result = m.run_cycle(state=directory, runs_dir=None, binary=None, auth_file=None, owner_context=None)
            self.assertEqual("owner_hold", result["status"])
        self.assertEqual(before, empty.path.read_bytes())
        self.assertFalse((self.root / "runs/coordinator.sqlite3").exists())

    def test_stale_during_briefing_does_not_invoke_launcher(self):
        original = m.briefing
        def brief(*args):
            result = original(*args)
            self.c.event({"id": "arrived-before-launch"})
            return result
        with patch.object(m, "briefing", brief):
            self.assert_hold(self.cycle())
        self.assertEqual(0, self.calls)

    def test_binary_and_durable_receipt_cannot_change(self):
        for mode in ("binary", "receipt"):
            def launch(args):
                receipt = self.launch(args)
                if mode == "binary":
                    f.write_file(args.binary, "different fixture binary", 0o700)
                else:
                    receipt["finished_at"] = "2000-01-01T00:00:00Z"
                return receipt
            with self.subTest(mode=mode):
                self.assert_hold(self.cycle(launch))
                self.c.fail_manager(self.c.snapshot()["manager"]["id"], "offline fixture reset")

    def test_corrupt_original_digest_is_not_presented_as_evidence(self):
        with self.c.connection() as db:
            reference = self.c._reference(db, {"original": "true"})
            db.execute("UPDATE evidence SET body=? WHERE digest=?", ('{"original":"changed"}', reference["evidence_digest"]))
        self.c.event({"id": "corrupt", "result": reference})
        result = self.cycle()
        self.assert_hold(result)
        self.assertEqual("evidence_digest_mismatch", result["reason"])
        self.assertEqual(0, self.calls)

    def test_process_crashes_preserve_claim_or_committed_acceptance(self):
        for boundary in ("claim.json", "during-launch", "returned.json", "validated.json", "after-commit"):
            with self.subTest(boundary=boundary):
                proc = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()),
                                       "--crash", str(self.root), boundary], capture_output=True, text=True)
                self.assertEqual(71, proc.returncode, proc.stderr)
                state = self.c.snapshot()
                if boundary == "after-commit":
                    self.assertIsNone(state["manager"])
                    self.assertFalse(self.pending())
                    self.assertEqual("prepared", next(iter(state["actions"].values()))["status"])
                    self.assertEqual("empty", self.cycle()["status"])
                else:
                    self.assertIsNotNone(state["manager"])
                    self.assertTrue(self.pending())
                    self.assertEqual("owned", self.cycle()["status"])
                    self.c.fail_manager(state["manager"]["id"], "offline child reaped; fixture-only reconciliation")
        self.assertEqual(0, self.calls)


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--hot-journal":
        hot_journal_child(Path(sys.argv[2]))
    elif len(sys.argv) == 4 and sys.argv[1] == "--crash":
        crash_child(Path(sys.argv[2]), sys.argv[3])
    else:
        unittest.main()
