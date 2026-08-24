#!/usr/bin/env python3
"""Verify the agent-portable skill tree exactly mirrors tracked Codex skills."""

import stat
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOT = REPO_ROOT / ".codex" / "skills"
PORTABLE_ROOT = REPO_ROOT / ".agents" / "skills"
EXECUTABLE_BITS = stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH


def tracked_skill_paths(repo_root: Path = REPO_ROOT) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z", "--", ".codex/skills"],
        cwd=repo_root,
        check=True,
        capture_output=True,
    )
    prefix = Path(".codex") / "skills"
    return [
        Path(raw.decode("utf-8")).relative_to(prefix)
        for raw in result.stdout.split(b"\0")
        if raw
    ]


def compare_skill_trees(
    source_root: Path,
    portable_root: Path,
    expected_paths: list[Path],
) -> list[str]:
    expected = set(expected_paths)
    actual = (
        {
            path.relative_to(portable_root)
            for path in portable_root.rglob("*")
            if path.is_file() or path.is_symlink()
        }
        if portable_root.is_dir()
        else set()
    )
    errors: list[str] = []

    for relative in sorted(expected - actual):
        errors.append(f"missing portable skill file: {relative.as_posix()}")
    for relative in sorted(actual - expected):
        errors.append(f"unexpected portable skill file: {relative.as_posix()}")

    for relative in sorted(expected & actual):
        source = source_root / relative
        portable = portable_root / relative
        if source.is_symlink() != portable.is_symlink():
            errors.append(f"file type differs: {relative.as_posix()}")
            continue
        if source.is_symlink():
            if source.readlink() != portable.readlink():
                errors.append(f"symlink target differs: {relative.as_posix()}")
            continue
        if source.read_bytes() != portable.read_bytes():
            errors.append(f"content differs: {relative.as_posix()}")
        source_executable = bool(source.stat().st_mode & EXECUTABLE_BITS)
        portable_executable = bool(portable.stat().st_mode & EXECUTABLE_BITS)
        if source_executable != portable_executable:
            errors.append(f"executable mode differs: {relative.as_posix()}")

    return errors


def main() -> int:
    expected = tracked_skill_paths()
    errors = compare_skill_trees(SOURCE_ROOT, PORTABLE_ROOT, expected)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"portable skills: {len(expected)} files match .codex/skills")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
