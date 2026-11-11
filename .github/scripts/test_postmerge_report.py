#!/usr/bin/env python3

import json
import re
import unittest
from pathlib import Path
from typing import Callable

from postmerge_report import LABEL
from postmerge_report import Reporter

REPO = "CorbanuCore/CorbanuTerminal"
REPO_ROOT = Path(__file__).resolve().parents[2]
BOT = "github-actions[bot]"
V8 = "v8-canary / Build release x86_64-unknown-linux-gnu"
V8_SKIPPED = "v8-canary / Build ${{ matrix.variant }} ${{ matrix.target }}"
TESTS = "rust-ci-full / Tests — ubuntu-24.04 - x86_64-unknown-linux-gnu / Tests shard 1/4"
RESULTS = "Postmerge CI results"


def jobs(*, v8: str | None, tests: str | None = "success") -> dict[str, str]:
    """A postmerge run's jobs; v8=None means the V8 matrix was skipped and
    tests=None means the test shard did not run."""
    out = {"v8-canary / metadata": "success"}
    if tests is not None:
        out[TESTS] = tests
    if v8 is None:
        out[V8_SKIPPED] = "skipped"
    else:
        out[V8] = v8
    red = tests not in (None, "success") or v8 not in (None, "success")
    out[RESULTS] = "failure" if red else "success"
    out["Report main status"] = ""
    return out


def marker(run_id: int, created_at: str, failed: list[str]) -> str:
    data = {"run_id": run_id, "created_at": created_at, "failed_jobs": sorted(failed)}
    return f"<!-- postmerge-report {json.dumps(data, sort_keys=True)} -->"


def page(items: list[str], path: str, args: list[str]) -> str:
    """One page of a REST listing unless --paginate is passed, like gh."""
    if "--paginate" not in args and (match := re.search(r"[?&]per_page=(\d+)", path)):
        items = items[: int(match.group(1))]
    return "".join(f"{item}\n" for item in items)


