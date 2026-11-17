#!/usr/bin/env python3

"""npm staging fixture check and refresh (#2515), against a faked `gh api`."""

import io
import json
from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime, timezone
from pathlib import Path
import re
import tempfile
import unittest
from unittest import mock

import npm_staging_fixture as fixture
from npm_staging_fixture import FixtureError, verify


REPO = "openai/codex"
THIS_REPO = "CorbanuCore/CorbanuTerminal"
RUN_URL = f"https://github.com/{THIS_REPO}/actions/runs/99"
NOW = datetime(2026, 11, 17, 13, 0, tzinfo=timezone.utc)
FAR = "2027-02-15T03:09:19Z"  # about 90 days after NOW
TARGETS = fixture.stage.BINARY_TARGETS
WORKFLOW_PATH = fixture.stage.WORKFLOW_NAME


def run(run_id: int, created_at: str, **overrides) -> dict:
    data = {
        "id": run_id,
        "name": "rust-release",
        "path": WORKFLOW_PATH,
        "head_branch": f"rust-v0.0.{run_id}",
        "head_sha": f"{run_id:040d}",
        "status": "completed",
        "conclusion": "success",
        "created_at": created_at,
    }
    data.update(overrides)
    return data


def artifact(name: str, expires_at: str = FAR, expired: bool = False) -> dict:
    return {
        "name": name,
        "expired": expired,
        "expires_at": expires_at,
        "size_in_bytes": 1,
        "digest": f"sha256:{name}",
    }


def full_set(**overrides) -> list[dict]:
    """All six targets plus an unrelated artifact, like a real release run."""
    items = [artifact(target) for target in TARGETS]
    items.append(
        artifact(
            "provisioned-macos-signed-candidate-x86_64-apple-darwin",
            "2026-11-18T00:00:00Z",
        )
    )
    for name, replacement in overrides.items():
        items = [item for item in items if item["name"] != name]
        if replacement is not None:
            items.append(replacement)
    return items


class FakeGh:
    """Serve `gh api` calls from per-run tables; record every call."""

    def __init__(
        self,
        runs: dict[int, dict],
        artifacts: dict[int, list[dict]],
        issues: list[dict] | None = None,
    ):
        self.runs = runs
        self.artifacts = artifacts
        # Issues of THIS repo (all creators, open and closed, PRs included).
        self.issues = issues if issues is not None else []
        self.calls: list[list[str]] = []
        self.writes: list[tuple[str, str, dict]] = []

    def __call__(self, args, text=True, input=None):
        self.calls.append(list(args))
        assert args[:2] == ["gh", "api"], args
        path = next(a for a in args[2:] if a.startswith("repos/"))
        jq = args[args.index("--jq") + 1] if "--jq" in args else None
        if "--method" in args:
            method = args[args.index("--method") + 1]
            assert args[-2:] == ["--input", "-"], args
            payload = json.loads(input)
            self.writes.append((method, path, payload))
            if method == "POST" and path == f"repos/{THIS_REPO}/issues":
                number = max([i["number"] for i in self.issues] + [0]) + 1
                issue = {
                    "number": number,
                    "state": "open",
                    "user": {"login": "github-actions[bot]"},
                    "html_url": f"https://github.com/{THIS_REPO}/issues/{number}",
                    **payload,
                }
                self.issues.append(issue)
                return json.dumps(issue)
            m = re.fullmatch(rf"repos/{THIS_REPO}/issues/(\d+)", path)
            if method == "PATCH" and m:
                issue = next(i for i in self.issues if i["number"] == int(m.group(1)))
                issue.update(payload)
                return json.dumps(issue)
            raise AssertionError(f"unexpected gh write: {args}")
        if path == (
            f"repos/{THIS_REPO}/issues?state=open&creator=github-actions%5Bbot%5D"
            "&sort=created&direction=asc&per_page=100"
        ):
            assert jq == ".[]" and "--paginate" not in args
            # Like the API, filter by state and creator (PRs are included).
            listed = sorted(
                (
                    i
                    for i in self.issues
                    if i["state"] == "open"
                    and i["user"]["login"] == "github-actions[bot]"
                ),
                key=lambda i: i["number"],
            )
            return "".join(json.dumps(i) + "\n" for i in listed)
        m = re.fullmatch(rf"repos/{REPO}/actions/runs/(\d+)", path)
        if m:
            assert jq is None
            return json.dumps(self.runs[int(m.group(1))])
        m = re.fullmatch(
            rf"repos/{REPO}/actions/runs/(\d+)/artifacts\?per_page=100", path
        )
        if m:
            assert jq == ".artifacts[]" and "--paginate" in args
            return "".join(
                json.dumps(a) + "\n" for a in self.artifacts[int(m.group(1))]
            )
        m = re.fullmatch(
            rf"repos/{REPO}/actions/workflows/rust-release\.yml/runs"
            r"\?status=success&exclude_pull_requests=true&per_page=(\d+)",
            path,
        )
        if m:
            assert jq == ".workflow_runs[]" and "--paginate" not in args
            # The API pages newest first; hand the page back oldest first to
            # prove the script orders it itself.
            newest = sorted(
                self.runs.values(), key=lambda r: r["created_at"], reverse=True
            )
            page = list(reversed(newest[: int(m.group(1))]))
            return "".join(json.dumps(r) + "\n" for r in page)
        raise AssertionError(f"unexpected gh call: {args}")


