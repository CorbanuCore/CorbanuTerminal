#!/usr/bin/env python3
"""One deterministic refresh; enabled, mapped writeback follows explicit setup."""
import argparse
from pathlib import Path

from control import atomic_json, collect, now, publish, read_json
from tasknode import credentials, enrolled, enqueue, flush


def tick(root):
    try:
        refresh(root)
    except Exception:
        atomic_json(root / "site/health.json", {"ok": False, "attempted_at": now(),
                    "error": "Refresh failed; displaying the last successful snapshot. Inspect publisher service logs."})
        raise


def refresh(root):
    state = root / "state"
    data = collect(root / "source", state)
    config = read_json(state / "control.json", state)["tasknode"]
    result = {"checked_at": now(), "enabled": config.get("enabled") is True,
              "enrollment_verified": enrolled(state, config), "delivered_this_run": 0,
              "unmapped_runs": 0, "rejected_runs": 0}
    for run in data["runs"]:
        if not config.get("task_mappings", {}).get(run["sprint_id"]):
            result["unmapped_runs"] += 1
            continue
        try:
            enqueue(state, run)
        except (OSError, ValueError, KeyError, TypeError):
            result["rejected_runs"] += 1
            result["error"] = "A mapped report was rejected; inspect its size, fields and task mapping."
    if config.get("enabled") is True:
        try:
            result["delivered_this_run"] = flush(state, credentials(root / "private/tasknode.json"))
        except (OSError, ValueError, KeyError, TypeError):
            result["error"] = "Writeback needs credential/enrollment/operator recovery; no secrets logged"
    counts = {}
    for path in (state / "outbox").glob("*.json"):
        try:
            status = read_json(path, state)["status"]
            if status not in {"pending", "delivered", "blocked"}:
                raise ValueError("invalid outbox status")
        except (OSError, ValueError, KeyError, TypeError):
            status = "invalid"
            result["error"] = "An outbox record needs manager inspection; no payload is published."
        counts[status] = counts.get(status, 0) + 1
    result["outbox"] = counts
    atomic_json(state / "writeback-status.json", result)
    publish(root / "source", state, root / "site")
    print("Refresh complete; live writeback " + ("enabled" if result["enabled"] else "disabled"))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    tick(parser.parse_args().root)
