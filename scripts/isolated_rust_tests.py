#!/usr/bin/env python3
"""Ordinary debug tests: disposable profile and no native credential-store calls.

This is test hygiene, not an OS sandbox for hostile tests or release binaries.
See docs/development/test-isolation.md for the separate native qualification lane.
"""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def test_environment(parent: dict[str, str], home: Path) -> dict[str, str]:
    env = parent.copy()
    for name in (
        "CORBANU_HOME",
        "PFTERMINAL_HOME",
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "CLAUDE_CODE_OAUTH_TOKEN",
        "CODEX_APP_SERVER_MANAGED_CONFIG_PATH",
    ):
        env.pop(name, None)
    # Leave the higher-priority aliases unset: existing test fixtures override
    # CODEX_HOME themselves. Setting all three here would shadow those fixtures.
    env["CODEX_HOME"] = str(home)
    env["CORBANU_TEST_NO_NATIVE_KEYRING"] = "1"
    env["CARGO_PROFILE_DEV_DEBUG_ASSERTIONS"] = "true"
    env["CARGO_PROFILE_TEST_DEBUG_ASSERTIONS"] = "true"
    env["RUST_MIN_STACK"] = "8388608"
    env["NEXTEST_PROFILE"] = "local"
    return env


def validate_debug_run(args: list[str], env: dict[str, str]) -> None:
    # The existing native-store guard is deliberately absent in release builds.
    # Do not silently offer a no-Keychain promise for a different build lane.
    for arg in args:
        # nextest accepts short-option clusters such as -vr and -rj4. Do not
        # mistake the 'r' inside an attached package/filter value for a flag.
        release_short_option = (
            arg.startswith("-")
            and not arg.startswith(("--", "-p", "-E", "-j", "-F"))
            and "r" in arg[1:]
        )
        if release_short_option or arg.split("=", 1)[0] in (
            "--release",
            "--cargo-profile",
            "--config",
            "--archive-file",
            "--binaries-metadata",
            "--cargo-metadata",
        ):
            raise ValueError(
                "Custom/reused builds require the isolated native qualification lane"
            )
    for name, value in env.items():
        if "RUSTFLAGS" in name and "debug-assertions" in value:
            raise ValueError(
                "Remove debug-assertions Rust flag overrides for ordinary tests"
            )


def main(args: list[str]) -> int:
    try:
        validate_debug_run(args, dict(os.environ))
    except ValueError as error:
        print(f"Test isolation: {error}", file=sys.stderr)
        return 2
    cargo = shutil.which("cargo")
    if cargo is None:
        print("Test isolation: cargo is not on PATH", file=sys.stderr)
        return 2
    with tempfile.TemporaryDirectory(prefix="corbanu-tests-") as root:
        home = Path(root) / "profile"
        home.mkdir(mode=0o700)
        env = test_environment(dict(os.environ), home)
        print(
            "Test isolation: disposable profile; native keyring disabled (debug lane)",
            flush=True,
        )
        return subprocess.run(
            [cargo, "nextest", "run", "--no-fail-fast", *args],
            cwd=Path(__file__).resolve().parents[1] / "codex-rs",
            env=env,
            check=False,
        ).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
