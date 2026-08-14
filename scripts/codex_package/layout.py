"""Canonical Codex package directory layout."""

import hashlib
import json
import os
import shutil
import stat
import subprocess
from pathlib import Path

from .targets import PackageInputs
from .targets import PackageVariant
from .targets import REPO_ROOT
from .targets import TargetSpec
from .zsh import ZSH_RESOURCE_PATH


LAYOUT_VERSION = 1
TERMINAL_PACKAGE_VARIANTS = frozenset(("corbanu", "pfterminal"))


def prepare_package_dir(package_dir: Path, *, force: bool) -> None:
    if package_dir.exists():
        if not package_dir.is_dir():
            raise RuntimeError(
                f"Package output exists and is not a directory: {package_dir}"
            )
        if any(package_dir.iterdir()):
            if not force:
                raise RuntimeError(
                    f"Package output directory is not empty: {package_dir}. "
                    "Pass --force to replace it."
                )
            shutil.rmtree(package_dir)

    package_dir.mkdir(parents=True, exist_ok=True)


def build_package_dir(
    package_dir: Path,
    version: str,
    variant: PackageVariant,
    spec: TargetSpec,
    inputs: PackageInputs,
    *,
    symbols_dir: Path | None = None,
) -> None:
    bin_dir = package_dir / "bin"
    resources_dir = package_dir / "codex-resources"
    path_dir = package_dir / "codex-path"
    bin_dir.mkdir()
    resources_dir.mkdir()
    path_dir.mkdir()

    entrypoint_name = variant.entrypoint_name(spec)
    copy_executable(
        inputs.entrypoint_bin,
        bin_dir / entrypoint_name,
        is_windows=spec.is_windows,
    )
    strip_packaged_executable(
        bin_dir / entrypoint_name,
        spec,
        symbols_dir=symbols_dir,
    )
    copy_executable(
        inputs.code_mode_host_bin,
        bin_dir / f"codex-code-mode-host{spec.exe_suffix}",
        is_windows=spec.is_windows,
    )
    strip_packaged_executable(
        bin_dir / f"codex-code-mode-host{spec.exe_suffix}",
        spec,
        symbols_dir=symbols_dir,
    )
    placed_binaries = [(inputs.entrypoint_bin, bin_dir / entrypoint_name)]
    for extra in variant.extra_binaries:
        output_name = extra.entrypoint_name(spec)
        source = inputs.extra_bins[output_name]
        destination = bin_dir / output_name
        alias_target = None
        if not spec.is_windows:
            alias_target = next(
                (
                    placed_destination
                    for placed_source, placed_destination in placed_binaries
                    if executable_sources_match(source, placed_source)
                ),
                None,
            )
        if alias_target is not None:
            destination.symlink_to(os.path.relpath(alias_target, destination.parent))
        else:
            copy_executable(source, destination, is_windows=spec.is_windows)
            strip_packaged_executable(
                destination,
                spec,
                symbols_dir=symbols_dir,
            )
            placed_binaries.append((source, destination))
    copy_executable(inputs.rg_bin, path_dir / spec.rg_name, is_windows=spec.is_windows)

    if inputs.zsh_bin is not None:
        copy_executable(
            inputs.zsh_bin,
            resources_dir / ZSH_RESOURCE_PATH,
            is_windows=False,
        )

    if inputs.bwrap_bin is not None:
        copy_executable(inputs.bwrap_bin, resources_dir / "bwrap", is_windows=False)

    if inputs.codex_command_runner_bin is not None:
        copy_executable(
            inputs.codex_command_runner_bin,
            resources_dir / "codex-command-runner.exe",
            is_windows=True,
        )

    if inputs.codex_windows_sandbox_setup_bin is not None:
        copy_executable(
            inputs.codex_windows_sandbox_setup_bin,
            resources_dir / "codex-windows-sandbox-setup.exe",
            is_windows=True,
        )

    if variant.name in TERMINAL_PACKAGE_VARIANTS:
        copy_telegram_resources(resources_dir / "telegram", spec)

    metadata = {
        "layoutVersion": LAYOUT_VERSION,
        "version": version,
        "target": spec.target,
        "variant": variant.name,
        "entrypoint": f"bin/{entrypoint_name}",
        "extraBinaries": [
            f"bin/{extra.entrypoint_name(spec)}" for extra in variant.extra_binaries
        ],
        "resourcesDir": "codex-resources",
        "pathDir": "codex-path",
    }
    if variant.name in TERMINAL_PACKAGE_VARIANTS:
        metadata["telegramResourcesDir"] = "codex-resources/telegram"
    write_json(package_dir / "codex-package.json", metadata)


