#!/usr/bin/env python3

"""List the codex-rs packages whose tests PR CI runs on Linux (#1972).

Before this, no Linux or macOS PR job ran most crates' tests: the full Linux
suite runs only postmerge, so a PR that broke its own crate's tests (#1963
broke two codex-rmcp-client tests) went green. A package is selected when the
PR changes any file under it (sources, tests, fixtures, snapshots or its
Cargo.toml); a crate nested in another (`exec-server/tests/support`) selects
both. Cargo metadata supplies all members (including test-support crates)
and local dependency edges. Linux adds transitive reverse dependents, including
dev/build and target-specific dependencies. Above MAX_PACKAGES it runs every
member instead of truncating coverage; workspace-wide inputs do the same.

A change to this job itself seeds CANARY_PACKAGE and expands its dependents.
That currently exceeds the cap and runs every member, proving the fallback
and helper-binary path on workflow changes too.

Prints one package name per line, or with --helpers the extra `cargo build`
arguments for helper binaries the given packages' tests need.
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from pathlib import PurePosixPath

sys.path.insert(0, str(Path(__file__).resolve().parent))

from argument_comment_lint_packages import ROOT  # noqa: E402
from argument_comment_lint_packages import cargo_packages as tested_packages  # noqa: E402
from argument_comment_lint_packages import changed_files  # noqa: E402

CANARY_PACKAGE = "codex-rmcp-client"
# Keep the graph walk bounded, never the coverage: broad changes use the full
# suite in the same job, with its ci-test profile, cache and 85-minute timeout.
MAX_PACKAGES = 32
FULL_SUITE_PATHS = (
    "codex-rs/Cargo.lock",
    "codex-rs/Cargo.toml",
    "codex-rs/rust-toolchain.toml",
    "codex-rs/.cargo/",
    "codex-rs/.config/nextest.toml",
)
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


def selected_packages(files: list[str], packages: dict[str, str]) -> list[str]:
    selected = set()
    for path in files:
        for parent in PurePosixPath(path).parents:
            package = packages.get(str(parent))
            if package is not None:
                selected.add(package)
        # The local DNS adapter is outside Cargo workspace package directories.
        if path.startswith("third_party/rama-dns/"):
            selected.add("codex-network-proxy")
        if path.startswith(JOB_PATHS):
            selected.add(CANARY_PACKAGE)
    return sorted(selected)


def cargo_metadata(root: Path) -> dict:
    # --no-deps needs no registry downloads and still includes each member's
    # declared dependencies, including optional, renamed and target-specific
    # path dependencies. Do not filter to the selector runner's platform.
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=root / "codex-rs",
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def affected_packages(
    files: list[str], metadata: dict, root: Path, limit: int = MAX_PACKAGES
) -> list[str]:
    members = set(metadata["workspace_members"])
    packages = [p for p in metadata["packages"] if p["id"] in members]
    by_path = {Path(p["manifest_path"]).parent.resolve(): p["name"] for p in packages}
    directories = {
        path.relative_to(root.resolve()).as_posix(): name
        for path, name in by_path.items()
    }
    all_packages = sorted(by_path.values())

    def full_suite(reason: str) -> list[str]:
        print(f"Full suite ({len(all_packages)} packages): {reason}", file=sys.stderr)
        return all_packages

    if any(path.startswith(FULL_SUITE_PATHS) for path in files):
        return full_suite(
            "workspace manifest, lockfile, toolchain or configuration changed"
        )
    selected = set(selected_packages(files, directories))
    # A deleted/moved crate no longer has metadata at HEAD. Its former
    # dependents cannot be recovered from that graph: conservatively test all.
    if any(
        path.startswith("codex-rs/")
        and path.endswith("/Cargo.toml")
        and str(PurePosixPath(path).parent) not in directories
        for path in files
    ):
        return full_suite("removed or unrecognised crate manifest")
    if not selected:
        return []

    reverse = {name: set() for name in all_packages}
    for package in packages:
        for dependency in package["dependencies"]:
            path = dependency.get("path")
            if path is not None and (name := by_path.get(Path(path).resolve())):
                reverse[name].add(package["name"])
    pending = list(selected)
    while pending:
        if len(selected) > limit:
            return full_suite(f"more than {limit} affected packages")
        for dependent in reverse[pending.pop()] - selected:
            selected.add(dependent)
            pending.append(dependent)
    print(f"Affected packages: {len(selected)} (cap {limit})", file=sys.stderr)
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
    selected = affected_packages(
        changed_files(args.base, args.head), cargo_metadata(ROOT), ROOT
    )
    for package in selected:
        print(package)


if __name__ == "__main__":
    main()
