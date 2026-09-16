#!/usr/bin/env python3
"""Goal-event-only Campaign Tracker outbox. Explicit setup required; no task actions."""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import urllib.error
import urllib.request

from control import RUN_STATUSES, STALE_SECONDS, atomic_json, checked_run, delivery_hold, locked, now, read_json, safe_text, timestamp

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
    if reason := delivery_hold(run.get("sprint_id"), run):
        raise ValueError(reason + "; no enqueue or automatic alias")
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
            record = {"event": event, "status": "pending", "attempts": 0, "next_attempt_at": now()}
            # Local provenance only: preserve the immutable wire payload and ID.
            if "source_namespace" in run:
                record["source_namespace"] = run["source_namespace"]
            atomic_json(destination, record)
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


def post(path, payload, auth, *, idempotency_key=None):
    if path not in {"/enrollment", "/events"}:
        raise ValueError("only enrollment and progress events are supported")
    data = {**payload, "apiKey": auth["api_key"]}
    request = urllib.request.Request(PREFIX + path, data=json.dumps(data).encode(), method="POST",
                                     headers={"Content-Type": "application/json", "Authorization": "Bearer " + auth["terminal_session"]})
    if idempotency_key is not None:
        request.add_header("Idempotency-Key", idempotency_key)
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
    if delivery_hold(event["turnId"], record):
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


def request_digest(payload):
    """Digest of the logical request, excluding transport credentials."""
    return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()


def immutable_json(path, value):
    """Create once, fsync before transport; never replace an existing receipt."""
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    if path.parent.is_symlink():
        raise ValueError("receipt directory must not be a symlink")
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w") as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
    # Persist both the receipt and a newly created receipts-directory entry.
    for parent in (path.parent, path.parent.parent):
        directory = os.open(parent, os.O_RDONLY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)


def send(state, event_id, *, live=False, dry_run=False, activation_file=None,
         credentials_file=None, transport=None):
    """Select one PF80 event; retries return receipts and never re-POST.

    A durable intent without a result is uncertain, even if the process crashed
    before HTTP. Reconciliation is external; deleting receipts is not recovery.
    The batch enablement flag must stay OFF and queue records stay unchanged.
    """
    if type(live) is not bool or type(dry_run) is not bool or live == dry_run:
        raise ValueError("send requires exactly one of --dry-run or --live")
    if dry_run and (activation_file or credentials_file):
        raise ValueError("dry-run forbids activation and credentials")
    preview = prepare(state, event_id)
    event = preview["payload"]["event"]
    config = read_json(state / "control.json", state)["tasknode"]
    mapping = config.get("task_mappings", {}).get("PF-80-S01")
    if (event["turnId"] != "PF-80-S01" or not isinstance(mapping, list)
            or not mapping or event["taskIds"] != sorted(set(mapping))
            or event["workspaceId"] != config.get("workspace_id")):
        raise ValueError("explicit matching PF-80-S01 mapping and workspace required")
    digest = request_digest(preview["payload"])
    base = {"schema": 1, "event_id": event_id, "request_digest": digest,
            "idempotency_key": event_id, "selected_count": 1}
    if dry_run:
        return {**preview, **base, "mode": "dry_run"}
    if not activation_file or not credentials_file:
        raise ValueError("live requires owner activation and credentials file")
    # Shares the queue lock for exclusion only; never calls flush/retry/enqueue.
    with locked(state / ".outbox.lock"):
        preview = prepare(state, event_id)
        if request_digest(preview["payload"]) != digest:
            raise ValueError("selected request changed")
        config = read_json(state / "control.json", state)["tasknode"]
        if config.get("enabled") is not False:
            raise ValueError("batch posting must remain explicitly disabled")
        allowed_blockers = {"posting_disabled",
                            "live_authority_entitlement_owner_and_target_lifecycle_unverified"}
        if set(preview["blockers"]) - allowed_blockers:
            raise ValueError("selected event has unresolved local gates")
        activation_file = Path(activation_file)
        metadata = activation_file.lstat()
        if (not stat.S_ISREG(metadata.st_mode) or metadata.st_uid != os.getuid()
                or stat.S_IMODE(metadata.st_mode) & 0o077):
            raise ValueError("owner activation must be an owner-only regular file")
        activation = read_json(activation_file, activation_file.parent)
        required = {"schema", "owner", "enabled", "event_id", "request_digest",
                    "workspace_id", "task_ids", "origin", "expires_at", "gates"}
        gates = {"identity", "entitlement", "enrollment", "target_lifecycle", "payload_review"}
        if (not isinstance(activation, dict) or set(activation) != required
                or type(activation["schema"]) is not int or activation["schema"] != 1
                or activation["enabled"] is not True
                or activation["event_id"] != event_id or activation["request_digest"] != digest
                or activation["workspace_id"] != event["workspaceId"]
                or activation["task_ids"] != event["taskIds"]
                or activation["origin"] != ORIGIN
                or not isinstance(activation["gates"], dict)
                or set(activation["gates"]) != gates
                or any(v is not True for v in activation["gates"].values())
                or timestamp(activation["expires_at"]) <= timestamp(now())):
            raise ValueError("owner activation does not authorize this exact request")
        identifier(activation["owner"])
        directory = state / "send-receipts"
        intent_path = directory / (event_id + ".intent.json")
        result_path = directory / (event_id + ".result.json")
        if intent_path.exists() or result_path.exists():
            intent = read_json(intent_path, state)
            if any(intent.get(k) != v for k, v in base.items()):
                raise ValueError("existing intent differs from selected request")
            if not result_path.exists():
                return {**base, "outcome": "uncertain", "network_writes": False,
                        "replayed": True, "reconciliation_required": True}
            receipt = read_json(result_path, state)
            if any(receipt.get(k) != v for k, v in base.items()):
                raise ValueError("existing receipt differs from selected request")
            return {**receipt, "replayed": True, "network_writes": False}
        auth = credentials(Path(credentials_file))
        immutable_json(intent_path, {**base, "created_at": now(),
                       "activation_digest": request_digest(activation)})
        # Once intent is durable, every exception leaves a non-retriable attempt.
        try:
            if transport is None:
                status, response = post("/events", preview["payload"], auth, idempotency_key=event_id)
            else:
                status, response = transport("/events", preview["payload"], auth)
            ok = isinstance(response, dict) and response.get("ok") is True
            matches = isinstance(response, dict) and response.get("id") == event_id
            deleted = isinstance(response, dict) and response.get("summaryState") == "deleted"
            success = type(status) is int and 200 <= status < 300 and ok and matches and not deleted
            redacted = {"ok": ok, "event_id_matches": matches, "deleted": deleted}
            http_status = status if type(status) is int and 0 <= status <= 599 else 0
        except Exception:
            success, http_status, redacted = False, 0, {}
        receipt = {**base, "created_at": now(), "http_status": http_status,
                   "response": redacted, "outcome": "delivered" if success else "uncertain",
                   "network_writes": True, "reconciliation_required": not success}
        immutable_json(result_path, receipt)
        return receipt


