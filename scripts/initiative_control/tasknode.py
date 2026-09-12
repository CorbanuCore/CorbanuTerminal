#!/usr/bin/env python3
"""Goal-event-only Campaign Tracker outbox. Explicit setup required; no task actions."""
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import stat
import urllib.error
import urllib.request

from control import LEGACY_DELIVERY_SPRINT, RUN_STATUSES, STALE_SECONDS, atomic_json, checked_run, locked, now, read_json, safe_text, timestamp

ORIGIN = "https://tasknode.postfiat.org"
PREFIX = ORIGIN + "/api/terminal/tasknode/campaign-tracker"


def identifier(value):
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z0-9._:-]{1,200}", value):
        raise ValueError("invalid tracker identifier")
    return value


def checked_branch(value):
    safe_text(value, 300)
    if len(value.encode("utf-8")) > 500:
        raise ValueError("repository branch exceeds Tracker's 500-byte limit")
    return value


def event_for(run, workspace, task_ids, sequence):
    checked_run(run)
    checked_branch(run["branch"])
    identifier(workspace)
    if not task_ids or len(task_ids) > 32:
        raise ValueError("explicit owned Task Node task IDs required")
    for task in task_ids:
        identifier(task)
    # Deliberately omit transcripts, tool arguments, raw agent output and local paths.
    content = f"{run['sprint_id']}: worker reports {run['status']}. {run['summary']}\nProgress observation only; no acceptance or task-completion claim."
    safe_text(content)
    if len(content.encode()) > 2048:
        raise ValueError("progress event exceeds Tracker's 2048-byte limit")
    value = dict(instanceId="corbanu-control", sessionId=identifier(run["run_id"]),
                 turnId=run["sprint_id"], sequence=sequence, workspaceId=workspace,
                 kind="goal", occurredAt=timestamp(run["updated_at"]).astimezone(dt.timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z"),
                 content=content, taskIds=sorted(set(task_ids)),
                 coverage="manager_observed_worker_report_not_independent_acceptance",
                 goal={"id": run["sprint_id"], "status": "reported_" + run["status"], "active": run["status"] in {"working", "blocked", "awaiting_review"}},
                 repository={"label": "Corbanu Terminal", "branch": run["branch"], "commit": run["commit"]})
    value["id"] = "cc-" + hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()
    return value


def enqueue(state, run):
    if run.get("sprint_id") == LEGACY_DELIVERY_SPRINT:
        raise ValueError("PF-76-S01 requires source reconciliation; no enqueue or automatic alias")
    config = read_json(state / "control.json", state)["tasknode"]
    tasks = config.get("task_mappings", {}).get(run["sprint_id"], [])
    with locked(state / ".outbox.lock"):
        index_file = state / "outbox-index.json"
        index = read_json(index_file, state) if index_file.exists() else {}
        # Stable report hash prevents duplicates on repeated collector runs.
        observed = {k: v for k, v in run.items() if k != "updated_at"}
        key = hashlib.sha256(json.dumps({"run": observed, "workspace": config["workspace_id"], "tasks": tasks}, sort_keys=True).encode()).hexdigest()
        if key in index:
            return index[key]
        event = event_for(run, config["workspace_id"], tasks, len(index))
        destination = state / "outbox" / (event["id"] + ".json")
        if not destination.exists():
            atomic_json(destination, {"event": event, "status": "pending", "attempts": 0, "next_attempt_at": now()})
        index[key] = event["id"]
        atomic_json(index_file, index)
        return event["id"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        return None  # Never forward terminal/subscription credentials to another URL.


def credentials(path):
    if path.is_symlink() or stat.S_IMODE(path.stat().st_mode) & 0o077:
        raise ValueError("credentials file must be regular and owner-only (0600)")
    value = read_json(path, path.parent)
    if set(value) != {"terminal_session", "api_key"} or not all(isinstance(v, str) and v and "\n" not in v and "\r" not in v for v in value.values()):
        raise ValueError("credentials file must contain terminal_session and api_key")
    return value


def post(path, payload, auth):
    if path not in {"/enrollment", "/events"}:
        raise ValueError("only enrollment and progress events are supported")
    data = {**payload, "apiKey": auth["api_key"]}
    request = urllib.request.Request(PREFIX + path, data=json.dumps(data).encode(), method="POST",
                                     headers={"Content-Type": "application/json", "Authorization": "Bearer " + auth["terminal_session"]})
    opener = urllib.request.build_opener(NoRedirect)
    try:
        with opener.open(request, timeout=20) as response:
            result = json.loads(response.read(65537))
            if not isinstance(result, dict):
                raise ValueError("invalid tracker response")
            return response.status, result
    except urllib.error.HTTPError as error:
        return error.code, {}  # Never persist or echo response bodies that may contain sensitive data.
    except (urllib.error.URLError, TimeoutError, OSError, ValueError):
        return 0, {}


def enrolled(state, config):
    """Server-local receipt: a source sync cannot assert or erase enrollment."""
    try:
        receipt = read_json(state / "enrollment.json", state)
        return receipt.get("verified") is True and receipt.get("workspace_id") == config.get("workspace_id")
    except (OSError, ValueError, TypeError, AttributeError):
        return False


def enroll(state, auth, transport=post):
    config = read_json(state / "control.json", state)["tasknode"]
    workspace = identifier(config["workspace_id"])
    status, result = transport("/enrollment", {"workspaceId": workspace, "enabled": True}, auth)
    if not (200 <= status < 300 and result.get("ok") is True):
        raise ValueError(f"Enrollment not verified (HTTP {status}); no credentials logged")
    atomic_json(state / "enrollment.json", {"workspace_id": workspace, "verified": True, "verified_at": now()})


def checked_event(event, event_id):
    """Validate this adapter's immutable goal schema before preview or transport."""
    fields = {"id", "instanceId", "sessionId", "turnId", "sequence", "workspaceId",
              "kind", "occurredAt", "content", "taskIds", "coverage", "goal", "repository"}
    if not isinstance(event, dict) or set(event) != fields:
        raise ValueError("invalid immutable goal fields")
    if not re.fullmatch(r"cc-[a-f0-9]{64}", event_id) or event["id"] != event_id:
        raise ValueError("event identity differs from selected file")
    payload = {k: v for k, v in event.items() if k != "id"}
    if "cc-" + hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest() != event_id:
        raise ValueError("immutable event payload changed")
    for field in ("instanceId", "sessionId", "turnId", "workspaceId"):
        identifier(event[field])
    tasks = event["taskIds"]
    if not isinstance(tasks, list) or not tasks or len(tasks) > 32:
        raise ValueError("explicit task list required")
    for task in tasks:
        identifier(task)
    if tasks != sorted(set(tasks)):
        raise ValueError("task list must retain canonical identity")
    if type(event["sequence"]) is not int or not 0 <= event["sequence"] <= 2**53 - 1:
        raise ValueError("invalid goal sequence")
    if event["kind"] != "goal" or event["instanceId"] != "corbanu-control":
        raise ValueError("only delivery-control goal events supported")
    if not re.fullmatch(r"PF-\d{2}-S\d{2}", event["turnId"]):
        raise ValueError("invalid goal sprint")
    if not isinstance(event["goal"], dict) or set(event["goal"]) != {"id", "status", "active"} or event["goal"]["id"] != event["turnId"]:
        raise ValueError("invalid goal metadata")
    if event["goal"]["status"] not in {"reported_" + status for status in RUN_STATUSES} or type(event["goal"]["active"]) is not bool:
        raise ValueError("invalid reported goal state")
    if not isinstance(event["repository"], dict) or set(event["repository"]) != {"label", "branch", "commit"}:
        raise ValueError("invalid repository metadata")
    if event["coverage"] != "manager_observed_worker_report_not_independent_acceptance" or event["repository"]["label"] != "Corbanu Terminal":
        raise ValueError("invalid observation coverage")
    checked_branch(event["repository"]["branch"])
    if not re.fullmatch(r"[a-f0-9]{40}", event["repository"]["commit"]):
        raise ValueError("invalid source commit")
    if len(safe_text(event["content"]).encode()) > 2048:
        raise ValueError("goal content exceeds byte limit")
    safe_text(json.dumps(event), 12000)
    if not event["occurredAt"].endswith("Z") or timestamp(event["occurredAt"]) > timestamp(now()) + dt.timedelta(minutes=5):
        raise ValueError("invalid goal timestamp")
    return event


def prepare(state, event_id):
    """Read exactly one immutable record. No credentials, locks, writes or transport.

    This advisory snapshot is never a send authorization. Recheck all gates in
    a separately reviewed native sender immediately before any later delivery.
    """
    if not isinstance(event_id, str) or not re.fullmatch(r"cc-[a-f0-9]{64}", event_id):
        raise ValueError("prepare requires one exact immutable event ID")
    record = read_json(state / "outbox" / (event_id + ".json"), state)
    event = checked_event(record["event"], event_id)
    config = read_json(state / "control.json", state)["tasknode"]
    blockers = ["live_authority_entitlement_owner_and_target_lifecycle_unverified"]
    if event["turnId"] == LEGACY_DELIVERY_SPRINT:
        blockers.append("historical_source_reconciliation_required")
    if config.get("enabled") is not True:
        blockers.append("posting_disabled")
    local_enrollment = enrolled(state, config)
    if not local_enrollment:
        blockers.append("local_workspace_enrollment_unverified")
    if event["workspaceId"] != config.get("workspace_id") or event["taskIds"] != sorted(set(config.get("task_mappings", {}).get(event["turnId"], []))):
        blockers.append("mapping_or_workspace_changed")
    if record["status"] != "pending":
        blockers.append("record_not_pending")
    elif timestamp(record["next_attempt_at"]) > timestamp(now()):
        blockers.append("retry_backoff_active")
    if (timestamp(now()) - timestamp(event["occurredAt"])).total_seconds() > STALE_SECONDS:
        blockers.append("stale_observation_requires_review")
    return {"mode": "offline_single_event_preparation", "selected_count": 1,
            "network_writes": False, "send_authorized": False,
            "event_id": event_id, "local_workspace_enrollment": local_enrollment,
            "blockers": blockers, "payload": {"event": event}}


def flush(state, auth, transport=None):
    transport = transport or post
    config = read_json(state / "control.json", state)["tasknode"]
    if config.get("enabled") is not True or not enrolled(state, config):
        raise ValueError("live writeback disabled or workspace enrollment not verified")
    delivered = 0
    with locked(state / ".outbox.lock"):
        attempted = 0
        for path in sorted((state / "outbox").glob("*.json")):
            record = read_json(path, state)
            if record["event"].get("turnId") == LEGACY_DELIVERY_SPRINT:
                continue  # Retain raw historical payload AND delivery metadata unchanged.
            if record["status"] != "pending" or timestamp(record["next_attempt_at"]) > timestamp(now()):
                continue
            if attempted >= 20:
                break
            config = read_json(state / "control.json", state)["tasknode"]
            if config.get("enabled") is not True or not enrolled(state, config):
                break
            attempted += 1
            event = record["event"]
            try:
                checked_event(event, path.stem)
            except (ValueError, TypeError, KeyError, AttributeError):
                record.update(status="blocked", error="immutable_event_invalid")
                atomic_json(path, record)
                continue
            # Revalidate authority immediately before sending an old queued record.
            allowed = config.get("task_mappings", {}).get(event.get("turnId"), [])
            if event.get("kind") != "goal" or event.get("workspaceId") != config["workspace_id"] or not event.get("taskIds") or set(event["taskIds"]) != set(allowed):
                record.update(status="blocked", error="mapping_or_event_changed")
            else:
                status, result = transport("/events", {"event": event}, auth)
                record["attempts"] += 1
                if 200 <= status < 300 and result.get("ok") is True and result.get("id") == event["id"] and result.get("summaryState") != "deleted":
                    record.update(status="delivered", delivered_at=now())
                    delivered += 1
                elif status in {0, 429} or 500 <= status < 600:
                    delay = min(3600, 30 * 2 ** min(record["attempts"], 7))
                    record.update(next_attempt_at=(timestamp(now()) + dt.timedelta(seconds=delay)).isoformat(), error="transient_delivery_failure")
                    if record["attempts"] >= 8:
                        record["status"] = "blocked"
                else:
                    record.update(status="blocked", error=f"http_{status}_requires_operator")
            atomic_json(path, record)
    return delivered


def retry(state, event_id):
    if not re.fullmatch(r"cc-[a-f0-9]{64}", event_id):
        raise ValueError("invalid event identifier")
    with locked(state / ".outbox.lock"):
        path = state / "outbox" / (event_id + ".json")
        record = read_json(path, state)
        checked_event(record["event"], event_id)
        if record["event"]["turnId"] == LEGACY_DELIVERY_SPRINT:
            raise ValueError("historical events require reconciliation; retry is not migration")
        if record["status"] != "blocked":
            raise ValueError("only blocked deliveries can be explicitly retried")
        record.update(status="pending", attempts=0, next_attempt_at=now())
        record.pop("error", None)
        atomic_json(path, record)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "preview", "enqueue", "flush", "enroll", "retry", "status"))
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--credentials-file", type=Path)
    parser.add_argument("--confirm-live", action="store_true")
    parser.add_argument("--event-id", action="append")
    args = parser.parse_args()
    if args.event_id is not None and len(args.event_id) != 1:
        parser.error("--event-id must occur exactly once")
    args.event_id = args.event_id[0] if args.event_id else None
    if args.command in {"prepare", "preview"}:
        if not args.event_id or args.credentials_file or args.confirm_live or args.report:
            parser.error("offline preparation requires only --state and --event-id; live/report arguments forbidden")
        print(json.dumps(prepare(args.state, args.event_id), indent=2, sort_keys=True))
        return
    if args.event_id and args.command != "retry":
        parser.error("--event-id is only valid for prepare, preview or retry; flush is a batch operation")
    if args.command == "status":
        for path in sorted((args.state / "outbox").glob("*.json")):
            record = read_json(path, args.state)
            print(json.dumps({"id": record["event"]["id"], "status": record["status"], "attempts": record["attempts"], "error": record.get("error")}))
        return
    if args.command == "retry":
        if not args.event_id:
            parser.error("retry requires --event-id")
        retry(args.state, args.event_id)
        print("Same immutable event queued again; no network write performed")
        return
    if args.command == "enqueue":
        if not args.report:
            parser.error("enqueue requires --report")
        print(enqueue(args.state, read_json(args.report, args.report.parent)))
        return
    if not args.confirm_live or not args.credentials_file:
        parser.error("network writes require --confirm-live and private --credentials-file")
    auth = credentials(args.credentials_file)
    if args.command == "flush":
        print(f"Delivered {flush(args.state, auth)} progress events")
    else:
        enroll(args.state, auth)
        print("Enrollment verified; live event delivery still requires enabled: true")


if __name__ == "__main__":
    main()
