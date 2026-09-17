"""Default-OFF one-tick owner; admitted, journaled TMUX worker lifecycle."""
import argparse
from contextlib import closing, contextmanager
import fcntl
import os
from pathlib import Path
import sqlite3
import stat
import subprocess
import sys
import time
import uuid

import fable_launcher as f
from coordinator import Rejected, digest, encoded
from manager_cycle import ExistingCoordinator

WORKER_KINDS = frozenset({"implement", "revise", "review", "design", "functional_test",
                          "evidence_review", "repair", "reconcile"})


class DispatchDeferred(Exception):
    """Authority currently defers dispatch; no external effect was attempted."""


SCHEMA = {
    "meta": "singleton INTEGER PRIMARY KEY CHECK(singleton=1), schema_version INTEGER, config_digest TEXT, package_digest TEXT, control_generation INTEGER, requested_mode TEXT, activation_decision_id TEXT, activation_revision INTEGER, activation_digest TEXT",
    "boots": "boot_id TEXT, owner_epoch INTEGER, host_boot_id TEXT, pid INTEGER, process_start TEXT, package_digest TEXT, started_at REAL, stopped_at REAL, stop_reason TEXT, PRIMARY KEY(boot_id,owner_epoch)",
    "operations": "op_id TEXT PRIMARY KEY, domain TEXT, action_id TEXT, claim TEXT, allocation_digest TEXT, effect TEXT, ordinal INTEGER, expected_coordinator_revision INTEGER, config_generation INTEGER, authority_digest TEXT, request_digest TEXT, request_artifact TEXT, phase TEXT, receipt_digest TEXT, receipt_artifact TEXT, hold_reason TEXT, created_at REAL, updated_at REAL",
    "processes": "op_id TEXT PRIMARY KEY, role TEXT, host_boot_id TEXT, actual_pid INTEGER, pgid INTEGER, process_start TEXT, socket_path TEXT, tmux_session TEXT, pane_id TEXT, corbanu_session_id TEXT, thread_id TEXT, active_turn_id TEXT, model TEXT, provider TEXT, effort TEXT, binary_digest TEXT, worktree TEXT, private_run_root TEXT, terminal_status TEXT",
    "observations": "observation_id TEXT PRIMARY KEY, op_id TEXT, observed_at_utc TEXT, boot_id TEXT, monotonic_offset REAL, source_kind TEXT, source_cursor TEXT, content_digest TEXT, artifact_path TEXT, identity_valid INTEGER, liveness TEXT, progress_kind TEXT",
    "deliveries": "delivery_id TEXT PRIMARY KEY, source_domain TEXT, source_event_id TEXT, source_revision_or_watermark TEXT, target_domain TEXT, target_operation_id TEXT, payload_digest TEXT, phase TEXT, receipt_digest TEXT",
    "holds": "hold_id TEXT PRIMARY KEY, scope TEXT, op_id TEXT, reason_code TEXT, first_seen REAL, last_seen REAL, original_evidence_digest TEXT, resolution_evidence_digest TEXT, resolved_at REAL",
    "health": "component TEXT PRIMARY KEY, last_attempt_at REAL, last_success_at REAL, consecutive_failures INTEGER, next_probe_at REAL, status TEXT, evidence_digest TEXT",
}


def package_digest():
    root = Path(__file__).resolve().parent
    return digest({name: f.file_digest(root / name) for name in
                   ("owner_daemon.py", "owner_tmux.py", "coordinator.py", "manager_cycle.py", "fable_launcher.py")})


def private_file(path):
    path = f.no_links(path)
    info = path.stat()
    f.require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
              and stat.S_IMODE(info.st_mode) == 0o600 and info.st_nlink == 1, "unsafe_file")
    return path


def load(path):
    return f.strict_json(f.read_file(private_file(path), 262144, private=True))


def configuration(path):
    value = load(path)
    f.require(set(value) - {"transport"} == {"coordinator", "worktrees", "package_digest", "manager_enabled"},
              "invalid_config")
    if "transport" in value:
        from owner_tmux import validate
        validate(value["transport"])
    f.require(type(value["manager_enabled"]) is bool and isinstance(value["worktrees"], list)
              and bool(value["worktrees"]), "invalid_config")
    coordinator = ExistingCoordinator(value["coordinator"])
    f.require(not any(Path(str(coordinator.path) + suffix).exists()
                      for suffix in ("-journal", "-wal", "-shm")), "coordinator_recovery_required")
    coordinator.snapshot()
    for path in value["worktrees"]:
        f.require(Path(path).is_absolute() and f.no_links(path).is_dir(), "missing_worktree")
    f.require(value["package_digest"] == package_digest(), "package_drift")
    return value


