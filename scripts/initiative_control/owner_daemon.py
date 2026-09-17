"""Default-OFF one-tick owner; admitted, journaled TMUX worker lifecycle."""
import argparse
from contextlib import closing, contextmanager, nullcontext
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


def validate_authority(authority, config):
    f.require(type(authority) is dict and set(authority) == {
        "decision_id", "revision", "authority", "scope", "generation",
        "config_digest", "package_digest"}, "invalid_activation")
    for key, reason in (("decision_id", "activation_decision_id_required"),
                        ("authority", "activation_authority_required")):
        f.require(isinstance(authority[key], str) and bool(authority[key].strip()), reason)
    for key in ("revision", "generation"):
        f.require(type(authority[key]) is int and authority[key] > 0,
                  "activation_" + key + "_required")
    f.require(authority["scope"] == ("tmux-workers" if "transport" in config else "fixture-only"),
              "activation_scope_mismatch")
    f.require(authority["config_digest"] == digest(config), "activation_config_mismatch")
    f.require(authority["package_digest"] == package_digest(), "activation_package_mismatch")


def activation_pending(root):
    return any(os.path.lexists(root / name) for name in
               ("activation-transaction.json", "activation-transaction.json.pending",
                "activation.json.pending"))


def remove_activation_intent(root):
    # Called only after FULL-synchronous SQLite commit, with admission excluded.
    for name in ("activation-transaction.json", "activation-transaction.json.pending",
                 "activation.json.pending"):
        path = root / name
        if os.path.lexists(path):
            private_file(path).unlink()
    fd = os.open(root, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


PREVIEW_READ_SECONDS = 0.25


def preview_backup(source, destination):
    """Bound each read-only SQLite backup; never hold an owner flock."""
    deadline = time.monotonic() + PREVIEW_READ_SECONDS
    source.execute("PRAGMA busy_timeout=0")

    def progress(status, remaining, total):
        f.require(time.monotonic() < deadline, "preview_read_budget_exceeded")
        f.require(status not in (sqlite3.SQLITE_BUSY, sqlite3.SQLITE_LOCKED), "preview_database_busy")

    source.backup(destination, pages=32, progress=progress, sleep=0)


@contextmanager
def activation_store(config_path, readonly=False):
    # Disarm must remain possible after package/config drift. Only resolve the
    # existing store here; arm performs full live configuration validation.
    config = load(config_path)
    f.require(type(config) is dict and isinstance(config.get("coordinator"), str), "invalid_config")
    root = f.private_dir(config["coordinator"])
    with (nullcontext() if readonly else locked(root / "owner-daemon.lock")), \
            (nullcontext() if readonly else locked(root / "owner-admission.lock")):
        path = private_file(root / "owner.sqlite3")
        f.require(not any(Path(str(path) + suffix).exists() for suffix in
                          ("-journal", "-wal", "-shm")), "owner_recovery_required")
        with closing(sqlite3.connect(":memory:" if readonly else path.as_uri() + "?mode=rw",
                                     uri=not readonly)) as db:
            if readonly:
                with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=0)) as source:
                    f.require(source.execute("PRAGMA journal_mode").fetchone()[0] == "delete", "journal_mode")
                    preview_backup(source, db)
            db.row_factory = sqlite3.Row
            db.execute("PRAGMA synchronous=FULL")
            f.require(db.execute("PRAGMA journal_mode").fetchone()[0] ==
                      ("memory" if readonly else "delete"), "journal_mode")
            for table, columns in SCHEMA.items():
                row = db.execute("SELECT sql FROM sqlite_master WHERE name=?", (table,)).fetchone()
                f.require(row and row[0] == f"CREATE TABLE {table} ({columns})", "schema_drift")
            db.execute("BEGIN" if readonly else "BEGIN IMMEDIATE")
            row = db.execute("SELECT * FROM meta WHERE singleton=1").fetchone()
            f.require(row is not None, "state_drift")
            meta = dict(row)
            f.require(meta["schema_version"] == 1 and type(meta["control_generation"]) is int
                      and meta["control_generation"] >= 0, "state_drift")
            yield root, db, meta


