#!/usr/bin/env python3
"""Parse bounded libtest method evidence, without collecting files or running tests.

Stage 1 of #1394. Only canonical names/statuses and fixed issue categories are
returned; raw output and ignored-test reasons are excluded. Complete framing is
not proof that a test executed freshly. BEP binding and workflow activation are
separate follow-up stages.
"""

import re


MAX_METHODS = 100_000
METHOD = re.compile(
    r"test ([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)(?: - should panic)? \.\.\. (ok|FAILED|ignored)(?:, .*)?\Z"
)
RUNNING = re.compile(r"running ([0-9]{1,9}) tests?\Z")
SUMMARY = re.compile(
    r"test result: (?:ok|FAILED)\. ([0-9]{1,9}) passed; ([0-9]{1,9}) failed; ([0-9]{1,9}) ignored;.*\Z"
)


class EvidenceError(Exception):
    """Contains only a fixed public-safe category, never an input path/text."""


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