def artifact(root, relative, value):
    path = root / relative
    for directory in (*reversed(path.parent.relative_to(root).parents), path.parent.relative_to(root)):
        target = root / directory
        if not target.exists():
            target.mkdir(mode=0o700)
            fd = os.open(target.parent, os.O_RDONLY)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
        f.private_dir(target)
    if path.exists():
        f.require(load(path) == value, "artifact_conflict")
    else:
        f.write_json(path, value)
    return str(path.relative_to(root))


@contextmanager
def locked(path):
    path = private_file(path)
    fd = os.open(path, os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        info, current = os.fstat(fd), path.stat()
        f.require((info.st_dev, info.st_ino) == (current.st_dev, current.st_ino)
                  and info.st_nlink == 1 and stat.S_IMODE(info.st_mode) == 0o600,
                  "lock_identity_changed")
        yield
    finally:
        os.close(fd)


def setup(config_path):
    """Explicit offline setup only; creates OFF state, never activation authority."""
    config = configuration(config_path)
    root = f.private_dir(config["coordinator"])
    path = root / "owner.sqlite3"
    f.require(not path.exists(), "owner_state_exists")
    for name in ("owner-daemon.lock", "owner-admission.lock"):
        f.write_file(root / name, b"") if not (root / name).exists() else private_file(root / name)
    fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(fd)
    with closing(sqlite3.connect(path)) as db:
        db.execute("PRAGMA synchronous=FULL")
        for table, columns in SCHEMA.items():
            db.execute(f"CREATE TABLE {table} ({columns})")
        db.execute("INSERT INTO meta VALUES(1,1,?,?,0,'off',NULL,NULL,NULL)",
                   (digest(config), config["package_digest"]))
        db.commit()
    directory = os.open(root, os.O_RDONLY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def fixture_receipt(request):
    return {"fixture_only": True, "request_digest": digest(request),
            "event": {"id": "owner-fixture:" + request["identity"], "fixture_only": True}}


class FixedTestAdapter:
    """A deterministic fixture receipt; no callable, command, or transport configuration."""
    def observe(self, request):
        return fixture_receipt(request)


def configured_adapter(config):
    if "transport" in config:
        from owner_tmux import TmuxAdapter
        return TmuxAdapter(config["transport"])
    return FixedTestAdapter()


class Kernel:
    def __init__(self, config_path):
        self.config_path = Path(config_path)
        self.config = configuration(config_path)
        self.root = f.private_dir(self.config["coordinator"])
        self.c = ExistingCoordinator(self.root)
        self.db = None

    def admit(self):
        f.require(configuration(self.config_path) == self.config, "config_drift")
        meta = dict(self.db.execute("SELECT * FROM meta WHERE singleton=1").fetchone())
        f.require(meta["schema_version"] == 1 and meta["config_digest"] == digest(self.config)
                  and meta["package_digest"] == package_digest(), "state_drift")
        f.require(meta["requested_mode"] == "armed", "owner_off")
        authority = load(self.root / "activation.json")
        f.require(set(authority) == {"decision_id", "revision", "authority", "scope",
                                    "generation", "config_digest", "package_digest"},
                  "invalid_activation")
        f.require(isinstance(authority["authority"], str) and bool(authority["authority"].strip())
                  and authority["scope"] == ("tmux-workers" if "transport" in self.config else "fixture-only")
                  and type(authority["revision"]) is int and authority["revision"] > 0
                  and type(authority["generation"]) is int and authority["generation"] > 0
                  and bool(authority["decision_id"]), "activation_authority_required")
        f.require((meta["activation_decision_id"], meta["activation_revision"],
                   meta["activation_digest"], meta["control_generation"]) ==
                  (authority["decision_id"], authority["revision"], digest(authority),
                   authority["generation"]) and authority["config_digest"] == digest(self.config)
                  and authority["package_digest"] == package_digest(), "activation_mismatch")
        return meta

    def hold(self, op_id, reason):
        now = time.time()
        artifact(self.root, "runs/" + (op_id or "global") + "/holds/" + str(uuid.uuid4()) + ".json",
                 {"reason": reason, "op_id": op_id, "observed_at": now})
        self.db.execute("INSERT INTO holds VALUES(?,?,?,?,?,?,?,NULL,NULL) "
                        "ON CONFLICT(hold_id) DO UPDATE SET last_seen=excluded.last_seen",
                        (digest([op_id, reason]), "operation" if op_id else "global",
                         op_id, reason, now, now, digest({"reason": reason})))
        if op_id:
            self.db.execute("UPDATE operations SET phase='held',hold_reason=?,updated_at=? WHERE op_id=?",
                            (reason, now, op_id))
        self.db.commit()

    def replay(self, row):
        request = load(self.root / row["request_artifact"])
        f.require(digest(request) == row["request_digest"], "request_drift")
        receipt_path = "runs/" + row["op_id"] + "/receipt.json"
        if not (self.root / receipt_path).exists():
            self.hold(row["op_id"], "effect_uncertain")
            return
        meta = self.admit()
        if (row["config_generation"], row["authority_digest"]) != (
                meta["control_generation"], meta["activation_digest"]):
            self.hold(row["op_id"], "prior_activation")
            return
        receipt = load(self.root / receipt_path)
        f.require(receipt == fixture_receipt(request), "invalid_fixture_receipt")
        f.require(row["receipt_digest"] in (None, digest(receipt)), "receipt_drift")
        self.db.execute("UPDATE operations SET phase='observed',receipt_digest=?,receipt_artifact=? WHERE op_id=?",
                        (digest(receipt), receipt_path, row["op_id"]))
        self.db.commit()
        # Coordinator.event checks exact duplicate content before mutating revision.
        self.c.event(receipt["event"])
        self.db.execute("INSERT OR REPLACE INTO deliveries VALUES(?,?,?,?,?,?,?,?,?)",
                        (row["op_id"], "fixture", receipt["event"]["id"], "1", "coordinator",
                         row["op_id"], digest(receipt["event"]), "applied", digest(receipt)))
        self.db.execute("UPDATE operations SET phase='applied',updated_at=? WHERE op_id=?",
                        (time.time(), row["op_id"]))
        self.db.commit()

    def effect(self, adapter, meta):
        request = {"identity": digest([meta["control_generation"], "fixture-tick"]),
                   "fixture_only": True}
        op_id = digest([meta["control_generation"], "fixture", request["identity"],
                        None, None, "observe", 0])
        if self.db.execute("SELECT 1 FROM operations WHERE op_id=?", (op_id,)).fetchone():
            return
        relative = artifact(self.root, "runs/" + op_id + "/request.json", request)
        now = time.time()
        self.db.execute("INSERT INTO operations VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                        (op_id, "fixture", None, None, None, "observe", 0, self.c.snapshot()["revision"],
                         meta["control_generation"], meta["activation_digest"], digest(request),
                         relative, "intent", None, None, None, now, now))
        self.db.commit()
        receipt = adapter.observe(request)
        artifact(self.root, "runs/" + op_id + "/receipt.json", receipt)
        observation = artifact(self.root, "runs/" + op_id + "/observations/1.json", receipt)
        self.db.execute("INSERT INTO observations VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
                        (op_id, op_id, f.now(), self.boot, time.monotonic() - self.started,
                         "fixture", "1", digest(receipt), observation, 1, "fixture", "receipt"))
        self.db.commit()
        self.replay(self.db.execute("SELECT * FROM operations WHERE op_id=?", (op_id,)).fetchone())

    def worker_gate(self, action):
        meta = self.admit()
        state = self.c.snapshot()
        current = state["actions"][action["id"]]
        if not state["enabled"] or state["manager"] is not None:
            raise DispatchDeferred("dispatch_paused_or_owned")
        f.require(current["allocation_digest"] == action["allocation_digest"]
                  == digest(state["allocations"].get(action["inputs"]["allocation"]))
                  and current["inputs"] == action["inputs"], "allocation_drift")
        f.require(current.get("claim") == action.get("claim"), "claim_drift")
        self.c._executable(state, current)
        self.c._resources_available(state, current)
        f.require("deadline" not in current or time.time() < current["deadline"], "lease_expired")
        return meta

    def step(self, action, effect, request, perform):
        """Receipts permit continuation; a missing effect receipt never permits retry."""
        op_id = digest(["tmux", action["id"], effect])
        meta = self.admit()
        row = self.db.execute("SELECT * FROM operations WHERE op_id=?", (op_id,)).fetchone()
        if row:
            f.require(row["phase"] != "held", "operation_held")
            f.require((row["config_generation"], row["authority_digest"]) ==
                      (meta["control_generation"], meta["activation_digest"]), "prior_activation")
            f.require(row["request_digest"] == digest(request)
                      and load(self.root / row["request_artifact"]) == request, "request_drift")
        relative = "runs/" + op_id + "/receipt.json"
        if row and row["phase"] != "deferred":
            f.require((self.root / relative).exists(), "effect_uncertain")
            receipt = load(self.root / relative)
            f.require(receipt["request_digest"] == digest(request)
                      and row["receipt_digest"] in (None, digest(receipt)), "receipt_drift")
        else:
            f.require(not (self.root / relative).exists(), "unexpected_deferred_receipt")
            self.worker_gate(action)
            relative = artifact(self.root, "runs/" + op_id + "/request.json", request)
            now = time.time()
            self.db.execute("INSERT OR REPLACE INTO operations VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                            (op_id, "tmux", action["id"], action.get("claim"),
                             action["allocation_digest"], effect, 0, self.c.snapshot()["revision"],
                             meta["control_generation"], meta["activation_digest"], digest(request),
                             relative, "intent", None, None, None, now, now))
            self.db.commit()
            try:
                self.worker_gate(action)
            except DispatchDeferred:
                # Only this pre-effect gate proves no effect occurred. A crash
                # before this commit still leaves an uncertain intent, never a retry.
                self.db.execute("UPDATE operations SET phase='deferred',updated_at=? WHERE op_id=?",
                                (time.time(), op_id))
                self.db.commit()
                raise
            receipt = {"request_digest": digest(request), "result": perform()}
            relative = artifact(self.root, "runs/" + op_id + "/receipt.json", receipt)
        self.db.execute("UPDATE operations SET phase='applied',receipt_digest=?,receipt_artifact=?,"
                        "updated_at=? WHERE op_id=?", (digest(receipt), relative, time.time(), op_id))
        self.db.commit()
        return receipt["result"]

    def worker_observation(self, worker, action, launch_id):
        state = worker.inspect(deadline=action["deadline"])
        relative = artifact(self.root, "runs/" + launch_id + "/observations/" +
                            str(uuid.uuid4()) + ".json", state)
        self.db.execute("INSERT INTO observations VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
                        (str(uuid.uuid4()), launch_id, f.now(), self.boot,
                         time.monotonic() - self.started, "tmux", state.get("rollout_digest"),
                         digest(state), relative, int(state["identity_valid"]),
                         state["liveness"], "working" if state.get("submitted") else "poll"))
        proc, b = state.get("process", {}), worker.binding
        self.db.execute("INSERT OR REPLACE INTO processes VALUES(" + ",".join(["?"] * 19) + ")",
                        (launch_id, "worker", worker.meta["boot_id"], proc.get("pid"), proc.get("pgid"),
                         proc.get("start"), worker.meta["socket"], worker.meta["session"],
                         proc.get("pane"), state.get("session_id"), state.get("thread_id"),
                         state.get("turn_id"), b["model"], b["provider"], b["effort"],
                         worker.config["binary_sha256"], b["worktree"], str(worker.run),
                         "lease_expired" if time.time() >= action["deadline"] else state["liveness"]))
        self.db.commit()
        f.require(state["identity_valid"], state.get("evidence_reason", "inspection_uncertain"))
        # A correlated durable RETURN survives pane death. Worker.send still
        # requires liveness before any further key delivery.
        f.require(state.get("returned") is not None or state["liveness"] == "alive", "worker_dead")
        return state

    def worker_action(self, adapter, action):
        from owner_tmux import Worker
        f.require(action["kind"] in WORKER_KINDS, "unsupported_worker_kind")
        inputs = action["inputs"]
        runtime = inputs["worker"]
        f.require(set(runtime) == {"model", "provider", "effort", "worktree", "policy"}
                  and runtime["policy"] == "--yolo", "recorded_yolo_required")
        f.require(runtime["worktree"] in self.config["worktrees"], "unallocated_worktree")
        # Freeze the same action assignment used by NativeOwner, including scope.
        assignment = encoded({k: action[k] for k in
                              ("kind", "workstream", "sprint", "scope", "inputs", "timeout_seconds")})
        claim_request = {"action_id": action["id"], "allocation_digest": action["allocation_digest"]}
        action = self.step(action, "claim", claim_request, lambda: self.c.claim(action["id"]))
        f.require(self.c.snapshot()["actions"][action["id"]].get("claim") == action["claim"], "claim_drift")
        binding = {k: runtime[k] for k in ("model", "provider", "effort", "worktree")}
        binding.update(action_id=action["id"], claim=action["claim"],
                       allocation_digest=action["allocation_digest"],
                       sandbox="danger-full-access", approval="never")
        assignment += ("\nOwner constraints: never push, send Slack, change owner daemon/transport source, "
                       "access live profiles or credential stores, or approve additional permissions. "
                       "Use only the recorded --yolo policy and allocated scope.")
        prepare = {"binding": binding, "assignment": assignment}
        run = self.step(action, "prepare", prepare, lambda: str(adapter.prepare(binding, assignment).run))
        f.require(Path(run).parent == Path(adapter.config["runs_dir"]), "run_root_drift")
        worker = Worker(run)
        f.require(worker.binding == binding and worker.config == adapter.config, "worker_binding_drift")
        launch_id = digest(["tmux", action["id"], "launch"])
        request = {"run": run, "binding": binding}
        self.step(action, "launch", request, worker.launch)
        state = self.worker_observation(worker, action, launch_id)
        if time.time() >= action["deadline"]:
            # This is a lease failure even if a zombie or frozen child still has a PID.
            pending = self.db.execute("SELECT effect FROM operations WHERE action_id=?",
                                      (action["id"],)).fetchall()
            effects = {row[0] for row in pending}
            reason = ("start_not_working" if "start" in effects and "working" not in effects else
                      "missing_ack" if "prompt" in effects and "ack" not in effects else "lease_expired")
            raise f.LaunchError(reason)
        prompted = self.db.execute("SELECT 1 FROM operations WHERE action_id=? AND effect='prompt'",
                                   (action["id"],)).fetchone()
        if not prompted and not state.get("ready"):
            return "awaiting_ready"
        self.step(action, "prompt", request, worker.prompt)
        state = self.worker_observation(worker, action, launch_id)
        if not state.get("ack"):
            return "awaiting_ack"
        f.require(state.get("ack_line") == worker.meta["ack"], "wrong_ack")
        agent = worker.meta["session"]
        ack = self.step(action, "ack", request, lambda: state)
        self.step(action, "dispatched", request,
                  lambda: self.c.dispatched(action["id"], action["claim"], agent, ack))
        self.step(action, "acknowledged", request,
                  lambda: self.c.acknowledge(action["id"], agent, action["allocation_digest"], ack))
        self.step(action, "start", request, worker.start)
        state = self.worker_observation(worker, action, launch_id)
        if not state.get("submitted"):
            return "awaiting_working"
        # send-keys alone is never a working receipt: require correlated START turn.
        self.step(action, "working", request, lambda: state)
        if state.get("returned") is None:
            return "working"
        result = self.step(action, "return_observed", request, lambda: state)
        self.step(action, "returned", request,
                  lambda: self.c.returned(action["id"], agent, result))
        return "returned"

    def workers(self, adapter):
        """One bounded pass. An operation failure holds only its owning action."""
        outcomes = {}
        for action in self.c.snapshot()["actions"].values():
            if action["kind"] not in WORKER_KINDS:
                continue
            rows = self.db.execute("SELECT * FROM operations WHERE domain='tmux' AND action_id=?",
                                   (action["id"],)).fetchall()
            if not rows and action["status"] not in {"prepared", "dispatching", "dispatch_uncertain",
                                                     "dispatched", "running"}:
                continue
            try:
                f.require(not self.db.execute("SELECT 1 FROM holds WHERE op_id=? AND resolved_at IS NULL",
                          (digest(["tmux", action["id"], "claim"]),)).fetchone(), "operation_held")
                for row in rows:
                    f.require(row["phase"] != "held", "operation_held")
                    f.require((row["config_generation"], row["authority_digest"]) ==
                              (self.admit()["control_generation"], self.admit()["activation_digest"]),
                              "prior_activation")
                    f.require(row["phase"] != "intent" or
                              (self.root / ("runs/" + row["op_id"] + "/receipt.json")).exists(),
                              "effect_uncertain")
                if any(row["effect"] == "returned" and row["phase"] == "applied" for row in rows):
                    outcomes[action["id"]] = "returned"
                    continue
                if self.c.readiness() in {"paused", "owned"}:
                    if rows and time.time() >= action.get("deadline", float("inf")):
                        self.db.execute("UPDATE processes SET terminal_status='lease_expired' WHERE op_id=?",
                                        (digest(["tmux", action["id"], "launch"]),))
                        self.db.commit()
                        raise f.LaunchError("lease_expired")
                    outcomes[action["id"]] = "deferred"
                    continue
                # Unreceipted claims are not adopted; only our recorded claim can resume.
                f.require(bool(rows) or action["status"] == "prepared", "unowned_claim")
                outcomes[action["id"]] = self.worker_action(adapter, action)
            except DispatchDeferred:
                outcomes[action["id"]] = "deferred"
            except Exception as exc:
                self.db.rollback()
                reason = str(exc) if isinstance(exc, f.LaunchError) else type(exc).__name__
                op_id = digest(["tmux", action["id"], "claim"])
                self.hold(op_id, reason)
                # Fence all existing steps, including pending external effects.
                self.db.execute("UPDATE operations SET phase='held',hold_reason=? "
                                "WHERE domain='tmux' AND action_id=?", (reason, action["id"]))
                self.db.commit()
                outcomes[action["id"]] = "HOLD"
        watchdog = self.c.watchdog()
        now = time.time()
        self.db.execute("INSERT OR REPLACE INTO health VALUES(?,?,?,?,?,?,?)",
                        ("watchdog", now, now, 0, None, "ok", digest(watchdog)))
        self.db.commit()
        held = self.db.execute("SELECT 1 FROM holds WHERE resolved_at IS NULL").fetchone()
        readiness = self.c.readiness()
        return {"state": "HOLD" if held else "READY" if readiness in {"paused", "owned"} else "ACTIVE",
                "actions": outcomes, "routing": "tmux-workers", "fixture_only": False}

    def tick(self, adapter=None):
        adapter = configured_adapter(self.config) if adapter is None else adapter
        from owner_tmux import TmuxAdapter
        f.require(type(adapter) is FixedTestAdapter or (type(adapter) is TmuxAdapter
                  and adapter.config == self.config.get("transport")), "live_adapter_unavailable")
        path = private_file(self.root / "owner.sqlite3")
        f.require(not any(Path(str(path) + suffix).exists() for suffix in ("-journal", "-wal", "-shm")),
                  "owner_recovery_required")
        with locked(self.root / "owner-daemon.lock"):
            self.db = sqlite3.connect(path.as_uri() + "?mode=rw", uri=True)
            self.db.row_factory = sqlite3.Row
            boot = None
            try:
                self.db.execute("PRAGMA synchronous=FULL")
                f.require(self.db.execute("PRAGMA journal_mode").fetchone()[0] == "delete", "journal_mode")
                for table, columns in SCHEMA.items():
                    row = self.db.execute("SELECT sql FROM sqlite_master WHERE name=?", (table,)).fetchone()
                    f.require(row and row[0] == f"CREATE TABLE {table} ({columns})", "schema_drift")
                with locked(self.root / "owner-admission.lock"):
                    meta = self.admit()
                    boot = self.boot = str(uuid.uuid4())
                    self.started = time.monotonic()
                    host = subprocess.check_output(
                        ["/usr/sbin/sysctl", "-n", "kern.boottime"], timeout=2).decode().strip() if sys.platform == "darwin" else Path("/proc/sys/kernel/random/boot_id").read_text().strip()
                    start = subprocess.check_output(
                        ["/bin/ps", "-o", "lstart=", "-p", str(os.getpid())], timeout=2).decode().strip()
                    self.db.execute("INSERT INTO boots VALUES(?,?,?,?,?,?,?,?,?)",
                                    (boot, meta["control_generation"], host, os.getpid(), start,
                                     package_digest(), time.time(), None, None))
                    self.db.commit()
                    if type(adapter) is TmuxAdapter:
                        return self.workers(adapter)
                    states = ["RECOVERING"]
                    if self.c.readiness() != "owned":
                        for row in self.db.execute("SELECT * FROM operations WHERE phase IN ('intent','observed')").fetchall():
                            self.replay(row)
                    watchdog = self.c.watchdog()
                    now = time.time()
                    self.db.execute("INSERT OR REPLACE INTO health VALUES(?,?,?,?,?,?,?)",
                                    ("watchdog", now, now, 0, None, "ok", digest(watchdog)))
                    self.db.commit()
                    if self.db.execute("SELECT 1 FROM holds WHERE resolved_at IS NULL").fetchone():
                        return {"state": "HOLD", "states": states + ["HOLD"]}
                    states.append("READY")
                    readiness = self.c.readiness()
                    if readiness in {"ready", "empty"}:
                        states.append("ACTIVE")
                        self.effect(adapter, meta)
                    manager = "deferred" if self.config["manager_enabled"] and readiness == "ready" else readiness
                    return {"state": states[-1], "states": states, "manager": manager,
                            "routing": "deferred", "fixture_only": True}
            finally:
                if boot:
                    self.db.execute("UPDATE boots SET stopped_at=?,stop_reason=? WHERE boot_id=?",
                                    (time.time(), "tick_exit" if sys.exc_info()[0] is None else "interrupted", boot))
                    self.db.commit()
                self.db.close()
                self.db = None