class FakeGitHub:
    """Answers the gh calls the report makes, from an in-memory repository."""

    def __init__(self, head: str) -> None:
        self.head = head
        self.runs: dict[int, dict] = {}
        self.issues: list[dict] = []
        self.calls: list[list[str]] = []
        # Runs once just before the named command ("issue close", ...) is
        # applied, to interleave another report's write.
        self.before: dict[str, Callable[[], None]] = {}

    def add_run(self, run_id: int, sha: str, created_at: str, job_results: dict[str, str],
                status: str = "completed") -> None:
        self.runs[run_id] = {
            "id": run_id, "sha": sha, "created_at": created_at, "status": status,
            "jobs": job_results,
        }

    def add_issue(self, body: str, login: str = BOT) -> int:
        number = len(self.issues) + 1
        self.issues.append(
            {"number": number, "open": True, "body": body, "login": login, "comments": []}
        )
        return number

    def add_comment(self, number: int, body: str, login: str = BOT) -> None:
        self.issue(str(number))["comments"].append({"body": body, "login": login})

    def open_numbers(self) -> list[int]:
        return [issue["number"] for issue in self.issues if issue["open"]]

    def issue(self, number: str) -> dict:
        return self.issues[int(number) - 1]

    def bodies(self, number: int) -> list[str]:
        return [comment["body"] for comment in self.issue(str(number))["comments"]]

    def job_lookups(self) -> int:
        return sum(1 for call in self.calls if any("/jobs?" in arg for arg in call))

    def __call__(self, args: list[str]) -> str:
        self.calls.append(args)
        flags = {args[i]: args[i + 1] for i in range(len(args) - 1) if args[i].startswith("--")}
        if hook := self.before.pop(" ".join(args[:2]), None):
            hook()
        if args[:2] == ["label", "create"]:
            return ""
        if args[:2] == ["issue", "list"]:
            numbers = self.open_numbers()
            if flags.get("--jq") == ".[].number":
                return "".join(f"{n}\n" for n in numbers)
            return f"{numbers[0]}\n" if numbers else ""
        if args[:2] == ["issue", "view"]:
            issue = self.issue(args[2])
            comments = [
                {"body": c["body"], "author": {"login": c["login"]}} for c in issue["comments"]
            ]
            return json.dumps({"body": issue["body"], "comments": comments})
        if args[:2] == ["issue", "comment"]:
            self.add_comment(int(args[2]), flags["--body"])
            return ""
        if args[:2] == ["issue", "create"]:
            self.add_issue(flags["--body"])
            return ""
        if args[:2] in (["issue", "close"], ["issue", "reopen"]):
            issue = self.issue(args[2])
            issue["open"] = args[1] == "reopen"
            if "--comment" in flags:
                self.add_comment(issue["number"], flags["--comment"])
            return ""
        path = next(arg for arg in args[1:] if arg.startswith("repos/"))
        if path == f"repos/{REPO}/commits/main":
            return self.head + "\n"
        if match := re.fullmatch(rf"repos/{REPO}/issues/(\d+)", path):
            issue = self.issue(match.group(1))
            state = "open" if issue["open"] else "closed"
            if flags.get("--jq") == ".state":
                return state + "\n"
            return json.dumps(
                {"state": state, "body": issue["body"], "login": issue["login"], "type": "Bot"
                 if issue["login"] == BOT else "User"}
            )
        if match := re.fullmatch(rf"repos/{REPO}/issues/(\d+)/comments\?per_page=100", path):
            return page(
                [
                    json.dumps({"body": c["body"], "login": c["login"],
                                "type": "Bot" if c["login"] == BOT else "User"})
                    for c in self.issue(match.group(1))["comments"]
                ],
                path,
                args,
            )
        if match := re.fullmatch(rf"repos/{REPO}/actions/runs/(\d+)", path):
            run = self.runs[int(match.group(1))]
            return json.dumps({k: run[k] for k in ("id", "created_at", "status")})
        if match := re.fullmatch(rf"repos/{REPO}/actions/runs/(\d+)/jobs\?per_page=100", path):
            run = self.runs[int(match.group(1))]
            return page(
                [
                    json.dumps({"name": name, "conclusion": conclusion or None})
                    for name, conclusion in run["jobs"].items()
                ],
                path,
                args,
            )
        if match := re.fullmatch(
            rf"repos/{REPO}/actions/workflows/postmerge-ci\.yml/runs\?branch=main&event=push"
            r"&per_page=100&created=%3E%3D(.+)",
            path,
        ):
            # The API lists newest first.
            runs = sorted(
                self.runs.values(), key=lambda run: (run["created_at"], run["id"]), reverse=True
            )
            return page(
                [
                    json.dumps({k: run[k] for k in ("id", "created_at", "status")})
                    for run in runs
                    if run["created_at"] >= match.group(1)
                ],
                path,
                args,
            )
        raise AssertionError(f"unexpected gh call: {args}")


A, B, C, Z = "a" * 40, "b" * 40, "c" * 40, "9" * 40
T0, T1, T2, T3 = (f"2026-11-08T1{n}:00:00Z" for n in range(4))


class Scenario:
    """Main was red at commit Z (run 10, V8 failed); issue #1 records it."""

    def __init__(self, head: str = B) -> None:
        self.gh = FakeGitHub(head)
        self.gh.add_run(10, Z, T0, jobs(v8="failure"))
        self.gh.add_issue(
            f"postmerge-ci failed on {Z[:10]}\n\n" + marker(10, T0, [RESULTS, V8])
        )

    def report(self, run_id: int, result: str) -> str:
        sha = self.gh.runs[run_id]["sha"]
        return Reporter(self.gh, REPO).report(run_id, sha, result, f"run-{run_id}")


