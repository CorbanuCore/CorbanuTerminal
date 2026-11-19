#!/usr/bin/env python3
"""Check PR regressions against current main; keep full checks and reporting on main."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import tomllib

LABEL = "cargo-deny-main"
TITLE = "cargo-deny dependency check on main"
BOT = "github-actions[bot]"
MARKER = re.compile(r"<!-- cargo-deny-report (\{[^\n]*\}) -->")


def merge_base(repo: Path, base: str) -> str:
    """Fetch the current base, including on reruns of an old PR event."""

    def git(*args: str, check: bool = True) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["git", "-C", str(repo), *args], check=check, text=True, capture_output=True
        )

    git("check-ref-format", f"refs/heads/{base}")
    git("fetch", "--no-tags", "origin", f"refs/heads/{base}")
    sha = git("rev-parse", "FETCH_HEAD").stdout.strip()
    result = git(
        "-c",
        "user.name=cargo-deny CI",
        "-c",
        "user.email=ci@localhost",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.hooksPath=/dev/null",
        "merge",
        "--no-ff",
        "--no-edit",
        sha,
        check=False,
    )
    if result.returncode:
        git("merge", "--abort", check=False)
        raise RuntimeError(
            f"Cannot merge current base {base} ({sha}) into the PR checkout. "
            "Resolve the merge conflict with the base branch and rerun cargo-deny.\n"
            + result.stdout
            + result.stderr
        )
    print(
        f"cargo-deny checks PR head merged with current base {base} at {sha}",
        flush=True,
    )
    return sha


def dependency_paths(node: dict, trail: tuple[str, ...] = ()) -> list[str]:
    crate = node.get("Krate")
    feature = node.get("Feature")
    if crate:
        name = f"{crate['name']} {crate['version']}"
    elif feature:
        name = f"{feature['crate_name']} feature {feature['name']}"
    else:
        raise ValueError("Unknown cargo-deny dependency graph node")
    trail = (*trail, name)
    parents = node.get("parents", [])
    if not parents:
        suffix = " (remaining parents shown elsewhere)" if node.get("repeat") else ""
        return [" → ".join(reversed(trail)) + suffix]
    return [path for parent in parents for path in dependency_paths(parent, trail)]


def advisories(text: str) -> list[dict]:
    """Read cargo-deny 0.19 JSON diagnostics, retaining every emitted advisory."""
    found = {}
    for line in text.splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        if record.get("type") != "diagnostic":
            continue
        fields = record["fields"]
        if fields.get("code") not in (
            "vulnerability",
            "unmaintained",
            "unsound",
            "notice",
        ):
            continue
        advisory = fields["advisory"]
        advisory_id = advisory["id"]
        solution = next(
            (note for note in fields["notes"] if note.startswith("Solution: ")), ""
        )
        if solution == "Solution: No safe upgrade is available!":
            patched = []
        else:
            match = re.fullmatch(
                r"Solution: Upgrade to (.+) \(try `cargo update -p [^\n]+`\)", solution
            )
            if not match:
                raise ValueError(f"Unknown cargo-deny solution: {solution}")
            patched = match[1].split(" OR ")
        graphs = fields["graphs"]
        if not graphs:
            raise ValueError(f"Missing dependency graph for {advisory_id}")
        for graph in graphs:
            crate = graph["Krate"]
            key = (advisory_id, crate["name"], crate["version"])
            item = found.setdefault(
                key,
                {
                    "id": advisory_id,
                    "crate": crate["name"],
                    "locked": crate["version"],
                    "patched": patched,
                    "paths": [],
                },
            )
            item["paths"] = sorted(set(item["paths"] + dependency_paths(graph)))
    return [found[key] for key in sorted(found)]


def check(repo: Path, output: Path) -> int:
    # Capture outside the worktree so the existing clean-worktree gate still applies.
    result = subprocess.run(
        [
            "cargo-deny",
            "--log-level",
            "warn",
            "--format",
            "json",
            "--manifest-path",
            "./Cargo.toml",
            "--all-features",
            "check",
        ],
        cwd=repo / "codex-rs",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    print(result.stdout, end="")
    output.write_text(result.stdout)
    # Parsing/reporting never changes cargo-deny's blocking exit status.
    output.with_suffix(".status").write_text(str(result.returncode))
    return result.returncode if result.returncode >= 0 else 1


def advisory_errors(text: str, status: int, lock: Path) -> set[tuple]:
    """Accept only complete advisory results; never waive infrastructure/errors."""
    records = [json.loads(line) for line in text.splitlines() if line.strip()]
    summaries = [r["fields"] for r in records if r.get("type") == "summary"]
    errors = [
        r["fields"]
        for r in records
        if r.get("type") == "diagnostic" and r["fields"]["severity"] == "error"
    ]
    if (
        status not in (0, 1)
        or len(summaries) != 1
        or "advisories" not in summaries[0]
        or sum(s["errors"] for s in summaries[0].values()) != len(errors)
        or bool(errors) != bool(status)
        or any(
            r.get("type") == "log" and r["fields"]["level"] == "ERROR" for r in records
        )
    ):
        raise ValueError("Incomplete or failed cargo-deny advisory check")
    packages = tomllib.loads(lock.read_text())["package"]
    keys = set()
    for fields in errors:
        # Reuse the strict parser; unknown error codes produce no items and fail.
        items = advisories(json.dumps({"type": "diagnostic", "fields": fields}))
        if not items:
            raise ValueError("Non-advisory error cannot be downgraded")
        for item in items:
            sources = {
                p.get("source", "path")
                for p in packages
                if (p["name"], p["version"]) == (item["crate"], item["locked"])
            }
            # Graphs omit sources. Ambiguous identities must not inherit a waiver.
            if len(sources) != 1:
                raise ValueError("Missing or ambiguous advisory package source")
            keys.add((item["id"], item["crate"], item["locked"], sources.pop()))
    return keys


def check_pr(repo: Path, output: Path, base: str) -> int:
    """Use the exact fetched base and one frozen DB for both advisory checks."""
    if not re.fullmatch(r"[0-9a-f]{40}", base):
        raise ValueError("Expected the full SHA from merge-base")
    repo = repo.resolve()
    config = repo / "codex-rs/deny.toml"

    def run(root: Path, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [
                "cargo-deny",
                "--log-level",
                "warn",
                "--format",
                "json",
                "--manifest-path",
                "./Cargo.toml",
                "--all-features",
                "--locked",
                *args,
                "--config",
                str(config),
            ],
            cwd=root / "codex-rs",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )

    fetched = run(repo, "fetch", "db")
    if fetched.returncode:
        print(fetched.stdout, end="")
        return 1
    # A separate checkout preserves manifests, features and patches on the base.
    # It lives outside the PR checkout and never changes its head or approvals.
    with tempfile.TemporaryDirectory(prefix="cargo-deny-base-") as temp:
        baseline = Path(temp) / "base"
        subprocess.run(
            [
                "git",
                "-C",
                str(repo),
                "worktree",
                "add",
                "--detach",
                str(baseline),
                base,
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        try:
            before = run(baseline, "check", "--disable-fetch", "advisories")
            output.with_suffix(".base.jsonl").write_text(before.stdout)
            known = advisory_errors(
                before.stdout, before.returncode, baseline / "codex-rs/Cargo.lock"
            )
        finally:
            subprocess.run(
                ["git", "-C", str(repo), "worktree", "remove", str(baseline)],
                check=True,
                capture_output=True,
                text=True,
            )
    # Preserve the full licenses/bans/sources check and its original exit status.
    result = run(repo, "check", "--disable-fetch")
    output.write_text(result.stdout)
    print(result.stdout, end="")
    status = result.returncode if result.returncode >= 0 else 1
    if result.returncode in (0, 1):
        current = advisory_errors(result.stdout, status, repo / "codex-rs/Cargo.lock")
        introduced = current - known
        inherited = current & known
        # Link to the persistent main tracker, including before its first report.
        tracker = (
            f"https://github.com/{os.environ['GITHUB_REPOSITORY']}/issues"
            f"?q=is%3Aissue+label%3A{LABEL}"
        )
        for advisory, crate, version, _source in sorted(inherited):
            print(
                f"::warning::Already on base {base}: {advisory} {crate} {version}; "
                f"main dependency tracker: {tracker}"
            )
        for advisory, crate, version, _source in sorted(introduced):
            print(f"::error::PR introduces {advisory} {crate} {version}")
        status = int(bool(introduced))
    output.with_suffix(".status").write_text(str(status))
    return status


def marker(data: dict) -> str:
    return f"<!-- cargo-deny-report {json.dumps(data, sort_keys=True)} -->"


def parse_marker(body: str) -> dict | None:
    match = MARKER.search(body)
    if not match:
        return None
    try:
        data = json.loads(match[1])
        if (
            re.fullmatch(r"[0-9a-f]{40}", data["sha"])
            and re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", data["started"])
            and type(data["run"]) is int
            and data["run"] > 0
            and type(data["attempt"]) is int
            and data["attempt"] > 0
        ):
            return data
    except (ValueError, KeyError, TypeError):
        pass
    return None


def render(data: dict, status: int, items: list[dict], url: str) -> str:
    lines = [
        f"Full cargo-deny check on main at `{data['sha']}`: "
        + ("passed." if status == 0 else f"failed (exit {status})."),
        f"Run: {url}",
        "",
        "PRs block new advisories; advisories already on main warn. "
        "Licenses, bans and sources remain blocking.",
        "",
    ]
    if not items:
        lines.append(
            "No advisory diagnostics were emitted."
            + (
                " See the run log for the check failure; this is not a clean advisory result."
                if status
                else ""
            )
        )
    for item in items:
        # A workspace graph can have hundreds of repeated inclusion paths.
        # Prefer complete paths and keep room for every advisory's key details.
        paths = sorted(
            item["paths"],
            key=lambda path: (
                path.endswith("(remaining parents shown elsewhere)"),
                len(path),
                path,
            ),
        )[:3]
        lines += [
            f"### {item['id']} — {item['crate']} {item['locked']}",
            f"- Locked version: `{item['locked']}`",
            "- Patched versions: "
            + (", ".join(item["patched"]) or "No safe upgrade available"),
            "- Dependency paths (dependent → dependency):",
            *[f"  - `{path}`" for path in paths],
            "",
        ]
        if len(item["paths"]) > len(paths):
            lines.append(
                "Additional dependency paths are in the cargo-deny-report artifact."
            )
    lines += [
        "",
        "This single tracking issue is updated by main and daily runs, including recovery.",
        marker(data),
    ]
    body = "\n".join(lines)
    if len(body) > 60000:
        # Keep the marker even for unusually large graphs. Full JSON is in the run artifact.
        body = (
            body[:58000]
            + "\n\nReport truncated; see cargo-deny-report artifact.\n"
            + marker(data)
        )
    return body


def report(output: Path, gh=None) -> str:
    if gh is None:

        def gh(args):
            return subprocess.run(
                ["gh", *args], check=True, capture_output=True, text=True
            ).stdout

    repo, sha = os.environ["GH_REPO"], os.environ["GITHUB_SHA"]
    # Defence in depth: this entry point must never write from a PR or other branch.
    if (
        os.environ["GITHUB_EVENT_NAME"] not in ("push", "schedule")
        or os.environ["GITHUB_REF"] != "refs/heads/main"
    ):
        raise ValueError(
            "Only main push and scheduled runs may publish cargo-deny reports"
        )
    run_id = int(os.environ["GITHUB_RUN_ID"])
    run = json.loads(gh(["api", f"repos/{repo}/actions/runs/{run_id}"]))
    data = {
        "sha": sha,
        "started": run["run_started_at"],
        "run": run_id,
        "attempt": int(os.environ["GITHUB_RUN_ATTEMPT"]),
    }
    # An older commit finishing after a newer push must not replace main's report.
    head = gh(["api", f"repos/{repo}/commits/main", "--jq", ".sha"]).strip()
    if head != sha:
        return "Stale main commit; report unchanged"
    items = advisories(output.read_text())
    status = int(output.with_suffix(".status").read_text())
    issues = json.loads(
        gh(
            [
                "api",
                "--paginate",
                "--slurp",
                f"repos/{repo}/issues?state=all&labels={LABEL}&per_page=100",
            ]
        )
    )
    owned = sorted(
        (
            issue
            for page in issues
            for issue in page
            if "pull_request" not in issue
            and issue["user"]["login"] == BOT
            and issue["user"]["type"] == "Bot"
            and parse_marker(issue.get("body") or "")
        ),
        key=lambda issue: issue["number"],
    )
    for issue in owned:
        previous = parse_marker(issue["body"])
        if (previous["started"], previous["run"], previous["attempt"]) > (
            data["started"],
            data["run"],
            data["attempt"],
        ):
            return "Newer report already published"
    if not owned and status == 0 and not items:
        return "Clean main; no tracking issue needed"
    url = f"https://github.com/{repo}/actions/runs/{run_id}/attempts/{data['attempt']}"
    body = render(data, status, items, url)
    # The workflow serializes reporters across push and schedule. Reuse even a
    # closed issue so an advisory recurrence never opens a second tracker.
    with tempfile.TemporaryDirectory() as temp:
        body_file = Path(temp) / "body.md"
        body_file.write_text(body)
        if owned:
            number = str(owned[0]["number"])
            gh(["issue", "edit", number, "--repo", repo, "--body-file", str(body_file)])
            if owned[0]["state"] == "closed" and (status != 0 or items):
                gh(["issue", "reopen", number, "--repo", repo])
            return f"Updated #{number}"
        gh(
            [
                "label",
                "create",
                LABEL,
                "--repo",
                repo,
                "--color",
                "B60205",
                "--description",
                TITLE,
                "--force",
            ]
        )
        return gh(
            [
                "issue",
                "create",
                "--repo",
                repo,
                "--title",
                TITLE,
                "--label",
                LABEL,
                "--body-file",
                str(body_file),
            ]
        ).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("merge-base", "check", "report"))
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--base", help="Full base SHA captured by merge-base (PRs only)"
    )
    args = parser.parse_args()
    if args.mode == "merge-base":
        sha = merge_base(args.repo, os.environ["PR_BASE_REF"])
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            output.write(f"base={sha}\n")
        return 0
    if args.output is None:
        parser.error("--output is required for check/report")
    if args.mode == "check":
        if args.base:
            return check_pr(args.repo, args.output, args.base)
        if os.environ.get("GITHUB_EVENT_NAME") == "pull_request":
            parser.error("PR checks require --base from merge-base")
        return check(args.repo, args.output)
    print(report(args.output))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (RuntimeError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"cargo-deny CI: {error}", file=sys.stderr)
        sys.exit(1)
