#!/usr/bin/env python3
"""#1989 (owner decision D503): helpers for rust-ci-changed-crate-tests-macos.yml.

The Linux `Changed crate tests` job (#1984) runs a PR's changed crates on
Linux only. This non-blocking twin runs a few crates with macOS-only code
paths on macOS, so a macOS-only break shows up on the PR instead of nowhere:
no PR, postmerge or nightly job ran these crates' tests on macOS before.

Usage:
  macos_changed_crate_tests.py packages <base> <head>
      `packages=<JSON array>`: the MACOS_PACKAGES the PR changes (the same
      selection as the Linux job, via the shared `cargo_packages` list), or
      all of them when it changes one of this job's own files.
  macos_changed_crate_tests.py packages --all
      Every package (manual runs).
  macos_changed_crate_tests.py filter|summary <cargo package>
      The nextest filter excluding the package's known macOS failures
      (.github/macos-changed-crate-known-failures.txt), or the job summary.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from argument_comment_lint_packages import ROOT  # noqa: E402
from argument_comment_lint_packages import cargo_packages  # noqa: E402
from argument_comment_lint_packages import changed_files  # noqa: E402
from changed_crate_test_packages import selected_packages  # noqa: E402
from windows_integration_known_failures import KnownFailure  # noqa: E402
from windows_integration_known_failures import load  # noqa: E402
from windows_integration_known_failures import nextest_filter  # noqa: E402

LIST = ROOT / ".github" / "macos-changed-crate-known-failures.txt"
WORKFLOW = ".github/workflows/rust-ci-changed-crate-tests-macos.yml"
# Crates with macOS-only code paths that no job tested on macOS (#1989):
# Seatbelt policies (sandboxing), the macOS process and file-system sandboxes
# (exec-server), Keychain access (keyring-store), dyld/ptrace hardening
# (process-hardening), and native certificates plus the credential broker's
# Mach memory scan (network-proxy). Each took 1.6-7 minutes in the trial (#2068).
# codex-login (14 minutes: its tests build codex-core) and codex-vault (no
# macOS-only code) passed there but are left out.
MACOS_PACKAGES = (
    "codex-exec-server",
    "codex-keyring-store",
    "codex-network-proxy",
    "codex-process-hardening",
    "codex-sandboxing",
)
# Editing any of these (a directory: anything under it) runs every package, so
# the PR proves the job still works.
JOB_FILES = (
    WORKFLOW,
    ".github/actions/check-clean-worktree/",
    ".github/actions/setup-ci/",
    ".github/macos-changed-crate-known-failures.txt",
    ".github/scripts/argument_comment_lint_packages.py",
    ".github/scripts/changed_crate_test_packages.py",
    ".github/scripts/macos_changed_crate_tests.py",
    ".github/scripts/windows_integration_known_failures.py",
    "codex-rs/.config/nextest.toml",
    "codex-rs/Cargo.toml",
    "codex-rs/rust-toolchain.toml",
)


def changed_packages(files: list[str], packages: dict[str, str]) -> list[str]:
    if any(path.startswith(JOB_FILES) for path in files):
        return list(MACOS_PACKAGES)
    selected = selected_packages(files, packages)
    return [package for package in MACOS_PACKAGES if package in selected]


def summary(entries: list[KnownFailure], package: str) -> str:
    excluded = [entry for entry in entries if entry.package == package]
    lines = [
        f"### {package}: tests on macOS (#1989)",
        "",
        "Non-blocking: blocking-ci.yml does not call this workflow, so `CI required`"
        " neither waits for nor depends on it.",
    ]
    if excluded:
        lines += [
            "",
            f"Known macOS failures excluded ({len(excluded)}, "
            "`.github/macos-changed-crate-known-failures.txt`):",
            "",
        ]
        lines += [f"- `{entry.test}` (#{entry.issue})" for entry in excluded]
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    if argv[1:] == ["packages", "--all"]:
        selected = list(MACOS_PACKAGES)
    elif len(argv) == 4 and argv[1] == "packages":
        selected = changed_packages(changed_files(argv[2], argv[3]), cargo_packages(ROOT))
    elif len(argv) == 3 and argv[1] in ("filter", "summary"):
        entries = load(LIST)
        if argv[1] == "filter":
            sys.stdout.write(nextest_filter(entries, argv[2]))
        else:
            sys.stdout.write(summary(entries, argv[2]))
        return 0
    else:
        print(__doc__, file=sys.stderr)
        return 2
    sys.stdout.write(f"packages={json.dumps(selected)}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