class PostmergeReportTest(unittest.TestCase):
    def test_red_older_commit_finishing_after_a_green_head_opens_nothing(self) -> None:
        # #1697 item 1: commit A's multi-hour V8 run is red, but newer head B
        # ran V8 green and reported first.
        s = Scenario(head=B)
        s.gh.add_run(11, A, T1, jobs(v8="failure"), status="in_progress")
        s.gh.add_run(12, B, T2, jobs(v8="success"))
        self.assertEqual(s.report(12, "success"), "closed #1")
        s.gh.runs[11]["status"] = "completed"
        self.assertIn("stale", s.report(11, "failure"))
        self.assertEqual(s.gh.open_numbers(), [])
        self.assertEqual(len(s.gh.issues), 1)

    def test_green_head_that_skipped_v8_leaves_a_v8_failure_open(self) -> None:
        # #1697 item 1: a green commit that skipped the V8 matrix says nothing
        # about V8 at head.
        s = Scenario(head=B)
        s.gh.add_run(12, B, T2, jobs(v8=None))
        self.assertIn(V8, s.report(12, "success"))
        self.assertEqual(s.gh.open_numbers(), [1])

    def test_green_run_that_reran_the_failed_jobs_closes_the_issue(self) -> None:
        s = Scenario(head=B)
        s.gh.add_run(12, B, T2, jobs(v8="success"))
        self.assertEqual(s.report(12, "success"), "closed #1")
        self.assertEqual(s.gh.open_numbers(), [])

    def test_rerun_of_the_red_run_that_goes_green_closes_the_issue(self) -> None:
        # Review item 1: "Re-run failed jobs" keeps the run id and created_at;
        # the jobs listing then shows the latest attempt.
        s = Scenario(head=Z)
        s.gh.runs[10]["jobs"] = jobs(v8="success")
        self.assertEqual(s.report(10, "success"), "closed #1")
        self.assertEqual(s.gh.open_numbers(), [])

    def test_rerun_of_the_red_run_that_stays_red_keeps_the_issue_open(self) -> None:
        s = Scenario(head=Z)
        self.assertEqual(s.report(10, "failure"), "commented on #1")
        self.assertEqual(s.gh.open_numbers(), [1])

    def test_v8_fixed_on_an_older_green_closes_once_the_head_is_done(self) -> None:
        # A fixes V8 (slow run); B, the head, skips V8 and finishes first.
        s = Scenario(head=B)
        s.gh.add_run(11, A, T1, jobs(v8=""), status="in_progress")
        s.gh.add_run(12, B, T2, jobs(v8=None))
        self.assertIn("not passed again yet", s.report(12, "success"))
        s.gh.runs[11].update(status="completed", jobs=jobs(v8="success"))
        self.assertEqual(s.report(11, "success"), "closed #1")

    def test_green_run_waits_for_a_newer_run_still_in_progress(self) -> None:
        s = Scenario(head=C)
        s.gh.add_run(12, B, T2, jobs(v8="success"))
        s.gh.add_run(13, C, T3, jobs(v8=None, tests="failure"), status="in_progress")
        self.assertIn("still in progress", s.report(12, "success"))
        self.assertEqual(s.gh.open_numbers(), [1])
        s.gh.runs[13]["status"] = "completed"
        self.assertEqual(s.report(13, "failure"), "commented on #1")
        self.assertIn(TESTS, s.gh.bodies(1)[-1])

    def test_red_run_records_its_failed_jobs(self) -> None:
        s = Scenario(head=A)
        s.gh.add_run(11, A, T1, jobs(v8=None, tests="failure"))
        self.assertEqual(s.report(11, "failure"), "commented on #1")
        comment = s.gh.bodies(1)[-1]
        self.assertIn(f"postmerge-ci failed on {A[:10]}: run-11", comment)
        self.assertIn(marker(11, T1, [RESULTS, TESTS]), comment)

    def test_red_run_with_some_jobs_fixed_later_still_reports(self) -> None:
        s = Scenario(head=B)
        s.gh.add_run(11, A, T1, jobs(v8="failure", tests="failure"))
        s.gh.add_run(12, B, T2, jobs(v8=None))
        self.assertEqual(s.report(11, "failure"), "commented on #1")

    def test_red_with_no_issue_opens_one_and_green_head_closes_it(self) -> None:
        gh = FakeGitHub(head=B)
        gh.add_run(11, A, T1, jobs(v8=None, tests="failure"))
        reporter = Reporter(gh, REPO)
        self.assertEqual(reporter.report(11, A, "failure", "run-11"), "opened an issue")
        self.assertEqual(gh.open_numbers(), [1])
        self.assertIn(marker(11, T1, [RESULTS, TESTS]), gh.issues[0]["body"])
        self.assertIn("close it by hand", gh.issues[0]["body"])
        gh.add_run(12, B, T2, jobs(v8=None))
        self.assertEqual(Reporter(gh, REPO).report(12, B, "success", "run-12"), "closed #1")

    def test_duplicate_issues_from_concurrent_reports_are_merged(self) -> None:
        # #1697 item 2: without the single-slot group two red reports can both
        # open an issue; the next report keeps the oldest.
        s = Scenario(head=A)
        s.gh.add_issue("postmerge-ci failed on " + marker(10, T0, [RESULTS, V8]))
        s.gh.add_run(11, A, T1, jobs(v8="failure"))
        self.assertEqual(s.report(11, "failure"), "commented on #1")
        self.assertEqual(s.gh.open_numbers(), [1])
        self.assertEqual(s.gh.bodies(2), ["Duplicate of #1."])

    def test_dedupe_carries_the_duplicates_failures_to_the_kept_issue(self) -> None:
        # Review item 3: two red runs with different failed jobs open #1 and
        # #2 at once; closing #2 must not lose run 11's test failure.
        s = Scenario(head=B)
        s.gh.add_run(11, A, T1, jobs(v8=None, tests="failure"))
        s.gh.add_issue("postmerge-ci failed on " + marker(11, T1, [RESULTS, TESTS]))
        s.gh.add_run(12, B, T2, jobs(v8="success", tests=None))
        self.assertIn("not passed again yet: " + TESTS, s.report(12, "success"))
        self.assertEqual(s.gh.open_numbers(), [1])
        self.assertIn(marker(11, T1, [RESULTS, TESTS]), s.gh.bodies(1)[0])
        self.assertEqual(s.gh.bodies(2), ["Duplicate of #1."])

    def test_red_comment_landing_while_a_green_report_closes_reopens(self) -> None:
        # Review item 3: green run 12 has read the markers when older red run
        # 11 comments, then 12 closes the issue.
        s = Scenario(head=B)
        s.gh.add_run(11, A, T1, jobs(v8=None, tests="failure"))
        s.gh.add_run(12, B, T2, jobs(v8="success", tests=None))
        s.gh.before["issue close"] = lambda: s.gh.add_comment(
            1, "postmerge-ci failed on a: run-11\n\n" + marker(11, T1, [RESULTS, TESTS])
        )
        self.assertIn("reopened #1", s.report(12, "success"))
        self.assertEqual(s.gh.open_numbers(), [1])

    def test_red_comment_on_an_issue_closed_meanwhile_reopens_it(self) -> None:
        # Review item 3: the issue was open when red run 11 listed it, and a
        # green report closed it before 11's comment landed.
        s = Scenario(head=A)
        s.gh.add_run(11, A, T1, jobs(v8=None, tests="failure"))

        def close() -> None:
            s.gh.issue("1")["open"] = False

        s.gh.before["issue comment"] = close
        self.assertIn("reopened", s.report(11, "failure"))
        self.assertEqual(s.gh.open_numbers(), [1])
        self.assertIn(marker(11, T1, [RESULTS, TESTS]), s.gh.bodies(1)[-1])

    def test_markers_from_other_authors_are_ignored(self) -> None:
        # Review item 2: the issue is public; a made-up job name would keep it
        # open forever.
        s = Scenario(head=B)
        s.gh.add_comment(1, marker(99, T0, ["made-up job"]), login="someone")
        s.gh.add_run(12, B, T2, jobs(v8="success"))
        self.assertEqual(s.report(12, "success"), "closed #1")

    def test_malformed_markers_are_skipped(self) -> None:
        # Review item 2: a malformed marker crashed every later report, and
        # created_at went straight into the runs query.
        s = Scenario(head=B)
        for bad in (
            "{not json}",
            '{"run_id": "10", "created_at": "%s", "failed_jobs": ["x"]}' % T0,
            '{"run_id": 10, "created_at": "2026&per_page=1", "failed_jobs": ["x"]}',
            '{"run_id": 10, "created_at": "%s", "failed_jobs": "x"}' % T0,
            '{"run_id": 10, "created_at": "%s"}' % T0,
            "[1, 2]",
        ):
            s.gh.add_comment(1, f"<!-- postmerge-report {bad} -->")
        s.gh.add_run(12, B, T2, jobs(v8="success"))
        self.assertEqual(s.report(12, "success"), "closed #1")

    def test_green_path_checks_a_bounded_number_of_runs(self) -> None:
        # Review item 4: a long stretch of green runs that all skip V8 must
        # not cost one jobs call per run.
        s = Scenario(head=B)
        for i in range(80):
            s.gh.add_run(100 + i, B, f"2026-11-09T{i // 60:02}:{i % 60:02}:00Z", jobs(v8=None))
        self.assertIn("not passed again", s.report(179, "success"))
        self.assertEqual(s.gh.open_numbers(), [1])
        # At most RUNS_CHECKED (50) newer runs plus the recorded run itself (a re-run).
        self.assertLessEqual(s.gh.job_lookups(), 51)

    def test_marker_on_a_later_comment_page_is_read(self) -> None:
        # The fake returns one page unless --paginate is passed, like gh.
        s = Scenario(head=B)
        for i in range(120):
            s.gh.add_comment(1, f"chatter {i}", login="someone")
        s.gh.add_run(11, A, T1, jobs(v8=None, tests="failure"))
        s.gh.add_comment(1, marker(11, T1, [RESULTS, TESTS]))
        s.gh.add_run(12, B, T2, jobs(v8="success", tests=None))
        self.assertIn(TESTS, s.report(12, "success"))

    def test_issue_without_markers_keeps_the_head_rule(self) -> None:
        gh = FakeGitHub(head=B)
        gh.add_issue("postmerge-ci failed on 9999999999: run-0")
        gh.add_run(11, A, T1, jobs(v8=None))
        gh.add_run(12, B, T2, jobs(v8=None))
        self.assertIn("older commit", Reporter(gh, REPO).report(11, A, "success", "run-11"))
        self.assertEqual(Reporter(gh, REPO).report(12, B, "success", "run-12"), "closed #1")


class PostmergeWorkflowTest(unittest.TestCase):
    def report_job(self) -> str:
        workflow = (REPO_ROOT / ".github" / "workflows" / "postmerge-ci.yml").read_text()
        match = re.search(r"(?ms)^  report:\n(.*?)(?=^  \S|\Z)", workflow)
        assert match, "postmerge-ci.yml has no report job"
        return match.group(1)

    def test_report_job_has_no_single_slot_concurrency_group(self) -> None:
        # #1697 item 2: a shared group keeps one pending run and cancels the rest.
        self.assertNotRegex(self.report_job(), r"(?m)^    concurrency:")

    def test_report_job_runs_the_tested_helper_with_run_read_access(self) -> None:
        job = self.report_job()
        self.assertIn("run: python3 .github/scripts/postmerge_report.py", job)
        self.assertRegex(job, r"(?m)^      actions: read$")
        self.assertIn("RUN_ID: ${{ github.run_id }}", job)
        self.assertIn(LABEL, (REPO_ROOT / ".github/scripts/postmerge_report.py").read_text())


if __name__ == "__main__":
    unittest.main()
