#!/usr/bin/env python3
"""One deterministic refresh; enabled, mapped writeback follows explicit setup."""
import argparse
from pathlib import Path

from control import atomic_json, collect, now, publish, read_json
from tasknode import CLI_CONTEXT, QueueLockTimeout, credentials, delivery_status, enrolled, enqueue, flush


def tick(root):
    try:
        refresh(root)
    except QueueLockTimeout as error:
        atomic_json(root / "site/health.json", {"ok": False, "attempted_at": now(), "error": str(error)})
        raise
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
    lock_error = None
    for run in data["runs"]:
        if not config.get("task_mappings", {}).get(run["sprint_id"]):
            result["unmapped_runs"] += 1
            continue
        try:
            enqueue(state, run)
        except QueueLockTimeout as error:
            lock_error = error
            result["error"] = str(error)
            break
        except (OSError, ValueError, KeyError, TypeError):
            result["rejected_runs"] += 1
            result["error"] = "A mapped report was rejected; inspect its size, fields and task mapping."
    if config.get("enabled") is True and lock_error is None:
        try:
            result["delivered_this_run"] = flush(state, credentials(root / "private/tasknode.json"))
        except QueueLockTimeout as error:
            lock_error = error
            result["error"] = str(error)
        except (OSError, ValueError, KeyError, TypeError):
            result["error"] = "Writeback needs credential/enrollment/operator recovery; no secrets logged"
    counts = {}
    for path in (state / "outbox").glob("*.json"):
        try:
            status = delivery_status(state, path.stem, read_json(path, state))
            if status not in {"pending", "delivered", "blocked", "uncertain"}:
                raise ValueError("invalid outbox status")
        except (OSError, ValueError, KeyError, TypeError):
            status = "invalid"
            result["error"] = " ".join(filter(None, (result.get("error"),
                "An outbox record needs manager inspection; no payload is published.")))
        counts[status] = counts.get(status, 0) + 1
    result["outbox"] = counts
    if counts.get("uncertain"):
        reason = "Delivery is uncertain; no automatic retry; external reconciliation or explicit named batch retry required."
        result["error"] = " ".join(filter(None, (result.get("error"), reason)))
    atomic_json(state / "writeback-status.json", result)
    publish(root / "source", state, root / "site")
    if lock_error is not None:
        raise lock_error
    print("Refresh complete; live writeback " + ("enabled" if result["enabled"] else "disabled"))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    token = CLI_CONTEXT.set(True)
    try:
        tick(parser.parse_args().root)
    except QueueLockTimeout as error:
        parser.exit(2, f"Task Node refresh refused: {error}\n")
    finally:
        CLI_CONTEXT.reset(token)
