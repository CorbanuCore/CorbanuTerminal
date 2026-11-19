import contextlib
import io
import json
import math
import os
import shlex
import signal
import subprocess
import sys
import time
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch

import bazel_resource_sampler as sampler


class ResourceMetricsTest(unittest.TestCase):
    def test_only_allowlisted_numeric_counters_are_emitted(self) -> None:
        contents = {
            sampler.MEMINFO: "MemTotal: 100 kB\nMemAvailable: 40 kB\n"
            "SwapTotal: 20 kB\nSwapFree: 10 kB\nSecret: TOKEN123\n"
            "MemAvailable: TOKEN123 kB\n",
            sampler.CGROUP / "memory.current": "123\n",
            sampler.CGROUP / "memory.max": "max\n",
            sampler.CGROUP / "memory.swap.current": "4\n",
            sampler.CGROUP / "memory.swap.max": "TOKEN123\n",
            sampler.CGROUP / "memory.events": "oom 1\noom_kill 2\nsecret TOKEN123\n",
        }
        with (
            patch.object(
                sampler, "read_counter_file", side_effect=lambda p: contents.get(p, "")
            ),
            patch.object(
                sampler.shutil, "disk_usage", return_value=SimpleNamespace(free=500)
            ),
            patch.object(
                sampler.os, "getloadavg", return_value=(1.5, math.inf, math.nan)
            ),
            patch.dict(
                os.environ, {"BAZEL_OUTPUT_USER_ROOT": "/private/TOKEN123"}, clear=True
            ),
        ):
            metrics = sampler.collect_metrics()
        self.assertEqual(
            metrics,
            {
                "memory_total_bytes": 102400,
                "memory_available_bytes": 40960,
                "swap_total_bytes": 20480,
                "swap_free_bytes": 10240,
                "cgroup_memory_current_bytes": 123,
                "cgroup_swap_current_bytes": 4,
                "cgroup_memory_oom_events": 1,
                "cgroup_memory_oom_kill_events": 2,
                "workspace_disk_free_bytes": 500,
                "bazel_disk_free_bytes": 500,
                "load_1_minutes": 1.5,
            },
        )
        self.assertNotIn("TOKEN123", json.dumps(metrics))

    def test_missing_metrics_do_not_become_zero_or_leak_errors(self) -> None:
        with (
            patch.object(sampler, "read_counter_file", return_value=""),
            patch.object(sampler.shutil, "disk_usage", side_effect=OSError("TOKEN123")),
            patch.object(sampler.os, "getloadavg", side_effect=OSError("TOKEN123")),
        ):
            self.assertEqual(sampler.collect_metrics(), {})
        stderr, stdout = io.StringIO(), io.StringIO()
        with (
            contextlib.redirect_stderr(stderr),
            contextlib.redirect_stdout(stdout),
            patch.object(
                sampler, "collect_metrics", side_effect=RuntimeError("TOKEN123")
            ),
        ):
            self.assertTrue(sampler.emit_sample())
        self.assertEqual(stdout.getvalue(), "")
        self.assertEqual(stderr.getvalue(), "Bazel resource metrics unavailable\n")

    def test_numeric_sample_uses_only_stderr(self) -> None:
        stderr, stdout = io.StringIO(), io.StringIO()
        with (
            contextlib.redirect_stderr(stderr),
            contextlib.redirect_stdout(stdout),
            patch.object(
                sampler, "collect_metrics", return_value={"memory_available_bytes": 12}
            ),
        ):
            self.assertTrue(sampler.emit_sample())
        self.assertEqual(stdout.getvalue(), "")
        self.assertEqual(
            json.loads(stderr.getvalue().removeprefix(sampler.PREFIX)),
            {"memory_available_bytes": 12},
        )

    def test_sampler_does_not_outlive_its_original_parent(self) -> None:
        with (
            patch.object(sys, "argv", ["sampler", "123"]),
            patch.object(sampler.os, "getppid", side_effect=[123, 1]),
            patch.object(sampler, "emit_sample", return_value=True) as emit,
            patch.object(sampler.time, "sleep") as sleep,
        ):
            sampler.main()
        emit.assert_called_once_with()
        sleep.assert_called_once_with(15)


