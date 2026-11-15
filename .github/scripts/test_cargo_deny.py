"""cargo-deny 0.19 recording, reporter API doubles, and real temporary Git repos."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import cargo_deny as deny

FIXTURE = Path(__file__).parent / "fixtures/cargo-deny-0.19.jsonl"
SHA = "a" * 40
RECORD = {"sha": SHA, "started": "2026-11-15T08:00:00Z", "run": 42, "attempt": 1}


class DiagnosticTests(unittest.TestCase):
    def test_recorded_json_and_dependency_path(self):
        # Recorded with 0.19.0 against rustls-webpki =0.103.15 on 2026-11-15.
        items = deny.advisories(FIXTURE.read_text())
        self.assertEqual(
            items,
            [
                {
                    "id": "RUSTSEC-2026-0382",
                    "crate": "rustls-webpki",
                    "locked": "0.103.15",
                    "patched": [">=0.103.16, <0.104.0-alpha.1", ">=0.104.0-alpha.8"],
                    "paths": ["cargo-deny-proof 0.1.0 → rustls-webpki 0.103.15"],
                }
            ],
        )
        body = deny.render(RECORD, 1, items, "https://example.test/run")
        for expected in (
            "RUSTSEC-2026-0382",
            "0.103.15",
            ">=0.103.16",
            "cargo-deny-proof",
        ):
            self.assertIn(expected, body)

    def test_no_patch_multiple_parents_and_duplicate_diagnostics(self):
        diagnostic = json.loads(FIXTURE.read_text().splitlines()[1])
        diagnostic["fields"]["notes"] = ["Solution: No safe upgrade is available!"]
        diagnostic["fields"]["graphs"][0]["parents"].append(
            {
                "Krate": {"name": "another-root", "version": "2.0.0"},
            }
        )
        text = json.dumps(diagnostic)
        items = deny.advisories(text + "\n" + text)
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]["patched"], [])
        self.assertEqual(len(items[0]["paths"]), 2)

    def test_non_advisory_errors_are_not_mistaken_for_clean_check(self):
        text = '{"type":"diagnostic","fields":{"code":"banned","severity":"error"}}'
        self.assertEqual(deny.advisories(text), [])
        self.assertIn("not a clean advisory result", deny.render(RECORD, 1, [], "run"))

    def test_malformed_output_fails(self):
        with self.assertRaises(ValueError):
            deny.advisories("not JSON")
        with self.assertRaises(KeyError):
            deny.advisories('{"type":"diagnostic","fields":{"code":"vulnerability"}}')

    def test_marker_round_trip_and_malformed_markers(self):
        self.assertEqual(deny.parse_marker(deny.marker(RECORD)), RECORD)
        for value in (None, [], {}, {**RECORD, "run": True}, {**RECORD, "sha": "bad"}):
            self.assertIsNone(deny.parse_marker(deny.marker(value)))
        self.assertIsNone(deny.parse_marker("<!-- cargo-deny-report {bad} -->"))

    def test_large_graphs_keep_every_advisory_in_the_report(self):
        item = deny.advisories(FIXTURE.read_text())[0]
        paths = [
            f"root-{i} → " + "dependency → " * 20 + "rustls-webpki 0.103.15"
            for i in range(600)
        ]
        complete = "root → rustls-webpki 0.103.15"
        paths += [complete, "webpki (remaining parents shown elsewhere)"]
        items = [
            {**item, "id": f"RUSTSEC-2026-{i:04}", "paths": paths} for i in range(20)
        ]
        body = deny.render(RECORD, 1, items, "run")
        self.assertLess(len(body), 60000)
        for advisory in items:
            self.assertIn(advisory["id"], body)
        self.assertEqual(body.count(complete), len(items))
        self.assertIn("Additional dependency paths", body)
        self.assertEqual(deny.parse_marker(body), RECORD)

    def test_missing_solution_note_fails_clearly(self):
        diagnostic = json.loads(FIXTURE.read_text().splitlines()[1])
        diagnostic["fields"]["notes"] = []
        with self.assertRaisesRegex(ValueError, "Unknown cargo-deny solution"):
            deny.advisories(json.dumps(diagnostic))

    def test_feature_graph_nodes(self):
        graph = {
            "Krate": {"name": "dep", "version": "1.0.0"},
            "parents": [
                {
                    "Feature": {"crate_name": "consumer", "name": "tls"},
                    "parents": [{"Krate": {"name": "root", "version": "2.0.0"}}],
                }
            ],
        }
        self.assertEqual(
            deny.dependency_paths(graph),
            ["root 2.0.0 → consumer feature tls → dep 1.0.0"],
        )

    def test_full_check_exit_status_is_preserved(self):
        with tempfile.TemporaryDirectory() as temp:
            for status in (0, 1, 2, 101):
                with (
                    self.subTest(status=status),
                    patch.object(
                        deny.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess(
                            [], status, FIXTURE.read_text()
                        ),
                    ) as run,
                ):
                    output = Path(temp) / "deny.jsonl"
                    self.assertEqual(deny.check(Path(temp), output), status)
                    command = run.call_args.args[0]
                    self.assertEqual(command[-1], "check")
                    self.assertIn("--all-features", command)
                    self.assertEqual(
                        output.with_suffix(".status").read_text(), str(status)
                    )


class MergeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.origin = self.root / "origin"
        self.pr = self.root / "pr"
        self.origin.mkdir()
        self.git(self.origin, "init", "-b", "main")
        self.git(self.origin, "config", "user.name", "Test")
        self.git(self.origin, "config", "user.email", "test@localhost")
        (self.origin / "Cargo.lock").write_text("vulnerable version\n")
        self.commit(self.origin, "initial")
        self.git(self.root, "clone", str(self.origin), str(self.pr))
        self.git(self.pr, "config", "user.name", "Test")
        self.git(self.pr, "config", "user.email", "test@localhost")
        self.git(self.pr, "checkout", "-b", "stale-pr")
        (self.pr / "unrelated.txt").write_text("PR change")
        self.commit(self.pr, "unrelated PR change")
        self.head = self.git(self.pr, "rev-parse", "HEAD")

    def git(self, repo, *args):
        return subprocess.run(
            ["git", "-C", str(repo), "-c", "commit.gpgsign=false", *args],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    def commit(self, repo, message):
        self.git(repo, "add", ".")
        self.git(repo, "commit", "-m", message)

    def test_stale_pr_uses_newly_fetched_fixed_base_without_changing_remote_head(self):
        (self.origin / "Cargo.lock").write_text("patched version\n")
        self.commit(self.origin, "fix advisory on base after PR creation")
        base = self.git(self.origin, "rev-parse", "HEAD")
        self.assertEqual((self.pr / "Cargo.lock").read_text(), "vulnerable version\n")
        self.assertEqual(deny.merge_base(self.pr, "main"), base)
        self.assertEqual((self.pr / "Cargo.lock").read_text(), "patched version\n")
        self.assertEqual(self.git(self.pr, "rev-parse", "HEAD^1"), self.head)
        self.assertEqual(self.git(self.pr, "rev-parse", "HEAD^2"), base)
        self.assertEqual(self.git(self.pr, "status", "--porcelain"), "")
        self.assertEqual(self.git(self.origin, "rev-parse", "HEAD"), base)

    def test_merge_does_not_hide_pr_added_bad_lock(self):
        (self.pr / "Cargo.lock").write_text(
            "PR introduces another vulnerable version\n"
        )
        self.commit(self.pr, "bad PR dependency change")
        (self.origin / "base.txt").write_text("unrelated base change")
        self.commit(self.origin, "advance base")
        deny.merge_base(self.pr, "main")
        self.assertIn("PR introduces", (self.pr / "Cargo.lock").read_text())

    def test_conflict_is_clear_and_checkout_is_clean_after_abort(self):
        for repo, value in ((self.origin, "base fix"), (self.pr, "PR dependency")):
            (repo / "Cargo.lock").write_text(value)
            self.commit(repo, value)
        head = self.git(self.pr, "rev-parse", "HEAD")
        with self.assertRaisesRegex(
            RuntimeError, "Resolve the merge conflict with the base branch"
        ):
            deny.merge_base(self.pr, "main")
        self.assertEqual(self.git(self.pr, "rev-parse", "HEAD"), head)
        self.assertEqual(self.git(self.pr, "status", "--porcelain"), "")


class ReporterTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.output = Path(self.temp.name) / "deny.jsonl"
        self.output.write_text(FIXTURE.read_text())
        self.output.with_suffix(".status").write_text("1")
        self.calls = []
        self.body = None
        self.issues = []
        self.head = SHA
        self.env = patch.dict(
            os.environ,
            {
                "GH_REPO": "owner/repo",
                "GITHUB_SHA": SHA,
                "GITHUB_EVENT_NAME": "push",
                "GITHUB_REF": "refs/heads/main",
                "GITHUB_RUN_ID": "42",
                "GITHUB_RUN_ATTEMPT": "1",
            },
        )
        self.env.start()
        self.addCleanup(self.env.stop)

    def gh(self, args):
        self.calls.append(args)
        if args == ["api", "repos/owner/repo/actions/runs/42"]:
            return json.dumps({"run_started_at": RECORD["started"]})
        if "repos/owner/repo/commits/main" in args:
            return self.head
        if "--slurp" in args:
            return json.dumps([self.issues])
        if "--body-file" in args:
            self.body = Path(args[args.index("--body-file") + 1]).read_text()
        return "https://github.com/owner/repo/issues/100"

    def issue(self, **changes):
        return {
            "number": 100,
            "state": "open",
            "body": deny.marker(RECORD),
            "user": {"login": deny.BOT, "type": "Bot"},
            **changes,
        }

    def test_create_then_update_same_issue_with_full_details(self):
        deny.report(self.output, self.gh)
        self.assertTrue(any(call[:2] == ["issue", "create"] for call in self.calls))
        self.assertIn("RUSTSEC-2026-0382", self.body)
        self.issues = [self.issue(body=self.body)]
        self.calls.clear()
        deny.report(self.output, self.gh)
        self.assertTrue(
            any(call[:3] == ["issue", "edit", "100"] for call in self.calls)
        )
        self.assertFalse(any(call[:2] == ["issue", "create"] for call in self.calls))

    def test_clean_run_updates_closed_tracker_without_reopening(self):
        self.issues = [self.issue(state="closed")]
        self.output.write_text("")
        self.output.with_suffix(".status").write_text("0")
        deny.report(self.output, self.gh)
        self.assertIn("passed.", self.body)
        self.assertFalse(any(call[:2] == ["issue", "reopen"] for call in self.calls))

    def test_advisory_recurrence_reopens_same_closed_tracker(self):
        self.issues = [self.issue(state="closed")]
        deny.report(self.output, self.gh)
        self.assertTrue(
            any(call[:3] == ["issue", "reopen", "100"] for call in self.calls)
        )
        self.assertFalse(any(call[:2] == ["issue", "create"] for call in self.calls))

    def test_clean_first_run_does_not_create_issue(self):
        self.output.write_text("")
        self.output.with_suffix(".status").write_text("0")
        deny.report(self.output, self.gh)
        self.assertFalse(any(call[0] in ("issue", "label") for call in self.calls))

    def test_stale_commit_or_out_of_order_run_does_not_write(self):
        self.head = "b" * 40
        deny.report(self.output, self.gh)
        self.assertIsNone(self.body)
        self.head = SHA
        self.issues = [self.issue(body=deny.marker({**RECORD, "run": 43}))]
        deny.report(self.output, self.gh)
        self.assertIsNone(self.body)

    def test_human_marker_is_not_trusted(self):
        self.issues = [self.issue(user={"login": "someone", "type": "User"})]
        deny.report(self.output, self.gh)
        self.assertTrue(any(call[:2] == ["issue", "create"] for call in self.calls))

    def test_pr_cannot_report(self):
        os.environ["GITHUB_EVENT_NAME"] = "pull_request"
        with self.assertRaisesRegex(ValueError, "Only main"):
            deny.report(self.output, self.gh)
        self.assertEqual(self.calls, [])

    def test_invalid_output_preserves_existing_report(self):
        self.output.write_text("download error")
        with self.assertRaises(ValueError):
            deny.report(self.output, self.gh)
        self.assertIsNone(self.body)


if __name__ == "__main__":
    unittest.main()
