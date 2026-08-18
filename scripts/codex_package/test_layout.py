#!/usr/bin/env python3

from pathlib import Path
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from codex_package.archive import write_archive
from codex_package.layout import build_package_dir
from codex_package.layout import validate_package_dir
from codex_package.targets import PACKAGE_VARIANTS
from codex_package.targets import PackageInputs
from codex_package.targets import TARGET_SPECS


class PackageLayoutTest(unittest.TestCase):
    def test_telegram_powershell_installer_uses_identifier_safe_parameters(
        self,
    ) -> None:
        script = (
            Path(__file__).resolve().parents[2]
            / "codex-rs"
            / "scripts"
            / "install-telegram-task.ps1"
        ).read_text(encoding="utf-8")

        self.assertIn("[string]$TerminalPath", script)
        self.assertIn("Get-Command corbanu", script)
        self.assertNotIn("Get-Command pfterminal", script)
        declarations = [line for line in script.splitlines() if "[string]$" in line]
        self.assertTrue(declarations)
        for declaration in declarations:
            self.assertRegex(
                declaration,
                r"^\s*\[string\]\$[A-Za-z_][A-Za-z0-9_]*\s*=",
            )

    def test_macos_package_preserves_prebuilt_resource_binaries(self) -> None:
        for variant_name in ("corbanu", "pfterminal", "codex", "codex-app-server"):
            for target in ("aarch64-apple-darwin", "x86_64-apple-darwin"):
                with self.subTest(variant=variant_name, target=target):
                    with tempfile.TemporaryDirectory() as temp_dir:
                        root = Path(temp_dir)
                        package_dir = root / "package"
                        package_dir.mkdir()
                        rg_bin = touch_executable(root / "signed-rg")
                        zsh_bin = touch_executable(root / "signed-zsh")
                        rg_bin.write_bytes(b"signed ripgrep binary")
                        zsh_bin.write_bytes(b"signed zsh binary")
                        variant = PACKAGE_VARIANTS[variant_name]
                        spec = TARGET_SPECS[target]
                        inputs = PackageInputs(
                            entrypoint_bin=touch_executable(
                                root / variant.executable_stem
                            ),
                            code_mode_host_bin=touch_executable(
                                root / "codex-code-mode-host"
                            ),
                            extra_bins={
                                extra.entrypoint_name(spec): touch_executable(
                                    root / extra.entrypoint_name(spec)
                                )
                                for extra in variant.extra_binaries
                            },
                            rg_bin=rg_bin,
                            zsh_bin=zsh_bin,
                            bwrap_bin=None,
                            codex_command_runner_bin=None,
                            codex_windows_sandbox_setup_bin=None,
                        )

                        build_package_dir(package_dir, "1.2.3", variant, spec, inputs)
                        validate_package_dir(
                            package_dir, variant, spec, include_zsh=True
                        )

                        self.assertEqual(
                            {
                                "rg": (package_dir / "codex-path" / "rg").read_bytes(),
                                "zsh": (
                                    package_dir
                                    / "codex-resources"
                                    / "zsh"
                                    / "bin"
                                    / "zsh"
                                ).read_bytes(),
                            },
                            {
                                "rg": b"signed ripgrep binary",
                                "zsh": b"signed zsh binary",
                            },
                        )

    def test_terminal_packages_include_telegram_resources(self) -> None:
        for variant_name in ("corbanu", "pfterminal"):
            with self.subTest(variant=variant_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    package_dir = root / "package"
                    package_dir.mkdir()
                    variant = PACKAGE_VARIANTS[variant_name]
                    spec = TARGET_SPECS["x86_64-unknown-linux-musl"]
                    inputs = PackageInputs(
                        entrypoint_bin=touch_executable(root / variant.executable_stem),
                        code_mode_host_bin=touch_executable(
                            root / "codex-code-mode-host"
                        ),
                        extra_bins={
                            extra.entrypoint_name(spec): touch_executable(
                                root / extra.entrypoint_name(spec)
                            )
                            for extra in variant.extra_binaries
                        },
                        rg_bin=touch_executable(root / "rg"),
                        zsh_bin=None,
                        bwrap_bin=touch_executable(root / "bwrap"),
                        codex_command_runner_bin=None,
                        codex_windows_sandbox_setup_bin=None,
                    )

                    build_package_dir(package_dir, "1.2.3", variant, spec, inputs)
                    validate_package_dir(package_dir, variant, spec, include_zsh=False)

                    self.assertTrue(
                        (
                            package_dir
                            / "codex-resources"
                            / "telegram"
                            / "setup-telegram.sh"
                        ).is_file()
                    )

    def test_app_server_package_places_code_mode_host_beside_entrypoint(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            package_dir = root / "package"
            package_dir.mkdir()
            inputs = PackageInputs(
                entrypoint_bin=touch_executable(root / "codex-app-server"),
                code_mode_host_bin=touch_executable(root / "codex-code-mode-host"),
                extra_bins={},
                rg_bin=touch_executable(root / "rg"),
                zsh_bin=None,
                bwrap_bin=touch_executable(root / "bwrap"),
                codex_command_runner_bin=None,
                codex_windows_sandbox_setup_bin=None,
            )

            build_package_dir(
                package_dir,
                "1.2.3",
                PACKAGE_VARIANTS["codex-app-server"],
                TARGET_SPECS["x86_64-unknown-linux-musl"],
                inputs,
            )
            validate_package_dir(
                package_dir,
                PACKAGE_VARIANTS["codex-app-server"],
                TARGET_SPECS["x86_64-unknown-linux-musl"],
                include_zsh=False,
            )

            self.assertTrue((package_dir / "bin" / "codex-code-mode-host").is_file())

    def test_corbanu_debug_alias_is_relative_and_survives_archive(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            package_dir = root / "package"
            package_dir.mkdir()
            entrypoint = write_version_executable(root / "corbanu", "corbanu")
            acp = write_version_executable(root / "corbanu-acp", "corbanu-acp")
            walletd = write_version_executable(
                root / "corbanu-walletd", "corbanu-walletd"
            )
            variant = PACKAGE_VARIANTS["corbanu"]
            spec = TARGET_SPECS["x86_64-unknown-linux-musl"]
            source_by_stem = {
                "corbanu": entrypoint,
                "corbanu-acp": acp,
                "corbanu-walletd": walletd,
            }
            inputs = PackageInputs(
                entrypoint_bin=entrypoint,
                code_mode_host_bin=write_version_executable(
                    root / "codex-code-mode-host", "codex-code-mode-host"
                ),
                extra_bins={
                    extra.entrypoint_name(spec): source_by_stem[
                        extra.alias_of or extra.executable_stem
                    ]
                    for extra in variant.extra_binaries
                },
                rg_bin=touch_executable(root / "rg"),
                zsh_bin=None,
                bwrap_bin=touch_executable(root / "bwrap"),
                codex_command_runner_bin=None,
                codex_windows_sandbox_setup_bin=None,
            )

            build_package_dir(package_dir, "1.2.3", variant, spec, inputs)
            validate_package_dir(package_dir, variant, spec, include_zsh=False)

            expected_aliases = {
                "corbanu-debug": "corbanu",
            }
            for alias, target in expected_aliases.items():
                alias_path = package_dir / "bin" / alias
                self.assertTrue(alias_path.is_symlink(), alias)
                self.assertFalse(os.path.isabs(os.readlink(alias_path)), alias)
                self.assertEqual(alias_path.resolve(), package_dir / "bin" / target)
            for canonical in ("corbanu", "corbanu-acp", "corbanu-walletd"):
                self.assertFalse((package_dir / "bin" / canonical).is_symlink())
            for legacy in (
                "pfterminal",
                "pfterminal-debug",
                "pfterminal-acp",
                "pfterminal-walletd",
            ):
                self.assertFalse((package_dir / "bin" / legacy).exists())

            archive_path = root / "corbanu.tar.gz"
            write_archive(package_dir, archive_path, force=False)
            extracted = root / "extracted"
            extracted.mkdir()
            with tarfile.open(archive_path, "r:gz") as archive:
                archive.extractall(extracted, filter="data")
            for alias in expected_aliases:
                result = subprocess.run(
                    [str(extracted / "bin" / alias), "--version"],
                    capture_output=True,
                    check=False,
                    text=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr)

    @unittest.skipUnless(shutil.which("cc"), "C compiler is required")
    def test_unix_native_binaries_are_stripped_with_external_sidecars(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "main.c"
            source.write_text("int main(void) { return 0; }\n", encoding="utf-8")
            entrypoint = root / "codex-app-server"
            code_mode_host = root / "codex-code-mode-host"
            for output in (entrypoint, code_mode_host):
                subprocess.run(
                    ["cc", "-g", "-O0", source, "-o", output],
                    check=True,
                )
            unstripped_size = entrypoint.stat().st_size
            package_dir = root / "package"
            package_dir.mkdir()
            symbols_dir = root / "symbols"
            spec = TARGET_SPECS["x86_64-unknown-linux-gnu"]
            variant = PACKAGE_VARIANTS["codex-app-server"]
            inputs = PackageInputs(
                entrypoint_bin=entrypoint,
                code_mode_host_bin=code_mode_host,
                extra_bins={},
                rg_bin=touch_executable(root / "rg"),
                zsh_bin=None,
                bwrap_bin=touch_executable(root / "bwrap"),
                codex_command_runner_bin=None,
                codex_windows_sandbox_setup_bin=None,
            )

            build_package_dir(
                package_dir,
                "1.2.3",
                variant,
                spec,
                inputs,
                symbols_dir=symbols_dir,
            )

            self.assertLess(
                (package_dir / "bin" / "codex-app-server").stat().st_size,
                unstripped_size,
            )
            self.assertTrue((symbols_dir / "codex-app-server.debug").is_file())
            self.assertTrue((symbols_dir / "codex-code-mode-host.debug").is_file())
            self.assertFalse(
                (package_dir / "codex-resources" / "debug-symbols").exists()
            )


def touch_executable(path: Path) -> Path:
    path.touch(mode=0o755)
    return path


def write_version_executable(path: Path, name: str) -> Path:
    path.write_text(f"#!/bin/sh\nprintf '{name} 1.2.3\\n'\n", encoding="utf-8")
    path.chmod(0o755)
    return path


if __name__ == "__main__":
    unittest.main()