PROPOSED_TASKS = (
    "task_865f75c6911f953c6586cdc1f4531e4f",
    "task_b3e8506327fb906173fd68b2f642221b",
    "task_789a0f3bd75b41d1eca20cae698f04cf",
)


def identity_check():
    """Read-only installed helper probes; stdout is a redacted receipt.

    No auth files, task acceptance, balance, signing or enrollment commands.
    CLI status does not establish Campaign Tracker enrollment or entitlement.
    """
    receipt = {"schema": 1, "mode": "identity_check", "checked_at": now(),
               "network_writes": False, "profile": "unverified",
               "entitlement": "unverified", "enrollment": "unverified",
               "tasks": {task: "unverified" for task in PROPOSED_TASKS},
               "commands": [], "blockers": []}
    home, raw_profile = os.environ.get("CODEX_HOME"), os.environ.get("CORBANU_TASKNODE_PROFILE")
    if home and any(os.environ.get(alias) and Path(os.environ[alias]).resolve() != Path(home).resolve()
                    for alias in ("CORBANU_HOME", "PFTERMINAL_HOME")):
        receipt["blockers"].append("conflicting_inherited_home")
        return receipt
    if not home or not raw_profile:
        receipt["blockers"].append("missing_inherited_scope")
        return receipt
    try:
        profile = json.loads(raw_profile)
        if profile is not None and (not isinstance(profile, str) or not profile):
            raise ValueError()
    except (ValueError, TypeError):
        receipt["blockers"].append("invalid_inherited_profile")
        return receipt
    debug_homes = {os.environ.get("CORBANU_DEBUG_HOME", str(Path.home() / ".corbanu-debug")),
                   os.environ.get("PFTERMINAL_DEBUG_HOME", str(Path.home() / ".pfterminal-debug"))}
    names = (("corbanu-debug", "corbanu", "pfterminal-debug", "pfterminal") if home in debug_homes
             else ("corbanu", "pfterminal", "corbanu-debug", "pfterminal-debug"))
    binary = next((found for name in names if (found := shutil.which(name))), None)
    if not binary:
        receipt["blockers"].append("installed_helper_missing")
        return receipt

    def run_read(args, *, parse=True):
        receipt["commands"].append(["tasknode", *args])
        result = subprocess.run([binary, "tasknode", *args], stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                text=True, timeout=15, check=False)
        # Never persist or display helper stderr or unfiltered JSON.
        if result.returncode != 0 or len(result.stdout) > 1048576:
            raise ValueError("helper_read_failed")
        if any(word in (result.stdout + result.stderr).lower()
               for word in ("keychain", "securityagent", "credential prompt")):
            raise ValueError("native_credential_prompt")
        return json.loads(result.stdout) if parse else result.stdout

    try:
        help_text = run_read(["--help"], parse=False)
        if "--profile" not in help_text:
            receipt["blockers"].append("installed_helper_lacks_profile_scope")
            return receipt
        local = run_read(["link", "status", "--json"])
        if (not isinstance(local, dict) or local.get("ok") is not True
                or "profile" not in local or local["profile"] != profile
                or local.get("origin") != ORIGIN):
            raise ValueError("profile_or_origin_unverified")
        receipt["profile"] = "matches_inherited_scope"
        status = run_read(["status", "--origin", ORIGIN, "--json"])
        if not isinstance(status, dict) or status.get("ok") is not True:
            raise ValueError("account_status_unverified")
        receipt["account_status"] = "read_successfully"
        receipt["blockers"].extend(["tracker_entitlement_not_exposed_by_read_only_cli",
                                    "tracker_enrollment_not_exposed_by_read_only_cli"])
        for task in PROPOSED_TASKS:
            detail = run_read(["task", "show", task, "--origin", ORIGIN, "--json"])
            card = detail.get("task", {}) if isinstance(detail, dict) else {}
            if not isinstance(card, dict):
                raise ValueError("invalid_target_card")
            state = card.get("statusKey", card.get("status"))
            if card.get("id", card.get("taskId")) == task and state in (
                    "Proposed", "Accepted", "Submitted", "Rewarded", "Refused",
                    "proposed", "accepted", "submitted", "rewarded", "refused"):
                receipt["tasks"][task] = state.lower()
                if state.lower() != "proposed":
                    receipt["blockers"].append("target_no_longer_proposed")
            else:
                receipt["blockers"].append("target_state_unverified")
    except (OSError, ValueError, subprocess.TimeoutExpired):
        receipt["blockers"].append("helper_read_failed_or_prompt_stop")
    return receipt


