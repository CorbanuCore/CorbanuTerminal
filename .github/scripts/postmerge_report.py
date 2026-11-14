#!/usr/bin/env python3

"""Keep one labelled issue open while postmerge-ci is red on main (#1697).

postmerge-ci runs overlap on main (each gets its own v8-canary group, #1696),
so a run's report can arrive after a newer commit's. Each red report records
its run and failed jobs in a hidden marker on the issue. A failure counts as
fixed once a later main run (by creation time, which follows push order) ran
that same job and it passed, or once a re-run of the failed run itself passed
it (a re-run keeps the run id; the jobs listing shows the latest attempt). A
job that a later run skipped, such as the V8 matrix on a commit that doesn't
touch V8, proves nothing. So:

- a red run whose failed jobs all passed in a later run is stale: no comment,
  no new issue;
- a green run closes the issue only when every recorded failure has been
  fixed that way and no newer main run is still in progress (that run's
  report decides instead).

Only markers posted by github-actions[bot] count, and malformed ones are
skipped: the issue is public. The report job has no concurrency group (a
single-slot group cancelled pending reports), so reports race:

- if two red reports open an issue at once, the newer duplicate is closed and
  its markers are copied to the kept issue first;
- a red report that comments on an issue a green report has just closed
  reopens it, and a green report that finds a new marker after closing the
  issue reopens it.
"""

import json
import os
import re
import subprocess
from dataclasses import dataclass
from typing import Callable

LABEL = "postmerge-red"
TITLE = "postmerge-ci is failing on main"
WORKFLOW = "postmerge-ci.yml"
REPORT_JOB = "Report main status"
RESULTS_JOB = "Postmerge CI results"
FAILED = ("failure", "timed_out")
BOT = "github-actions[bot]"
# Newer runs checked per recorded job (newest first, stopping at the first
# pass), so a long red stretch or a renamed job costs a bounded number of
# API calls against GITHUB_TOKEN's hourly limit.
RUNS_CHECKED = 50
NOTE = (
    "PR checks do not run postmerge-ci (full Linux tests, musl and Windows "
    "clippy). This issue closes itself once every failed job has passed again "
    f"in a later main run (one of the newest {RUNS_CHECKED}) or in a re-run. A "
    "maintainer may also close it by hand, for example after a failed job was "
    "renamed or removed, since such a failure can never pass again."
)
MARKER = re.compile(r"<!-- postmerge-report (\{[^\n]*\}) -->")
TIMESTAMP = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")

Gh = Callable[[list[str]], str]


def run_gh(args: list[str]) -> str:
    return subprocess.run(
        ["gh", *args], check=True, capture_output=True, text=True
    ).stdout


@dataclass(frozen=True)
class Run:
    id: int
    created_at: str
    status: str

    def key(self) -> tuple[str, int]:
        return (self.created_at, self.id)


@dataclass(frozen=True)
class Record:
    run_id: int
    created_at: str
    failed_jobs: tuple[str, ...]

    def key(self) -> tuple[str, int]:
        return (self.created_at, self.run_id)

    def marker(self) -> str:
        data = {
            "run_id": self.run_id,
            "created_at": self.created_at,
            "failed_jobs": list(self.failed_jobs),
        }
        return f"<!-- postmerge-report {json.dumps(data, sort_keys=True)} -->"


def parse_marker(text: str) -> Record | None:
    """The marker's record, or None when it is not the shape a report writes."""
    try:
        raw = json.loads(text)
    except json.JSONDecodeError:
        return None
    if not isinstance(raw, dict):
        return None
    run_id, created_at, failed = (
        raw.get("run_id"),
        raw.get("created_at"),
        raw.get("failed_jobs"),
    )
    if type(run_id) is not int or run_id <= 0:
        return None
    # created_at goes into the runs query string.
    if not isinstance(created_at, str) or not TIMESTAMP.fullmatch(created_at):
        return None
    if not isinstance(failed, list) or not failed:
        return None
    if not all(isinstance(job, str) and job for job in failed):
        return None
    return Record(run_id, created_at, tuple(failed))


