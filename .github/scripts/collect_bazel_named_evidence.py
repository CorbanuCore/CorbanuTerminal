#!/usr/bin/env python3
"""Write sanitized named evidence for one Bazel invocation, never raw logs."""

import argparse
import json
from pathlib import Path

from bazel_evidence_bep import collect
from bazel_evidence_io import EvidenceError, bounded_read


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("bep", "testlogs-root-file", "invocation-id", "checkout-sha", "output"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--workflow-outcome", default="unknown")
    args = parser.parse_args()
    root = None
    try:
        value = bounded_read(args.testlogs_root_file, 8192).decode("utf-8").strip()
        if value and "\n" not in value and "\r" not in value and Path(value).is_absolute():
            root = value
    except (EvidenceError, UnicodeError, ValueError):
        pass
    report = collect(args.bep, root, args.invocation_id, args.checkout_sha, args.workflow_outcome)
    try:
        with Path(args.output).open("x", encoding="utf-8") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    except (OSError, ValueError):
        print("Named result capture: output unavailable")
        return 1
    print("Named result capture: " + report["captureStatus"])
    return 0 if report["captureStatus"] == "complete" else 1


if __name__ == "__main__":
    raise SystemExit(main())
