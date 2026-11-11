"""Associate named outcomes with one bounded BEP invocation.

Adapted from citadelculture #1394. IDs/checksums associate artifacts; they are
not authentication against a hostile producer, nor proof of fresh execution.
"""

import hashlib
import json
import re
from uuid import UUID

from bazel_evidence_io import EvidenceError, bounded_read, local_log
from collect_bazel_test_results import libtest_rows

MAX_BEP_BYTES = 32 * 1024 * 1024
MAX_EVENTS = 100_000
MAX_ATTEMPTS = 10_000
MAX_TOTAL_LOG_BYTES = 64 * 1024 * 1024
MAX_METHODS = 100_000
# Match the current native Windows target groups, not the original four crates.
CRATES = ("tui", "core", "state", "login", "windows-sandbox-rs", "network-proxy",
          "process-hardening", "sandboxing", "exec-server", "utils/pty", "app-server")
TARGET = re.compile(r"//codex-rs/(?:" + "|".join(CRATES) + r")(?:/[A-Za-z0-9_.+-]+)*:[A-Za-z0-9_.+-]+\Z")
CONFIG = re.compile(r"[A-Za-z0-9_-]{1,128}\Z")
STATUSES = {"PASSED", "FLAKY", "TIMEOUT", "FAILED", "INCOMPLETE", "REMOTE_FAILURE",
            "FAILED_TO_BUILD", "TOOL_HALTED_BEFORE_TESTING", "NO_STATUS"}


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceError("duplicate_json_key")
        result[key] = value
    return result


def attempt_identity(value):
    if not isinstance(value, dict):
        raise EvidenceError("invalid_attempt_identity")
    target = value.get("label")
    config = value.get("configuration", {})
    config = config.get("id") if isinstance(config, dict) else None
    if (not isinstance(target, str) or len(target) > 2048
            or not TARGET.fullmatch(target) or any(p in (".", "..") for p in target.split("/"))
            or not isinstance(config, str) or not CONFIG.fullmatch(config)):
        raise EvidenceError("invalid_attempt_identity")
    result = {"target": target, "configuration": config}
    for field in ("run", "shard", "attempt"):
        number = value.get(field, 0)  # Protobuf int32 default.
        if type(number) is not int or not 0 <= number < 2**31:
            raise EvidenceError("invalid_attempt_identity")
        result[field] = number
    return result


def execution_evidence(result):
    execution = result.get("executionInfo")
    strategy = execution.get("strategy") if isinstance(execution, dict) else None
    evidence = {"strategy": strategy if strategy in (
        "local", "remote", "sandboxed", "worker", "windows-sandbox"
    ) else "unknown"}
    for field in ("testAttemptStartMillisEpoch", "testAttemptDurationMillis"):
        value = result.get(field)
        if isinstance(value, str) and re.fullmatch(r"[0-9]{1,18}", value):
            evidence[field] = value
        elif type(value) is int and 0 <= value < 10**18:
            evidence[field] = str(value)
    return evidence


