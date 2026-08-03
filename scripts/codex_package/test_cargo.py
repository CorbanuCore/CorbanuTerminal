#!/usr/bin/env python3

from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from codex_package.cargo import build_source_binaries
from codex_package.cargo import source_binaries_for_target
from codex_package.cli import parse_args
from codex_package.targets import PACKAGE_VARIANTS
from codex_package.targets import TARGET_SPECS


class SourceBinariesForTargetTest(unittest.TestCase):
    def test_cli_accepts_repeated_prebuilt_extra_binaries(self) -> None:
        with patch.object(
            sys,
            "argv",
            [
                "build_codex_package.py",
                "--variant",
                "pfterminal",
                "--extra-bin",
                "pfterminal-debug=/tmp/pfterminal-debug",
                "--extra-bin",
                "pfterminal-walletd=/tmp/pfterminal-walletd",
            ],
        ):
            args = parse_args()

        self.assertEqual(
            args.extra_bin,
            [
                "pfterminal-debug=/tmp/pfterminal-debug",
                "pfterminal-walletd=/tmp/pfterminal-walletd",
            ],
        )

    def test_pfterminal_package_builds_required_debug_and_wallet_binaries(self) -> None:
        self.assertEqual(
            source_binaries_for_target(
                TARGET_SPECS["aarch64-apple-darwin"],
                PACKAGE_VARIANTS["pfterminal"],
                build_entrypoint=False,
                extra_cargo_bins=["pfterminal-debug", "pfterminal-walletd"],
                build_code_mode_host=False,
                build_bwrap=False,
                build_codex_command_runner=False,
                build_codex_windows_sandbox_setup=False,
            ),
            ["pfterminal-debug", "pfterminal-walletd"],
        )

    def test_pfterminal_package_accepts_prebuilt_debug_and_wallet_binaries(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            entrypoint = touch_file(root / "pfterminal")
            code_mode_host = touch_file(root / "codex-code-mode-host")
            debug_entrypoint = touch_file(root / "pfterminal-debug")
            wallet_daemon = touch_file(root / "pfterminal-walletd")

            outputs = build_source_binaries(
                TARGET_SPECS["aarch64-apple-darwin"],
                PACKAGE_VARIANTS["pfterminal"],
                cargo=str(root / "cargo-that-should-not-run"),
                profile="release",
                entrypoint_bin=entrypoint,
                code_mode_host_bin=code_mode_host,
                extra_bins={
                    "pfterminal-debug": debug_entrypoint,
                    "pfterminal-walletd": wallet_daemon,
                },
                bwrap_bin=None,
                codex_command_runner_bin=None,
                codex_windows_sandbox_setup_bin=None,
            )

        self.assertEqual(outputs.entrypoint_bin, entrypoint)
        self.assertEqual(outputs.extra_bins["pfterminal-debug"], debug_entrypoint)
        self.assertEqual(outputs.extra_bins["pfterminal-walletd"], wallet_daemon)

    def test_macos_package_with_prebuilt_entrypoint_builds_nothing(self) -> None:
        self.assertEqual(
            source_binaries_for_target(
                TARGET_SPECS["aarch64-apple-darwin"],
                PACKAGE_VARIANTS["codex"],
                build_entrypoint=False,
                build_code_mode_host=False,
                build_bwrap=False,
                build_codex_command_runner=False,
                build_codex_windows_sandbox_setup=False,
            ),
            [],
        )

    def test_linux_package_with_prebuilt_entrypoint_and_bwrap_builds_nothing(
        self,
    ) -> None:
        self.assertEqual(
            source_binaries_for_target(
                TARGET_SPECS["x86_64-unknown-linux-musl"],
                PACKAGE_VARIANTS["codex"],
                build_entrypoint=False,
                build_code_mode_host=False,
                build_bwrap=False,
                build_codex_command_runner=False,
                build_codex_windows_sandbox_setup=False,
            ),
            [],
        )

    def test_windows_package_with_prebuilt_entrypoint_and_helpers_builds_nothing(
        self,
    ) -> None:
        self.assertEqual(
            source_binaries_for_target(
                TARGET_SPECS["x86_64-pc-windows-msvc"],
                PACKAGE_VARIANTS["codex"],
                build_entrypoint=False,
                build_code_mode_host=False,
                build_bwrap=False,
                build_codex_command_runner=False,
                build_codex_windows_sandbox_setup=False,
            ),
            [],
        )

    def test_missing_windows_helpers_are_built(self) -> None:
        self.assertEqual(
            source_binaries_for_target(
                TARGET_SPECS["x86_64-pc-windows-msvc"],
                PACKAGE_VARIANTS["codex"],
                build_entrypoint=False,
                build_code_mode_host=False,
                build_bwrap=False,
                build_codex_command_runner=True,
                build_codex_windows_sandbox_setup=True,
            ),
            ["codex-command-runner", "codex-windows-sandbox-setup"],
        )

    def test_missing_code_mode_host_is_built_for_app_server(self) -> None:
        self.assertEqual(
            source_binaries_for_target(
                TARGET_SPECS["aarch64-apple-darwin"],
                PACKAGE_VARIANTS["codex-app-server"],
                build_entrypoint=False,
                build_code_mode_host=True,
                build_bwrap=False,
                build_codex_command_runner=False,
                build_codex_windows_sandbox_setup=False,
            ),
            ["codex-code-mode-host"],
        )

    def test_build_uses_prebuilt_windows_helpers_without_running_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            entrypoint = touch_file(root / "codex.exe")
            code_mode_host = touch_file(root / "codex-code-mode-host.exe")
            command_runner = touch_file(root / "codex-command-runner.exe")
            sandbox_setup = touch_file(root / "codex-windows-sandbox-setup.exe")
            pfterminal = touch_file(root / "pfterminal.exe")

            outputs = build_source_binaries(
                TARGET_SPECS["x86_64-pc-windows-msvc"],
                PACKAGE_VARIANTS["codex"],
                cargo=str(root / "cargo-that-should-not-run"),
                profile="release",
                entrypoint_bin=entrypoint,
                code_mode_host_bin=code_mode_host,
                bwrap_bin=None,
                codex_command_runner_bin=command_runner,
                codex_windows_sandbox_setup_bin=sandbox_setup,
                extra_bins={"pfterminal.exe": pfterminal},
            )

        self.assertEqual(outputs.entrypoint_bin, entrypoint)
        self.assertEqual(outputs.code_mode_host_bin, code_mode_host)
        self.assertEqual(outputs.codex_command_runner_bin, command_runner)
        self.assertEqual(outputs.codex_windows_sandbox_setup_bin, sandbox_setup)


def touch_file(path: Path) -> Path:
    path.write_text("", encoding="utf-8")
    return path.resolve()


if __name__ == "__main__":
    unittest.main()
