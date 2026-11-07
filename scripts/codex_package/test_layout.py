#!/usr/bin/env python3

from pathlib import Path
<<<<<<< HEAD
import os
||||||| 413492cd6c
=======
import hashlib
import io
import tarfile
import json
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

<<<<<<< HEAD
from codex_package.archive import write_archive
||||||| 413492cd6c
=======
from build_winget_package import prepare_winget_package
from codex_package.archive import write_archive, resolve_zstd_command
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
from codex_package.layout import build_package_dir
from codex_package.layout import validate_package_dir
from codex_package.targets import PACKAGE_VARIANTS
from codex_package.targets import PackageInputs
from codex_package.targets import TARGET_SPECS
from codex_package.targets import default_target


class PackageLayoutTest(unittest.TestCase):
<<<<<<< HEAD
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
||||||| 413492cd6c
=======
    @unittest.skipIf(sys.platform == "win32", "MSVC symbols are separate PDB files")
    def test_strip_policy_preserves_inputs_and_runs_packaged_executables(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / "input-executable"
            # System executables may already be stripped. Compile with debug
            # information so a no-op strip cannot pass this regression test.
            subprocess.run(
                ["cc", "-g", "-O0", "-x", "c", "-", "-o", str(source)],
                input=r"""#include <stdio.h>
int main(void) {
    fputs("package-ok", stdout);
    return 0;
}
""",
                text=True,
                check=True,
                capture_output=True,
            )
            original = source.read_bytes()
            target = default_target()
            for mode in ("auto", "all", "none"):
                with self.subTest(mode=mode):
                    package = root / mode
                    command = [
                        sys.executable,
                        str(
                            Path(__file__).resolve().parents[1]
                            / "build_codex_package.py"
                        ),
                        "--target",
                        target,
                        "--cargo-profile",
                        "release",
                        "--package-dir",
                        str(package),
                        "--strip",
                        mode,
                    ]
                    flags = [
                        "--entrypoint-bin",
                        "--code-mode-host-bin",
                        "--exec-server-bin",
                        "--rg-bin",
                    ]
                    if "linux" in target:
                        flags.append("--bwrap-bin")
                    for flag in flags:
                        command.extend((flag, str(source)))
                    if mode != "all":
                        # Preserved prebuilt inputs must not even need a strip tool.
                        command.extend(("--strip-tool", str(root / "absent-strip")))
                    subprocess.run(command, check=True, capture_output=True)
                    for name in ("codex", "codex-code-mode-host", "codex-exec-server"):
                        executable = package / "bin" / name
                        self.assertEqual(
                            subprocess.check_output([str(executable)]),
                            b"package-ok",
                        )
                        if mode != "all":
                            self.assertEqual(executable.read_bytes(), original)
                        else:
                            self.assertLess(executable.stat().st_size, len(original))
                    self.assertEqual(source.read_bytes(), original)
                    self.assertEqual((package / "codex-path/rg").read_bytes(), original)
                    if "linux" in target:
                        self.assertEqual(
                            (package / "codex-resources/bwrap").read_bytes(), original
                        )

    def test_winget_preserves_signed_files_and_voice_hashes(self) -> None:
        for target in ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as temp:
                package = Path(temp)
                files = {
                    "bin/codex.exe": b"signed CLI",
                    "bin/codex-code-mode-host.exe": b"signed code mode host",
                    "bin/codex-exec-server.exe": b"signed executor",
                    "codex-resources/codex-command-runner.exe": b"signed runner",
                    "codex-resources/codex-windows-sandbox-setup.exe": b"signed setup",
                    "codex-resources/voice/bin/codex-voice-host.exe": b"signed voice host",
                    "codex-resources/voice/bin/gstreamer-1.0-0.dll": b"signed audio DLL",
                    "codex-resources/voice/NOTICE.md": b"license notices",
                    "codex-path/rg.exe": b"ripgrep",
                }
                for name, contents in files.items():
                    path = package / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(contents)
                metadata = {
                    "layoutVersion": 1,
                    "target": target,
                    "entrypoint": "bin/codex.exe",
                }
                (package / "codex-package.json").write_text(json.dumps(metadata))
                manifest = {
                    "schemaVersion": 1,
                    "sha256": {
                        name: hashlib.sha256(contents).hexdigest()
                        for name, contents in files.items()
                        if name == "bin/codex.exe"
                        or name.startswith("codex-resources/voice/")
                    },
                }
                manifest_path = package / "codex-resources/voice/manifest.json"
                manifest_path.write_text(json.dumps(manifest))
                prepare_winget_package(package)
                entrypoint = f"codex-{target}.exe"
                files[entrypoint] = files.pop("bin/codex.exe")
                files["codex-code-mode-host.exe"] = files.pop(
                    "bin/codex-code-mode-host.exe"
                )
                for helper in (
                    "codex-command-runner.exe",
                    "codex-windows-sandbox-setup.exe",
                ):
                    files[helper] = files[f"codex-resources/{helper}"]
                actual = {
                    str(path.relative_to(package)).replace("\\", "/"): path.read_bytes()
                    for path in package.rglob("*")
                    if path.is_file()
                }
                actual_metadata = json.loads(actual.pop("codex-package.json"))
                actual_manifest = json.loads(
                    actual.pop("codex-resources/voice/manifest.json")
                )
                self.assertEqual(actual, files)
                metadata["entrypoint"] = entrypoint
                self.assertEqual(actual_metadata, metadata)
                manifest["sha256"][entrypoint] = manifest["sha256"].pop("bin/codex.exe")
                self.assertEqual(actual_manifest, manifest)
                for name, digest in actual_manifest["sha256"].items():
                    self.assertEqual(hashlib.sha256(actual[name]).hexdigest(), digest)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

    def test_macos_package_preserves_prebuilt_resource_binaries(self) -> None:
        for variant_name in ("corbanu", "pfterminal", "codex", "codex-app-server"):
            for target in ("aarch64-apple-darwin", "x86_64-apple-darwin"):
                with self.subTest(variant=variant_name, target=target):
                    with tempfile.TemporaryDirectory() as temp_dir:
                        root = Path(temp_dir)
                        package_dir = root / "package"
                        package_dir.mkdir()
                        rg_bin = touch_executable(root / "signed-rg")
                        rg_bin.write_bytes(b"signed ripgrep binary")
                        variant = PACKAGE_VARIANTS[variant_name]
                        spec = TARGET_SPECS[target]
                        inputs = PackageInputs(
                            entrypoint_bin=touch_executable(
                                root / variant.executable_stem
                            ),
                            code_mode_host_bin=touch_executable(
                                root / "codex-code-mode-host"
                            ),
<<<<<<< HEAD
                            extra_bins={
                                extra.entrypoint_name(spec): touch_executable(
                                    root / extra.entrypoint_name(spec)
                                )
                                for extra in variant.extra_binaries
                            },
||||||| 413492cd6c
=======
                            exec_server_bin=rg_bin,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                            rg_bin=rg_bin,
                            bwrap_bin=None,
                            codex_command_runner_bin=None,
                            codex_windows_sandbox_setup_bin=None,
                        )

                        build_package_dir(package_dir, "1.2.3", variant, spec, inputs)
                        validate_package_dir(package_dir, variant, spec)
                        for suffix in (".tar.gz", ".tar.zst"):
                            archive_path = root / f"package{suffix}"
                            write_archive(package_dir, archive_path, force=False)
                            data = (
                                subprocess.check_output(
                                    [
                                        *resolve_zstd_command(),
                                        "-d",
                                        "--stdout",
                                        str(archive_path),
                                    ]
                                )
                                if suffix == ".tar.zst"
                                else archive_path.read_bytes()
                            )
                            with tarfile.open(
                                fileobj=io.BytesIO(data), mode="r:*"
                            ) as archive:
                                member = archive.getmember("bin/codex-exec-server")
                                self.assertTrue(member.mode & 0o111)
                                self.assertEqual(
                                    archive.extractfile(member).read(),
                                    rg_bin.read_bytes(),
                                )

                        self.assertEqual(
                            {
                                "rg": (package_dir / "codex-path" / "rg").read_bytes(),
                                "executor": (
                                    package_dir / "bin/codex-exec-server"
                                ).read_bytes(),
                            },
                            {
                                "rg": b"signed ripgrep binary",
                                "executor": b"signed ripgrep binary",
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
<<<<<<< HEAD
                extra_bins={},
||||||| 413492cd6c
=======
                exec_server_bin=touch_executable(root / "codex-exec-server"),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                rg_bin=touch_executable(root / "rg"),
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
            )

            self.assertTrue((package_dir / "bin" / "codex-code-mode-host").is_file())

    def test_codex_package_requires_wallet_runtime_companion(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            variant = PACKAGE_VARIANTS["codex"]
            spec = TARGET_SPECS["x86_64-unknown-linux-gnu"]
            inputs = PackageInputs(
                entrypoint_bin=touch_executable(root / "codex"),
                code_mode_host_bin=touch_executable(root / "codex-code-mode-host"),
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
            package = root / "package"
            package.mkdir()
            build_package_dir(package, "1.2.3", variant, spec, inputs)
            validate_package_dir(package, variant, spec, include_zsh=False)
            companion = package / "bin" / "pfterminal-walletd"
            self.assertTrue(companion.is_file())
            companion.unlink()
            with self.assertRaises(RuntimeError):
                validate_package_dir(package, variant, spec, include_zsh=False)

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
                self.assertEqual(
                    alias_path.resolve(), (package_dir / "bin" / target).resolve()
                )
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

    @unittest.skipUnless(
        os.name == "posix" and shutil.which("cc"), "Unix C compiler is required"
    )
    def test_unix_native_binaries_are_stripped_with_external_sidecars(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "main.c"
            source.write_text("int main(void) { return 0; }\n", encoding="utf-8")
            entrypoint = root / "codex-app-server"
            code_mode_host = root / "codex-code-mode-host"
            # dsymutil needs the original object to collect Mach-O debug data.
            # A single compile-and-link cc invocation deletes that object.
            object_file = root / "main.o"
            subprocess.run(
                ["cc", "-g", "-O0", "-c", source, "-o", object_file],
                check=True,
            )
            for output in (entrypoint, code_mode_host):
                subprocess.run(
                    ["cc", "-g", object_file, "-o", output],
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
            for name in ("codex-app-server", "codex-code-mode-host"):
                sidecar = (
                    symbols_dir
                    / f"{name}.dSYM"
                    / "Contents"
                    / "Resources"
                    / "DWARF"
                    / name
                    if sys.platform == "darwin"
                    else symbols_dir / f"{name}.debug"
                )
                self.assertTrue(sidecar.is_file(), sidecar)
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
