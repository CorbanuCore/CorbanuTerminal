"""Validate manual lane selection without changing normal Bazel CI coverage."""

import json
import os
import sys
from pathlib import Path


def boolean_input(raw: str, name: str, *, optional: bool = False) -> bool:
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(f"{name} must be a JSON boolean") from error
    # Reusable callers do not expose the manual-only input. Missing expression
    # properties serialize as an empty string or null in GitHub Actions.
    if optional and (value is None or value == ""):
        return False
    if type(value) is not bool:
        raise ValueError(f"{name} must be a JSON boolean")
    return value


def select_lanes(event: str, cold_raw: str, gnu_raw: str) -> dict[str, str]:
    cold = boolean_input(cold_raw, "cold_bazel")
    gnu_only = boolean_input(gnu_raw, "linux_gnu_only", optional=True)
    if gnu_only and event != "workflow_dispatch":
        raise ValueError("linux_gnu_only is reserved for manual dispatch")
    if gnu_only and not cold:
        raise ValueError("linux_gnu_only requires cold_bazel=true")
    targets = ["x86_64-unknown-linux-gnu"]
    if not gnu_only:
        targets.append("x86_64-unknown-linux-musl")
    return {
        "run_cold": str(cold).lower(),
        "run_full": str(not gnu_only).lower(),
        "test_matrix": json.dumps(
            {
                "include": [
                    {"os": "ubuntu-24.04", "target": target} for target in targets
                ]
            },
            separators=(",", ":"),
        ),
    }


def main() -> int:
    try:
        selection = select_lanes(
            os.environ["GITHUB_EVENT_NAME"],
            os.environ["BAZEL_COLD_INPUT"],
            os.environ.get("BAZEL_GNU_ONLY_INPUT", "null"),
        )
    except (KeyError, ValueError) as error:
        print(f"::error::Invalid Bazel lane selection: {error}", file=sys.stderr)
        return 1
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
        output.writelines(f"{name}={value}\n" for name, value in selection.items())
    mode = "full default coverage" if selection["run_full"] == "true" else "GNU only"
    print(f"Bazel lane selection: {mode}; cold={selection['run_cold']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