def coordinator_activation_impact(root, dispatcher=None):
    """Read one coordinator snapshot without running/recovering its watchdog."""
    coordinator = ExistingCoordinator(root)
    f.require(not any(Path(str(coordinator.path) + suffix).exists()
                      for suffix in ("-journal", "-wal", "-shm")), "coordinator_recovery_required")
    with coordinator.connection(readonly=True) as db:
        db.execute("PRAGMA busy_timeout=250")
        row = db.execute("SELECT body FROM state WHERE id=1").fetchone()
        f.require(row is not None, "uninitialized_state")
        snapshot = f.strict_json(row[0])
    observed_at = time.time()
    actions = sorted(snapshot["actions"].values(), key=lambda value: value["id"])
    reportable = [a for a in actions if coordinator.watchdog_action_reportable(a, observed_at)]
    covered = [a["id"] for a in reportable if coordinator.watchdog_covers(snapshot, a, dispatcher)]
    excluded = [a["id"] for a in reportable if not coordinator.watchdog_covers(snapshot, a, dispatcher)]
    default_action = {}
    future_covered = coordinator.watchdog_covers(snapshot, default_action, dispatcher)
    coverage = dict(
        dispatcher=dispatcher, covered_actions=covered, excluded_actions=excluded,
        future_default_owner=coordinator.dispatch_owner(snapshot, default_action),
        future_default_covered=future_covered,
        manager_covered=coordinator.watchdog_covers(snapshot, None, dispatcher),
        counted_actions="unreported overdue dispatching/dispatched/running actions",
        condition="On an admitted tick reaching the watchdog; OFF, refusal or earlier failure runs no watchdog.",
        hand_stall_detection="manager responsibility (--hand-run is manual; no scheduled hand watchdog)"
        if not coordinator.watchdog_covers(snapshot, {"dispatch_owner": "hand"}, dispatcher)
        else "covered by this watchdog",
    )
    coverage["summary"] = (
        f"Owner watchdog on admitted ticks (unreported overdue actions): "
        f"covered={len(covered)}, excluded={len(excluded)}; "
        f"manager={'covered' if coverage['manager_covered'] else 'excluded'}; "
        f"new default actions={'covered' if future_covered else 'excluded'}; "
        f"hand stall detection={coverage['hand_stall_detection']}."
    )
    in_flight = []
    for action in actions:
        if action["status"] not in {"dispatching", "dispatched", "running",
                                    "dispatch_uncertain", "returned"}:
            continue
        overdue = action["deadline"] < observed_at
        covered_action = coordinator.watchdog_covers(snapshot, action, dispatcher)
        will_report = covered_action and coordinator.watchdog_action_reportable(action, observed_at)
        in_flight.append(dict(id=action["id"], status=action["status"], deadline=action["deadline"],
                              overdue=overdue, stall_reported=bool(action.get("stall_reported")),
                              dispatch_owner=coordinator.dispatch_owner(snapshot, action),
                              watchdog_covered=covered_action, watchdog_will_report=will_report,
                              watchdog_status="dispatch_uncertain" if will_report and
                              action["status"] == "dispatching" else action["status"]))
    manager = snapshot["manager"]
    if manager is not None:
        overdue = manager["deadline"] < observed_at
        manager = dict(id=manager["id"], deadline=manager["deadline"], overdue=overdue,
                       stall_reported=bool(manager.get("stall_reported")),
                       watchdog_will_report=coordinator.watchdog_manager_reportable(
                           snapshot, observed_at, dispatcher))
    return dict(observed_at=observed_at, revision=snapshot["revision"], enabled=snapshot["enabled"],
                in_flight=in_flight, overdue=[row for row in in_flight if row["overdue"]],
                manager=manager, watchdog_coverage=coverage,
                dispatch_control=snapshot.get("dispatch_control"),
                ownership={a["id"]: dict(owner=coordinator.dispatch_owner(snapshot, a),
                           claim=a.get("claim"), allocation_digest=a.get("allocation_digest"),
                           status=a["status"]) for a in snapshot["actions"].values()},
                warning=coverage["summary"] + " Armed operation is not read-only. "
                "For covered actions only, watchdog_will_report predicts unreported overdue "
                "dispatching/dispatched/running stalls if an admitted tick reaches the watchdog; "
                "dispatching becomes dispatch_uncertain. "
                "Covered watchdog mutations can occur while dispatch is paused; "
                "fixture-only processing can append events when dispatch is ready. "
                "This snapshot is advisory, not an ownership gate or reservation; claims and "
                "deadlines can change before a tick.")


def unresolved_holds(db):
    """Read receipts, including holds whose action is now in the other lane."""
    return [dict(row, dispatch_owner=None, excluded_from_owner_lane=None,
                 ownership_status="not_checked") for row in db.execute(
        "SELECT h.op_id,h.reason_code,h.first_seen,h.last_seen,o.action_id "
        "FROM holds h LEFT JOIN operations o ON o.op_id=h.op_id "
        "WHERE h.resolved_at IS NULL ORDER BY h.op_id")]