def schedule_pins(python, runtime, config, expected_python):
    f.require(Path(python).is_absolute() and Path(runtime).is_absolute()
              and Path(config).is_absolute(), "absolute_paths_required")
    python = f.no_links(python)
    f.private_dir(Path(config).parent)
    info = python.stat()
    f.require(python.is_absolute() and stat.S_ISREG(info.st_mode)
              and info.st_uid in {0, os.getuid()} and not info.st_mode & 0o022, "unsafe_python")
    runtime = f.private_dir(runtime)
    f.require(f.file_digest(python) == expected_python, "python_pin_mismatch")
    return {"python": str(python), "python_sha256": expected_python,
            "version": subprocess.check_output([str(python), "-E", "-s", "-S", "--version"],
                                              timeout=5, env={}).decode().strip(),
            "runtime": str(runtime), "files": {p.name: f.file_digest(private_file(p))
                                              for p in sorted(runtime.iterdir())},
            "config": str(private_file(config)), "config_sha256": f.file_digest(config)}


def installation_domain(receipt):
    # Pre-domain receipts were installed exclusively into this user’s GUI domain.
    domain = receipt.get("domain", f"gui/{os.getuid()}")
    f.require(domain in (f"gui/{os.getuid()}", f"user/{os.getuid()}"), "invalid_domain")
    return domain


