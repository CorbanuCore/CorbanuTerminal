import contextlib
import io
import subprocess
import sys
import types
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_platform_probe as probe
import security_platform_self_test as runner
from test_security_platform_probe import synthetic_report


class SecurityPlatformSelfTestTests(unittest.TestCase):
    def capture(self):
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = runner.main()
        return code, stdout.getvalue(), stderr.getvalue()

    @contextlib.contextmanager
    def windows_identity(self, *, response=None, error=None, executable=True):
        def windows_directory(buffer, length):
            buffer.value = r"C:\SyntheticWindows"
            return len(buffer.value)

        kernel = types.SimpleNamespace(GetWindowsDirectoryW=windows_directory)
        with contextlib.ExitStack() as patches:
            patches.enter_context(
                mock.patch.object(probe, "target_os", return_value="windows")
            )
            for name in ("node", "system", "release", "machine"):
                patches.enter_context(
                    mock.patch.object(probe.platform, name, return_value="synthetic")
                )
            patches.enter_context(
                mock.patch.object(
                    probe.ctypes,
                    "windll",
                    types.SimpleNamespace(kernel32=kernel),
                    create=True,
                )
            )
            patches.enter_context(
                mock.patch.object(probe.Path, "is_file", return_value=executable)
            )
            query = patches.enter_context(
                mock.patch.object(
                    probe.subprocess, "run", return_value=response, side_effect=error
                )
            )
            # Exercise the real identity failure through the real self_test,
            # without measuring any capability or launching a child process.
            report = patches.enter_context(
                mock.patch.object(
                    probe,
                    "run_probe",
                    side_effect=lambda label: probe.target_identity(),
                )
            )
            yield query, report

    def assert_failure(self, captured, category):
        self.assertEqual(
            captured,
            (1, "", f"security-platform-self-test: failed; category={category}\n"),
        )

    def test_real_self_test_success_delegates_once_without_live_probes(self):
        with synthetic_report("self-test") as report, contextlib.ExitStack() as patches:
            measured = patches.enter_context(
                mock.patch.object(probe, "run_probe", return_value=report)
            )
            delegated = patches.enter_context(
                mock.patch.object(probe, "self_test", wraps=probe.self_test)
            )
            guards = [
                patches.enter_context(
                    mock.patch.object(
                        probe, name, side_effect=AssertionError("live entrypoint")
                    )
                )
                for name in (
                    "target_identity",
                    "target_metadata",
                    "start_worker",
                    "run_worker",
                    "internal_worker",
                    "probe_ipc",
                    "probe_network",
                    "probe_signing",
                    "probe_elevation",
                    "probe_store",
                )
            ]
            child = patches.enter_context(
                mock.patch.object(
                    probe.subprocess, "run", side_effect=AssertionError("child process")
                )
            )
            self.assertEqual(
                self.capture(),
                (0, "security-platform-probe: 8 contract regressions passed\n", ""),
            )
            delegated.assert_called_once_with()
            measured.assert_called_once_with("self-test")
            child.assert_not_called()
            for guard in guards:
                guard.assert_not_called()
            self.assertFalse(report["protected_mode_eligible"])

    def test_real_windows_timeout_has_safe_category_and_no_retry(self):
        failure = subprocess.TimeoutExpired(
            ["PRIVATE_COMMAND"], 8, output="PRIVATE_STDOUT", stderr="PRIVATE_STDERR"
        )
        with self.windows_identity(error=failure) as (query, report):
            self.assert_failure(self.capture(), "boot_identity_timeout")
            query.assert_called_once()
            report.assert_called_once_with("self-test")
            self.assertEqual(query.call_args.kwargs["timeout"], 8)
            self.assertEqual(query.call_args.kwargs["stderr"], subprocess.DEVNULL)
            self.assertFalse(query.call_args.kwargs["check"])

    def test_real_windows_os_error_has_safe_category_and_no_retry(self):
        failure = OSError(5, "PRIVATE_ERROR", "PRIVATE_PATH")
        with self.windows_identity(error=failure) as (query, report):
            self.assert_failure(self.capture(), "boot_identity_os_error")
            query.assert_called_once()
            report.assert_called_once_with("self-test")

    def test_real_windows_nonzero_and_empty_output_keep_cause_unknown(self):
        for code, output in ((5, "PRIVATE_STDOUT"), (0, "")):
            with self.subTest(returncode=code):
                response = subprocess.CompletedProcess(
                    ["PRIVATE_COMMAND"], code, output, "PRIVATE_STDERR"
                )
                with self.windows_identity(response=response) as (query, report):
                    self.assert_failure(self.capture(), "boot_identity_cause_unknown")
                    query.assert_called_once()
                    report.assert_called_once_with("self-test")

    def test_real_windows_missing_executable_keeps_cause_unknown(self):
        with self.windows_identity(executable=False) as (query, report):
            self.assert_failure(self.capture(), "boot_identity_cause_unknown")
            query.assert_not_called()
            report.assert_called_once_with("self-test")

    def test_implicit_context_and_unrecognized_explicit_cause_are_not_guessed(self):
        for explicit in (None, RuntimeError("PRIVATE_CAUSE")):
            with self.subTest(explicit=type(explicit).__name__):
                error = probe.ContractError("boot_identity_unavailable")
                error.__context__ = subprocess.TimeoutExpired(["PRIVATE_COMMAND"], 8)
                error.__cause__ = explicit
                with mock.patch.object(
                    probe, "self_test", side_effect=error
                ) as delegated:
                    self.assert_failure(self.capture(), "boot_identity_cause_unknown")
                    delegated.assert_called_once_with()

    def test_other_errors_fail_closed_without_exception_text_or_traceback(self):
        for error in (
            ValueError("PRIVATE_VALUE"),
            TypeError("PRIVATE_TYPE"),
            OSError(5, "PRIVATE_ERROR", "PRIVATE_PATH"),
            AssertionError("PRIVATE_ASSERTION"),
            RuntimeError("PRIVATE_RUNTIME"),
            probe.ContractError("PRIVATE_CONTRACT"),
        ):
            with self.subTest(error=type(error).__name__):
                with mock.patch.object(
                    probe, "self_test", side_effect=error
                ) as delegated:
                    self.assert_failure(self.capture(), "self_test_error")
                    delegated.assert_called_once_with()

    def test_cause_type_alone_cannot_relabel_an_unrelated_contract_error(self):
        error = probe.ContractError("wrong_probe_identity")
        error.__cause__ = subprocess.TimeoutExpired(["PRIVATE_COMMAND"], 8)
        with mock.patch.object(probe, "self_test", side_effect=error) as delegated:
            self.assert_failure(self.capture(), "self_test_error")
            delegated.assert_called_once_with()


if __name__ == "__main__":
    unittest.main()
