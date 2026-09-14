"""Default-OFF one-tick kernel; explicit TMUX transport, lifecycle routing deferred."""
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
from coordinator import digest, encoded
from manager_cycle import ExistingCoordinator

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
                  and authority["scope"] == "fixture-only"
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

    def tick(self, adapter=None):
        adapter = configured_adapter(self.config) if adapter is None else adapter
        from owner_tmux import TmuxAdapter
        f.require(type(adapter) is FixedTestAdapter or (type(adapter) is TmuxAdapter
                  and adapter.config == self.config.get("transport")), "live_adapter_unavailable")
        # Increment C must supply admitted action/claim routing before any live effect.
        if type(adapter) is TmuxAdapter:
            return {"state": "HOLD", "reason": "tmux_lifecycle_routing_unavailable", "fixture_only": False}
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


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="store_true")
    parser.add_argument("--config", type=Path)
    args = parser.parse_args(argv)
    if not args.run:
        print('{"state":"OFF"}')
        return 0
    try:
        f.require(args.config is not None, "config_required")
        kernel = Kernel(args.config)
        # Configuration selects only built-in adapters; default remains the fixture.
        print(encoded(kernel.tick()))
        return 0
    except (f.LaunchError, OSError, sqlite3.Error, ValueError, TypeError, KeyError):
        print('{"state":"HOLD","reason":"owner_run_refused"}')
        return 2


if __name__ == "__main__":
    sys.exit(main())