def service(label, domain=None):
    import re
    domain = installation_domain({"domain": domain} if domain is not None else {})
    f.require(re.fullmatch(r"com\.corbanu\.initiative-owner(?:\.[a-zA-Z0-9-]+)?", label), "invalid_label")
    result = subprocess.run(["/bin/launchctl", "print", f"{domain}/{label}"],
                            capture_output=True, text=True, timeout=5, env={})
    if result.returncode == 0:
        return "present", result.stdout
    if result.returncode == 113 and f'Could not find service "{label}"' in result.stderr:
        return "absent", ""
    raise f.LaunchError("service_observation_unavailable")


def firing_source(root, receipt):
    """Fail closed: launchd diagnostics are not a stable API or activation authority."""
    try:
        presence, output = service(receipt["label"], installation_domain(receipt))
        lines = set(output.splitlines())
        if (presence != "present" or f"\tpid = {os.getpid()}" not in lines
                or "\tstate = running" not in lines):
            return "manual"
        f.require(f"path = {root / 'owner.plist'}\n" in output and
                  f.file_digest(private_file(root / "owner.plist")) == receipt["plist_sha256"],
                  "unowned_service")
        return "interval" if "\timmediate reason = interval" in lines else "other"
    except Exception:
        return "unknown"


def observe_schedule(root, label="com.corbanu.initiative-owner"):
    result = dict(observed_at=time.time(), service="unknown", installed=False,
                  started_at=None, completed_at=None, last_success=None, hold=None,
                  reason="observation-unavailable", previous_success=None, interval=30,
                  errors=0, consecutive_errors=0, last_error=None, firing="unknown",
                  publication_errors=0, last_publication_error=None)
    try:
        root = f.private_dir(root)
        receipt = load(root / "installation.json") if os.path.lexists(root / "installation.json") else None
        f.require(receipt is not None or not os.path.lexists(root / "tick.json"), "installation_receipt_missing")
        if receipt:
            label = receipt["label"]
        result["installed"] = receipt is not None and receipt["phase"] != "uninstalled"
        result["service"], output = service(label, installation_domain(receipt or {}))
        if receipt and result["service"] == "present":
            plist = root / "owner.plist"
            f.require(f"path = {plist}\n" in output and
                      f.file_digest(private_file(plist)) == receipt["plist_sha256"], "unowned_service")
        if receipt:
            result["interval"] = receipt["interval"]
            status = load(root / "tick.json")
            result.update({key: status.get(key, result[key]) for key in
                           ("previous_success", "errors", "consecutive_errors", "last_error",
                            "firing", "publication_errors", "last_publication_error")})
            result.update({key: status[key] for key in
                           ("started_at", "completed_at", "last_success", "hold")})
        result["reason"] = None
    except Exception:
        result.update(service="unknown", reason="observation-unavailable")
    return result