class VerifyTest(unittest.TestCase):
    def test_success_reports_provenance_and_earliest_expiry(self) -> None:
        items = full_set(**{TARGETS[0]: artifact(TARGETS[0], "2027-02-10T00:00:00Z")})
        got = verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 21)
        self.assertEqual(got.run_id, 7)
        self.assertEqual(got.tag, "rust-v0.0.7")
        self.assertEqual(got.head_sha, f"{7:040d}")
        self.assertEqual(got.expires_at, "2027-02-10T00:00:00Z")
        self.assertEqual([t for t, _, _ in got.artifacts], list(TARGETS))
        self.assertEqual(got.url, "https://github.com/openai/codex/actions/runs/7")

    def test_missing_target_fails_closed_and_names_it(self) -> None:
        items = full_set(**{"aarch64-pc-windows-msvc": None})
        with self.assertRaisesRegex(
            FixtureError, "not found for target aarch64-pc-windows-msvc"
        ):
            verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 21)

    def test_run_without_artifacts_fails_on_first_target(self) -> None:
        # Run 32199727293 after retention: `total_count: 0` (#2515).
        with self.assertRaisesRegex(FixtureError, f"not found for target {TARGETS[0]}"):
            verify(REPO, run(32199727293, "2026-08-19T00:03:26Z"), [], NOW, 21)

    def test_expired_target_fails(self) -> None:
        name = "x86_64-apple-darwin"
        items = full_set(**{name: artifact(name, "2026-11-01T00:00:00Z", expired=True)})
        with self.assertRaisesRegex(
            FixtureError, f"target {name} \\(artifact {name}\\) has expired"
        ):
            verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 21)

    def test_past_expiry_fails_even_if_not_flagged_yet(self) -> None:
        name = "x86_64-apple-darwin"
        items = full_set(**{name: artifact(name, "2026-11-17T12:00:00Z")})
        with self.assertRaisesRegex(FixtureError, "expired at 2026-11-17T12:00:00Z"):
            verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 0)

    def test_expiry_inside_threshold_fails(self) -> None:
        name = "aarch64-unknown-linux-musl"
        items = full_set(**{name: artifact(name, "2026-12-01T13:00:00Z")})
        with self.assertRaisesRegex(
            FixtureError, f"target {name} .* in 14.0 days \\(need at least 21\\)"
        ):
            verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 21)
        # The same artifact is fine under a shorter threshold.
        verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 14)

    def test_missing_expiry_metadata_fails_closed(self) -> None:
        name = TARGETS[2]
        bad = artifact(name)
        del bad["expired"]
        with self.assertRaisesRegex(FixtureError, f"target {name} .* has expired"):
            verify(
                REPO, run(7, "2026-11-17T03:09:19Z"), full_set(**{name: bad}), NOW, 21
            )

    def test_failed_or_unfinished_run_fails(self) -> None:
        for overrides in (
            {"conclusion": "failure"},
            {"conclusion": "cancelled"},
            {"status": "in_progress", "conclusion": None},
        ):
            with self.subTest(**overrides):
                with self.assertRaisesRegex(FixtureError, "not a successful release"):
                    verify(
                        REPO,
                        run(7, "2026-11-17T03:09:19Z", **overrides),
                        full_set(),
                        NOW,
                        21,
                    )

    def test_other_workflow_fails(self) -> None:
        other = run(7, "2026-11-17T03:09:19Z", path=".github/workflows/rust-ci.yml")
        with self.assertRaisesRegex(
            FixtureError, "not .github/workflows/rust-release.yml"
        ):
            verify(REPO, other, full_set(), NOW, 21)

    def test_unsigned_fallback_matches_staging(self) -> None:
        name = "x86_64-apple-darwin"
        items = full_set(**{name: artifact(f"{name}-unsigned")})
        got = verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 21)
        self.assertIn(
            (name, f"{name}-unsigned", f"sha256:{name}-unsigned"), got.artifacts
        )

    def test_signed_artifact_wins_like_staging(self) -> None:
        # Staging downloads the signed artifact when both exist, so its expiry
        # is the one that matters, even if the unsigned twin is still valid.
        name = "aarch64-apple-darwin"
        items = full_set(**{name: artifact(name, "2026-11-20T00:00:00Z")})
        items.append(artifact(f"{name}-unsigned"))
        with self.assertRaisesRegex(
            FixtureError, f"target {name} \\(artifact {name}\\) expires"
        ):
            verify(REPO, run(7, "2026-11-17T03:09:19Z"), items, NOW, 21)