def validate_package_dir(
    package_dir: Path,
    variant: PackageVariant,
    spec: TargetSpec,
    *,
    include_zsh: bool,
) -> None:
    required_dirs = [
        Path("bin"),
        Path("codex-resources"),
        Path("codex-path"),
    ]
    for relative_dir in required_dirs:
        path = package_dir / relative_dir
        if not path.is_dir():
            raise RuntimeError(f"Missing package directory: {relative_dir}")

    metadata_path = package_dir / "codex-package.json"
    if not metadata_path.is_file():
        raise RuntimeError("Missing package metadata: codex-package.json")

    with open(metadata_path, encoding="utf-8") as fh:
        metadata = json.load(fh)

    expected_metadata = {
        "layoutVersion": LAYOUT_VERSION,
        "target": spec.target,
        "variant": variant.name,
        "entrypoint": f"bin/{variant.entrypoint_name(spec)}",
        "extraBinaries": [
            f"bin/{extra.entrypoint_name(spec)}" for extra in variant.extra_binaries
        ],
        "resourcesDir": "codex-resources",
        "pathDir": "codex-path",
    }
    if variant.name in TERMINAL_PACKAGE_VARIANTS:
        expected_metadata["telegramResourcesDir"] = "codex-resources/telegram"
    for key, expected in expected_metadata.items():
        actual = metadata.get(key)
        if actual != expected:
            raise RuntimeError(
                f"Invalid package metadata field {key!r}: expected {expected!r}, got {actual!r}"
            )

    required_files = [
        Path("bin") / variant.entrypoint_name(spec),
        Path("bin") / f"codex-code-mode-host{spec.exe_suffix}",
        Path("codex-path") / spec.rg_name,
    ]
    executable_files = list(required_files)
    for extra in variant.extra_binaries:
        extra_path = Path("bin") / extra.entrypoint_name(spec)
        required_files.append(extra_path)
        executable_files.append(extra_path)

    if include_zsh:
        zsh_path = Path("codex-resources") / ZSH_RESOURCE_PATH
        required_files.append(zsh_path)
        executable_files.append(zsh_path)

    if spec.is_linux:
        required_files.append(Path("codex-resources") / "bwrap")
        executable_files.append(Path("codex-resources") / "bwrap")

    if spec.is_windows:
        required_files.extend(
            [
                Path("codex-resources") / "codex-command-runner.exe",
                Path("codex-resources") / "codex-windows-sandbox-setup.exe",
            ]
        )

    if variant.name in TERMINAL_PACKAGE_VARIANTS:
        telegram_dir = Path("codex-resources") / "telegram"
        required_files.extend(
            [
                telegram_dir / "setup-telegram.sh",
                telegram_dir / "install-telegram-task.ps1",
                telegram_dir / "dist" / "AGENTS.md.template",
                telegram_dir / "dist" / "pfterminal-telegram.service",
                telegram_dir / "dist" / "net.postfiat.pfterminal.telegram.plist",
            ]
        )
        if not spec.is_windows:
            executable_files.append(telegram_dir / "setup-telegram.sh")

    for relative_file in required_files:
        path = package_dir / relative_file
        if not path.is_file():
            raise RuntimeError(f"Missing package file: {relative_file}")

    if not spec.is_windows:
        for relative_file in executable_files:
            path = package_dir / relative_file
            if not is_executable(path):
                raise RuntimeError(f"Package file is not executable: {relative_file}")


def copy_executable(src: Path, dest: Path, *, is_windows: bool) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dest)
    if not is_windows:
        mode = dest.stat().st_mode
        dest.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def executable_sources_match(left: Path, right: Path) -> bool:
    try:
        if os.path.samefile(left, right):
            return True
    except OSError:
        pass

    if left.stat().st_size != right.stat().st_size:
        return False
    return file_digest(left) == file_digest(right)


