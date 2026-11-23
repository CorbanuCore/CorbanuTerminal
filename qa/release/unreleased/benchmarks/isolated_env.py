#!/usr/bin/env python3
"""Run one benchmark agent with per-run state directories and frozen settings.

Used as the first element of an explicit `command` template in the runner
configs next to this file, because explicit templates bypass the runner's
built-in per-agent home isolation. Never pass credential values here: agents
inherit provider keys from the runner's environment.

Usage:
  isolated_env.py [--home NAME=DIR]... [--home-under NAME=BASE::KEYDIR]...
                  [--set NAME=VALUE]...
                  [--set-rel NAME=FILE]... -- PROGRAM [ARG]...

--home creates DIR (parents included) and exports NAME=DIR.
--home-under NAME=BASE::KEYDIR creates BASE/run-<sha256(KEYDIR)[:16]>, exports
NAME as that path and records it in KEYDIR/NAME.path. Both directories must
exist. Hermes uses this: a home under <installed root>/profiles/ shares the
installed dependency environment instead of copying ~420 MB into every run.
--set exports NAME=VALUE.
--set-rel exports NAME as the absolute path of FILE relative to this script's
directory, and fails if FILE does not exist.
"""

from __future__ import annotations

import hashlib
import os
import sys
from pathlib import Path


def split_assignment(flag: str, raw: str) -> tuple[str, str]:
    name, sep, value = raw.partition("=")
    if not sep or not name or not value:
        raise SystemExit(f"{flag} expects NAME=VALUE, got {raw!r}")
    return name, value


def main(argv: list[str]) -> int:
    if "--" not in argv:
        raise SystemExit("missing -- before the program to run")
    split = argv.index("--")
    options, program = argv[:split], argv[split + 1 :]
    if not program:
        raise SystemExit("missing program after --")
    if len(options) % 2:
        raise SystemExit("every option needs exactly one NAME=VALUE argument")
    here = Path(__file__).resolve().parent
    updates: dict[str, str] = {}
    for flag, raw in zip(options[::2], options[1::2]):
        name, value = split_assignment(flag, raw)
        if flag == "--home":
            path = Path(value).resolve()
            path.mkdir(parents=True, exist_ok=True)
            updates[name] = str(path)
        elif flag == "--home-under":
            base_raw, sep, key_raw = value.partition("::")
            base, key_dir = Path(base_raw).resolve(), Path(key_raw).resolve()
            if not sep or not base.is_dir() or not key_dir.is_dir():
                raise SystemExit(
                    f"--home-under expects NAME=EXISTING_BASE::EXISTING_DIR, got {raw!r}"
                )
            digest = hashlib.sha256(str(key_dir).encode()).hexdigest()[:16]
            path = base / f"run-{digest}"
            path.mkdir(exist_ok=True)
            (key_dir / f"{name}.path").write_text(f"{path}\n", encoding="utf-8")
            updates[name] = str(path)
        elif flag == "--set":
            updates[name] = value
        elif flag == "--set-rel":
            path = (here / value).resolve()
            if not path.is_file():
                raise SystemExit(f"--set-rel file not found: {path}")
            updates[name] = str(path)
        else:
            raise SystemExit(f"unknown option {flag!r}")
    os.environ.update(updates)
    os.execvp(program[0], program)
    return 127


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