@unittest.skipUnless(os.name == "posix", "Bazel Linux CI uses POSIX process signals")
class ResourceWrapperTest(unittest.TestCase):
    def run_wrapper(
        self, codes, *, sampler_fails=False, terminate=False, stop_signal=signal.SIGTERM
    ):
        script = Path(__file__).with_name("run-bazel-ci.sh")
        with TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pid_file, count_file, ready = (
                root / "sampler-pid",
                root / "calls",
                root / "ready",
            )
            shim = root / "python-shim.py"
            shim.write_text(
                "import os, sys\nfrom pathlib import Path\n"
                "if len(sys.argv) > 1 and Path(sys.argv[1]).name == 'bazel_resource_sampler.py':\n"
                f"    Path({str(pid_file)!r}).write_text(str(os.getpid()))\n"
                f"    if {sampler_fails!r}: sys.exit(9)\n"
                "os.execv(sys.executable, [sys.executable, *sys.argv[1:]])\n",
                encoding="utf-8",
            )
            python = root / "python3"
            python.write_text(
                f'#!/bin/sh\nexec {shlex.quote(sys.executable)} {shlex.quote(str(shim))} "$@"\n',
                encoding="utf-8",
            )
            python.chmod(0o755)
            bazel = root / "fake-bazel"
            bazel.write_text(
                "#!/usr/bin/env python3\nimport sys, time\nfrom pathlib import Path\n"
                f"pid_file, counter, ready = map(Path, {[str(pid_file), str(count_file), str(ready)]!r})\n"
                "deadline = time.monotonic() + 10\n"
                "while not pid_file.exists():\n"
                "    if time.monotonic() > deadline: sys.exit(88)\n"
                "    time.sleep(0.01)\n"
                "calls = int(counter.read_text()) + 1 if counter.exists() else 1\n"
                "counter.write_text(str(calls))\n"
                "ready.write_text('ready')\n"
                f"if {terminate!r}: time.sleep(60)\n"
                f"codes = {codes!r}\n"
                "sys.exit(codes[min(calls, len(codes)) - 1])\n",
                encoding="utf-8",
            )
            bazel.chmod(0o755)
            # Synthetic environment only: no credentials or shared runtime home.
            env = {
                "PATH": str(root) + os.pathsep + os.defpath,
                "GITHUB_ACTIONS": "true",
                "RUNNER_OS": "Linux",
                "CODEX_BAZEL_BIN": str(bazel),
                "BAZEL_OUTPUT_USER_ROOT": str(root),
            }
            process = subprocess.Popen(
                ["bash", str(script), "--", "build", "--", "//fake:target"],
                cwd=script.parents[2],
                env=env,
                start_new_session=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            try:
                if terminate:
                    deadline = time.monotonic() + 10
                    while not ready.exists() and process.poll() is None:
                        self.assertLess(
                            time.monotonic(), deadline, "fake Bazel did not start"
                        )
                        time.sleep(0.01)
                    self.assertTrue(ready.exists())
                    # This isolated process group contains only this test's children.
                    os.killpg(process.pid, stop_signal)
                stdout, stderr = process.communicate(timeout=15)
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.communicate(timeout=5)
            self.assertTrue(pid_file.exists(), stderr)
            sampler_pid = int(pid_file.read_text())
            with self.assertRaises(ProcessLookupError, msg="sampler was not reaped"):
                os.kill(sampler_pid, 0)
            self.assertNotIn(sampler.PREFIX, stdout)
            return process.returncode, int(count_file.read_text()), stderr

    def test_success_failure_and_retry_status_are_preserved(self) -> None:
        for codes, expected, calls in [
            ([0], 0, 1),
            ([3], 3, 1),
            ([37, 0], 0, 2),
            ([37, 37], 37, 2),
        ]:
            with self.subTest(codes=codes):
                actual, actual_calls, stderr = self.run_wrapper(codes)
                self.assertEqual(actual, expected, stderr)
                self.assertEqual(actual_calls, calls)

    def test_sampler_failure_does_not_replace_bazel_failure(self) -> None:
        code, calls, stderr = self.run_wrapper([3], sampler_fails=True)
        self.assertEqual(code, 3, stderr)
        self.assertEqual(calls, 1)

    def test_signals_are_preserved_and_sampler_is_reaped(self) -> None:
        for stop_signal in (signal.SIGTERM, signal.SIGINT):
            with self.subTest(signal=stop_signal):
                code, calls, stderr = self.run_wrapper(
                    [0], terminate=True, stop_signal=stop_signal
                )
                self.assertEqual(code, -stop_signal, stderr)
                self.assertEqual(calls, 1)


if __name__ == "__main__":
    unittest.main()
