#!/usr/bin/env python3
"""Summarize a tag's main ledger and every sibling split, excluding previews."""

import argparse
from collections import Counter
import json
from pathlib import Path
import re

from ledger import COLS


def summarize(main_ledger):
    if not re.fullmatch(r"ledger-rust-v\d+\.\d+\.\d+\.tsv", main_ledger.name):
        raise ValueError("pass the main ledger, ledger-rust-v<version>.tsv")
    # Require the main file; a typo must not silently report zero rows.
    paths = [main_ledger, *sorted(main_ledger.parent.glob(f"{main_ledger.stem}-*.tsv"))]
    counts = {key: Counter() for key in ("status", "kind", "slice")}
    total = 0
    for path in paths:
        with path.open(encoding="utf-8") as source:
            lines = (
                (number, line.rstrip("\n"))
                for number, line in enumerate(source, 1)
                if line.strip() and not line.startswith("#")
            )
            _, header = next(lines, (0, ""))
            if header.split("\t") != COLS:
                raise ValueError(f"{path}: unexpected ledger columns")
            for number, line in lines:
                values = line.split("\t")
                if len(values) != len(COLS):
                    raise ValueError(f"{path}:{number}: unexpected column count")
                row = dict(zip(COLS, values))
                if row["status"] not in ("open", "resolved"):
                    raise ValueError(f"{path}:{number}: unexpected status")
                total += 1
                for key, counter in counts.items():
                    counter[row[key]] += 1
    return {"rows": total, **{key: dict(value) for key, value in counts.items()}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ledger", type=Path, help="main ledger-<tag>.tsv path")
    args = parser.parse_args()
    try:
        summary = summarize(args.ledger)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
