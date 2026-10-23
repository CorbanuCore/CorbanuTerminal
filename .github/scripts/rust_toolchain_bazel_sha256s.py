#!/usr/bin/env python3
"""Pin the Rust toolchain archives that Bazel downloads through rules_rust.

`MODULE.bazel` declares Rust toolchains through the rules_rust extension that
rules_rs re-exports (the argument-comment-lint nightly and the native Windows
gnullvm sets). rules_rust checks each archive it downloads against its
`sha256s` attribute, then its built-in `known_shas.bzl`; when neither has the
archive it downloads it from static.rust-lang.org with no checksum (#1032).
That table has no gnullvm entries and lags new releases, so this script pins
every archive explicitly in the `RUST_TOOLCHAIN_SHA256S` block of
`MODULE.bazel`.

It asks Bazel for the toolchain repositories the extension defines
(`bazel mod show_extension` / `show_repo`), works out the archives each one
downloads (the same names rules_rust uses), and takes their sha256 from the
Rust release manifests (`channel-rust-*.toml`) on static.rust-lang.org.

  python3 .github/scripts/rust_toolchain_bazel_sha256s.py           # rewrite the block
  python3 .github/scripts/rust_toolchain_bazel_sha256s.py --check   # exit 1 if stale

Run it (without --check) after changing any Rust version in MODULE.bazel or
the rules_rs version (rules_rust's default rustfmt nightly is pinned too). The
rust-analyzer repository (IDE only; its archives depend on the host) is not
pinned. Needs Python 3.11+ and bazel.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    sys.exit("rust_toolchain_bazel_sha256s.py needs Python 3.11+ (tomllib)")

REPO_ROOT = Path(__file__).resolve().parents[2]
MODULE = REPO_ROOT / "MODULE.bazel"
EXTENSION = "@rules_rs//rs:rules_rust_reexported_extensions.bzl%rust"
REPO_PREFIX = "@@rules_rs++rust+"
DIST = "https://static.rust-lang.org/dist/"
URL_TEMPLATE = DIST + "{}.tar.xz"
BEGIN = "# BEGIN RUST_TOOLCHAIN_SHA256S"
END = "# END RUST_TOOLCHAIN_SHA256S"


def bazel(*args: str) -> str:
    # `--lockfile_mode=error`: never rewrite a stale MODULE.bazel.lock (the
    # default `update` would, hiding it from the lockfile check). Bazel's
    # stderr goes to ours, so a failure shows why.
    command, *rest = args
    return subprocess.run(
        [
            os.environ.get("CODEX_BAZEL_BIN", "bazel"),
            command,
            *rest,
            "--lockfile_mode=error",
        ],
        cwd=REPO_ROOT,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout


def toolchain_repos() -> list[dict[str, object]]:
    """The tools repositories the extension defines, with their attributes."""
    listing = bazel("mod", "show_extension", EXTENSION)
    names = sorted(set(re.findall(r"^\s+- (\S+_tools)$", listing, re.MULTILINE)))
    if not names:
        sys.exit(f"no toolchain repositories in `bazel mod show_extension {EXTENSION}`")
    shown = bazel("mod", "show_repo", *(REPO_PREFIX + name for name in names))
    repos = []
    for block in re.split(r"^## ", shown, flags=re.MULTILINE)[1:]:
        rule = re.search(r"^(\w+)\($", block, re.MULTILINE)
        attrs: dict[str, object] = dict(
            re.findall(r'^\s+(\w+) = "([^"]*)",$', block, re.MULTILINE)
        )
        attrs["dev_components"] = (
            re.search(r"^\s+dev_components = True,$", block, re.MULTILINE) is not None
        )
        urls = re.search(r"^\s+urls = \[(.*)\],$", block, re.MULTILINE)
        if urls and urls.group(1) != f'"{URL_TEMPLATE}"':
            sys.exit(
                f"{attrs.get('name')}: unexpected urls {urls.group(1)}; update this script"
            )
        attrs["rule"] = rule.group(1) if rule else ""
        repos.append(attrs)
    if len(repos) != len(names):
        sys.exit(
            f"parsed {len(repos)} of {len(names)} repositories from `bazel mod show_repo`"
        )
    return repos


def archive(tool: str, version: str, triple: str) -> str:
    """The archive key rules_rust looks up (`produce_tool_suburl` + extension)."""
    channel, _, iso_date = version.partition("/")
    if channel in ("nightly", "beta"):
        return f"{iso_date}/{tool}-{channel}-{triple}.tar.xz"
    return f"{tool}-{version}-{triple}.tar.xz"


def archives(repo: dict[str, object]) -> set[str]:
    """The archives one tools repository downloads (see rules_rust's
    private/repositories.bzl `*_tools_repository_impl`)."""
    rule = repo["rule"]
    exec_triple = str(repo.get("exec_triple", ""))
    version = str(repo.get("version", ""))
    if rule == "rust_toolchain_tools_repository":
        target = str(repo["target_triple"])
        # rules_rust skips llvm-tools only for releases older than 1.45 or
        # nightly 2020-05-22, which this script does not support.
        keys = {
            archive(tool, version, exec_triple)
            for tool in ("rustc", "clippy", "cargo", "llvm-tools")
        }
        keys.add(archive("rust-std", version, target))
        rustfmt_version = str(repo.get("rustfmt_version", ""))
        if rustfmt_version in ("nightly", "beta"):
            # A bare channel takes the toolchain's own date.
            rustfmt_version += "/" + version.partition("/")[2]
        if rustfmt_version:
            keys.add(archive("rustfmt", rustfmt_version, exec_triple))
        if repo["dev_components"]:
            keys.add(archive("rustc-dev", version, target))
        return keys
    if rule == "rustfmt_toolchain_tools_repository":
        return {
            archive("rustc", version, exec_triple),
            archive("rustfmt", version, exec_triple),
        }
    if rule == "rust_analyzer_toolchain_tools_repository":
        return set()
    sys.exit(f"{repo.get('name')}: unknown rule {rule}; update this script")


def release(key: str) -> str:
    """The release (`nightly/DATE` or `X.Y.Z`) an archive key belongs to."""
    if "/" in key:
        iso_date, name = key.split("/", 1)
        return ("beta/" if "-beta-" in name else "nightly/") + iso_date
    match = re.search(r"-(\d+\.\d+\.\d+)-", key)
    if not match:
        sys.exit(f"cannot tell the release of {key}")
    return match.group(1)


def download(url: str) -> bytes:
    for attempt in range(1, 4):
        try:
            with urllib.request.urlopen(url, timeout=60) as response:
                return response.read()
        except OSError as error:
            if attempt == 3:
                sys.exit(f"cannot download {url}: {error}")
            print(f"retrying {url} after {error}", file=sys.stderr)
            time.sleep(5 * attempt)
    raise AssertionError("unreachable")


def manifest_hashes(version: str) -> dict[str, str]:
    """Archive key -> sha256 from the release manifest of `version`.

    The manifest comes over HTTPS from static.rust-lang.org and is checked
    against its published `.sha256`, as rustup does by default; its GPG
    signature is not checked. Committing the result means later downloads
    must match what was seen here.
    """
    channel, _, iso_date = version.partition("/")
    nightly = channel in ("nightly", "beta")
    url = (
        f"{DIST}{iso_date}/channel-rust-{channel}.toml"
        if nightly
        else f"{DIST}channel-rust-{version}.toml"
    )
    data = download(url)
    published = download(url + ".sha256").decode().split()[0]
    if hashlib.sha256(data).hexdigest() != published:
        sys.exit(f"{url} does not match its .sha256")
    manifest = tomllib.loads(data.decode())
    hashes = {}
    for package in manifest["pkg"].values():
        for target in package.get("target", {}).values():
            xz_url = target.get("xz_url")
            if not target.get("available") or not xz_url or not xz_url.startswith(DIST):
                continue
            path = xz_url.removeprefix(DIST)
            # Stable archives sit under a date directory but are keyed by file
            # name; nightly and beta keys keep the date.
            hashes[path if nightly else path.rsplit("/", 1)[-1]] = target["xz_hash"]
    return hashes


def expected() -> tuple[dict[str, str], list[str]]:
    needed: dict[str, set[str]] = {}
    for repo in toolchain_repos():
        for key in archives(repo):
            needed.setdefault(key, set()).add(
                str(repo.get("_original_name", repo.get("name")))
            )
    published: dict[str, str] = {}
    for version in sorted({release(key) for key in needed}):
        published.update(manifest_hashes(version))
    pins = {key: published[key] for key in sorted(needed) if key in published}
    # e.g. rustc-dev for wasm targets: rules_rust would fail to fetch it, and
    # only if a build used that toolchain.
    unpublished = [
        f"{key} ({len(needed[key])} repositories)"
        for key in sorted(needed)
        if key not in published
    ]
    return pins, unpublished


def render(pins: dict[str, str]) -> str:
    lines = [BEGIN, "RUST_TOOLCHAIN_SHA256S = {"]
    lines += [f'    "{key}": "{sha}",' for key, sha in pins.items()]
    lines += ["}", END]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--check", action="store_true", help="exit 1 if the block is stale"
    )
    args = parser.parse_args()

    text = MODULE.read_text()
    block = re.search(re.escape(BEGIN) + r".*?" + re.escape(END), text, re.DOTALL)
    if not block:
        sys.exit(f"MODULE.bazel has no {BEGIN} ... {END} block")
    pins, unpublished = expected()
    for entry in unpublished:
        print(f"not published, not pinned: {entry}", file=sys.stderr)
    wanted = render(pins)
    if block.group(0) == wanted:
        print(f"RUST_TOOLCHAIN_SHA256S is up to date ({len(pins)} archives).")
        return 0
    if args.check:
        print(
            "RUST_TOOLCHAIN_SHA256S is stale: run .github/scripts/rust_toolchain_bazel_sha256s.py"
        )
        return 1
    MODULE.write_text(text[: block.start()] + wanted + text[block.end() :])
    print(f"Updated RUST_TOOLCHAIN_SHA256S ({len(pins)} archives).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
