#!/usr/bin/env python3

"""Partition the Linux `bazel test //...` target set across GitHub jobs.

Without remote execution, V8 is built from source (target and exec
configuration) in every job that needs it. Only the shards before the last
receive the targets that link the V8 crate, hashed between them; the last
shard receives every other target and so builds no V8. The shards together
cover exactly the input set; the V8 list only decides where a target runs.
"""

import argparse
import sys
import zlib
from pathlib import Path


def read_labels(path: Path) -> list[str]:
    labels = [line.strip() for line in path.read_text(encoding="utf-8").splitlines()]
    labels = [label for label in labels if label]
    if len(labels) != len(set(labels)):
        raise ValueError(f"{path}: duplicate Bazel target labels")
    if any(not label.startswith("//") for label in labels):
        raise ValueError(f"{path}: expected main-repository labels")
    return labels


def assign_shard(
    all_labels: list[str], v8_labels: list[str], shard: int, shard_count: int
) -> list[str]:
    if shard_count < 2:
        raise ValueError("shard count must be at least 2")
    if not 1 <= shard <= shard_count:
        raise ValueError("shard must be between 1 and the shard count")
    if not all_labels:
        raise ValueError("no Bazel targets were provided")
    targets = set(all_labels)
    v8 = set(v8_labels)
    if not v8:
        raise ValueError("no V8-consuming Bazel targets were provided")
    # Classification only decides which shard builds a target; the shards
    # always partition all_labels, so labels outside it are ignored.
    v8 &= targets
    if not v8:
        raise ValueError("no V8-consuming Bazel targets are in the target set")
    if shard == shard_count:
        selected = targets - v8
    else:
        selected = {
            label
            for label in v8
            if zlib.crc32(label.encode("utf-8")) % (shard_count - 1) + 1 == shard
        }
    if not selected:
        raise ValueError(f"shard {shard}/{shard_count} selected no Bazel targets")
    return sorted(selected)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--all", type=Path, required=True)
    parser.add_argument("--v8", type=Path, required=True)
    parser.add_argument("--shard", type=int, required=True)
    parser.add_argument("--shard-count", type=int, required=True)
    args = parser.parse_args(argv)
    try:
        all_labels = read_labels(args.all)
        v8_labels = read_labels(args.v8)
        extra = sorted(set(v8_labels) - set(all_labels))
        if extra:
            print(
                f"Ignoring {len(extra)} V8-consuming labels outside the target set, "
                f"e.g. {', '.join(extra[:5])}",
                file=sys.stderr,
            )
        selected = assign_shard(all_labels, v8_labels, args.shard, args.shard_count)
    except ValueError as error:
        print(f"::error::Invalid Linux Bazel shard selection: {error}", file=sys.stderr)
        return 1
    sys.stdout.write("".join(f"{label}\n" for label in selected))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
