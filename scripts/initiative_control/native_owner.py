"""Bounded owner-only native handoff; never a scheduler or worker acceptance API.

CLI: one JSON request line in, at most one native_request line out, one trusted
host response line in, then one owner_receipt line out. The host invokes actual
native tools; no command execution, dynamic adapters, credentials or model calls.
Only the owner may supply the adapter or stdio responses. See the QA contract.
"""

import argparse
import json
import os
import select
import sys
import time
import uuid

from coordinator import Coordinator, Rejected, TERMINAL, digest, encoded, ident, require


LIMIT = 65536
WORK_KINDS = {"implement", "revise", "review", "design", "functional_test",
              "evidence_review", "repair", "reconcile"}
EFFECTS = {"spawn": "spawned", "start-work": "submitted", "close": "closed"}


def bounded(value):
    encoded(value, LIMIT)
    return value


def binding(action):
    return {"action_id": action["id"], "claim": action["claim"],
            "allocation_digest": action["allocation_digest"]}


class NativeOwner:
    """Adapter(request, timeout) performs exactly one host operation.

    Adapter exceptions mean unknown outcome. Callers must impose the supplied
    timeout; the concrete CLI transport enforces it. Poll is native read-only,
    though accepted observations are journaled locally. No effect is retried.
    """

    def __init__(self, coordinator, adapter):
        self.c, self.adapter = coordinator, adapter
        with self.c.connection() as db:
            db.execute("CREATE TABLE IF NOT EXISTS native_owner "
                       "(action TEXT PRIMARY KEY, body TEXT NOT NULL)")

    @staticmethod
    def _load(db, action_id):
        row = db.execute("SELECT body FROM native_owner WHERE action=?", (action_id,)).fetchone()
        return json.loads(row[0]) if row else {}

    @staticmethod
    def _save(db, action_id, journal):
        db.execute("INSERT OR REPLACE INTO native_owner VALUES(?,?)",
                   (action_id, encoded(journal)))

    def journal(self, action_id):
        with self.c.connection() as db:
            return self._load(db, action_id)

    def _action(self, action_id):
        state = self.c.snapshot()
        if action_id in state["actions"]:
            return state["actions"][action_id]
        with self.c.connection() as db:
            row = db.execute("SELECT body FROM action_history WHERE id=?", (action_id,)).fetchone()
            require(row is not None, "unknown action")
            return json.loads(row[0])

    def _gate(self, state, action):
        require(action["kind"] in WORK_KINDS, "not a native worker action")
        require(action["allocation_digest"] == digest(state["allocations"].get(action["inputs"]["allocation"])),
                "stale allocation")
        self.c._executable(state, action)
        for other in state["actions"].values():
            if other["id"] != action["id"] and other["status"] not in TERMINAL | {"prepared"}:
                require(not set(action["resources"]) & set(other["resources"]), "resource owned")

    def _sync(self, action_id):
        """Replay durable receipts into existing core APIs, never external tools."""
        journal = self.journal(action_id)
        spawn = journal.get("spawn", {}).get("response")
        if not spawn:
            return
        action = self._action(action_id)
        agent = spawn["agent_id"]
        if action["status"] == "dispatching":
            self.c.dispatched(action_id, action["claim"], agent, spawn)
        elif action["status"] == "dispatch_uncertain":
            self.c.reconcile_dispatch(action_id, spawn, agent)
        else:
            require(action.get("agent") == agent, "core/native identity conflict")
        action = self._action(action_id)
        if journal.get("ack") and action["status"] == "dispatched":
            self.c.acknowledge(action_id, agent, action["allocation_digest"], journal["ack"])
        terminal = journal.get("terminal")
        action = self._action(action_id)
        if terminal and action["status"] == "running":
            self.c.returned(action_id, agent, terminal)
        elif terminal and action["status"] == "dispatched":
            # An actual startup failure, not an observation timeout.
            self.c.reconcile_dispatch(action_id, terminal)

    def inspect(self, action_id):
        ident(action_id)
        action, journal = self._action(action_id), self.journal(action_id)
        pending = [op for op in EFFECTS if op in journal and "response" not in journal[op]]
        terminal = journal.get("terminal", {}).get("status")
        if pending or action["status"] in {"dispatching", "dispatch_uncertain"} and not journal:
            status, next_action = "reconciliation_required", "inspect actual native effect; reconcile exact receipt"
        elif action["status"] == "returned":
            status, next_action = "awaiting_owner_verification", "owner verifies evidence, then integrates if authorized"
        elif terminal == "failed":
            status, next_action = "native_failed", "owner inspects failure and closes native agent"
        elif journal.get("ack") and "start-work" not in journal:
            status, next_action = "awaiting_work_submission", "owner invokes start-work after fresh authority checks"
        elif journal.get("spawn", {}).get("response") and not journal.get("ack"):
            status, next_action = "awaiting_ack", "poll actual native identity for allocation-bound ACK"
        else:
            status, next_action = action["status"], "owner chooses next bounded operation"
        return {"type": "owner_receipt", "action_id": action_id, "status": status,
                "core_status": action["status"], "agent_id": action.get("agent"),
                "native_status": terminal or journal.get("observation", {}).get("status"),
                "effects": {op: {"request_id": journal[op]["request"]["request_id"],
                                  "receipted": "response" in journal[op]}
                            for op in EFFECTS if op in journal},
                "closed": "response" in journal.get("close", {}), "next_owner_action": next_action}

    def _validate(self, request, response):
        bounded(response)
        common = {"request_id", "binding", "agent_id", "status", "receipt"}
        require(isinstance(response, dict) and common <= set(response), "invalid host response")
        require(response["request_id"] == request["request_id"] and response["binding"] == request["binding"],
                "wrong request/binding")
        ident(response["agent_id"])
        require(request["agent_id"] in {None, response["agent_id"]}, "wrong native identity")
        require(isinstance(response["receipt"], dict) and response["receipt"], "actual native receipt required")
        operation, status = request["operation"], response["status"]
        extra = {"ack"} if status == "ready" else {"result"} if status in {"returned", "failed"} else set()
        require(set(response) == common | extra, "unexpected response fields")
        if operation in EFFECTS:
            require(status == EFFECTS[operation], "unknown effect outcome; reconcile")
        else:
            require(status in {"ready", "running", "timed_out", "returned", "failed"}, "invalid observation")
        if status == "ready":
            require(response["ack"] == {**request["binding"], "agent_id": response["agent_id"],
                                       "ack": "ready_no_work"}, "wrong allocation-bound ACK")
        if status in {"returned", "failed"}:
            require(isinstance(response["result"], dict) and response["result"], "native result required")

    def _receipt(self, action_id, request, response, evidence=None):
        self._validate(request, response)
        operation = request["operation"]
        with self.c.mutation("native_receipt", {"action": action_id, "operation": operation}) as (db, state):
            journal = self._load(db, action_id)
            action = state["actions"][action_id]
            require(binding(action) == request["binding"], "changed native binding")
            if operation in EFFECTS:
                entry = journal[operation]
                require(entry["request"] == request, "wrong durable intent")
                require("response" not in entry or entry["response"] == response, "conflicting receipt")
                if operation == "spawn":
                    for row in db.execute("SELECT body FROM native_owner WHERE action<>?", (action_id,)):
                        other = json.loads(row[0]).get("spawn", {}).get("response", {})
                        require(other.get("agent_id") != response["agent_id"], "native identity already bound")
                entry["response"] = response
                if operation == "start-work":
                    journal.pop("observation", None)
                if evidence:
                    entry["reconciliation"] = evidence
            else:
                require(action.get("agent") == response["agent_id"], "wrong observed identity")
                status = response["status"]
                if status == "ready":
                    require(not journal.get("terminal"), "native worker already terminal")
                    journal["ack"] = response
                if status in {"returned", "failed"}:
                    require(not journal.get("terminal") or journal["terminal"] == response,
                            "conflicting terminal observation")
                    require(status == "failed" or "response" in journal.get("start-work", {}),
                            "return before confirmed work submission")
                    journal["terminal"] = response
                journal["observation"] = response
            self._save(db, action_id, journal)
        self._sync(action_id)

    def reconcile(self, action_id, operation, response, evidence):
        """Owner supplies an actual lookup receipt; absence/uncertainty grants no retry."""
        require(operation in EFFECTS and isinstance(evidence, dict) and evidence,
                "effect and owner reconciliation evidence required")
        bounded(evidence)
        journal = self.journal(action_id)
        if operation == "spawn" and not journal:
            # Recover the core-claim/journal gap, or import an owner-inspected
            # external claim. This path only records evidence; it emits no tool.
            action = self._action(action_id)
            require(action["status"] in {"dispatching", "dispatch_uncertain"}, "no dispatch to reconcile")
            request = {"type": "native_request", "request_id": ident(response["request_id"]),
                       "operation": "spawn", "binding": binding(action), "agent_id": None}
            with self.c.mutation("native_recovered_intent", evidence) as (db, _):
                require(not self._load(db, action_id), "native intent already recorded")
                self._save(db, action_id, {"spawn": {"request": request}})
            journal = self.journal(action_id)
        require(operation in journal, "no recorded native intent")
        self._receipt(action_id, journal[operation]["request"], response, evidence)
        return self.inspect(action_id)

    def run(self, action_id, operation, timeout=30):
        ident(action_id)
        require(operation in {*EFFECTS, "poll"}, "unsupported native operation")
        require(type(timeout) in {int, float} and 0 < timeout <= 60, "timeout must be 0..60 seconds")
        self._sync(action_id)
        journal, action = self.journal(action_id), self._action(action_id)
        if operation in journal or journal.get("terminal") and operation == "poll":
            return self.inspect(action_id)
        if operation == "spawn":
            require(action["kind"] in WORK_KINDS, "not a native worker action")
            require(action["status"] == "prepared", "existing claim requires native reconciliation")
            action = self.c.claim(action_id)
        request = {"type": "native_request", "request_id": str(uuid.uuid4()),
                   "operation": operation, "binding": binding(action), "agent_id": action.get("agent")}
        if operation == "poll":
            require(request["agent_id"] is not None, "spawn identity requires reconciliation")
        else:
            with self.c.mutation("native_intent", {"action": action_id, "operation": operation}) as (db, state):
                journal = self._load(db, action_id)
                require(operation not in journal, "native intent already recorded")
                action = state["actions"][action_id]
                self._gate(state, action)
                require(binding(action) == request["binding"], "stale dispatch binding")
                if operation == "spawn":
                    request["startup"] = {"instruction": "Acknowledge binding and scope only; do no work until start-work.",
                                          "scope": action["scope"], "ack": "ready_no_work"}
                else:
                    require(action.get("agent") == request["agent_id"] and request["agent_id"] is not None,
                            "native identity missing or changed")
                    require(not any("response" not in journal[op] for op in EFFECTS if op in journal),
                            "unresolved native effect")
                    if operation == "start-work":
                        require(action["status"] == "running" and journal.get("ack") and not journal.get("terminal"),
                                "strict ACK required before work")
                        request["assignment"] = {key: action[key] for key in
                                                 ("kind", "workstream", "sprint", "scope", "inputs", "timeout_seconds")}
                    else:
                        require(journal.get("terminal"), "observe native terminal state before close")
                bounded(request)
                journal[operation] = {"request": request}
                self._save(db, action_id, journal)
            # Recheck just before the boundary. A committed intent remains held
            # if authority changes here; it is never cleared to authorize retry.
            state = self.c.snapshot()
            self._gate(state, state["actions"][action_id])
        try:
            response = self.adapter(request, timeout)
            self._receipt(action_id, request, response)
        except (Exception, KeyboardInterrupt):
            # Do not export host exception text (which may contain private data).
            result = self.inspect(action_id)
            result["observation"] = "unknown" if operation in EFFECTS else "unavailable"
            result["next_owner_action"] = "inspect native state and reconcile; no automatic effect retry"
            return result
        return self.inspect(action_id)