def file_digest(path: Path) -> bytes:
    digest = hashlib.sha256()
    with open(path, "rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.digest()


def strip_packaged_executable(
    path: Path,
    spec: TargetSpec,
    *,
    symbols_dir: Path | None,
) -> None:
    binary_format = native_binary_format(path)
    if spec.is_windows or binary_format is None:
        return

    strip_tool = resolve_binary_tool("STRIP", ("llvm-strip", "strip"))
    if binary_format == "elf":
        if symbols_dir is not None:
            symbols_dir.mkdir(parents=True, exist_ok=True)
            debug_path = symbols_dir / f"{path.name}.debug"
            objcopy_tool = resolve_binary_tool("OBJCOPY", ("llvm-objcopy", "objcopy"))
            subprocess.run(
                [objcopy_tool, "--only-keep-debug", path, debug_path],
                check=True,
            )
        subprocess.run(
            [strip_tool, "--strip-debug", "--strip-unneeded", path],
            check=True,
        )
        if symbols_dir is not None:
            subprocess.run(
                [objcopy_tool, f"--add-gnu-debuglink={debug_path}", path],
                check=True,
            )
    elif binary_format == "mach-o":
        if symbols_dir is not None:
            symbols_dir.mkdir(parents=True, exist_ok=True)
            dsymutil = resolve_binary_tool("DSYMUTIL", ("dsymutil",))
            subprocess.run(
                [dsymutil, path, "-o", symbols_dir / f"{path.name}.dSYM"],
                check=True,
            )
        subprocess.run([strip_tool, "-S", "-x", path], check=True)


def native_binary_format(path: Path) -> str | None:
    with open(path, "rb") as binary:
        magic = binary.read(4)
    if magic == b"\x7fELF":
        return "elf"
    if magic in {
        b"\xcf\xfa\xed\xfe",
        b"\xfe\xed\xfa\xcf",
        b"\xca\xfe\xba\xbe",
        b"\xbe\xba\xfe\xca",
    }:
        return "mach-o"
    return None


def resolve_binary_tool(environment_variable: str, candidates: tuple[str, ...]) -> str:
    configured = os.environ.get(environment_variable)
    if configured:
        resolved = shutil.which(configured)
        if resolved is not None:
            return resolved
        raise RuntimeError(
            f"{environment_variable} names an unavailable executable: {configured}"
        )
    for candidate in candidates:
        resolved = shutil.which(candidate)
        if resolved is not None:
            return resolved
    raise RuntimeError(
        f"Required packaging tool is unavailable: {', '.join(candidates)}"
    )


def copy_telegram_resources(dest: Path, spec: TargetSpec) -> None:
    sources = {
        Path("setup-telegram.sh"): REPO_ROOT
        / "codex-rs"
        / "scripts"
        / "setup-telegram.sh",
        Path("install-telegram-task.ps1"): REPO_ROOT
        / "codex-rs"
        / "scripts"
        / "install-telegram-task.ps1",
        Path("dist/AGENTS.md.template"): REPO_ROOT
        / "codex-rs"
        / "telegram"
        / "dist"
        / "AGENTS.md.template",
        Path("dist/pfterminal-telegram.service"): REPO_ROOT
        / "codex-rs"
        / "telegram"
        / "dist"
        / "pfterminal-telegram.service",
        Path("dist/net.postfiat.pfterminal.telegram.plist"): REPO_ROOT
        / "codex-rs"
        / "telegram"
        / "dist"
        / "net.postfiat.pfterminal.telegram.plist",
    }
    for relative_path, source in sources.items():
        target = dest / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    if not spec.is_windows:
        setup = dest / "setup-telegram.sh"
        setup.chmod(setup.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def write_json(path: Path, value: object) -> None:
    with open(path, "w", encoding="utf-8") as out:
        json.dump(value, out, indent=2)
        out.write("\n")


def is_executable(path: Path) -> bool:
    return bool(path.stat().st_mode & stat.S_IXUSR)
