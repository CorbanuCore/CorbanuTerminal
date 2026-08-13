#!/usr/bin/env python3

import json
import os
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


class RunBazelCiTest(unittest.TestCase):
    def test_keyless_windows_cross_compile_uses_msvc_execution_host(self) -> None:
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

            self.assertEqual(result.returncode, 0, result.stderr)
            args = json.loads(capture_path.read_text(encoding="utf-8"))
            self.assertIn("--host_platform=//:local_windows_msvc", args)
            self.assertIn("--jobs=8", args)
            self.assertIn("--remote_download_toplevel", args)
            self.assertIn("//codex-rs/cli:codex", args)


if __name__ == "__main__":
    unittest.main()