def activation_status(config_path):
    """Independent advisory reads: unavailable state is unknown, never empty/off."""
    config = load(config_path)
    f.require(type(config) is dict and isinstance(config.get("coordinator"), str), "invalid_config")
    root = f.private_dir(config["coordinator"])
    result = dict(state=None, generation=None, next_generation=None, stored=None,
                  config_digest=digest(config), package_digest=package_digest(),
                  scope="tmux-workers" if "transport" in config else "fixture-only",
                  recovery_required=activation_pending(root), coordinator=None,
                  complete=False, unavailable={}, unresolved_holds=None,
                  warning="Advisory independent reads, not an atomic snapshot or arming clearance. "
                  "Armed ticks can mutate shared coordinator watchdog and fixture events; "
                  "unavailable components must be inspected again while quiescent.")
    errors = (f.LaunchError, Rejected, OSError, sqlite3.Error, ValueError, TypeError, KeyError)
    try:
        with activation_store(config_path) as (_, db, meta):
            result["unresolved_holds"] = unresolved_holds(db)
            for hold in result["unresolved_holds"]:
                hold["ownership_status"] = "coordinator_unavailable"
            result.update(state=meta["requested_mode"], generation=meta["control_generation"],
                          next_generation=meta["control_generation"] + 1, stored=meta)
    except errors as exc:
        reason = str(exc) if isinstance(exc, (f.LaunchError, Rejected)) else type(exc).__name__
        result["unavailable"]["owner"] = dict(error=type(exc).__name__, reason=reason)
    try:
        result["coordinator"] = coordinator_activation_impact(root, "owner" if "transport" in config else None)
    except errors as exc:
        reason = str(exc) if isinstance(exc, (f.LaunchError, Rejected)) else type(exc).__name__
        result["unavailable"]["coordinator"] = dict(error=type(exc).__name__, reason=reason)
    if result["coordinator"] is not None and result["unresolved_holds"] is not None:
        ownership = result["coordinator"]["ownership"]
        claims = {digest(["tmux", key, "claim"]): key for key in ownership}
        for hold in result["unresolved_holds"]:
            hold["action_id"] = hold["action_id"] or claims.get(hold["op_id"])
            action = ownership.get(hold["action_id"])
            hold["ownership_status"] = ("resolved" if action is not None else
                                        "action_missing" if hold["action_id"] else "unmapped")
            if action is not None:
                hold["dispatch_owner"] = action["owner"]
                hold["excluded_from_owner_lane"] = (
                    "transport" in config and hold["dispatch_owner"] == "hand")
    result["complete"] = not result["unavailable"]
    return result


def preview_changes(before, after, path=()):
    """JSON field deltas, including missing versus null, for an advisory preview."""
    if isinstance(before, dict) and isinstance(after, dict):
        changes = []
        for key in sorted(before.keys() | after.keys()):
            if key not in before:
                changes.append(dict(path=[*path, key], operation="insert", after=after[key]))
            elif key not in after:
                changes.append(dict(path=[*path, key], operation="delete", before=before[key]))
            else:
                changes.extend(preview_changes(before[key], after[key], (*path, key)))
        return changes
    return [] if before == after else [dict(path=list(path), operation="update",
                                            before=before, after=after)]


