"""utm-par2 transport check (method: 'Transport check'): for one run, the guest mediator's requests in the tester's
window (start - 120 s .. end + 5 s), the host relay's requests in the same window, and the tunnel log's reconnects.
usage: transport_check.py RUN_DIR EVIDENCE_DIR RELAY_LOG"""
import datetime
import json
import pathlib
import sys

run, ev, relay = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])


def ns(stamp):
    return datetime.datetime.strptime(stamp.strip(), "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=datetime.timezone.utc).timestamp() * 1e9


t0, t1 = ns((run / "executor-start.txt").read_text()) - 120e9, ns((run / "executor-end.txt").read_text()) + 5e9


def rows(path):
    out = []
    for line in path.read_text().splitlines():
        try:
            e = json.loads(line)
        except ValueError:
            continue
        if t0 <= e.get("time_ns", 0) <= t1:
            out.append(e)
    return out


med = rows(run / "mediator-at-end.jsonl")
fwd = [e for e in med if e.get("event") == "forwarded"]
rel = [e for e in rows(relay) if "status" in e]
tunnel = [l for l in (ev / "tunnel.log").read_text().splitlines()
          if t0 <= ns(l.split(" ")[0]) <= t1] if (ev / "tunnel.log").exists() else []
secs = sorted(float(e.get("seconds", 0)) for e in fwd if e.get("role") == "product")
res = {
    "run": run.name,
    "mediator_forwarded": len(fwd),
    "mediator_non_200": sum(str(e.get("status")) != "200" for e in fwd),
    "mediator_session_error": sum(e.get("event") == "session_error" for e in med),
    "mediator_by_role": {r: sum(e.get("role") == r for e in fwd) for r in ("product", "executor")},
    "product_median_seconds": secs[len(secs) // 2] if secs else None,
    "relay_requests": len(rel),
    "relay_non_200": sum(str(e.get("status")) != "200" for e in rel),
    "tunnel_events_in_window": tunnel,
}
res["ok"] = (res["mediator_non_200"] == 0 and res["mediator_session_error"] == 0 and res["relay_non_200"] == 0
             and not any("ssh exited" in l for l in tunnel))
print(json.dumps(res))
