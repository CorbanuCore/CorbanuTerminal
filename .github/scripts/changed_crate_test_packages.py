#!/usr/bin/env python3

"""List the codex-rs packages whose tests PR CI runs on Linux (#1972).

Before this, no Linux or macOS PR job ran most crates' tests: the full Linux
suite runs only postmerge, so a PR that broke its own crate's tests (#1963
broke two codex-rmcp-client tests) went green. A package is selected when the
PR changes any file under it (sources, tests, fixtures, snapshots or its
Cargo.toml); a crate nested in another (`exec-server/tests/support`) selects
both. Crates depending on a changed crate are not added, and a change to
Cargo.lock alone selects nothing; postmerge covers those.

A change to this job itself selects CANARY_PACKAGE, which also needs a helper
binary from another crate, so the PR proves the whole path still works.

Prints one package name per line, or with --helpers the extra `cargo build`
arguments for helper binaries the given packages' tests need.
"""

import argparse
import os
import sys
import tomllib
from pathlib import Path
from pathlib import PurePosixPath

sys.path.insert(0, str(Path(__file__).resolve().parent))

from argument_comment_lint_packages import ROOT  # noqa: E402
from argument_comment_lint_packages import WORKSPACE  # noqa: E402
from argument_comment_lint_packages import changed_files  # noqa: E402
from argument_comment_lint_packages import workspace_packages  # noqa: E402

CANARY_PACKAGE = "codex-rmcp-client"
# Editing any of these runs the canary's tests.
JOB_PATHS = (
    ".github/actions/check-clean-worktree/",
    ".github/actions/setup-ci/",
    ".github/scripts/argument_comment_lint_packages.py",
    ".github/scripts/changed_crate_test_packages.py",
    ".github/workflows/rust-ci-changed-crate-tests.yml",
    ".github/workflows/rust-ci.yml",
    "codex-rs/.config/nextest.toml",
    "codex-rs/Cargo.toml",
    "codex-rs/rust-toolchain.toml",
    "scripts/codex_package/rg",
)
# Binaries from other packages that a package's tests start through
# `codex_utils_cargo_bin::cargo_bin` (Bazel's `extra_binaries`). Cargo builds
# only the tested packages' own binaries.
HELPER_BINARIES = {
    "codex-app-server": (
        ("codex-cli", "codex"),
        ("codex-code-mode-host", "codex-code-mode-host"),
        ("codex-rmcp-client", "test_stdio_server"),
    ),
    "codex-core": (
        ("codex-cli", "codex"),
        ("codex-code-mode-host", "codex-code-mode-host"),
        ("codex-linux-sandbox", "codex-linux-sandbox"),
        ("codex-rmcp-client", "test_stdio_server"),
        ("codex-rmcp-client", "test_streamable_http_server"),
    ),
    "codex-rmcp-client": (("codex-cli", "codex"),),
    "codex-tui": (
        ("codex-cli", "codex"),
        # The containment tests (#743) skip without it.
        ("codex-linux-sandbox", "codex-linux-sandbox"),
        ("codex-wallet-daemon", "corbanu-walletd"),
    ),
}


DEPENDENCY_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")


def path_dependencies(table: dict) -> list[str]:
    """The `path`s of a manifest's dependency tables (target tables too)."""
    found = []
    for name in DEPENDENCY_TABLES:
        for dependency in table.get(name, {}).values():
            if isinstance(dependency, dict) and "path" in dependency:
                found.append(dependency["path"])
    for target in table.get("target", {}).values():
        found.extend(path_dependencies(target))
    return found


def tested_packages(root: Path) -> dict[str, str]:
    """The workspace members plus the crates Cargo makes members because a
    member depends on them by path (#2027: missing these, the job never ran
    codex-windows-sandbox's tests, and #2011 broke two of them): path
    dependencies inside the workspace and not excluded, in `[workspace.
    dependencies]` or a member's own tables, and theirs in turn."""
    packages = workspace_packages(root)
    manifest = tomllib.loads((root / WORKSPACE / "Cargo.toml").read_text())
    workspace = manifest["workspace"]
    excluded = [os.path.normpath(path) for path in workspace.get("exclude", [])]
    # (directory relative to the workspace, dependency paths relative to it)
    pending = [(".", path_dependencies(workspace))]
    for directory in packages:
        member = tomllib.loads((root / directory / "Cargo.toml").read_text())
        pending.append(
            (os.path.relpath(directory, WORKSPACE), path_dependencies(member))
        )
    while pending:
        base, paths = pending.pop()
        for path in paths:
            crate = os.path.normpath(os.path.join(base, path))
            # Outside the workspace, or the virtual root manifest itself.
            if os.path.isabs(crate) or crate in (".", "..") or crate.startswith("../"):
                continue
            if any(crate == ex or crate.startswith(ex + "/") for ex in excluded):
                continue
            directory = f"{WORKSPACE}/{crate}"
            if directory in packages:
                continue
            member = tomllib.loads((root / directory / "Cargo.toml").read_text())
            packages[directory] = member["package"]["name"]
            pending.append((crate, path_dependencies(member)))
    return packages


def selected_packages(files: list[str], packages: dict[str, str]) -> list[str]:
    selected = set()
    for path in files:
        for parent in PurePosixPath(path).parents:
            package = packages.get(str(parent))
            if package is not None:
                selected.add(package)
        if path.startswith(JOB_PATHS):
            selected.add(CANARY_PACKAGE)
    return sorted(selected)


def helper_build_args(selected: list[str]) -> list[str]:
    helpers = sorted(
        {helper for package in selected for helper in HELPER_BINARIES.get(package, ())}
    )
    return [
        arg for package, binary in helpers for arg in ("-p", package, "--bin", binary)
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--helpers",
        nargs="*",
        metavar="PACKAGE",
        help="print the helper `cargo build` arguments for these packages",
    )
    parser.add_argument("base", nargs="?", help="base commit of the PR")
    parser.add_argument("head", nargs="?", help="head commit of the PR")
    args = parser.parse_args()
    if args.helpers is not None:
        print(" ".join(helper_build_args(args.helpers)))
        return
    if args.base is None or args.head is None:
        parser.error("base and head are required")
    selected = selected_packages(
        changed_files(args.base, args.head), tested_packages(ROOT)
    )
    for package in selected:
        print(package)


if __name__ == "__main__":
    main()
