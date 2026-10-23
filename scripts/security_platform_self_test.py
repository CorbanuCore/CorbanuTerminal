#!/usr/bin/env python3
"""Run the platform self-test with bounded, non-sensitive failure diagnostics."""

import subprocess
import sys

import security_platform_probe as probe


def main() -> int:
    try:
        probe.self_test()
    except Exception as error:
        category = "self_test_error"
        if isinstance(error, probe.ContractError) and error.args == (
            "boot_identity_unavailable",
        ):
            category = "boot_identity_cause_unknown"
            if isinstance(error.__cause__, subprocess.TimeoutExpired):
                category = "boot_identity_timeout"
            elif isinstance(error.__cause__, OSError):
                category = "boot_identity_os_error"
        # Exception text can contain commands, paths or captured process output.
        # Only classify the actual explicit cause; do not infer from elapsed time.
        print(
            f"security-platform-self-test: failed; category={category}", file=sys.stderr
        )
        return 1
    print("security-platform-probe: 8 contract regressions passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