class FlushResult(int):
    """Keep the delivered-count contract and expose single-send skips."""

    def __new__(cls, delivered, single_sent):
        result = super().__new__(cls, delivered)
        result.single_sent = single_sent
        return result


def flush(state, auth, transport=None):
    transport = transport or post
    config = read_json(state / "control.json", state)["tasknode"]
    if config.get("enabled") is not True or not enrolled(state, config):
        raise ValueError("live writeback disabled or workspace enrollment not verified")
    delivered = 0
    single_sent = []
    with locked(state / ".outbox.lock"):
        attempted = 0
        for path in sorted((state / "outbox").glob("*.json")):
            record = read_json(path, state)
            if (state / "send-receipts" / (path.stem + ".intent.json")).exists():
                single_sent.append(path.stem)
                continue  # Intent alone blocks retransmission, even without a result.
            if delivery_hold(record["event"].get("turnId"), record):
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
    return FlushResult(delivered, single_sent)


def retry(state, event_id):
    if not re.fullmatch(r"cc-[a-f0-9]{64}", event_id):
        raise ValueError("invalid event identifier")
    with locked(state / ".outbox.lock"):
        path = state / "outbox" / (event_id + ".json")
        record = read_json(path, state)
        checked_event(record["event"], event_id)
        if delivery_hold(record["event"]["turnId"], record):
            raise ValueError("historical events require reconciliation; retry is not migration")
        if record["status"] != "blocked":
            raise ValueError("only blocked deliveries can be explicitly retried")
        record.update(status="pending", attempts=0, next_attempt_at=now())
        record.pop("error", None)
        atomic_json(path, record)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("send", "identity-check", "prepare", "preview", "enqueue", "flush", "enroll", "retry", "status"))
    parser.add_argument("--state", type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true")
    mode.add_argument("--live", action="store_true")
    parser.add_argument("--owner-activation-file", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--credentials-file", type=Path)
    parser.add_argument("--confirm-live", action="store_true")
    parser.add_argument("--event-id", action="append")
    args = parser.parse_args()
    if args.event_id is not None and len(args.event_id) != 1:
        parser.error("--event-id must occur exactly once")
    args.event_id = args.event_id[0] if args.event_id else None
    if args.command == "identity-check":
        if any((args.state, args.event_id, args.credentials_file, args.confirm_live,
                args.report, args.dry_run, args.live, args.owner_activation_file)):
            parser.error("identity-check accepts no state, auth or write arguments")
        print(json.dumps(identity_check(), indent=2, sort_keys=True))
        return
    if not args.state:
        parser.error("--state is required")
    if args.command == "send":
        if (not args.event_id or args.report or args.confirm_live
                or args.live == args.dry_run
                or (args.dry_run and (args.credentials_file or args.owner_activation_file))
                or (args.live and (not args.credentials_file or not args.owner_activation_file))):
            parser.error("send requires one --event-id and --dry-run OR --live with owner activation and credentials")
        print(json.dumps(send(args.state, args.event_id, live=args.live, dry_run=args.dry_run,
                              activation_file=args.owner_activation_file,
                              credentials_file=args.credentials_file), indent=2, sort_keys=True))
        return
    if args.live or args.dry_run or args.owner_activation_file:
        parser.error("single-event flags require send")
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
        result = flush(args.state, auth)
        print(f"Delivered {result} progress events")
        if result.single_sent:
            print(json.dumps({"single_sent": result.single_sent}))
    else:
        enroll(args.state, auth)
        print("Enrollment verified; live event delivery still requires enabled: true")


if __name__ == "__main__":
    main()