def publish_schedule(root):
    receipt = load(root / "installation.json")
    f.write_json(f.private_dir(receipt["publish_state"]) / "owner-recurrence.json",
                 observe_schedule(root))


def scheduled_tick(root, recover=None):
    root = f.private_dir(root)
    with locked(root / "tick.lock"):
        receipt = load(root / "installation.json")
        f.require(receipt["phase"] == "installed", "schedule_not_installed")
        status = load(root / "tick.json")
        if recover is not None:
            f.require(isinstance(recover, str) and 0 < len(recover.strip()) <= 1000, "recovery_evidence_required")
            artifact(root, "recovery/" + str(uuid.uuid4()) + ".json",
                     {"at": time.time(), "evidence": recover, "previous": status})
            status.update(hold=None, started_at=None, completed_at=None,
                          last_success=None, previous_success=None)
            f.write_json(root / "tick.json", status)
            return {"state": "RECOVERED"}
        if status["started_at"] is not None and status["completed_at"] is None:
            status["hold"] = status["hold"] or "interrupted_tick"
        if status["hold"]:
            status["skipped"] += 1
            status["last_probe"] = time.time()
            f.write_json(root / "tick.json", status)
            return {"state": "HOLD", "reason": status["hold"]}
        previous_firing = status.get("firing")
        status.update(started_at=time.time(), completed_at=None, firing=firing_source(root, receipt))
        f.write_json(root / "tick.json", status)
        try:
            pins = receipt["pins"]
            f.require(schedule_pins(Path(pins["python"]), Path(pins["runtime"]),
                                    Path(pins["config"]), pins["python_sha256"]) == pins, "schedule_pin_drift")
            result = Kernel(Path(pins["config"])).tick()
        except (f.LaunchError, Rejected):
            result = {"state": "HOLD", "reason": "owner_run_refused"}
        except Exception as exc:
            result = {"state": "ERROR", "reason": type(exc).__name__}
        status["completed_at"] = time.time()
        if result.get("reason") == "owner_run_refused":
            status.update(hold="owner_run_refused", first_refusal=status.get("first_refusal") or time.time(),
                          last_refusal=time.time())
        elif result.get("reason"):
            status.update(errors=status.get("errors", 0) + 1,
                          consecutive_errors=status.get("consecutive_errors", 0) + 1,
                          last_error=result["reason"], previous_success=None)
        else:
            status.update(previous_success=status.get("last_success") if not status.get("consecutive_errors")
                          and previous_firing == status["firing"] == "interval" else None,
                          last_success=status["completed_at"], consecutive_errors=0, last_error=None)
        status["ticks"] += 1
        f.write_json(root / "tick.json", status)
        artifact(root, "ticks/" + str(uuid.uuid4()) + ".json", status)
        return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="store_true")
    parser.add_argument("--config", type=Path)
    parser.add_argument("--schedule", type=Path)
    parser.add_argument("--recover")
    parser.add_argument("--observe", action="store_true")
    parser.add_argument("--label", default="com.corbanu.initiative-owner")
    parser.add_argument("--publish-state", type=Path)
    args = parser.parse_args(argv)
    if args.observe:
        result = observe_schedule(args.schedule, args.label)
        if args.publish_state:
            f.write_json(f.private_dir(args.publish_state) / "owner-recurrence.json", result)
        print(encoded(result))
        return 0
    if not args.run and args.recover is None:
        print('{"state":"OFF"}')
        return 0
    try:
        if args.schedule:
            try:
                result = scheduled_tick(args.schedule, args.recover)
            except (f.LaunchError, OSError, sqlite3.Error, ValueError, TypeError, KeyError):
                result = {"state": "HOLD", "reason": "owner_run_refused"}
            try:
                publish_schedule(args.schedule)
            except Exception as exc:
                result = dict(result, publication_error=type(exc).__name__, publication_recorded=False)
                try:
                    with locked(args.schedule / "tick.lock"):
                        status = load(args.schedule / "tick.json")
                        status.update(publication_errors=status.get("publication_errors", 0) + 1,
                                      last_publication_error=type(exc).__name__)
                        f.write_json(args.schedule / "tick.json", status)
                        result["publication_recorded"] = True
                except Exception:
                    pass  # stdout still reports both outcomes if local recording also fails.
            print(encoded(result))
            return 2 if result.get("reason") else 3 if result.get("publication_error") else 0
        f.require(args.config is not None and args.recover is None, "config_required")
        kernel = Kernel(args.config)
        # Configuration selects only built-in adapters; default remains the fixture.
        print(encoded(kernel.tick()))
        return 0
    except (f.LaunchError, OSError, sqlite3.Error, ValueError, TypeError, KeyError):
        print('{"state":"HOLD","reason":"owner_run_refused"}')
        return 2


if __name__ == "__main__":
    sys.exit(main())
