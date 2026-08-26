import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_credential_canary as canary


class SecurityCredentialCanaryTests(unittest.TestCase):
    def test_sanitized_environment_removes_credential_material(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.dict(
                os.environ,
                {
                    "PATH": "/bin",
                    "OPENAI_API_KEY": "credential-value",
                    "GITHUB_TOKEN": "credential-value",
                    "SERVICE_PASSWORD": "credential-value",
                    "SAFE_SETTING": "kept",
                },
                clear=True,
            ):
                result = canary.sanitized_environment(Path(directory))
        self.assertEqual(result["PATH"], "/bin")
        self.assertEqual(result["SAFE_SETTING"], "kept")
        self.assertEqual(result["TMPDIR"], directory)
        self.assertNotIn("OPENAI_API_KEY", result)
        self.assertNotIn("GITHUB_TOKEN", result)
        self.assertNotIn("SERVICE_PASSWORD", result)

    def test_secret_shaped_output_fails_closed(self) -> None:
        with self.assertRaisesRegex(
            canary.QualificationError, "credential-shaped material"
        ):
            canary.assert_secret_free(
                "Authorization: Bearer sk-escaped-credential-value", "stdout"
            )

    def test_validate_probe_output_requires_each_named_test(self) -> None:
        probe = canary.Probe(
            probe_id="fixture",
            package="fixture",
            cargo_args=("--lib", "credential"),
            expected_tests=("first_case", "second_case"),
            source_paths=("fixture.rs",),
            covers=("surface",),
        )
        passed = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=(
                "test module::first_case ... ok\n"
                "test module::second_case ... ok\n"
                "test result: ok. 2 passed; 0 failed; 0 ignored\n"
            ),
            stderr="",
        )
        self.assertEqual(canary.validate_probe_output(probe, passed), 2)

        missing = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=(
                "test module::first_case ... ok\n"
                "test result: ok. 1 passed; 0 failed; 0 ignored\n"
            ),
            stderr="",
        )
        with self.assertRaisesRegex(
            canary.QualificationError, "did not execute expected tests"
        ):
            canary.validate_probe_output(probe, missing)

    def test_parse_canary_result_requires_exact_surface_and_use_counts(self) -> None:
        payload = {
            "canary_sha256": "a" * 64,
            "outgoing_request_count": 1,
            "raw_secret_observations": 1,
            "scanned_surfaces": sorted(canary.REQUIRED_CANARY_SURFACES),
        }
        result = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=f"{canary.CANARY_SENTINEL}{json.dumps(payload)}\n",
            stderr="",
        )
        self.assertEqual(canary.parse_canary_result([result]), payload)

        payload["outgoing_request_count"] = 2
        result = canary.CommandResult(
            command=["cargo", "test"],
            returncode=0,
            stdout=f"{canary.CANARY_SENTINEL}{json.dumps(payload)}\n",
            stderr="",
        )
        with self.assertRaisesRegex(
            canary.QualificationError, "exactly one outgoing request"
        ):
            canary.parse_canary_result([result])

    def test_build_candidate_rejects_non_workspace_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            arbitrary = root / "arbitrary"
            arbitrary.write_text("fixture", encoding="utf-8")
            with self.assertRaisesRegex(
                canary.QualificationError, "must be the workspace binary"
            ):
                canary.build_candidate(root, arbitrary, {})

    def test_write_report_is_atomic_and_secret_free(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            destination = canary.write_report(
                Path(directory), {"status": "passed", "digest": "a" * 64}
            )
            self.assertEqual(
                json.loads(destination.read_text(encoding="utf-8"))["status"],
                "passed",
            )
            with self.assertRaisesRegex(
                canary.QualificationError, "credential-shaped material"
            ):
                canary.write_report(
                    Path(directory),
                    {"status": "failed", "output": "Bearer sk-leaked-test-secret"},
                )


if __name__ == "__main__":
    unittest.main()
