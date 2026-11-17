#!/usr/bin/env python3

"""Check and refresh the npm staging fixture that repo-checks stages from (#2515).

Full npm staging in repo-checks downloads the six native target artifacts from
one pinned, successful upstream rust-release run. GitHub deletes those
artifacts after their retention period, so a pin that is never moved breaks
staging on a known date (#2514). The pin lives in
`.github/npm-staging-fixture.json` (run ID plus provenance), not in the workflow.

  url      print the pinned run URL (repo-checks passes it to staging).
  check    verify the pinned run: a completed, successful rust-release run whose
           six required target artifacts (picked with staging's own signed /
           `-unsigned` rule) are present, unexpired, and valid for at least
           --min-days more days. Fails closed and names the target.
  refresh  keep the pin if it passes `check`; otherwise search the newest
           successful rust-release runs (at most --limit) for the first one that
           passes `check` and rewrite the pin file. Fails if none does.
"""

import argparse
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys


REPO_ROOT = Path(__file__).resolve().parents[2]
PIN_FILE = REPO_ROOT / ".github" / "npm-staging-fixture.json"
STAGE_SCRIPT = REPO_ROOT / "scripts" / "stage_npm_packages.py"
DEFAULT_MIN_DAYS = 21
DEFAULT_LIMIT = 20
MAX_LIMIT = 100
PIN_KEYS = ("repo", "run_id", "tag", "head_sha", "created_at", "expires_at")


