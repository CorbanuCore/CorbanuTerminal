"""Owner-only JSON bridge to the durable coordinator; no native tools are faked.

Each command takes a bounded JSON object on stdin and emits a result on stdout.
Use claim -> actual native tool -> dispatched -> actual worker ACK -> acknowledge.
A crash between those steps must go through native inspection and reconciliation.
This interface must never be exposed as a Slack/web arbitrary-command endpoint.
"""

import argparse
import json
import sqlite3
import sys

from coordinator import Coordinator, Rejected, encoded


OPERATIONS = {
    "initialize", "snapshot", "event", "set_enabled", "begin_manager",
    "fail_manager", "accept_decision", "claim", "dispatched", "acknowledge",
    "returned", "verify", "reconcile_dispatch", "watchdog",
    "read_evidence",
}


def execute(directory, operation, payload):
    if operation not in OPERATIONS or not isinstance(payload, dict):
        raise Rejected("unknown operation or non-object input")
    coordinator = Coordinator(directory)
    result = getattr(coordinator, operation)(**payload)
    return {"operation": operation, "result": result, "revision": coordinator.snapshot()["revision"]}


def main(argv=None, source=None, sink=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=sorted(OPERATIONS))
    parser.add_argument("--state", required=True)
    args = parser.parse_args(argv)
    source, sink = source or sys.stdin, sink or sys.stdout
    try:
        data = source.read(262145)
        if len(data.encode()) > 262144:
            raise Rejected("input exceeds 256 KiB")
        payload = json.loads(data) if data.strip() else {}
        # Reject NaN and other unbounded JSON values before opening state.
        encoded(payload)
        result = execute(args.state, args.operation, payload)
        # Output contains bounded state/packet plus wrapper; it has its own cap.
        rendered = encoded({"status": "ok", **result}, limit=524288)
    except (Rejected, ValueError, KeyError, TypeError, OSError, AttributeError, RecursionError, sqlite3.Error) as exc:
        sink.write(encoded({"status": "rejected", "error": str(exc)[:500]}) + "\n")
        return 2
    sink.write(rendered + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
