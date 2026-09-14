import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import isolated_rust_tests as runner


class IsolatedRustTests(unittest.TestCase):
    def test_live_aliases_and_credentials_do_not_reach_child(self):
        with tempfile.TemporaryDirectory() as root:
            live = Path(root) / "live"
            live.mkdir()
            canary = live / "auth-canary"
            canary.write_text("synthetic-do-not-touch", encoding="utf-8")
            fixture = Path(root) / "fixture"
            fixture.mkdir()
            parent = {
                "PATH": os.environ.get("PATH", ""),
                "CORBANU_HOME": str(live),
                "PFTERMINAL_HOME": str(live),
                "CODEX_HOME": str(live),
                "CLAUDE_CODE_OAUTH_TOKEN": "synthetic-token",
                "ANTHROPIC_API_KEY": "synthetic-key",
                "OPENAI_API_KEY": "synthetic-key",
                "CORBANU_TEST_NO_NATIVE_KEYRING": "0",
                "CARGO_TARGET_DIR": str(Path(root) / "build-cache"),
            }
            before = parent.copy()
            env = runner.test_environment(parent, fixture)
            result = subprocess.run(
                [
                    sys.executable,
                    "-c",
                    "import json,os; print(json.dumps(dict(os.environ)))",
                ],
                env=env,
                capture_output=True,
                text=True,
                check=True,
            )
            child = json.loads(result.stdout)
            self.assertEqual(child["CODEX_HOME"], str(fixture))
            self.assertEqual(child["CORBANU_TEST_NO_NATIVE_KEYRING"], "1")
            for name in (
                "CORBANU_HOME",
                "PFTERMINAL_HOME",
                "CLAUDE_CODE_OAUTH_TOKEN",
                "ANTHROPIC_API_KEY",
                "OPENAI_API_KEY",
            ):
                self.assertNotIn(name, child)
            self.assertEqual(child["CARGO_TARGET_DIR"], parent["CARGO_TARGET_DIR"])
            self.assertEqual(parent, before)
            self.assertEqual(
                canary.read_text(encoding="utf-8"), "synthetic-do-not-touch"
            )

    def test_existing_fixture_can_override_codex_home_without_alias_shadowing(self):
        env = runner.test_environment({"CORBANU_HOME": "host"}, Path("run-profile"))
        env["CODEX_HOME"] = "individual-fixture"
        selected = (
            env.get("CORBANU_HOME") or env.get("PFTERMINAL_HOME") or env["CODEX_HOME"]
        )
        self.assertEqual(selected, "individual-fixture")

    def test_launch_forces_guard_preserves_exit_and_cleans_only_owned_profile(self):
        profiles = []

        def fake_cargo(argv, *, cwd, env, check):
            self.assertEqual(
                argv,
                [
                    "/synthetic/cargo",
                    "nextest",
                    "run",
                    "--no-fail-fast",
                    "-p",
                    "fixture",
                ],
            )
            self.assertEqual(cwd.name, "codex-rs")
            self.assertFalse(check)
            profiles.append(Path(env["CODEX_HOME"]))
            self.assertTrue(profiles[-1].is_dir())
            self.assertEqual(env["CORBANU_TEST_NO_NATIVE_KEYRING"], "1")
            self.assertNotIn("CORBANU_HOME", env)
            return subprocess.CompletedProcess(argv, 100)

        with (
            patch.dict(os.environ, {"CORBANU_HOME": "host-canary"}, clear=True),
            patch.object(runner.shutil, "which", return_value="/synthetic/cargo"),
            patch.object(runner.subprocess, "run", side_effect=fake_cargo),
        ):
            self.assertEqual(runner.main(["-p", "fixture"]), 100)
            self.assertEqual(runner.main(["-p", "fixture"]), 100)
            self.assertEqual(os.environ["CORBANU_HOME"], "host-canary")
        self.assertNotEqual(profiles[0], profiles[1])
        self.assertTrue(all(not path.exists() for path in profiles))

    def test_unsafe_build_lane_fails_before_cargo_starts(self):
        for option in (
            "--release",
            "-r",
            "-vr",
            "-rj4",
            "--cargo-profile=release",
            "--config=x",
            "--archive-file=old.tar.zst",
            "--binaries-metadata=old.json",
        ):
            with (
                self.subTest(option=option),
                patch.object(runner.subprocess, "run") as run,
            ):
                self.assertEqual(runner.main([option]), 2)
                run.assert_not_called()

    def test_debug_assertions_override_is_rejected(self):
        for name in (
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS",
        ):
            with self.subTest(name=name), self.assertRaises(ValueError):
                runner.validate_debug_run([], {name: "-Cdebug-assertions=no"})

    def test_attached_package_and_filter_values_are_not_release_flags(self):
        runner.validate_debug_run(["-pcodex-keyring-store", "-Etest(guard)", "-j4"], {})

    def test_missing_cargo_is_explicit(self):
        with patch.object(runner.shutil, "which", return_value=None):
            self.assertEqual(runner.main([]), 2)


if __name__ == "__main__":
    unittest.main()