def arming_preview(root, db, meta, config, authority, impact):
    """All arm checks have passed; predict fixture effects using only RAM writes.

    Deliberately refuse unsupported transport/replay previews instead of presenting
    a partial plan as exact. These are preview limits, not new arming policy.
    """
    f.require("transport" not in config, "preview_requires_fixture_only")
    f.require(not db.execute("SELECT 1 FROM operations WHERE phase IN ('intent','observed')").fetchone(),
              "preview_requires_settled_operations")
    after = dict(meta, requested_mode="armed", control_generation=authority["generation"],
                 activation_decision_id=authority["decision_id"],
                 activation_revision=authority["revision"], activation_digest=digest(authority))
    request = dict(identity=digest([authority["generation"], "fixture-tick"]), fixture_only=True)
    op_id = digest([authority["generation"], "fixture", request["identity"], None, None, "observe", 0])
    source = ExistingCoordinator(root)
    observed_at = time.time()
    with closing(sqlite3.connect(":memory:", isolation_level=None)) as memory:
        memory.row_factory = sqlite3.Row
        # The source connection is read-only; backup writes solely into RAM.
        with source.connection(readonly=True) as original:
            preview_backup(original, memory)
        row = memory.execute("SELECT body FROM state WHERE id=1").fetchone()
        f.require(row is not None and f.strict_json(row[0])["revision"] == impact["revision"],
                  "preview_coordinator_changed")

        class PreviewCoordinator(ExistingCoordinator):
            def __init__(self):
                self.clock = lambda: observed_at

            @contextmanager
            def connection(self, readonly=False):
                try:
                    yield memory
                finally:
                    # Match a real per-call connection's close/rollback semantics.
                    if memory.in_transaction:
                        memory.rollback()

        def rows():
            tables = {}
            for table, key in (("state", "id"), ("events", "seq"), ("audit", "seq"),
                               ("action_history", "id"), ("evidence", "digest"),
                               ("sqlite_sequence", "name")):
                tables[table] = {}
                for row in memory.execute(f"SELECT * FROM {table}"):
                    value = dict(row)
                    if "body" in value:
                        value["body"] = f.strict_json(value["body"])
                    tables[table][str(value[key])] = value
            return tables

        coordinator = PreviewCoordinator()
        before = rows()
        watchdog = coordinator.watchdog()
        held = bool(db.execute("SELECT 1 FROM holds WHERE resolved_at IS NULL").fetchone())
        readiness = coordinator.readiness()
        effect = not held and readiness in {"ready", "empty"} and not db.execute(
            "SELECT 1 FROM operations WHERE op_id=?", (op_id,)).fetchone()
        fixture = fixture_receipt(request) if effect else None
        if fixture:
            coordinator.event(fixture["event"])
        changes = preview_changes(before, rows())
    return dict(
        state="WOULD_ARM", dry_run=True, generation=authority["generation"],
        read_policy=dict(owner_locks=False, sqlite_backup_seconds=PREVIEW_READ_SECONDS,
                         sqlite_backup_pages=32, busy_wait_seconds=0,
                         warning="Advisory independent snapshots. SQLite briefly takes read locks; "
                         "each backup refuses on contention or its 250ms budget. "
                         "No owner lock spans the preview; revalidate at arm."),
        activation_digest=digest(authority), coordinator=impact,
        arm=dict(
            database=str(root / "owner.sqlite3"), table="meta", key=dict(singleton=1),
            before=meta, after=after,
            activation_file=dict(path=str(root / "activation.json"),
                                 operation="replace" if (root / "activation.json").exists() else "create",
                                 after=authority),
            transient_files=[str(root / name) for name in
                             ("activation-transaction.json.pending", "activation-transaction.json",
                              "activation.json.pending", "owner.sqlite3-journal")],
            intent=dict(authority=authority, previous_generation=meta["control_generation"])),
        first_admitted_tick=dict(
            observed_at=observed_at, coordinator_database=str(source.path),
            coordinator_changes=changes, watchdog_events=watchdog,
            state="HOLD" if held else "ACTIVE" if readiness in {"ready", "empty"} else "READY",
            readiness=readiness, fixture_event=fixture["event"] if fixture else None,
            owner_database=str(root / "owner.sqlite3"),
            owner_rows=dict(
                boots="INSERT boot_id=<new UUID>, owner_epoch=generation, host_boot_id, pid, "
                      "process_start, package_digest, started_at; UPDATE stopped_at, stop_reason=tick_exit",
                health=dict(component="watchdog", operation="INSERT OR REPLACE",
                            after=dict(last_attempt_at="<tick time>", last_success_at="<tick time>",
                                       consecutive_failures=0, next_probe_at=None, status="ok",
                                       evidence_digest=digest(watchdog))),
                operations=dict(op_id=op_id, final_phase="applied") if effect else None,
                observations=dict(observation_id=op_id, op_id=op_id) if effect else None,
                deliveries=dict(delivery_id=op_id, phase="applied") if effect else None),
            files=[str(root / "runs" / op_id / name) for name in
                   ("request.json", "receipt.json", "observations/1.json")] if effect else [],
            transient_files=[str(root / "owner.sqlite3-journal")]
                            + ([str(root / "coordinator.sqlite3-journal")] if changes else []),
            warning="WATCHDOG/EVENT HISTORY IS NOT UNDONE BY DISARM. No worker launch in fixture-only scope."),
        schedule="Arming neither fires a tick nor clears a latched schedule HOLD. "
                 "An installed schedule must be explicitly recovered before kernel admission; "
                 "held probes still update tick.json and publish owner-recurrence.json.",
        limitations="Point-in-time advisory, not a reservation. Inspect while coordinator is quiescent; "
                    "deadlines and state may change before admission. UUIDs, PIDs and wall times are "
                    "allocated only by the actual tick. Schedule/service/pin health is separate; "
                    "this preview does not clear or inspect its HOLD.")