class StdioHost:
    """Concrete bounded JSON-lines transport to the owner's actual native tools."""

    def __init__(self, fd, sink):
        self.fd, self.sink = fd, sink

    def read(self, timeout):
        deadline, data = time.monotonic() + timeout, bytearray()
        while len(data) <= LIMIT:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([self.fd], [], [], remaining)[0]:
                raise TimeoutError("host response deadline")
            byte = os.read(self.fd, 1)
            require(bool(byte), "host disconnected")
            if byte == b"\n":
                return bounded(json.loads(data))
            data.extend(byte)
        raise Rejected("host frame exceeds limit")

    def __call__(self, request, timeout):
        self.emit(request, timeout)
        return self.read(timeout)

    def emit(self, value, timeout):
        data = memoryview((encoded(value, LIMIT) + "\n").encode())
        fd, deadline = self.sink.fileno(), time.monotonic() + timeout
        blocking = os.get_blocking(fd)
        os.set_blocking(fd, False)
        try:
            while data:
                remaining = deadline - time.monotonic()
                if remaining <= 0 or not select.select([], [fd], [], remaining)[1]:
                    raise TimeoutError("host output deadline")
                try:
                    data = data[os.write(fd, data):]
                except BlockingIOError:
                    continue
        finally:
            os.set_blocking(fd, blocking)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=[*EFFECTS, "poll", "inspect", "reconcile"])
    parser.add_argument("--state", required=True, help="existing private coordinator directory")
    parser.add_argument("--timeout", type=float, default=30)
    args = parser.parse_args(argv)
    try:
        require(0 < args.timeout <= 60, "timeout must be 0..60 seconds")
        host = StdioHost(sys.stdin.fileno(), sys.stdout)
        payload = host.read(args.timeout)
        require(isinstance(payload, dict), "object required")
        fields = {"action_id", "effect", "response", "evidence"} if args.operation == "reconcile" else {"action_id"}
        require(set(payload) == fields, "unexpected input fields")
        require(os.path.isfile(os.path.join(args.state, "coordinator.sqlite3")), "existing coordinator required")
        owner = NativeOwner(Coordinator(args.state), host)
        if args.operation == "inspect":
            result = owner.inspect(payload["action_id"])
        elif args.operation == "reconcile":
            result = owner.reconcile(payload["action_id"], payload["effect"], payload["response"], payload["evidence"])
        else:
            result = owner.run(payload["action_id"], args.operation, args.timeout)
        failed = result["status"] in {"reconciliation_required", "native_failed"} or result.get("observation")
        failed = failed or result["native_status"] in {"timed_out", "failed"}
        code = 2 if failed else 0
    except (Exception, KeyboardInterrupt):
        result, code = {"type": "owner_receipt", "status": "rejected",
                        "next_owner_action": "inspect durable state; request denied or outcome unknown"}, 2
    try:
        StdioHost(sys.stdin.fileno(), sys.stdout).emit(result, min(60, max(0.01, args.timeout)))
    except (OSError, TimeoutError):
        return 2
    return code


if __name__ == "__main__":
    raise SystemExit(main())
