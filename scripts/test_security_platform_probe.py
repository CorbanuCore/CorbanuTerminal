import contextlib
import copy
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_platform_probe as probe


class SecurityPlatformProbeTests(unittest.TestCase):
    def test_contract_regressions(self) -> None:
        probe.self_test()

    def test_malformed_result_uses_stable_error_path(self) -> None:
        report = probe.run_probe("malformed-result-test")
        report["capabilities"][0]["status"] = []
        with tempfile.TemporaryDirectory() as directory:
            evidence = Path(directory) / "evidence.json"
            evidence.write_text(json.dumps(report), encoding="utf-8")
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                self.assertEqual(
                    probe.main(["--validate-evidence", str(evidence)]),
                    1,
                )
        output = stderr.getvalue()
        self.assertTrue(output.startswith("security-platform-probe: TypeError:"))
        self.assertNotIn("Traceback", output)

    def test_unknown_os_has_no_unbound_boot_identity(self) -> None:
        with mock.patch.object(probe, "target_os", return_value="unknown"):
            with self.assertRaisesRegex(
                probe.ContractError, "boot_identity_unavailable"
            ):
                probe.target_identity()

    def test_require_eligible_applies_to_validation_modes(self) -> None:
        report = probe.run_probe("require-eligible-test")
        self.assertFalse(report["protected_mode_eligible"])
        with tempfile.TemporaryDirectory() as directory:
            evidence = Path(directory) / "evidence.json"
            evidence.write_text(json.dumps(report), encoding="utf-8")
            for mode in ("--validate", "--validate-evidence"):
                with self.subTest(mode=mode):
                    with contextlib.redirect_stdout(io.StringIO()):
                        self.assertEqual(
                            probe.main([mode, str(evidence), "--require-eligible"]),
                            2,
                        )

    def test_target_metadata_matches_schema_constraints(self) -> None:
        report = probe.run_probe("target-metadata-test")
        for field, value in (("cpu", []), ("architecture", "x" * 65)):
            with self.subTest(field=field):
                malformed = copy.deepcopy(report)
                malformed["target"][field] = value
                with self.assertRaisesRegex(
                    probe.ContractError, "invalid_target_metadata"
                ):
                    probe.validate_report(malformed)

    def test_windows_elevation_context_is_fail_closed(self) -> None:
        with mock.patch.object(probe, "target_os", return_value="windows"):
            cases = (
                (True, "allowed", "worker_already_elevated"),
                (
                    False,
                    "unavailable",
                    "worker_unelevated_no_noninteractive_attempt",
                ),
            )
            for elevated, outcome, code in cases:
                with self.subTest(elevated=elevated):
                    with mock.patch.object(
                        probe, "windows_token_is_elevated", return_value=elevated
                    ):
                        self.assertEqual(
                            probe.internal_worker("elevation", {}),
                            {"outcome": outcome, "code": code},
                        )
            for error_type in (OSError, AttributeError, TypeError, ValueError):
                with self.subTest(error_type=error_type.__name__):
                    with mock.patch.object(
                        probe,
                        "windows_token_is_elevated",
                        side_effect=error_type("synthetic"),
                    ):
                        self.assertEqual(
                            probe.internal_worker("elevation", {}),
                            {
                                "outcome": "error",
                                "code": "windows_token_probe_error",
                            },
                        )


if __name__ == "__main__":
    unittest.main()