def arm_owner(config_path, authority, dry_run=False):
    """Explicit local authority; never installs a schedule or invokes transport.

    File + SQLite have no shared physical transaction. Both owner locks exclude
    readers; a durable intent fences admission across crashes, including a crash
    after the SQL commit. Only explicit disarm clears an interrupted transaction.
    """
    with activation_store(config_path, readonly=dry_run) as (root, db, meta):
        f.require(not activation_pending(root), "activation_recovery_required")
        config = configuration(config_path)
        f.require(config["coordinator"] == str(root), "config_drift")
        f.require(meta["config_digest"] == digest(config)
                  and meta["package_digest"] == package_digest(), "state_drift")
        validate_authority(authority, config)
        f.require(authority["generation"] == meta["control_generation"] + 1,
                  "activation_generation_mismatch")
        f.require(meta["requested_mode"] == "off", "owner_already_armed")
        if os.path.lexists(root / "activation.json"):
            private_file(root / "activation.json")
        impact = coordinator_activation_impact(root, "owner" if "transport" in config else None)
        if "transport" in config:
            f.require(impact["dispatch_control"] is not None, "dispatch_handoff_required")
        if dry_run:
            return arming_preview(root, db, meta, config, authority, impact)
        f.write_json(root / "activation-transaction.json",
                     dict(authority=authority, previous_generation=meta["control_generation"]))
        f.write_json(root / "activation.json", authority)
        db.execute("UPDATE meta SET requested_mode='armed',control_generation=?,"
                   "activation_decision_id=?,activation_revision=?,activation_digest=? WHERE singleton=1",
                   (authority["generation"], authority["decision_id"],
                    authority["revision"], digest(authority)))
        db.commit()
        remove_activation_intent(root)
        return dict(state="ARMED", generation=authority["generation"], activation_digest=digest(authority),
                    coordinator=impact)


def disarm_owner(config_path, generation):
    """Revoke admission, consume a generation and recover interrupted activation.

    Retain activation.json as evidence. Existing processes and schedule HOLDs
    are deliberately unaffected; this is revocation, not worker termination.
    """
    f.require(type(generation) is int and generation >= 0, "activation_generation_required")
    with activation_store(config_path) as (root, db, meta):
        f.require(generation == meta["control_generation"], "activation_generation_mismatch")
        db.execute("UPDATE meta SET requested_mode='off',control_generation=? WHERE singleton=1",
                   (generation + 1,))
        db.commit()
        remove_activation_intent(root)
        return dict(state="OFF", generation=generation + 1)


def handoff(config_path, request):
    """One SQLite commit defines the cutover, with both dispatchers excluded.

    Cooperating hand dispatch uses --hand-run and the same locks/journal. Initial
    cutover requires the operator to quiesce legacy/manual key senders first.
    """
    f.require(type(request) is dict and set(request) ==
              {"expected_revision", "assignments", "evidence"}, "invalid_handoff")
    with activation_store(config_path) as (root, db, meta):
        coordinator = ExistingCoordinator(root)
        state = coordinator.snapshot()
        f.require(isinstance(request["assignments"], dict), "invalid_handoff")
        for key, selection in request["assignments"].items():
            action = state["actions"][key]
            if selection.get("to") == "owner" and action["status"] != "prepared":
                # A legacy hand claim has no daemon effect receipts. Never
                # fabricate those or relaunch it to make ownership appear valid.
                row = db.execute("SELECT * FROM operations WHERE action_id=? AND effect='claim'",
                                 (key,)).fetchone()
                f.require(row is not None and row["phase"] == "applied"
                          and row["config_generation"] == meta["control_generation"]
                          and row["authority_digest"] == meta["activation_digest"],
                          "handoff_requires_receipted_claim")
                receipt = load(root / row["receipt_artifact"])
                f.require(digest(receipt) == row["receipt_digest"]
                          and receipt["result"]["claim"] == action.get("claim")
                          and receipt["result"]["allocation_digest"] == action["allocation_digest"],
                          "handoff_claim_mismatch")
        control = coordinator._handoff(**request)
        return dict(state="HANDED_OFF", control=control,
                    coordinator=str(root), assignments=request["assignments"])


