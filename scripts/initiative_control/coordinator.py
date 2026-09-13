"""Owner-operated durable coordinator. Worker output is evidence, never authority.

This module performs no native-tool, Git, Slack or inference calls. Durable claims
surround those effects in the owner bridge; an interrupted claim is uncertain,
not permission to repeat a side effect. Local database access is owner authority.
"""

from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import stat
import time
import uuid


class Rejected(ValueError):
    pass


KINDS = frozenset({"implement", "revise", "review", "design", "functional_test",
                   "evidence_review", "repair", "reconcile", "integrate",
                   "verify_integration", "ask_human", "prepare_successor",
                   "complete_sprint", "activate_successor", "wait", "pause", "cancel"})
PASSIVE = frozenset({"wait", "pause", "cancel", "ask_human", "prepare_successor"})
TERMINAL = frozenset({"accepted", "failed", "cancelled"})
ACTION_FIELDS = frozenset({"id", "kind", "workstream", "sprint", "rationale", "inputs",
                           "timeout_seconds", "expected_revision"})


def encoded(value, limit=262144):
    try:
        result = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    except (TypeError, ValueError) as exc:
        raise Rejected("not bounded JSON") from exc
    if len(result.encode()) > limit:
        raise Rejected("record exceeds JSON byte limit")
    return result


def digest(value):
    return hashlib.sha256(encoded(value).encode()).hexdigest()


def ident(value):
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.:-]{0,159}", value):
        raise Rejected("invalid identifier")
    return value


def require(condition, reason):
    if not condition:
        raise Rejected(reason)


