#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


DEFAULT_MAX_BYTES = 500 * 1024


@dataclass(frozen=True)
class ChangedBlob:
    path: str
    size_bytes: int
    base_size_bytes: int | None
    is_allowlisted: bool
    is_binary: bool


def run_git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def load_allowlist(path: Path) -> set[str]:
    allowlist: set[str] = set()
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if line:
            allowlist.add(line)
    return allowlist


def get_changed_paths(base: str, head: str) -> list[str]:
    output = run_git(
        "diff",
        "--name-only",
        "--diff-filter=AM",
        "--no-renames",
        "-z",
        base,
        head,
    )
    return [path for path in output.split("\0") if path]


def is_binary_change(base: str, head: str, path: str) -> bool:
    output = run_git(
        "diff",
        "--numstat",
        "--diff-filter=AM",
        "--no-renames",
        base,
        head,
        "--",
        path,
    ).strip()
    if not output:
        return False

    added, deleted, _ = output.split("\t", 2)
    return added == "-" and deleted == "-"


def blob_size(commit: str, path: str) -> int:
    return int(run_git("cat-file", "-s", f"{commit}:{path}").strip())


def base_blob_size(base: str, path: str) -> int | None:
    result = subprocess.run(
        ["git", "cat-file", "-s", f"{base}:{path}"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return None
    return int(result.stdout.strip())


def collect_changed_blobs(
    base: str, head: str, allowlist: set[str]
) -> list[ChangedBlob]:
    blobs: list[ChangedBlob] = []
    for path in get_changed_paths(base, head):
        blobs.append(
            ChangedBlob(
                path=path,
                size_bytes=blob_size(head, path),
                base_size_bytes=base_blob_size(base, path),
                is_allowlisted=path in allowlist,
                is_binary=is_binary_change(base, head, path),
            )
        )
    return blobs


def format_kib(size_bytes: int) -> str:
    return f"{size_bytes / 1024:.1f} KiB"


def violates_size_policy(blob: ChangedBlob, max_bytes: int) -> bool:
    if blob.is_allowlisted or blob.size_bytes <= max_bytes:
        return False

    # Existing oversized blobs are migration debt, but changing an unrelated line in one
    # must not force a risky refactor into the same pull request. Grandfather only blobs
    # that were already oversized and did not grow; new or growing oversized blobs fail.
    return (
        blob.base_size_bytes is None
        or blob.base_size_bytes <= max_bytes
        or blob.size_bytes > blob.base_size_bytes
    )


def blob_status(blob: ChangedBlob, max_bytes: int, is_violation: bool) -> str:
    if is_violation:
        return "blocked"
    if blob.is_allowlisted:
        return "allowlisted"
    if blob.size_bytes > max_bytes:
        return "grandfathered (not grown)"
    return "ok"


def write_step_summary(
    max_bytes: int,
    blobs: list[ChangedBlob],
    violations: list[ChangedBlob],
) -> None:
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return

    lines = [
        "## Blob Size Policy",
        "",
        f"Default max: `{max_bytes}` bytes ({format_kib(max_bytes)})",
        f"Changed files checked: `{len(blobs)}`",
        f"Violations: `{len(violations)}`",
        "",
    ]

    if blobs:
        lines.extend(
            [
                "| Path | Kind | Size | Status |",
                "| --- | --- | ---: | --- |",
            ]
        )
        for blob in blobs:
            status = blob_status(blob, max_bytes, blob in violations)
            kind = "binary" if blob.is_binary else "non-binary"
            lines.append(
                f"| `{blob.path}` | {kind} | `{blob.size_bytes}` bytes ({format_kib(blob.size_bytes)}) | {status} |"
            )
    else:
        lines.append("No changed files were detected.")

    lines.append("")
    Path(summary_path).write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fail if changed blobs exceed the configured size budget."
    )
    parser.add_argument(
        "--base", required=True, help="Base git revision to diff against."
    )
    parser.add_argument("--head", required=True, help="Head git revision to inspect.")
    parser.add_argument(
        "--max-bytes",
        type=int,
        default=DEFAULT_MAX_BYTES,
        help=f"Maximum allowed blob size in bytes. Default: {DEFAULT_MAX_BYTES}.",
    )
    parser.add_argument(
        "--allowlist",
        type=Path,
        required=True,
        help="Path to the newline-delimited allowlist file.",
    )
    args = parser.parse_args()

    allowlist = load_allowlist(args.allowlist)
    blobs = collect_changed_blobs(args.base, args.head, allowlist)
    violations = [blob for blob in blobs if violates_size_policy(blob, args.max_bytes)]

    write_step_summary(args.max_bytes, blobs, violations)

    if not blobs:
        print("No changed files were detected.")
        return 0

    print(
        f"Checked {len(blobs)} changed file(s) against the {args.max_bytes}-byte limit."
    )
    for blob in blobs:
        status = blob_status(blob, args.max_bytes, blob in violations)
        kind = "binary" if blob.is_binary else "non-binary"
        print(
            f"- {blob.path}: {blob.size_bytes} bytes ({format_kib(blob.size_bytes)}) [{kind}, {status}]"
        )

    if violations:
        print("\nFile(s) exceed the configured limit:")
        for blob in violations:
            print(f"- {blob.path}: {blob.size_bytes} bytes > {args.max_bytes} bytes")
        print(
            "\nIf one of these is a real checked-in asset we want to keep, add its "
            "repo-relative path to .github/blob-size-allowlist.txt. Otherwise, "
            "shrink it or keep it out of git."
        )
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