def collect(bep_path, testlogs_root, invocation_id, checkout_sha, workflow_outcome="unknown"):
    report = {
        "schemaVersion": 1, "invocationId": None, "checkoutSha": None,
        "workflowOutcome": workflow_outcome if workflow_outcome in (
            "success", "failure", "cancelled", "skipped"
        ) else "unknown",
        "captureStatus": "incomplete", "issues": [], "attempts": [],
        "limitations": [
            "IDs and checksums associate artifacts, not independent producer authenticity.",
            "Named rows do not establish fresh execution or every conditional test body.",
            "Absent names are not passes; filtered tests are not inferred.",
        ],
    }
    issues = report["issues"]
    try:
        valid_uuid = isinstance(invocation_id, str) and str(UUID(invocation_id)) == invocation_id.lower()
    except ValueError:
        valid_uuid = False
    if not valid_uuid or not isinstance(checkout_sha, str) or not re.fullmatch(r"[0-9a-f]{40}", checkout_sha):
        issues.append("invalid_invocation_binding")
        return report
    report.update(invocationId=invocation_id, checkoutSha=checkout_sha)
    try:
        data = bounded_read(bep_path, MAX_BEP_BYTES)
        report["bepSha256"] = hashlib.sha256(data).hexdigest()
        lines = data.splitlines()
        if len(lines) > MAX_EVENTS:
            raise EvidenceError("event_limit")
        events = [json.loads(line, object_pairs_hook=unique_object) for line in lines]
        if any(not isinstance(event, dict) for event in events):
            raise EvidenceError("invalid_event")
    except (EvidenceError, ValueError, UnicodeError, RecursionError) as error:
        issues.append(str(error) if isinstance(error, EvidenceError) else "malformed_bep")
        return report
    starts = [event["started"] for event in events if "started" in event]
    metadata = [event["buildMetadata"] for event in events if "buildMetadata" in event]
    if len(starts) != 1 or not isinstance(starts[0], dict) or starts[0].get("uuid") != invocation_id:
        issues.append("invocation_mismatch")
        return report
    if (len(metadata) != 1 or not isinstance(metadata[0], dict)
            or not isinstance(metadata[0].get("metadata"), dict)
            or metadata[0]["metadata"].get("CORBANU_CHECKOUT_SHA") != checkout_sha):
        issues.append("checkout_mismatch")
        return report
    report["observedInvocationId"] = invocation_id
    report["observedCheckoutSha"] = checkout_sha
    finishes = [event["finished"] for event in events if "finished" in event]
    if not events or events[-1].get("lastMessage") is not True or len(finishes) != 1 or not isinstance(finishes[0], dict):
        issues.append("incomplete_bep")
    # A terminal marker alone does not close the announced event graph.
    # Canonical JSON keys are kept only in memory, never exposed in the report.
    observed, announced = set(), set()
    for event in events:
        event_id = event.get("id")
        if isinstance(event_id, dict):
            observed.add(json.dumps(event_id, sort_keys=True, separators=(",", ":")))
        children = event.get("children", [])
        if not isinstance(children, list):
            issues.append("invalid_event_children")
            continue
        for child in children:
            if not isinstance(child, dict):
                issues.append("invalid_event_children")
                continue
            announced.add(json.dumps(child, sort_keys=True, separators=(",", ":")))
    if announced - observed:
        issues.append("missing_announced_events")
    identities = set()
    remaining_bytes, remaining_methods = MAX_TOTAL_LOG_BYTES, MAX_METHODS
    for event in events:
        event_id = event.get("id", {})
        has_attempt_id = isinstance(event_id, dict) and "testResult" in event_id
        if "testResult" not in event:
            if has_attempt_id:
                # Includes aborted attempts. Their absence is neither success
                # nor failure, and must prevent a complete capture.
                issues.append("unavailable_test_result")
            continue
        if len(report["attempts"]) >= MAX_ATTEMPTS:
            issues.append("attempt_limit")
            break
        try:
            event_id = event.get("id", {})
            item = attempt_identity(event_id.get("testResult") if isinstance(event_id, dict) else None)
            key = tuple(item.values())
            if key in identities:
                issues.append("duplicate_attempt_identity")
            identities.add(key)
            result = event["testResult"]
            if not isinstance(result, dict):
                raise EvidenceError("invalid_test_result")
            status = result.get("status")
            item["targetStatus"] = status if isinstance(status, str) and status in STATUSES else "UNKNOWN"
            if item["targetStatus"] == "UNKNOWN":
                issues.append("invalid_target_status")
            execution = result.get("executionInfo")
            cached = (result.get("cachedLocally", False),
                      execution.get("cachedRemotely", False) if isinstance(execution, dict) else None)
            item["cache"] = dict(zip(("local", "remote"), (v if type(v) is bool else None for v in cached)))
            item["provenance"] = "cached" if True in item["cache"].values() else "unknown"
            item["executionEvidence"] = execution_evidence(result)
            item.update(logStatus="missing_log_reference", methods=[], issues=[])
            outputs = result.get("testActionOutput", [])
            if not isinstance(outputs, list):
                raise EvidenceError("invalid_log_reference")
            logs = [value for value in outputs if isinstance(value, dict) and value.get("name") == "test.log"]
            if len(logs) != 1:
                item["logStatus"] = "ambiguous_log_reference" if logs else "missing_log_reference"
            else:
                try:
                    if testlogs_root is None:
                        raise EvidenceError("unavailable_local_root")
                    log = local_log(logs[0].get("uri"), testlogs_root, remaining_bytes)
                    remaining_bytes -= len(log)
                    item["logSha256"] = hashlib.sha256(log).hexdigest()
                    item["methods"], item["issues"] = libtest_rows(log)
                    if len(item["methods"]) > remaining_methods:
                        item["methods"] = item["methods"][:remaining_methods]
                        item["issues"].append("method_limit")
                    remaining_methods -= len(item["methods"])
                    item["logStatus"] = "available"
                except EvidenceError as error:
                    item["logStatus"] = str(error)
            if item["logStatus"] != "available" or item["issues"]:
                issues.append("incomplete_named_evidence")
            report["attempts"].append(item)
        except EvidenceError as error:
            issues.append(str(error))
    if not report["attempts"]:
        issues.append("no_test_results")
    report["issues"] = sorted(set(issues))
    if not report["issues"]:
        report["captureStatus"] = "complete"
    return report