class Coordinator:
    def __init__(self, directory, clock=time.time):
        self.directory = Path(directory).absolute()
        self.directory.mkdir(mode=0o700, parents=False, exist_ok=True)
        info = self.directory.lstat()
        require(stat.S_ISDIR(info.st_mode) and not self.directory.is_symlink()
                and info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700,
                "coordinator directory must be owner-only and not a symlink")
        self.path = self.directory / "coordinator.sqlite3"
        fd = os.open(self.path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
        try:
            info = os.fstat(fd)
            require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                    and stat.S_IMODE(info.st_mode) == 0o600 and info.st_nlink == 1,
                    "database must be owner-only regular file")
        finally:
            os.close(fd)
        self.clock = clock
        with self.connection() as db:
            db.executescript("""
                CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY CHECK(id=1), body TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS events (seq INTEGER PRIMARY KEY AUTOINCREMENT,
                    id TEXT NOT NULL UNIQUE, body TEXT NOT NULL, meaningful INTEGER NOT NULL,
                    consumed TEXT);
                CREATE TABLE IF NOT EXISTS audit (seq INTEGER PRIMARY KEY AUTOINCREMENT,
                    at REAL NOT NULL, operation TEXT NOT NULL, body TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS action_history (id TEXT PRIMARY KEY, body TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS evidence (digest TEXT PRIMARY KEY, body TEXT NOT NULL);
            """)

    @contextmanager
    def connection(self):
        db = sqlite3.connect(self.path, timeout=5, isolation_level=None)
        db.row_factory = sqlite3.Row
        try:
            yield db
        finally:
            db.close()

    @contextmanager
    def mutation(self, operation, evidence):
        with self.connection() as db:
            db.execute("BEGIN IMMEDIATE")
            try:
                row = db.execute("SELECT body FROM state WHERE id=1").fetchone()
                require(row is not None, "coordinator not initialized")
                state = json.loads(row[0])
                yield db, state
                self._archive_actions(db, state)
                state["revision"] += 1
                db.execute("UPDATE state SET body=? WHERE id=1", (encoded(state),))
                db.execute("INSERT INTO audit(at,operation,body) VALUES(?,?,?)",
                           (self.clock(), operation, encoded(evidence)))
                db.execute("COMMIT")
            except BaseException:
                db.execute("ROLLBACK")
                raise

    @staticmethod
    def _archive_actions(db, state):
        # Keep durable history in SQLite, not every past assignment in each fresh
        # manager packet. Retain the last three terminal actions per workstream.
        for stream in state["workstreams"]:
            terminal = sorted((a for a in state["actions"].values()
                               if a["workstream"] == stream and a["status"] in TERMINAL),
                              key=lambda a: a["sequence"])
            for action in terminal[:-3]:
                db.execute("INSERT INTO action_history(id,body) VALUES(?,?)", (action["id"], encoded(action)))
                del state["actions"][action["id"]]

    def initialize(self, workstreams, sprints, allocations):
        """Trusted owner seed, not a manager/worker-authored configuration."""
        require(1 <= len(workstreams) <= 3, "one to three workstreams required")
        for key, stream in workstreams.items():
            ident(key)
            require(stream["sprint"] in sprints and stream["mode"] in {"paused", "enabled"},
                    "invalid workstream")
        for key, sprint in sprints.items():
            ident(key)
            require(sprint["workstream"] in workstreams, "unknown sprint workstream")
            require(sprint["status"] in {"draft", "in_progress", "blocked", "completed"}, "invalid lifecycle")
            require(all(dep in sprints and dep != key for dep in sprint["dependencies"]), "unknown dependency")
        def visit(key, trail):
            require(key not in trail, "dependency cycle")
            for dep in sprints[key]["dependencies"]:
                visit(dep, trail | {key})
        for key in sprints:
            visit(key, set())
        for key, allocation in allocations.items():
            ident(key)
            require(allocation["sprint"] in sprints, "unknown allocation sprint")
            require(set(allocation["kinds"]) <= KINDS, "unknown action kind")
            require(isinstance(allocation["resources"], list) and bool(allocation["scope"]), "missing scope/resources")
            require(isinstance(allocation["inputs"], dict), "missing frozen assignment inputs")
            require(1 <= allocation["timeout_seconds"] <= 86400, "invalid timeout")
        state = {"revision": 0, "enabled": False, "workstreams": workstreams,
                 "sprints": sprints, "allocations": allocations, "actions": {}, "manager": None}
        with self.connection() as db:
            try:
                db.execute("INSERT INTO state(id,body) VALUES(1,?)", (encoded(state),))
            except sqlite3.IntegrityError as exc:
                raise Rejected("already initialized; never erase prior state") from exc
        return self.snapshot()

    def snapshot(self):
        with self.connection() as db:
            row = db.execute("SELECT body FROM state WHERE id=1").fetchone()
            require(row is not None, "coordinator not initialized")
            return json.loads(row[0])

    @staticmethod
    def _reference(db, value):
        body = encoded(value)
        key = digest(value)
        db.execute("INSERT OR IGNORE INTO evidence(digest,body) VALUES(?,?)", (key, body))
        # Full evidence remains retrievable, but never grows every manager packet.
        return {"evidence_digest": key, "bytes": len(body.encode()), "preview": body[:400]}

    def read_evidence(self, evidence_digest):
        require(re.fullmatch("[a-f0-9]{64}", evidence_digest) is not None, "invalid evidence digest")
        with self.connection() as db:
            row = db.execute("SELECT body FROM evidence WHERE digest=?", (evidence_digest,)).fetchone()
            require(row is not None, "unknown evidence")
            return json.loads(row[0])

    @staticmethod
    def _event(db, event, meaningful=True):
        ident(event["id"])
        body = encoded(event)
        prior = db.execute("SELECT body FROM events WHERE id=?", (event["id"],)).fetchone()
        if prior:
            require(prior[0] == body, "event ID reused with different content")
            return False
        db.execute("INSERT INTO events(id,body,meaningful) VALUES(?,?,?)",
                   (event["id"], body, int(meaningful)))
        return True

    def event(self, event, meaningful=True):
        # Duplicate delivery does not invalidate a running manager's revision.
        with self.connection() as db:
            db.execute("BEGIN IMMEDIATE")
            try:
                added = self._event(db, event, meaningful)
                if added:
                    row = db.execute("SELECT body FROM state WHERE id=1").fetchone()
                    require(row is not None, "coordinator not initialized")
                    state = json.loads(row[0])
                    state["revision"] += 1
                    db.execute("UPDATE state SET body=? WHERE id=1", (encoded(state),))
                db.execute("COMMIT")
                return added
            except BaseException:
                db.execute("ROLLBACK")
                raise

    def set_enabled(self, enabled, evidence):
        require(type(enabled) is bool and bool(evidence), "owner evidence required")
        with self.mutation("owner_enable", evidence) as (_, state):
            state["enabled"] = enabled
        # Stop new effects immediately; already dispatched work needs explicit shutdown.

    def begin_manager(self, timeout_seconds=600):
        require(1 <= timeout_seconds <= 3600, "invalid manager timeout")
        with self.mutation("begin_manager", {}) as (db, state):
            require(state["enabled"], "dispatch paused")
            require(state["manager"] is None, "manager cycle already owned")
            pending = db.execute("SELECT seq,body FROM events WHERE meaningful=1 AND consumed IS NULL ORDER BY seq LIMIT 24").fetchall()
            require(bool(pending), "no meaningful pending event")
            run = {"id": str(uuid.uuid4()), "revision": state["revision"] + 1,
                   "events": [row["seq"] for row in pending], "deadline": self.clock() + timeout_seconds}
            state["manager"] = run
            packet = {"state_revision": run["revision"], "manager_run": run["id"],
                      "events": [{"id": json.loads(row["body"])["id"],
                                  **self._reference(db, json.loads(row["body"]))} for row in pending],
                      "workstreams": state["workstreams"], "sprints": state["sprints"],
                      "allocations": state["allocations"], "actions": state["actions"]}
            packet["last_three_actions"] = {
                key: sorted((a for a in state["actions"].values() if a["workstream"] == key),
                            key=lambda a: a["sequence"])[-3:]
                for key in state["workstreams"]}
            # Validate before committing ownership, not while printing afterward.
            encoded(packet, limit=240000)
        return packet

    def fail_manager(self, run_id, reason):
        with self.mutation("manager_failed", {"run": run_id, "reason": reason}) as (db, state):
            require(state["manager"] and state["manager"]["id"] == run_id, "wrong manager")
            state["manager"] = None
            self._event(db, {"id": "manager-failed:" + run_id, "reason": reason})

    @staticmethod
    def _dependencies(state, sprint):
        return all(state["sprints"][dep]["status"] == "completed"
                   and state["sprints"][dep].get("archived") is True
                   for dep in state["sprints"][sprint]["dependencies"])

    def accept_decision(self, run_id, decision, launcher_receipt):
        with self.mutation("manager_decision", {"run": run_id, "receipt": launcher_receipt}) as (db, state):
            run = state["manager"]
            require(state["enabled"] and run and run["id"] == run_id, "wrong/paused manager")
            require(run["deadline"] >= self.clock(), "manager deadline expired")
            require(decision["state_revision"] == run["revision"] == state["revision"], "stale decision")
            actions = decision["actions"]
            require(isinstance(actions, list) and 1 <= len(actions) <= 24, "invalid action count")
            require(bool(launcher_receipt), "verified launcher receipt required")
            require(sum(a["status"] not in TERMINAL for a in state["actions"].values()) + len(actions) <= 48,
                    "pending assignment limit; reconcile existing work first")
            for index, action in enumerate(actions):
                require(isinstance(action, dict) and set(action) == ACTION_FIELDS, "unexpected action fields")
                key = ident(action["id"])
                require(len(key) <= 120, "action ID too long for derived event IDs")
                require(key not in state["actions"], "duplicate action ID")
                require(db.execute("SELECT id FROM action_history WHERE id=?", (key,)).fetchone() is None,
                        "action ID already archived")
                require(action["kind"] in KINDS and bool(action["rationale"]), "invalid action")
                require(isinstance(action["rationale"], str) and len(action["rationale"].encode()) <= 1000,
                        "rationale must be bounded text")
                require(action["expected_revision"] == run["revision"], "wrong action revision")
                allocation = state["allocations"].get(action["inputs"].get("allocation"))
                require(allocation is not None, "unallocated action")
                require(action["kind"] in allocation["kinds"] and action["sprint"] == allocation["sprint"], "out of scope")
                sprint = state["sprints"][action["sprint"]]
                require(action["workstream"] == sprint["workstream"], "wrong workstream")
                require(action["inputs"] == {"allocation": action["inputs"]["allocation"], **allocation["inputs"]},
                        "manager cannot alter frozen assignment inputs")
                require(0 < action["timeout_seconds"] <= allocation["timeout_seconds"], "timeout exceeds authority")
                if action["kind"] not in PASSIVE:
                    require(state["workstreams"][action["workstream"]]["mode"] == "enabled", "workstream paused")
                    require(self._dependencies(state, action["sprint"]), "unfinished dependency")
                state["actions"][key] = {**action, "status": "prepared", "manager_run": run_id,
                                         "resources": allocation["resources"], "scope": allocation["scope"],
                                         "allocation_digest": digest(allocation), "created": self.clock(),
                                         "sequence": [state["revision"], index]}
            for seq in run["events"]:
                db.execute("UPDATE events SET consumed=? WHERE seq=? AND consumed IS NULL", (run_id, seq))
            state["manager"] = None

    def claim(self, action_id):
        with self.mutation("claim", {"action": action_id}) as (_, state):
            require(state["enabled"], "dispatch paused")
            action = state["actions"][action_id]
            require(action["status"] == "prepared", "action already claimed")
            if action["kind"] not in PASSIVE:
                require(state["workstreams"][action["workstream"]]["mode"] == "enabled", "workstream paused")
                require(self._dependencies(state, action["sprint"]), "unfinished dependency")
            for other in state["actions"].values():
                if other["id"] != action_id and other["status"] not in TERMINAL | {"prepared"}:
                    require(not set(action["resources"]) & set(other["resources"]), "resource owned")
            action.update(status="dispatching", claim=str(uuid.uuid4()),
                          deadline=self.clock() + action["timeout_seconds"], updated=self.clock(), dispatch_epoch=0)
            receipt = json.loads(encoded(action))
        return receipt

    def dispatched(self, action_id, claim, agent_id, native_receipt):
        ident(agent_id)
        require(bool(native_receipt), "native dispatch receipt required")
        with self.mutation("dispatched", {"action": action_id, "native": native_receipt}) as (db, state):
            action = state["actions"][action_id]
            require(action["status"] == "dispatching" and action["claim"] == claim, "wrong dispatch claim")
            action.update(status="dispatched", agent=agent_id, dispatch_receipt=self._reference(db, native_receipt), updated=self.clock())

    def acknowledge(self, action_id, agent_id, allocation_digest, native_receipt):
        with self.mutation("acknowledged", {"action": action_id, "native": native_receipt}) as (db, state):
            action = state["actions"][action_id]
            require(action["status"] == "dispatched" and action["agent"] == agent_id, "wrong agent/state")
            require(action["allocation_digest"] == allocation_digest and bool(native_receipt), "wrong allocation ACK")
            action.update(status="running", ack_receipt=self._reference(db, native_receipt), updated=self.clock())

    def returned(self, action_id, agent_id, result):
        with self.mutation("returned", {"action": action_id, "result": result}) as (db, state):
            action = state["actions"][action_id]
            require(action["status"] == "running" and action["agent"] == agent_id, "wrong agent/state")
            reference = self._reference(db, result)
            action.update(status="returned", result=reference, updated=self.clock())
            self._event(db, {"id": "returned:" + action_id, "action": action_id, "result": reference})

    def verify(self, action_id, evidence, accepted):
        require(type(accepted) is bool and bool(evidence), "owner verification evidence required")
        with self.mutation("verified", {"action": action_id, "evidence": evidence}) as (db, state):
            action = state["actions"][action_id]
            require(action["status"] == "returned", "no returned evidence")
            reference = self._reference(db, evidence)
            action.update(status="accepted" if accepted else "failed", verification=reference, updated=self.clock())
            self._event(db, {"id": "verified:" + action_id, "accepted": accepted, "evidence": reference})

    def reconcile_dispatch(self, action_id, evidence, agent_id=None):
        """Owner inspects native state first. No automatic duplicate launch."""
        require(bool(evidence), "native reconciliation evidence required")
        with self.mutation("reconcile_dispatch", evidence) as (db, state):
            action = state["actions"][action_id]
            allowed = {"dispatching", "dispatch_uncertain"} if agent_id else {"dispatching", "dispatch_uncertain", "dispatched", "running"}
            require(action["status"] in allowed, "not a reconcilable dispatch")
            action.update(status="dispatched" if agent_id else "failed", updated=self.clock())
            if not agent_id:
                action["owner_failure"] = self._reference(db, evidence)
            if agent_id:
                action["agent"] = ident(agent_id)
                action["deadline"] = self.clock() + action["timeout_seconds"]
                action.pop("stall_reported", None)
                action["dispatch_epoch"] += 1
            self._event(db, {"id": f"dispatch-reconciled:{action_id}:{state['revision']}", "evidence": evidence})

    def watchdog(self):
        """Report each stall once; never re-launch on timeout alone."""
        found = []
        snapshot = self.snapshot()
        now = self.clock()
        overdue = [a["id"] for a in snapshot["actions"].values()
                   if a["status"] in {"dispatching", "dispatched", "running"}
                   and a["deadline"] < now and not a.get("stall_reported")]
        manager = snapshot["manager"]
        if not overdue and not (manager and manager["deadline"] < now and not manager.get("stall_reported")):
            return found
        with self.mutation("watchdog", {"observed_at": now}) as (db, state):
            manager = state["manager"]
            if manager and manager["deadline"] < now and not manager.get("stall_reported"):
                # Launcher/process reconciliation required before another manager.
                event = {"id": "manager-stall:" + manager["id"], "manager": manager["id"]}
                if self._event(db, event):
                    found.append(event)
                manager["stall_reported"] = True
            for action in state["actions"].values():
                if (action["status"] not in {"dispatching", "dispatched", "running"}
                        or action["deadline"] >= now or action.get("stall_reported")):
                    continue
                action["stall_reported"] = True
                if action["status"] == "dispatching":
                    action["status"] = "dispatch_uncertain"
                event = {"id": f"stall:{action['id']}:{action['dispatch_epoch']}", "action": action["id"], "status": action["status"]}
                self._event(db, event)
                found.append(event)
        return found
