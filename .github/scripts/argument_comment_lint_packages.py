#!/usr/bin/env python3

"""List the codex-rs packages a PR's argument-comment lint should check.

PR CI lints only the workspace crates whose Rust sources changed, test code
included (the nightly Bazel lint still covers everything). The crates are the
ones Cargo sees (`cargo_packages`, shared with the changed-crate test job),
not just the `members` list. A change to the lint job itself adds a small
canary crate so the job exercises its own setup.
Prints one package name per line.
"""

import argparse
import os
import subprocess
import tomllib
from pathlib import Path, PurePosixPath


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE = "codex-rs"
CANARY_PACKAGE = "codex-utils-string"
# Editing any of these lints the canary so the PR proves the job still works.
LINT_JOB_PATHS = (
    ".github/actions/setup-ci/",
    ".github/actions/setup-rusty-v8/",
    ".github/scripts/argument_comment_lint_packages.py",
    ".github/workflows/rust-ci.yml",
    "tools/argument-comment-lint/",
)


def workspace_packages(root: Path) -> dict[str, str]:
    """Map each workspace member directory (relative to the repo) to its package."""
    manifest = tomllib.loads((root / WORKSPACE / "Cargo.toml").read_text())
    packages = {}
    for member in manifest["workspace"]["members"]:
        member_manifest = tomllib.loads(
            (root / WORKSPACE / member / "Cargo.toml").read_text()
        )
        packages[f"{WORKSPACE}/{member}"] = member_manifest["package"]["name"]
    return packages


DEPENDENCY_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")


def path_dependencies(table: dict, base: str, shared: dict[str, str]) -> list[str]:
    """The directories (relative to the workspace) of the crates a manifest
    in `base` depends on by path, directly or through `workspace = true` on a
    `shared` (`[workspace.dependencies]`) path entry; target tables too."""
    found = []
    for name in DEPENDENCY_TABLES:
        for key, dependency in table.get(name, {}).items():
            if not isinstance(dependency, dict):
                continue
            if "path" in dependency:
                found.append(os.path.normpath(os.path.join(base, dependency["path"])))
            elif dependency.get("workspace") is True and key in shared:
                found.append(shared[key])
    for target in table.get("target", {}).values():
        found.extend(path_dependencies(target, base, shared))
    return found


def cargo_packages(root: Path) -> dict[str, str]:
    """The workspace members plus the crates Cargo makes members because a
    member depends on them by path (#2027: missing these, the changed-crate
    test job never ran codex-windows-sandbox's tests, and #2011 broke two of
    them; the lint job never linted codex-chatgpt), and theirs in
    turn, unless outside the workspace or excluded. An unused
    `[workspace.dependencies]` path entry is not a member."""
    packages = workspace_packages(root)
    workspace = tomllib.loads((root / WORKSPACE / "Cargo.toml").read_text())[
        "workspace"
    ]
    shared = {
        key: os.path.normpath(dependency["path"])
        for key, dependency in workspace.get("dependencies", {}).items()
        if isinstance(dependency, dict) and "path" in dependency
    }
    excluded = [os.path.normpath(path) for path in workspace.get("exclude", [])]
    pending = []
    for directory in packages:
        member = tomllib.loads((root / directory / "Cargo.toml").read_text())
        base = os.path.relpath(directory, WORKSPACE)
        pending.extend(path_dependencies(member, base, shared))
    while pending:
        crate = pending.pop()
        # Outside the workspace, or the virtual root manifest itself.
        if os.path.isabs(crate) or crate in (".", "..") or crate.startswith("../"):
            continue
        if any(crate == ex or crate.startswith(ex + "/") for ex in excluded):
            continue
        directory = f"{WORKSPACE}/{crate}"
        if directory in packages:
            continue
        manifest = tomllib.loads((root / directory / "Cargo.toml").read_text())
        packages[directory] = manifest["package"]["name"]
        pending.extend(path_dependencies(manifest, crate, shared))
    return packages


def packages_for(path: str, packages: dict[str, str]) -> list[str]:
    """Return every package owning `path`. A crate nested in another
    (`core/tests/common`) selects its parent too: the parent's tests are
    where its functions are called, and so where argument comments are."""
    return [
        package
        for parent in PurePosixPath(path).parents
        if (package := packages.get(str(parent))) is not None
    ]


def lint_packages(files: list[str], packages: dict[str, str]) -> list[str]:
    selected = set()
    for path in files:
        if path.endswith(".rs"):
            selected.update(packages_for(path, packages))
        if path.startswith(LINT_JOB_PATHS):
            selected.add(CANARY_PACKAGE)
    return sorted(selected)


def changed_files(base: str, head: str) -> list[str]:
    output = subprocess.run(
        # Three dots: only what the PR changed since it left `base`.
        [
            "git",
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            f"{base}...{head}",
        ],
        capture_output=True,
        check=True,
        cwd=ROOT,
    ).stdout
    # NUL records avoid Git's path quoting and preserve embedded newlines.
    return [os.fsdecode(path) for path in output.split(b"\0") if path]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", help="base commit of the PR")
    parser.add_argument("head", help="head commit of the PR")
    args = parser.parse_args()
    for package in lint_packages(
        changed_files(args.base, args.head), cargo_packages(ROOT)
    ):
        print(package)


if __name__ == "__main__":
    main()
