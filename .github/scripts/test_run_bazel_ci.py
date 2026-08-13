#!/usr/bin/env python3

import json
import os
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


class RunBazelCiTest(unittest.TestCase):
    def test_explicit_bazel_wrapper_job_timeouts_allow_cold_local_builds(self) -> None:
        workflows_dir = Path(__file__).parents[1] / "workflows"

        for workflow_path in workflows_dir.glob("*.yml"):
            lines = workflow_path.read_text(encoding="utf-8").splitlines()
            for line_number, line in enumerate(lines):
                if "run-bazel-ci.sh" not in line:
                    continue

                job_start = line_number
                while job_start >= 0:
                    candidate = lines[job_start]
                    if candidate.startswith("  ") and not candidate.startswith("    "):
                        break
                    job_start -= 1

                job_end = line_number + 1
                while job_end < len(lines):
                    candidate = lines[job_end]
                    if candidate.startswith("  ") and not candidate.startswith("    "):
                        break
                    job_end += 1

                timeout_lines = [
                    candidate.strip()
                    for candidate in lines[job_start:job_end]
                    if candidate.strip().startswith("timeout-minutes:")
                ]
                for timeout_line in timeout_lines:
                    timeout = int(timeout_line.split(":", 1)[1].strip())
                    self.assertGreaterEqual(
                        timeout,
                        30,
                        f"{workflow_path.name} Bazel wrapper job needs cold-build headroom",
                    )

    def test_keyless_windows_cross_compile_fails_closed(self) -> None:
        script = Path(__file__).with_name("run-bazel-ci.sh")

        with TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            capture_path = temp_path / "bazel-args.json"
            fake_bazel = temp_path / "bazel"
            fake_bazel.write_text(
                """#!/usr/bin/env python3
import json
import os
import sys
from pathlib import Path

Path(os.environ["BAZEL_ARGS_CAPTURE"]).write_text(
    json.dumps(sys.argv[1:]), encoding="utf-8"
)
""",
                encoding="utf-8",
            )
            fake_bazel.chmod(0o755)

            env = os.environ.copy()
            env.pop("BUILDBUDDY_API_KEY", None)
            env.update(
                {
                    "BAZEL_ARGS_CAPTURE": str(capture_path),
                    "CODEX_BAZEL_BIN": str(fake_bazel),
                    "CODEX_BAZEL_WINDOWS_PATH": "/usr/bin",
                    "RUNNER_OS": "Windows",
                }
            )

            result = subprocess.run(
                [
                    "bash",
                    str(script),
                    "--windows-cross-compile",
                    "--remote-download-toplevel",
                    "--",
                    "build",
                    "--",
                    "//codex-rs/cli:codex",
                ],
                cwd=script.parents[2],
                env=env,
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 2)
            self.assertIn("requires authenticated remote execution", result.stderr)
            self.assertFalse(capture_path.exists(), "Bazel must not run with a false local fallback")


if __name__ == "__main__":
    unittest.main()
