#!/usr/bin/env python3
"""Build a canonical Codex package directory and optional archive."""

import mmap
from pathlib import Path
import sys


# Some developer environments set PYTHONSAFEPATH=1, which prevents Python from
# adding the script directory to sys.path. Add it explicitly so the local helper
# package remains importable when this executable is launched from any cwd.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from codex_package import cli
from codex_package.cargo import build_source_binaries


def distribution_source_binaries(*args, **kwargs):
    outputs = build_source_binaries(*args, **kwargs)
    # Inspect actual prebuilt/source-built bytes, independent of profile, feature
    # flags or symbols. Reject directory-only handoffs as well as archives.
    paths = set(outputs.extra_bins.values())
    paths.update(value for value in vars(outputs).values() if isinstance(value, Path))
    for path in sorted(paths):
        with path.open("rb") as binary:
            if path.stat().st_size == 0:
                continue  # Empty files cannot carry developer code.
            with mmap.mmap(binary.fileno(), 0, access=mmap.ACCESS_READ) as data:
                if data.find(b"CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION") != -1:
                    raise SystemExit(
                        f"Refusing distribution package: developer-accounting enabled in {path}"
                    )
    return outputs


def main():
    # Apply the distribution policy before CLI assembly or archive creation.
    cli.build_source_binaries = distribution_source_binaries
    return cli.main()


if __name__ == "__main__":
    raise SystemExit(main())
