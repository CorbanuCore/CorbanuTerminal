#!/usr/bin/env python3

import json
from pathlib import Path
import stat
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from codex_package.layout import build_package_dir
from codex_package.layout import validate_package_dir
from codex_package.targets import PACKAGE_VARIANTS
from codex_package.targets import PackageInputs
from codex_package.targets import TARGET_SPECS


class PFTerminalPackageLayoutTest(unittest.TestCase):
    def test_pfterminal_package_contains_runnable_telegram_setup_resources(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            package_dir = root / "package"
            package_dir.mkdir()
            inputs = package_inputs(root, include_walletd=True)
            variant = PACKAGE_VARIANTS["pfterminal"]
            spec = TARGET_SPECS["aarch64-apple-darwin"]

            build_package_dir(package_dir, "0.0.0", variant, spec, inputs)
            validate_package_dir(package_dir, variant, spec, include_zsh=False)

            resources = package_dir / "codex-resources" / "telegram"
            self.assertTrue(
                (resources / "setup-telegram.sh").stat().st_mode & stat.S_IXUSR
            )
            self.assertTrue((resources / "install-telegram-task.ps1").is_file())
            self.assertTrue((resources / "dist" / "AGENTS.md.template").is_file())
            self.assertTrue(
                (resources / "dist" / "pfterminal-telegram.service").is_file()
            )
            self.assertTrue(
                (
                    resources / "dist" / "net.postfiat.pfterminal.telegram.plist"
                ).is_file()
            )
            metadata = json.loads((package_dir / "codex-package.json").read_text())
            self.assertEqual(
                metadata["telegramResourcesDir"], "codex-resources/telegram"
            )

    def test_stock_codex_package_does_not_gain_pfterminal_telegram_assets(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            package_dir = root / "package"
            package_dir.mkdir()
            inputs = package_inputs(root, include_walletd=False)
            variant = PACKAGE_VARIANTS["codex-app-server"]
            spec = TARGET_SPECS["aarch64-apple-darwin"]

            build_package_dir(package_dir, "0.0.0", variant, spec, inputs)
            validate_package_dir(package_dir, variant, spec, include_zsh=False)

            self.assertFalse((package_dir / "codex-resources" / "telegram").exists())
            metadata = json.loads((package_dir / "codex-package.json").read_text())
            self.assertNotIn("telegramResourcesDir", metadata)


def package_inputs(root: Path, *, include_walletd: bool) -> PackageInputs:
    entrypoint = executable(root / "entrypoint")
    rg = executable(root / "rg")
    extra_bins = {}
    if include_walletd:
        extra_bins["pfterminal-walletd"] = executable(root / "pfterminal-walletd")
    return PackageInputs(
        entrypoint_bin=entrypoint,
        extra_bins=extra_bins,
        rg_bin=rg,
        zsh_bin=None,
        bwrap_bin=None,
        codex_command_runner_bin=None,
        codex_windows_sandbox_setup_bin=None,
    )


def executable(path: Path) -> Path:
    path.write_text("", encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


if __name__ == "__main__":
    unittest.main()
