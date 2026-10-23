#!/usr/bin/env python3

import os
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


REPO_ROOT = Path(__file__).resolve().parents[2]
SETUP_SCRIPT = REPO_ROOT / "codex-rs" / "scripts" / "setup-telegram.sh"


class SetupTelegramHomeTest(unittest.TestCase):
    def run_setup(
        self,
        home: Path,
        extra_env: dict[str, str] | None = None,
        chat_id: str = "1",
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        for name in ("CORBANU_HOME", "PFTERMINAL_HOME", "CODEX_HOME"):
            env.pop(name, None)
        env["HOME"] = str(home)
        env.update(extra_env or {})
        return subprocess.run(
            [
                "bash",
                str(SETUP_SCRIPT),
                "--chat-id",
                chat_id,
                "--no-token-required",
                "--allow-danger-full-access",
            ],
            cwd=REPO_ROOT,
            env=env,
            # Never the caller's terminal: the script prompts when stdin is one.
            stdin=subprocess.DEVNULL,
            check=False,
            capture_output=True,
            text=True,
        )

    def test_fresh_setup_uses_corbanu_paths(self) -> None:
        with TemporaryDirectory() as temp_dir:
            home = Path(temp_dir)
            result = self.run_setup(home)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((home / ".corbanu" / "config.toml").is_file())
            self.assertTrue((home / ".config" / "corbanu" / "telegram.env").is_file())
            self.assertTrue((home / "corbanu-telegram" / "AGENTS.md").is_file())

    def test_lone_legacy_home_is_reused_in_place(self) -> None:
        with TemporaryDirectory() as temp_dir:
            home = Path(temp_dir)
            (home / ".pfterminal").mkdir()
            result = self.run_setup(home)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((home / ".pfterminal" / "config.toml").is_file())
            self.assertFalse((home / ".corbanu").exists())

    def test_both_homes_prefer_corbanu_without_merging(self) -> None:
        with TemporaryDirectory() as temp_dir:
            home = Path(temp_dir)
            (home / ".corbanu").mkdir()
            (home / ".pfterminal").mkdir()
            result = self.run_setup(home)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("without merging either home", result.stderr)
            self.assertTrue((home / ".corbanu" / "config.toml").is_file())
            self.assertFalse((home / ".pfterminal" / "config.toml").exists())

    def test_product_specific_override_precedence_is_stable(self) -> None:
        with TemporaryDirectory() as temp_dir:
            home = Path(temp_dir)
            corbanu = home / "corbanu-override"
            legacy = home / "legacy-override"
            codex = home / "codex-override"
            for path in (corbanu, legacy, codex):
                path.mkdir()
            result = self.run_setup(
                home,
                {
                    "CORBANU_HOME": str(corbanu),
                    "PFTERMINAL_HOME": str(legacy),
                    "CODEX_HOME": str(codex),
                },
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((corbanu / "config.toml").is_file())
            self.assertFalse((legacy / "config.toml").exists())
            self.assertFalse((codex / "config.toml").exists())

    def test_empty_chat_id_is_reported_not_an_unbound_variable(self) -> None:
        # macOS bash 3.2 treats an empty array as unset under `set -u` (#530).
        with TemporaryDirectory() as temp_dir:
            result = self.run_setup(Path(temp_dir), chat_id="")
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("unbound variable", result.stderr)
            self.assertIn("provide at least one --chat-id", result.stderr)


if __name__ == "__main__":
    unittest.main()