@contextmanager
def uninstalled_schedule(config_path, schedule_root):
    """Serialize with install/repin and bind the inspected receipt to this config."""
    f.require(schedule_root is not None, "reconfigure_requires_schedule")
    schedule_root = f.private_dir(schedule_root)
    with locked(schedule_root / "installation.lock"):
        receipt = load(schedule_root / "installation.json")
        f.require(receipt["pins"]["config"] == str(private_file(config_path)),
                  "reconfigure_schedule_config_mismatch")
        f.require(receipt["phase"] == "uninstalled", "reconfigure_requires_uninstalled")
        domain = installation_domain(receipt)
        sibling = ("user" if domain.startswith("gui/") else "gui") + "/" + str(os.getuid())
        for location in (domain, sibling):
            presence, _ = service(receipt["label"], location)
            f.require(presence == "absent" or
                      (location == sibling and presence == "domain_absent"),
                      "reconfigure_requires_absent_service")
        yield


def reconfigure_owner(config_path, replacement, schedule_root=None):
    """OFF-only repin of a recorded, uninstalled schedule; preserve history."""
    with uninstalled_schedule(config_path, schedule_root), activation_store(config_path) as (root, db, meta):
        f.require(meta["requested_mode"] == "off", "reconfigure_requires_off")
        config = configuration(replacement)
        f.require(config["coordinator"] == str(root), "coordinator_change_forbidden")
        f.require(not db.execute("SELECT 1 FROM operations WHERE phase != 'applied'").fetchone()
                  and not db.execute("SELECT 1 FROM holds WHERE resolved_at IS NULL").fetchone(),
                  "reconfigure_requires_settled_operations")
        state = ExistingCoordinator(root).snapshot()
        f.require(not any(a.get("dispatch_owner") == "owner" and a["status"] not in
                          {"prepared", "accepted", "failed", "cancelled"}
                          for a in state["actions"].values()), "reconfigure_requires_quiescent_owner")
        f.write_json(root / "activation-transaction.json",
                     dict(reconfigure=digest(config), previous_generation=meta["control_generation"]))
        f.write_json(config_path, config)
        db.execute("UPDATE meta SET config_digest=?,package_digest=? WHERE singleton=1",
                   (digest(config), package_digest()))
        db.commit()
        remove_activation_intent(root)
        return dict(state="OFF", config_digest=digest(config), package_digest=package_digest(),
                    generation=meta["control_generation"])


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
    def __init__(self, config_path, dispatcher="owner"):
        f.require(dispatcher in {"owner", "hand"}, "invalid_dispatcher")
        self.dispatcher = dispatcher
        self.config_path = Path(config_path)
        # Validate shared state only after acquiring the delivery lock in tick;
        # a cooperating dispatcher may currently be committing its SQLite journal.
        self.config = load(config_path)
        self.root = f.private_dir(self.config["coordinator"])
        self.c = ExistingCoordinator(self.root)
        if dispatcher == "hand":
            f.require("transport" in self.config, "hand_run_requires_transport")
        self.db = None

    def admit(self):
        f.require(configuration(self.config_path) == self.config, "config_drift")
        meta = dict(self.db.execute("SELECT * FROM meta WHERE singleton=1").fetchone())
        f.require(meta["schema_version"] == 1 and meta["config_digest"] == digest(self.config)
                  and meta["package_digest"] == package_digest(), "state_drift")
        f.require(meta["requested_mode"] == "armed", "owner_off")
        f.require(not activation_pending(self.root), "activation_recovery_required")
        authority = load(self.root / "activation.json")
        validate_authority(authority, self.config)
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
        self.c._dispatcher(state, current, self.dispatcher)
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
        action = self.step(action, "claim", claim_request, lambda: self.c.claim(action["id"], dispatcher=self.dispatcher))
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
                  lambda: self.c.dispatched(action["id"], action["claim"], agent, ack, dispatcher=self.dispatcher))
        self.step(action, "acknowledged", request,
                  lambda: self.c.acknowledge(action["id"], agent, action["allocation_digest"], ack, dispatcher=self.dispatcher))
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
                  lambda: self.c.returned(action["id"], agent, result, dispatcher=self.dispatcher))
        return "returned"

    def workers(self, adapter):
        """One bounded pass. An operation failure holds only its owning action."""
        outcomes = {}
        snapshot = self.c.snapshot()
        for action in snapshot["actions"].values():
            if (self.c.dispatch_owner(snapshot, action) not in (None, self.dispatcher)
                    or action["kind"] not in WORKER_KINDS):
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
                if not rows and action["status"] != "prepared" and self.dispatcher == "hand":
                    outcomes[action["id"]] = "external_hand_claim"
                    continue
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
        watchdog = self.c.watchdog(dispatcher=self.dispatcher)
        now = time.time()
        self.db.execute("INSERT OR REPLACE INTO health VALUES(?,?,?,?,?,?,?)",
                        ("watchdog", now, now, 0, None, "ok", digest(watchdog)))
        self.db.commit()
        # An action-local hand HOLD is visible in status but must not turn the
        # owner lane into HOLD (and vice versa). Unknown/global holds remain hard.
        excluded = {digest(["tmux", a["id"], "claim"]) for a in snapshot["actions"].values()
                    if self.c.dispatch_owner(snapshot, a) not in (None, self.dispatcher)}
        excluded.update(row["op_id"] for row in self.db.execute("SELECT op_id,action_id FROM operations")
                        if row["action_id"] in snapshot["actions"] and
                        self.c.dispatch_owner(snapshot, snapshot["actions"][row["action_id"]])
                        not in (None, self.dispatcher))
        held = any(row[0] not in excluded for row in self.db.execute(
            "SELECT op_id FROM holds WHERE resolved_at IS NULL"))
        readiness = self.c.readiness()
        return {"state": "HOLD" if held else "READY" if readiness in {"paused", "owned"} else "ACTIVE",
                "actions": outcomes, "routing": "tmux-workers", "fixture_only": False,
                "dispatcher": self.dispatcher, "unresolved_holds": unresolved_holds(self.db),
                "ownership": {a["id"]: self.c.dispatch_owner(snapshot, a)
                              for a in snapshot["actions"].values()}}

    def tick(self, adapter=None):
        adapter = configured_adapter(self.config) if adapter is None else adapter
        from owner_tmux import TmuxAdapter
        f.require(type(adapter) is FixedTestAdapter or (type(adapter) is TmuxAdapter
                  and adapter.config == self.config.get("transport")), "live_adapter_unavailable")
        path = private_file(self.root / "owner.sqlite3")
        with locked(self.root / "owner-daemon.lock"):
            f.require(not any(Path(str(path) + suffix).exists() for suffix in ("-journal", "-wal", "-shm")),
                      "owner_recovery_required")
            if self.dispatcher == "hand":
                f.require("dispatch_control" in self.c.snapshot(), "dispatch_handoff_required")
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
    f.require(isinstance(label, str) and
              re.fullmatch(r"com\.corbanu\.initiative-owner(?:\.[a-zA-Z0-9-]+)?", label), "invalid_label")
    try:
        result = subprocess.run(["/bin/launchctl", "print", f"{domain}/{label}"],
                                capture_output=True, text=True, timeout=5, env={})
    except UnicodeError as exc:
        raise f.LaunchError(f"service_observation_unavailable: {domain}/{label}") from exc
    if result.returncode == 0:
        return "present", result.stdout
    if result.returncode == 113 and f'Could not find service "{label}"' in result.stderr:
        return "absent", ""
    kind, uid = domain.split("/")
    missing = f"Could not find domain for {'user gui' if kind == 'gui' else 'uid'}: {uid}"
    if result.returncode == 112 and missing in result.stderr.splitlines():
        return "domain_absent", missing
    unavailable = "Could not print domain: 125: Domain does not support specified action"
    if kind == "gui" and result.returncode == 125 and unavailable in result.stderr.splitlines():
        # A real user without an Aqua session can have only a Background domain.
        # Activation permits this observation only for the sibling domain.
        return "domain_absent", unavailable
    raise f.LaunchError(f"service_observation_unavailable: {domain}/{label}")


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
        f.require(result["service"] != "domain_absent", "service_observation_unavailable")
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
        except BlockingIOError:
            result = {"state": "BUSY", "dispatcher": "owner"}
        except (f.LaunchError, Rejected) as exc:
            result = {"state": "HOLD", "reason": "owner_run_refused", "refusal": str(exc)}
        except Exception as exc:
            result = {"state": "ERROR", "reason": type(exc).__name__}
        status["completed_at"] = time.time()
        if result.get("reason") == "owner_run_refused":
            status.update(hold="owner_run_refused", first_refusal=status.get("first_refusal") or time.time(),
                          last_refusal=time.time(), refusal=result.get("refusal"))
        elif result["state"] == "BUSY":
            status.update(skipped=status["skipped"] + 1, previous_success=None)
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
    commands = parser.add_mutually_exclusive_group()
    commands.add_argument("--run", action="store_true")
    commands.add_argument("--hand-run", action="store_true", help="one hand-owned TMUX pass using the shared journal")
    commands.add_argument("--handoff", type=Path, help="revision-bound ownership partition/transfer JSON")
    commands.add_argument("--reconfigure", type=Path,
                          help="OFF-only config/package repin; requires --schedule with an uninstalled receipt")
    commands.add_argument("--arm", action="store_true",
                          help="arm shared coordinator mutation; fixture-only is not read-only; "
                          "inspect --activation-status before arming")
    commands.add_argument("--disarm", action="store_true")
    commands.add_argument("--activation-status", action="store_true",
                          help="inspect in-flight/overdue deadlines and watchdog effects without arming")
    parser.add_argument("--dry-run", action="store_true",
                        help="with --arm: validate and preview fixture-only changes without writing")
    parser.add_argument("--authority", type=Path)
    parser.add_argument("--generation", type=int)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--schedule", type=Path)
    parser.add_argument("--recover")
    parser.add_argument("--observe", action="store_true")
    parser.add_argument("--label", default="com.corbanu.initiative-owner")
    parser.add_argument("--publish-state", type=Path)
    args = parser.parse_args(argv)
    if args.dry_run and not args.arm:
        parser.error("--dry-run requires --arm")
    if args.handoff or args.reconfigure or args.hand_run:
        if (args.config is None or (args.schedule is not None and not args.reconfigure)
                or (args.reconfigure and args.schedule is None)
                or args.recover is not None or args.observe or args.publish_state
                or args.authority or args.generation is not None):
            parser.error("handoff/hand-run require only --config and their input; "
                         "reconfigure also requires --schedule")
        try:
            if args.handoff:
                result = handoff(args.config, load(args.handoff))
            elif args.reconfigure:
                result = reconfigure_owner(args.config, args.reconfigure, args.schedule)
            else:
                result = Kernel(args.config, dispatcher="hand").tick()
            print(encoded(result))
            return 0
        except (f.LaunchError, Rejected, OSError, sqlite3.Error, ValueError, TypeError, KeyError) as exc:
            reason = str(exc) if isinstance(exc, (f.LaunchError, Rejected)) else type(exc).__name__
            print(encoded(dict(state="BUSY" if isinstance(exc, BlockingIOError) else "REFUSED", reason=reason)))
            return 0 if args.hand_run and isinstance(exc, BlockingIOError) else 2
    if args.arm or args.disarm or args.activation_status:
        if (args.config is None or args.schedule or args.recover is not None or args.observe
                or args.publish_state or (args.arm and (args.authority is None or args.generation is not None))
                or (not args.arm and args.authority is not None)
                or (args.disarm and args.generation is None)
                or (args.activation_status and args.generation is not None)):
            parser.error("activation commands require --config; --arm requires --authority; "
                         "--disarm requires --generation; schedule options cannot be combined")
        try:
            result = (arm_owner(args.config, load(args.authority), dry_run=args.dry_run) if args.arm else
                      disarm_owner(args.config, args.generation) if args.disarm else
                      activation_status(args.config))
            print(encoded(result))
            return 0
        except (f.LaunchError, Rejected, OSError, sqlite3.Error, ValueError, TypeError, KeyError) as exc:
            reason = str(exc) if isinstance(exc, (f.LaunchError, Rejected)) else type(exc).__name__
            print(encoded(dict(state="REFUSED", reason=reason)))
            return 2
    if args.authority is not None or args.generation is not None:
        parser.error("--authority/--generation require an activation command")
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
    except (f.LaunchError, Rejected, OSError, sqlite3.Error, ValueError, TypeError, KeyError) as exc:
        reason = str(exc) if isinstance(exc, (f.LaunchError, Rejected)) else type(exc).__name__
        print(encoded(dict(state="BUSY" if isinstance(exc, BlockingIOError) else "HOLD",
                           reason="dispatch_busy" if isinstance(exc, BlockingIOError) else "owner_run_refused",
                           refusal=reason)))
        return 0 if isinstance(exc, BlockingIOError) else 2


if __name__ == "__main__":
    sys.exit(main())