def json_lines(text: str) -> list[dict]:
    return [json.loads(line) for line in text.splitlines() if line.strip()]


class Reporter:
    def __init__(self, gh: Gh, repo: str) -> None:
        self.gh = gh
        self.repo = repo
        self.jobs_cache: dict[int, dict[str, str]] = {}

    def run(self, run_id: int) -> Run:
        data = json.loads(self.gh(["api", f"repos/{self.repo}/actions/runs/{run_id}"]))
        return Run(data["id"], data["created_at"], data["status"])

    def runs_since(self, created_at: str) -> list[Run]:
        """Main runs created at or after created_at, newest first.

        One page of the newest 100 is enough: fixed() looks at no more than
        RUNS_CHECKED of them. (The runs API also stops at 1,000 results for a
        query filtered by branch, event or created, so paginating further
        would not reach the oldest runs of a very long red stretch anyway.)
        """
        query = f"branch=main&event=push&per_page=100&created=%3E%3D{created_at}"
        out = self.gh(
            [
                "api",
                f"repos/{self.repo}/actions/workflows/{WORKFLOW}/runs?{query}",
                "--jq",
                ".workflow_runs[] | {id, created_at, status}",
            ]
        )
        runs = [Run(r["id"], r["created_at"], r["status"]) for r in json_lines(out)]
        return sorted(runs, key=Run.key, reverse=True)

    def jobs(self, run_id: int) -> dict[str, str]:
        """Conclusion by job display name, for the run's latest attempt.

        Keyed by display name, which is what the markers record; two jobs
        with the same name would overwrite each other (postmerge-ci has none).
        """
        if run_id not in self.jobs_cache:
            out = self.gh(
                [
                    "api",
                    "--paginate",
                    f"repos/{self.repo}/actions/runs/{run_id}/jobs?per_page=100",
                    "--jq",
                    ".jobs[] | {name, conclusion}",
                ]
            )
            self.jobs_cache[run_id] = {
                job["name"]: job["conclusion"] or "" for job in json_lines(out)
            }
        return self.jobs_cache[run_id]

    def open_issues(self) -> list[int]:
        out = self.gh(
            [
                "issue",
                "list",
                "--state",
                "open",
                "--label",
                LABEL,
                "--json",
                "number",
                "--jq",
                ".[].number",
            ]
        )
        return sorted(int(line) for line in out.split())

    def is_closed(self, number: int) -> bool:
        path = f"repos/{self.repo}/issues/{number}"
        return self.gh(["api", path, "--jq", ".state"]).strip() == "closed"

    def records(self, number: int) -> list[Record]:
        """Well-formed markers that github-actions[bot] posted on the issue."""
        path = f"repos/{self.repo}/issues/{number}"
        fields = "{body, login: .user.login, type: .user.type}"
        posts = [json.loads(self.gh(["api", path, "--jq", fields]))]
        comments = ["api", "--paginate", f"{path}/comments?per_page=100"]
        posts += json_lines(self.gh([*comments, "--jq", f".[] | {fields}"]))
        records = []
        for post in posts:
            if post.get("login") != BOT or post.get("type") != "Bot":
                continue
            for match in MARKER.finditer(post.get("body") or ""):
                if record := parse_marker(match.group(1)):
                    records.append(record)
        return records

    def fixed(self, record: Record, job: str, runs: list[Run]) -> bool:
        """Whether a run after the record, or a re-run of the record's own run,
        ran the job and it passed. runs is newest first; checks at most
        RUNS_CHECKED newer runs and stops at the first pass."""
        newer = [run.id for run in runs if run.key() > record.key()][:RUNS_CHECKED]
        return any(
            self.jobs(run_id).get(job) == "success"
            for run_id in [*newer, record.run_id]
        )

    def dedupe(self, issues: list[int]) -> int | None:
        if not issues:
            return None
        keep, *duplicates = issues
        for number in duplicates:
            if carried := self.records(number):
                markers = "\n".join(record.marker() for record in carried)
                body = f"Failures recorded on #{number}, closed as a duplicate:\n\n{markers}"
                self.gh(["issue", "comment", str(keep), "--body", body])
            self.gh(
                ["issue", "close", str(number), "--comment", f"Duplicate of #{keep}."]
            )
        return keep

    def report(self, run_id: int, sha: str, result: str, run_url: str) -> str:
        try:
            self.gh(
                ["label", "create", LABEL, "--color", "B60205", "--description", TITLE]
            )
        except subprocess.CalledProcessError:
            pass  # The label already exists.
        me = self.run(run_id)
        issue = self.dedupe(self.open_issues())

        if result == "failure":
            failed = sorted(
                name
                for name, conclusion in self.jobs(run_id).items()
                if conclusion in FAILED and name != REPORT_JOB
            ) or [RESULTS_JOB]
            mine = Record(run_id, me.created_at, tuple(failed))
            runs = self.runs_since(me.created_at)
            if all(self.fixed(mine, job, runs) for job in failed):
                return (
                    f"stale: every failed job of {sha[:10]} passed in a later main run"
                )
            jobs = "\n".join(f"- {job}" for job in failed)
            body = (
                f"postmerge-ci failed on {sha[:10]}: {run_url}\n\nFailed jobs:\n{jobs}\n\n"
                f"{mine.marker()}"
            )
            if issue is not None:
                self.gh(["issue", "comment", str(issue), "--body", body])
                # A green report may have closed it since we listed it.
                if self.is_closed(issue):
                    self.gh(["issue", "reopen", str(issue)])
                    return f"commented on #{issue} and reopened it"
                return f"commented on #{issue}"
            create = ["issue", "create", "--title", TITLE, "--label", LABEL]
            self.gh([*create, "--body", f"{body}\n\n{NOTE}"])
            self.dedupe(self.open_issues())
            return "opened an issue"

        if issue is None:
            return "green, no open issue"
        records = self.records(issue)
        if not records:
            # An issue opened before these markers existed: keep the old rule.
            head = self.gh(
                ["api", f"repos/{self.repo}/commits/main", "--jq", ".sha"]
            ).strip()
            if head != sha:
                return "green on an older commit; left open"
        else:
            # Only the newest record of each job matters: a run that fixes it
            # is also later than the older records.
            newest: dict[str, Record] = {}
            for record in records:
                for job in record.failed_jobs:
                    if job not in newest or record.key() > newest[job].key():
                        newest[job] = record
            runs = self.runs_since(min(record.created_at for record in newest.values()))
            if any(run.key() > me.key() and run.status != "completed" for run in runs):
                return "a newer main run is still in progress; it decides"
            unfixed = sorted(
                job
                for job, record in newest.items()
                if not self.fixed(record, job, runs)
            )
            if unfixed:
                return "left open; not passed again yet: " + ", ".join(unfixed)
        comment = f"postmerge-ci is green again on {sha[:10]}: {run_url}"
        self.gh(["issue", "close", str(issue), "--comment", comment])
        # A red report may have commented after we read the markers.
        if any(record not in records for record in self.records(issue)):
            reopen = (
                "Reopened: a failure was reported while this issue was being closed."
            )
            self.gh(["issue", "reopen", str(issue), "--comment", reopen])
            return f"closed and reopened #{issue}"
        return f"closed #{issue}"


def main() -> None:
    reporter = Reporter(run_gh, os.environ["GH_REPO"])
    print(
        reporter.report(
            int(os.environ["RUN_ID"]),
            os.environ["SHA"],
            os.environ["RESULT"],
            os.environ["RUN_URL"],
        )
    )


if __name__ == "__main__":
    main()
