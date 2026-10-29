#!/usr/bin/env python3
"""Retain allowlisted libtest rows referenced by one Bazel invocation.

This is evidence collection, not a test runner. A named row is distinct from a
target result and from fresh execution: cache metadata and missing evidence are
retained, and no remote output is fetched. Raw BEP/log bytes are never emitted.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
from urllib.parse import unquote, urlsplit
from urllib.request import url2pathname


MAX_BEP_BYTES = 32 * 1024 * 1024
MAX_LOG_BYTES = 16 * 1024 * 1024
MAX_TOTAL_LOG_BYTES = 64 * 1024 * 1024
MAX_EVENTS = 100_000
MAX_ATTEMPTS = 10_000
MAX_METHODS = 100_000
TARGET = re.compile(
    r"//codex-rs/(?:tui|core|state|login)(?:/[A-Za-z0-9_.+-]+)*:[A-Za-z0-9_.+-]+\Z"
)
CONFIG = re.compile(r"[A-Za-z0-9_-]{1,128}\Z")
METHOD = re.compile(
    r"test ([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)(?: - should panic)? \.\.\. (ok|FAILED|ignored)(?:, .*)?\Z"
)
RUNNING = re.compile(r"running ([0-9]{1,9}) tests?\Z")
SUMMARY = re.compile(
    r"test result: (?:ok|FAILED)\. ([0-9]{1,9}) passed; ([0-9]{1,9}) failed; ([0-9]{1,9}) ignored;.*\Z"
)
STATUSES = {
    "PASSED",
    "FLAKY",
    "TIMEOUT",
    "FAILED",
    "INCOMPLETE",
    "REMOTE_FAILURE",
    "FAILED_TO_BUILD",
    "TOOL_HALTED_BEFORE_TESTING",
    "NO_STATUS",
}


class EvidenceError(Exception):
    """Contains only a fixed public-safe category, never an input path/text."""


def bounded_read(path, limit):
    try:
        if path.is_symlink():
            raise EvidenceError("symlink_rejected")
        if not stat.S_ISREG(path.stat().st_mode):
            raise EvidenceError("nonregular_file")
        flags = (
            os.O_RDONLY
            | getattr(os, "O_NOFOLLOW", 0)
            | getattr(os, "O_NONBLOCK", 0)
            | getattr(os, "O_BINARY", 0)
        )
        with os.fdopen(os.open(path, flags), "rb") as stream:
            metadata = os.fstat(stream.fileno())
            if not stat.S_ISREG(metadata.st_mode):
                raise EvidenceError("nonregular_file")
            if metadata.st_size > limit:
                raise EvidenceError("size_limit")
            data = stream.read(limit + 1)
    except OSError:
        raise EvidenceError("missing_local_file") from None
    if len(data) > limit:
        raise EvidenceError("size_limit")
    return data


def local_log(uri, root, remaining_bytes):
    if not isinstance(uri, str) or len(uri) > 8192:
        raise EvidenceError("invalid_reference")
    try:
        parsed = urlsplit(uri)
    except ValueError:
        raise EvidenceError("invalid_reference") from None
    if parsed.scheme != "file" or parsed.netloc not in ("", "localhost"):
        raise EvidenceError("unavailable_nonlocal_output")
    if parsed.query or parsed.fragment:
        raise EvidenceError("invalid_reference")
    decoded = unquote(parsed.path)
    if "\x00" in decoded or ".." in decoded.replace("\\", "/").split("/"):
        raise EvidenceError("path_escape")
    path = Path(url2pathname(parsed.path))
    if not path.is_absolute():
        raise EvidenceError("invalid_reference")
    # Bazel's workspace testlogs link may itself be a symlink; it is the
    # explicitly supplied boundary. No link beneath that boundary is followed.
    root = root.resolve(strict=True)
    try:
        relative = path.relative_to(root)
    except ValueError:
        raise EvidenceError("path_escape") from None
    # The BEP output name is test.log; retry URIs may point at an archived
    # attempt filename instead. Its explicit reference, not its basename,
    # determines selection.
    if not relative.parts:
        raise EvidenceError("invalid_reference")
    current = root
    for part in relative.parts:
        current = current / part
        if current.is_symlink() or getattr(current, "is_junction", lambda: False)():
            raise EvidenceError("symlink_rejected")
    try:
        path.resolve(strict=True).relative_to(root)
    except ValueError:
        raise EvidenceError("path_escape") from None
    return bounded_read(path, min(MAX_LOG_BYTES, remaining_bytes))


def libtest_rows(data):
    try:
        text = data.decode("utf-8")
    except UnicodeError:
        raise EvidenceError("invalid_log_encoding") from None
    rows, issues, seen = [], [], set()
    expected, batch = None, []
    completed_batches = 0
    for line in text.splitlines():
        running = RUNNING.fullmatch(line)
        row = METHOD.fullmatch(line) if len(line) <= 2048 else None
        summary = SUMMARY.fullmatch(line)
        if running:
            if expected is not None:
                issues.append("incomplete_libtest_batch")
            expected, batch = int(running[1]), []
        elif row:
            if len(rows) >= MAX_METHODS:
                issues.append("method_limit")
                break
            name, status = row.groups()
            if len(name) > 1024:
                issues.append("unrecognized_named_row")
                continue
            if expected is None:
                issues.append("unframed_named_row")
            if name in seen:
                issues.append("duplicate_named_row")
            seen.add(name)
            result = {"name": name, "status": status}
            rows.append(result)
            batch.append(result)
        elif summary:
            counts = [
                sum(row["status"] == status for row in batch)
                for status in ("ok", "FAILED", "ignored")
            ]
            if (
                expected is None
                or len(batch) != expected
                or counts != [int(value) for value in summary.groups()]
            ):
                issues.append("incomplete_libtest_batch")
            completed_batches += 1
            expected, batch = None, []
        elif line.startswith("test ") and " ... " in line:
            issues.append("unrecognized_named_row")
    if expected is not None or completed_batches == 0:
        issues.append("incomplete_libtest_batch")
    return rows, sorted(set(issues))


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
    if (
        not isinstance(target, str)
        or not TARGET.fullmatch(target)
        or "/../" in target
        or not isinstance(config, str)
        or not CONFIG.fullmatch(config)
    ):
        raise EvidenceError("invalid_attempt_identity")
    result = {"target": target, "configuration": config}
    for field in ("run", "shard", "attempt"):
        number = value.get(field, 0)  # TestResultId protobuf int32 defaults.
        if type(number) is not int or not 0 <= number < 2**31:
            raise EvidenceError("invalid_attempt_identity")
        result[field] = number
    return result


def optional_bool(value):
    return value if type(value) is bool else None


def execution_evidence(result, execution):
    # These are protocol observations, not proof of current/fresh execution.
    evidence = {"strategy": "unknown"}
    if (
        isinstance(execution, dict)
        and isinstance(execution.get("strategy"), str)
        and execution["strategy"]
        in {"local", "remote", "sandboxed", "worker", "windows-sandbox"}
    ):
        evidence["strategy"] = execution["strategy"]
    for field in ("testAttemptStartMillisEpoch", "testAttemptDurationMillis"):
        value = result.get(field)
        if isinstance(value, str) and re.fullmatch(r"[0-9]{1,18}", value):
            evidence[field] = value
        elif type(value) is int and 0 <= value < 10**18:
            evidence[field] = str(value)
    # Recent BEP versions use protobuf Timestamp/Duration JSON strings.
    for field, pattern in (
        ("testAttemptStart", r"[0-9TZ:.-]{1,40}"),
        ("testAttemptDuration", r"[0-9.]{1,30}s"),
    ):
        value = result.get(field)
        if isinstance(value, str) and re.fullmatch(pattern, value):
            evidence[field] = value
    return evidence


def collect(
    bep_path, testlogs_root, invocation_id, checkout_sha, workflow_outcome="unknown"
):
    report = {
        "schemaVersion": 1,
        "invocationId": invocation_id
        if re.fullmatch(r"[0-9a-fA-F-]{36}", invocation_id)
        else None,
        "checkoutSha": checkout_sha
        if re.fullmatch(r"[0-9a-f]{40}", checkout_sha)
        else None,
        "workflowOutcome": workflow_outcome
        if workflow_outcome in {"success", "failure", "cancelled", "skipped"}
        else "unknown",
        "captureStatus": "incomplete",
        "issues": [],
        "attempts": [],
        "limitations": [
            "Named rows do not establish fresh execution or execution of every conditional test body.",
            "Absent names are not passes; filtered tests are not inferred from missing rows.",
        ],
    }
    issues = report["issues"]
    if report["invocationId"] is None or report["checkoutSha"] is None:
        issues.append("invalid_invocation_binding")
        return report
    try:
        data = bounded_read(Path(bep_path), MAX_BEP_BYTES)
        report["bepSha256"] = hashlib.sha256(data).hexdigest()
        lines = data.splitlines()
        if len(lines) > MAX_EVENTS:
            raise EvidenceError("event_limit")
        events = []
        for line in lines:
            event = json.loads(line, object_pairs_hook=unique_object)
            if not isinstance(event, dict):
                raise EvidenceError("invalid_event")
            events.append(event)
    except (EvidenceError, ValueError, UnicodeError, RecursionError) as error:
        issues.append(
            str(error) if isinstance(error, EvidenceError) else "malformed_bep"
        )
        return report
    starts = [event["started"] for event in events if "started" in event]
    if (
        len(starts) != 1
        or not isinstance(starts[0], dict)
        or starts[0].get("uuid") != invocation_id
    ):
        issues.append("invocation_mismatch")
        return report
    report["observedInvocationId"] = starts[0]["uuid"]
    if (
        not events
        or events[-1].get("lastMessage") is not True
        or not any(isinstance(event.get("finished"), dict) for event in events)
    ):
        issues.append("incomplete_bep")
    identities = set()
    remaining_bytes, remaining_methods = MAX_TOTAL_LOG_BYTES, MAX_METHODS
    for event in events:
        if "testResult" not in event:
            continue
        if len(report["attempts"]) >= MAX_ATTEMPTS:
            issues.append("attempt_limit")
            break
        try:
            event_id = event.get("id", {})
            item = attempt_identity(
                event_id.get("testResult") if isinstance(event_id, dict) else None
            )
            key = tuple(item.values())
            if key in identities:
                issues.append("duplicate_attempt_identity")
            identities.add(key)
            result = event["testResult"]
            if not isinstance(result, dict):
                raise EvidenceError("invalid_test_result")
            status = result.get("status")
            item["targetStatus"] = (
                status if isinstance(status, str) and status in STATUSES else "UNKNOWN"
            )
            execution = result.get("executionInfo")
            item["cache"] = {
                "local": optional_bool(result.get("cachedLocally", False)),
                "remote": optional_bool(execution.get("cachedRemotely", False))
                if isinstance(execution, dict)
                else None,
            }
            item["provenance"] = (
                "cached" if True in item["cache"].values() else "unknown"
            )
            item["executionEvidence"] = execution_evidence(result, execution)
            item["logStatus"], item["methods"] = "missing_log_reference", []
            item["issues"] = []
            outputs = result.get("testActionOutput", [])
            if not isinstance(outputs, list):
                raise EvidenceError("invalid_log_reference")
            logs = [
                output
                for output in outputs
                if isinstance(output, dict) and output.get("name") == "test.log"
            ]
            if len(logs) != 1:
                item["logStatus"] = (
                    "ambiguous_log_reference" if logs else "missing_log_reference"
                )
            else:
                try:
                    if testlogs_root is None:
                        raise EvidenceError("unavailable_local_root")
                    log = local_log(
                        logs[0].get("uri"), Path(testlogs_root), remaining_bytes
                    )
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
                except (OSError, ValueError):
                    item["logStatus"] = "unavailable_local_root"
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("bep", "invocation-id", "checkout-sha", "output"):
        parser.add_argument("--" + name, required=True)
    root = parser.add_mutually_exclusive_group(required=True)
    root.add_argument("--testlogs-root")
    root.add_argument("--testlogs-root-file")
    parser.add_argument("--workflow-outcome", default="unknown")
    args = parser.parse_args()
    testlogs_root = args.testlogs_root
    if args.testlogs_root_file:
        try:
            value = (
                bounded_read(Path(args.testlogs_root_file), 8192)
                .decode("utf-8")
                .strip()
            )
            testlogs_root = (
                value
                if value
                and "\n" not in value
                and "\r" not in value
                and Path(value).is_absolute()
                else None
            )
        except (EvidenceError, UnicodeError, ValueError):
            testlogs_root = None
    report = collect(
        args.bep,
        testlogs_root,
        args.invocation_id,
        args.checkout_sha,
        args.workflow_outcome,
    )
    output = Path(args.output)
    try:
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("x", encoding="utf-8") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    except OSError:
        print("Named result capture: output unavailable (no raw diagnostic emitted)")
        return 1
    print("Named result capture: " + report["captureStatus"])
    return 0 if report["captureStatus"] == "complete" else 1


if __name__ == "__main__":
    raise SystemExit(main())