class CommandTest(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = Path(tmp.name)
        self.pin = self.dir / "pin.json"
        self.summary = self.dir / "summary.md"
        self.output = self.dir / "github_output"
        patcher = mock.patch.dict(
            "os.environ", {"GITHUB_OUTPUT": str(self.output)}, clear=False
        )
        patcher.start()
        self.addCleanup(patcher.stop)
        self.addCleanup(mock.patch.stopall)
        mock.patch.object(fixture, "utcnow", return_value=NOW).start()
        # Never let the default step-summary path leak in from a CI runner.
        mock.patch.dict("os.environ", {"GITHUB_STEP_SUMMARY": ""}).start()

    def write_pin(self, run_id: int) -> str:
        text = (
            json.dumps({"repo": REPO, "run_id": run_id, "tag": "old"}, indent=2) + "\n"
        )
        self.pin.write_text(text)
        return text

    def main(self, gh: FakeGh, *args: str) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(fixture.subprocess, "check_output", gh):
            with redirect_stdout(out), redirect_stderr(err):
                code = fixture.main(["--pin-file", str(self.pin), *args])
        return code, out.getvalue(), err.getvalue()

    def test_url_prints_pinned_run(self) -> None:
        self.write_pin(44008150412)
        code, out, _ = self.main(FakeGh({}, {}), "url")
        self.assertEqual(
            (code, out),
            (0, "https://github.com/openai/codex/actions/runs/44008150412\n"),
        )

    def test_pin_for_another_repo_fails_closed(self) -> None:
        self.pin.write_text(json.dumps({"repo": "someone/else", "run_id": 1}))
        code, _, err = self.main(FakeGh({}, {}), "url")
        self.assertEqual(code, 1)
        self.assertIn("staging downloads from openai/codex", err)

    def test_check_passes_on_valid_pin(self) -> None:
        self.write_pin(10)
        gh = FakeGh({10: run(10, "2026-11-17T03:09:19Z")}, {10: full_set()})
        code, out, err = self.main(gh, "check")
        self.assertEqual((code, err), (0, ""))
        self.assertIn("Artifacts expire: 2027-02-15T03:09:19Z", out)

    def test_check_names_missing_target_for_dead_pin(self) -> None:
        self.write_pin(32199727293)
        gh = FakeGh(
            {32199727293: run(32199727293, "2026-08-19T00:03:26Z")}, {32199727293: []}
        )
        code, _, err = self.main(gh, "check")
        self.assertEqual(code, 1)
        self.assertIn(
            "run 32199727293: Expected workflow artifact not found for target x86_64-unknown-linux-musl",
            err,
        )

    def test_check_api_failure_fails_closed(self) -> None:
        self.write_pin(10)

        def broken(args, text=True):
            raise fixture.subprocess.CalledProcessError(1, args)

        code, _, err = self.main(broken, "check")
        self.assertEqual(code, 1)
        self.assertIn("GitHub API call failed", err)

    def test_refresh_keeps_valid_pin(self) -> None:
        before = self.write_pin(10)
        gh = FakeGh(
            {10: run(10, "2026-11-17T03:09:19Z"), 11: run(11, "2026-11-17T09:00:00Z")},
            {10: full_set(), 11: full_set()},
        )
        candidate = self.dir / "out" / "candidate.json"
        code, out, _ = self.main(
            gh,
            "refresh",
            "--output",
            str(candidate),
            "--summary-file",
            str(self.summary),
        )
        self.assertEqual(code, 0)
        self.assertEqual(self.pin.read_text(), before)
        self.assertIn("changed=false", self.output.read_text())
        self.assertIn("no tracking issue", out)
        self.assertFalse(
            any("workflows" in " ".join(c) for c in gh.calls),
            "no search when the pin is valid",
        )
        # Healthy: no candidate, no summary, so no artifact and no issue.
        self.assertFalse(candidate.exists())
        self.assertFalse(self.summary.exists())
        self.assertEqual(gh.writes, [])

    def test_refresh_writes_candidate_to_output_and_leaves_pin(self) -> None:
        before = self.write_pin(10)
        runs = {
            10: run(10, "2026-08-19T00:03:26Z"),
            11: run(11, "2026-11-17T00:00:00Z"),
        }
        candidate = self.dir / "out" / "npm-staging-fixture.json"
        code, _, err = self.main(
            FakeGh(runs, {10: [], 11: full_set()}),
            "refresh",
            "--output",
            str(candidate),
            "--summary-file",
            str(self.summary),
        )
        self.assertEqual((code, err), (0, ""))
        self.assertEqual(self.pin.read_text(), before)
        self.assertEqual(json.loads(candidate.read_text())["run_id"], 11)
        summary = self.summary.read_text()
        self.assertIn(f"```json\n{candidate.read_text()}```", summary)
        self.assertIn(
            "fails the check: run 10: Expected workflow artifact not found", summary
        )
        self.assertIn("in a normal PR", summary)
        self.assertIn("stays disabled", summary)
        self.assertNotIn(fixture.ISSUE_MARKER, summary)

    def test_refresh_picks_newest_valid_run_and_skips_invalid_ones(self) -> None:
        self.write_pin(10)
        expiring = "2026-11-25T00:00:00Z"
        runs = {
            10: run(10, "2026-08-19T00:03:26Z"),
            11: run(11, "2026-10-01T00:00:00Z"),  # valid, but older than 12
            12: run(12, "2026-11-10T00:00:00Z"),  # newest valid
            13: run(13, "2026-11-15T00:00:00Z"),  # missing a target
            14: run(14, "2026-11-16T00:00:00Z", conclusion="failure"),
            15: run(15, "2026-11-17T00:00:00Z"),  # a target expires inside 21 days
        }
        artifacts = {
            10: [],
            11: full_set(),
            12: full_set(
                **{"x86_64-apple-darwin": artifact("x86_64-apple-darwin-unsigned")}
            ),
            13: full_set(**{"aarch64-unknown-linux-musl": None}),
            14: full_set(),
            15: full_set(**{TARGETS[5]: artifact(TARGETS[5], expiring)}),
        }
        gh = FakeGh(runs, artifacts)
        code, out, err = self.main(gh, "refresh", "--summary-file", str(self.summary))
        self.assertEqual((code, err), (0, ""))
        pin = json.loads(self.pin.read_text())
        self.assertEqual(
            pin,
            {
                "repo": REPO,
                "run_id": 12,
                "tag": "rust-v0.0.12",
                "head_sha": f"{12:040d}",
                "created_at": "2026-11-10T00:00:00Z",
                "expires_at": FAR,
            },
        )
        self.assertEqual(list(pin), list(fixture.PIN_KEYS))
        self.assertTrue(self.pin.read_text().endswith("}\n"))
        for skipped in (
            "run 15: target",
            "run 14 is not a successful release",
            "run 13: Expected workflow artifact not found for target aarch64-unknown-linux-musl",
        ):
            self.assertIn(skipped, out)
        # Newest first; it stops at the first valid run, so 11 is never read.
        examined = [c[3].split("/")[5] for c in gh.calls if c[2] == "--paginate"]
        self.assertEqual(examined, ["10", "15", "14", "13", "12"])
        summary = self.summary.read_text()
        self.assertIn(
            "| x86_64-apple-darwin | x86_64-apple-darwin-unsigned | sha256:x86_64-apple-darwin-unsigned |",
            summary,
        )
        self.assertIn("https://github.com/openai/codex/actions/runs/10", summary)
        self.assertIn("changed=true\nrun_id=12\n", self.output.read_text())

    def test_refresh_respects_limit(self) -> None:
        self.write_pin(10)
        runs = {i: run(i, f"2026-11-{i:02d}T00:00:00Z") for i in range(10, 13)}
        artifacts = {10: [], 11: full_set(), 12: []}
        code, _, err = self.main(FakeGh(runs, artifacts), "refresh", "--limit", "1")
        self.assertEqual(code, 1)
        self.assertIn("none of the newest 1 successful rust-release runs", err)

    def test_refresh_fails_closed_when_no_run_is_valid(self) -> None:
        before = self.write_pin(10)
        runs = {
            10: run(10, "2026-08-19T00:03:26Z"),
            11: run(11, "2026-11-17T00:00:00Z"),
        }
        candidate = self.dir / "out" / "candidate.json"
        code, _, err = self.main(
            FakeGh(runs, {10: [], 11: []}),
            "refresh",
            "--output",
            str(candidate),
            "--summary-file",
            str(self.summary),
        )
        self.assertEqual(code, 1)
        self.assertIn("the pin stays at run 10", err)
        self.assertEqual(self.pin.read_text(), before)
        # The job fails: no `changed` output, candidate or summary, so the
        # workflow uploads nothing and the report job does not run.
        self.assertFalse(self.output.exists())
        self.assertFalse(candidate.exists())
        self.assertFalse(self.summary.exists())

    # --- report: the one bot-owned tracking issue -------------------------

    def report(self, gh: FakeGh) -> tuple[int, str, str]:
        return self.main(
            gh,
            "report",
            "--repo",
            THIS_REPO,
            "--run-url",
            RUN_URL,
            "--summary-file",
            str(self.summary),
        )

    @staticmethod
    def bot_issue(number: int, body: str, **overrides) -> dict:
        issue = {
            "number": number,
            "state": "open",
            "title": "old title",
            "body": body,
            "user": {"login": "github-actions[bot]"},
            "html_url": f"https://github.com/{THIS_REPO}/issues/{number}",
        }
        issue.update(overrides)
        return issue

    def test_report_opens_issue_when_none_exists(self) -> None:
        self.summary.write_text("## refresh needed\n\nproposed pin\n")
        marker = fixture.ISSUE_MARKER
        decoys = [
            # A human issue quoting the marker, a bot PR and a closed bot issue.
            self.bot_issue(1, marker, user={"login": "someone"}),
            self.bot_issue(2, marker, pull_request={"url": "x"}),
            self.bot_issue(3, marker, state="closed"),
            self.bot_issue(4, "an unrelated bot issue"),
        ]
        gh = FakeGh({}, {}, issues=decoys)
        code, out, err = self.report(gh)
        self.assertEqual((code, err), (0, ""))
        self.assertEqual(
            [(m, p) for m, p, _ in gh.writes], [("POST", f"repos/{THIS_REPO}/issues")]
        )
        body = gh.writes[0][2]["body"]
        self.assertTrue(body.startswith(marker + "\n## refresh needed"))
        self.assertIn(f"Workflow run: {RUN_URL}", body)
        self.assertIn(fixture.ARTIFACT_NAME, body)
        self.assertEqual(gh.writes[0][2]["title"], fixture.ISSUE_TITLE)
        self.assertIn("Opened tracking issue #5", out)
        self.assertIn("issue=5\naction=opened\n", self.output.read_text())

    def test_report_updates_existing_issue_instead_of_duplicating(self) -> None:
        self.summary.write_text("## refresh needed\n\nnewer candidate\n")
        older = self.bot_issue(7, f"{fixture.ISSUE_MARKER}\nolder candidate\n")
        newer = self.bot_issue(9, f"{fixture.ISSUE_MARKER}\nanother\n")
        gh = FakeGh({}, {}, issues=[newer, older])
        code, out, err = self.report(gh)
        self.assertEqual((code, err), (0, ""))
        self.assertEqual(
            [(m, p) for m, p, _ in gh.writes],
            [("PATCH", f"repos/{THIS_REPO}/issues/7")],
        )
        self.assertIn("newer candidate", older["body"])
        self.assertEqual(older["title"], fixture.ISSUE_TITLE)
        self.assertEqual(len(gh.issues), 2)
        self.assertIn("Updated tracking issue #7", out)

    def test_report_is_idempotent_across_weekly_runs(self) -> None:
        self.summary.write_text("## refresh needed\n\nsame candidate\n")
        gh = FakeGh({}, {})
        for _ in range(3):
            self.assertEqual(self.report(gh)[0], 0)
        self.assertEqual(len(gh.issues), 1)
        # Opened once; the identical second and third reports write nothing.
        self.assertEqual([m for m, _, _ in gh.writes], ["POST"])
        self.assertIn("action=unchanged", self.output.read_text())

    def test_lookup_only_trusts_bot_issues_with_the_marker(self) -> None:
        # Even if the listing returned other issues (the creator filter is the
        # API's), only an open bot issue, not a PR, with the marker counts.
        marker = fixture.ISSUE_MARKER
        listed = [
            self.bot_issue(1, marker, user={"login": "someone"}),
            self.bot_issue(2, marker, pull_request={"url": "x"}),
            self.bot_issue(3, "no marker"),
            self.bot_issue(4, f"text\n{marker}\n"),
        ]

        def lines(issues):
            return "".join(json.dumps(i) + "\n" for i in issues)

        with mock.patch.object(fixture, "gh_api", return_value=lines(listed)):
            self.assertEqual(fixture.find_tracking_issue(THIS_REPO)["number"], 4)
        with mock.patch.object(fixture, "gh_api", return_value=lines(listed[:3])):
            self.assertIsNone(fixture.find_tracking_issue(THIS_REPO))

    def test_report_fails_closed_without_summary(self) -> None:
        gh = FakeGh({}, {})
        code, _, err = self.report(gh)
        self.assertEqual(code, 1)
        self.assertIn("cannot read summary", err)
        self.summary.write_text("  \n")
        code, _, err = self.report(gh)
        self.assertEqual(code, 1)
        self.assertIn("is empty", err)
        self.assertEqual((gh.calls, gh.writes), ([], []))

    def test_rejects_bad_arguments(self) -> None:
        self.write_pin(10)
        for args in (
            ["check", "--min-days", "-1"],
            ["refresh", "--limit", "0"],
            ["refresh", "--limit", "101"],
        ):
            with self.subTest(args=args):
                with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                    fixture.main(["--pin-file", str(self.pin), *args])


class RepoWiringTest(unittest.TestCase):
    """The committed pin and repo-checks stay wired to this script."""

    ROOT = Path(__file__).resolve().parents[2]

    def test_committed_pin_is_well_formed(self) -> None:
        pin = fixture.read_pin(fixture.PIN_FILE)
        self.assertEqual(list(pin), list(fixture.PIN_KEYS))
        self.assertEqual(pin["repo"], fixture.stage.GITHUB_REPO)
        fixture.parse_time(pin["expires_at"], "pin")

    def test_repo_checks_stages_from_the_pin_file(self) -> None:
        workflow = (self.ROOT / ".github/workflows/repo-checks.yml").read_text()
        self.assertIn(
            'WORKFLOW_URL="$(python3 .github/scripts/npm_staging_fixture.py url)"',
            workflow,
        )
        self.assertNotRegex(workflow, r"actions/runs/\d+")
        # A pin change is a packaging input, so its PR runs full staging.
        self.assertIn(".github/npm-staging-fixture.json", workflow)
        self.assertIn(".github/scripts/npm_staging_fixture.py", workflow)

    def test_refresh_workflow_reports_by_issue_with_least_privilege(self) -> None:
        text = (self.ROOT / ".github/workflows/npm-staging-fixture.yml").read_text()
        for forbidden in (
            "contents: write",
            "pull-requests: write",
            "create-pull-request",
        ):
            self.assertNotIn(forbidden, text)
        self.assertRegex(text, r"(?m)^permissions:\n  contents: read\n")
        jobs = text.split("\njobs:\n", 1)[1]
        check, report = jobs.split("\n  report:\n", 1)
        self.assertNotIn("issues: write", check)
        self.assertIn("refresh\n", check)
        self.assertIn("name: npm-staging-fixture-candidate", check)
        self.assertIn("if: steps.fixture.outputs.changed == 'true'", check)
        # Only the reporting job may write issues, and only after a check
        # that found a valid replacement (a failed check skips it).
        self.assertIn("needs: check\n", report)
        self.assertIn("if: needs.check.outputs.changed == 'true'", report)
        self.assertRegex(
            report, r"permissions:\n      contents: read\n      issues: write\n"
        )
        self.assertIn("npm_staging_fixture.py report", report)


if __name__ == "__main__":
    unittest.main()