def _load_stage_module():
    spec = importlib.util.spec_from_file_location("stage_npm_packages", STAGE_SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load module from {STAGE_SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# Staging owns the target list, the upstream repo and the fallback rule.
stage = _load_stage_module()


class FixtureError(Exception):
    pass


@dataclass(frozen=True)
class Fixture:
    repo: str
    run_id: int
    tag: str
    head_sha: str
    created_at: str
    expires_at: str
    # (target, artifact name, digest) in BINARY_TARGETS order.
    artifacts: tuple[tuple[str, str, str], ...]

    @property
    def url(self) -> str:
        return run_url(self.repo, self.run_id)

    def pin(self) -> dict:
        return {key: getattr(self, key) for key in PIN_KEYS}


def run_url(repo: str, run_id: int) -> str:
    return f"https://github.com/{repo}/actions/runs/{run_id}"


def utcnow() -> datetime:
    return datetime.now(timezone.utc)


def parse_time(value: object, what: str) -> datetime:
    if not isinstance(value, str) or not value:
        raise FixtureError(f"{what} has no timestamp")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise FixtureError(f"{what} has an unreadable timestamp {value!r}") from exc
    if parsed.tzinfo is None:
        raise FixtureError(f"{what} timestamp {value!r} has no time zone")
    return parsed


def gh_api(path: str, jq: str | None = None, paginate: bool = False) -> str:
    args = ["gh", "api"]
    if paginate:
        args.append("--paginate")
    args.append(path)
    if jq is not None:
        args += ["--jq", jq]
    return subprocess.check_output(args, text=True)


def get_run(repo: str, run_id: int) -> dict:
    return json.loads(gh_api(f"repos/{repo}/actions/runs/{run_id}"))


def list_artifacts(repo: str, run_id: int) -> list[dict]:
    stdout = gh_api(
        f"repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100",
        jq=".artifacts[]",
        paginate=True,
    )
    return [json.loads(line) for line in stdout.splitlines() if line.strip()]


def list_successful_runs(repo: str, limit: int) -> list[dict]:
    # One page only: the search is bounded by design.
    workflow = Path(stage.WORKFLOW_NAME).name
    stdout = gh_api(
        f"repos/{repo}/actions/workflows/{workflow}/runs"
        f"?status=success&exclude_pull_requests=true&per_page={limit}",
        jq=".workflow_runs[]",
    )
    runs = [json.loads(line) for line in stdout.splitlines() if line.strip()]
    # Newest first; the run ID breaks ties so the choice is deterministic.
    runs.sort(
        key=lambda run: (
            parse_time(run.get("created_at"), f"run {run.get('id')}"),
            run["id"],
        ),
        reverse=True,
    )
    return runs[:limit]


def verify(
    repo: str,
    run: dict,
    artifacts: list[dict],
    now: datetime,
    min_days: int,
) -> Fixture:
    run_id = run.get("id")
    where = f"run {run_id}"
    if run.get("path") != stage.WORKFLOW_NAME:
        raise FixtureError(f"{where} is {run.get('path')!r}, not {stage.WORKFLOW_NAME}")
    if run.get("status") != "completed" or run.get("conclusion") != "success":
        raise FixtureError(
            f"{where} is not a successful release "
            f"(status={run.get('status')}, conclusion={run.get('conclusion')})"
        )

    by_name = {artifact["name"]: artifact for artifact in artifacts}
    try:
        selected = stage.select_target_artifact_names(by_name)
    except FileNotFoundError as exc:
        raise FixtureError(f"{where}: {exc}") from exc

    deadline = now + timedelta(days=min_days)
    chosen: list[tuple[str, str, str]] = []
    earliest: tuple[datetime, str] | None = None
    for target, name in selected.items():
        artifact = by_name[name]
        label = f"{where}: target {target} (artifact {name})"
        if artifact.get("expired") is not False:
            raise FixtureError(f"{label} has expired")
        expires = parse_time(artifact.get("expires_at"), label)
        if expires <= now:
            raise FixtureError(f"{label} expired at {artifact['expires_at']}")
        if expires < deadline:
            days = (expires - now) / timedelta(days=1)
            raise FixtureError(
                f"{label} expires at {artifact['expires_at']}, in {days:.1f} days "
                f"(need at least {min_days})"
            )
        if earliest is None or expires < earliest[0]:
            earliest = (expires, artifact["expires_at"])
        chosen.append((target, name, artifact.get("digest") or ""))

    assert earliest is not None  # BINARY_TARGETS is never empty.
    return Fixture(
        repo=repo,
        run_id=int(run_id),
        tag=str(run.get("head_branch") or ""),
        head_sha=str(run.get("head_sha") or ""),
        created_at=str(run.get("created_at") or ""),
        expires_at=earliest[1],
        artifacts=tuple(chosen),
    )


def check_run(repo: str, run_id: int, now: datetime, min_days: int) -> Fixture:
    return verify(
        repo, get_run(repo, run_id), list_artifacts(repo, run_id), now, min_days
    )


def read_pin(path: Path) -> dict:
    try:
        pin = json.loads(path.read_text())
    except (OSError, ValueError) as exc:
        raise FixtureError(f"cannot read pin file {path}: {exc}") from exc
    if not isinstance(pin, dict) or not isinstance(pin.get("run_id"), int):
        raise FixtureError(f"pin file {path} has no integer run_id")
    if pin.get("repo") != stage.GITHUB_REPO:
        raise FixtureError(
            f"pin file {path} names repo {pin.get('repo')!r}; staging downloads "
            f"from {stage.GITHUB_REPO}"
        )
    return pin


def write_pin(path: Path, fixture: Fixture) -> None:
    path.write_text(json.dumps(fixture.pin(), indent=2) + "\n")


def describe(fixture: Fixture, now: datetime) -> str:
    days = (parse_time(fixture.expires_at, "fixture") - now) / timedelta(days=1)
    lines = [
        f"Run: {fixture.url}",
        f"Tag: {fixture.tag}",
        f"Head SHA: {fixture.head_sha}",
        f"Created: {fixture.created_at}",
        f"Artifacts expire: {fixture.expires_at} ({days:.1f} days from now)",
        "",
        "| Target | Artifact | Digest |",
        "|---|---|---|",
    ]
    lines += [f"| {t} | {n} | {d or '-'} |" for t, n, d in fixture.artifacts]
    return "\n".join(lines) + "\n"


def write_github_output(values: dict[str, str]) -> None:
    path = os.environ.get("GITHUB_OUTPUT")
    if not path:
        return
    with open(path, "a", encoding="utf-8") as out:
        for key, value in values.items():
            out.write(f"{key}={value}\n")


def write_summary(path: Path | None, text: str) -> None:
    targets = [p for p in (path, os.environ.get("GITHUB_STEP_SUMMARY")) if p]
    for target in targets:
        with open(target, "a", encoding="utf-8") as out:
            out.write(text)


def cmd_url(args: argparse.Namespace) -> int:
    pin = read_pin(args.pin_file)
    print(run_url(pin["repo"], pin["run_id"]))
    return 0


def cmd_check(args: argparse.Namespace) -> int:
    now = utcnow()
    if args.run_id is not None:
        repo, run_id = stage.GITHUB_REPO, args.run_id
    else:
        pin = read_pin(args.pin_file)
        repo, run_id = pin["repo"], pin["run_id"]
    fixture = check_run(repo, run_id, now, args.min_days)
    print(f"OK: fixture valid for at least {args.min_days} more days.")
    print(describe(fixture, now), end="")
    return 0


def cmd_refresh(args: argparse.Namespace) -> int:
    now = utcnow()
    pin = read_pin(args.pin_file)
    repo, current = pin["repo"], pin["run_id"]
    try:
        fixture = check_run(repo, current, now, args.min_days)
    except FixtureError as exc:
        print(f"Pinned run needs a refresh: {exc}")
    else:
        print(
            f"Pinned run {current} is valid for at least {args.min_days} more days; no change."
        )
        write_github_output({"changed": "false", "run_id": str(current)})
        return 0

    skipped: list[str] = []
    for run in list_successful_runs(repo, args.limit):
        if run.get("id") == current:
            continue  # Already failed above.
        try:
            fixture = verify(
                repo, run, list_artifacts(repo, run["id"]), now, args.min_days
            )
        except FixtureError as exc:
            skipped.append(str(exc))
            print(f"Skipped: {exc}")
            continue
        break
    else:
        raise FixtureError(
            f"none of the newest {args.limit} successful rust-release runs passes "
            f"the check ({len(skipped)} examined); the pin stays at run {current}"
        )

    write_pin(args.pin_file, fixture)
    print(f"Pin moved from run {current} to run {fixture.run_id}.")
    summary = (
        f"Moves the npm staging fixture (#2515) from run {run_url(repo, current)} "
        f"to the newest successful rust-release run whose six required target "
        f"artifacts are present and valid for at least {args.min_days} more days.\n\n"
        + describe(fixture, now)
        + "\nRepo-checks runs full npm staging on this PR because the pin file "
        "is a packaging input.\n"
    )
    print(summary, end="")
    write_summary(args.summary_file, summary)
    write_github_output(
        {
            "changed": "true",
            "run_id": str(fixture.run_id),
            "tag": fixture.tag,
            "expires_at": fixture.expires_at,
        }
    )
    return 0


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--pin-file", type=Path, default=PIN_FILE)
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("url", help="Print the pinned run URL.").set_defaults(func=cmd_url)

    check = sub.add_parser("check", help="Verify the pinned (or a given) run.")
    check.add_argument("--run-id", type=int, help="Check this run instead of the pin.")
    check.add_argument("--min-days", type=int, default=DEFAULT_MIN_DAYS)
    check.set_defaults(func=cmd_check)

    refresh = sub.add_parser("refresh", help="Move the pin if it fails the check.")
    refresh.add_argument("--min-days", type=int, default=DEFAULT_MIN_DAYS)
    refresh.add_argument("--limit", type=int, default=DEFAULT_LIMIT)
    refresh.add_argument("--summary-file", type=Path)
    refresh.set_defaults(func=cmd_refresh)

    args = parser.parse_args(argv)
    if getattr(args, "min_days", 0) < 0:
        parser.error("--min-days must not be negative")
    if not 1 <= getattr(args, "limit", 1) <= MAX_LIMIT:
        parser.error(f"--limit must be between 1 and {MAX_LIMIT}")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        return args.func(args)
    except FixtureError as exc:
        print(f"npm staging fixture: {exc}", file=sys.stderr)
        return 1
    except subprocess.CalledProcessError as exc:
        print(f"npm staging fixture: GitHub API call failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
