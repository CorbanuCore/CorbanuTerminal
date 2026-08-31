import copy
import json
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from security_upstream_seams_check import ManifestError, validate_manifest  # noqa: E402


class SecurityUpstreamSeamsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = json.loads(
            (ROOT / "qa/security-levels/upstream-seams.json").read_text()
        )

    def test_committed_manifest_is_valid(self):
        validate_manifest(self.manifest, ROOT)

    def test_missing_required_hook_field_fails(self):
        manifest = copy.deepcopy(self.manifest)
        del manifest["seams"][0]["upstream_symbol"]
        with self.assertRaises(ManifestError):
            validate_manifest(manifest, ROOT)

    def test_missing_owner_commit_command_or_evidence_fails(self):
        for field in ("owner", "upstream_revision", "regression_command", "evidence"):
            with self.subTest(field=field):
                manifest = copy.deepcopy(self.manifest)
                manifest["seams"][0][field] = ""
                with self.assertRaises(ManifestError):
                    validate_manifest(manifest, ROOT)

    def test_filename_without_exact_symbol_fails(self):
        manifest = copy.deepcopy(self.manifest)
        manifest["seams"][0]["upstream_symbol"] = "ToolRouter::not_a_real_symbol"
        with self.assertRaises(ManifestError):
            validate_manifest(manifest, ROOT)

    def test_pending_requires_blocker_and_verified_forbids_one(self):
        pending = copy.deepcopy(self.manifest)
        pending["seams"][0]["blocker"] = None
        with self.assertRaises(ManifestError):
            validate_manifest(pending, ROOT)

        verified = copy.deepcopy(self.manifest)
        verified["seams"][2]["blocker"] = "not empty"
        with self.assertRaises(ManifestError):
            validate_manifest(verified, ROOT)

    def test_all_required_categories_and_requalification_steps_are_required(self):
        missing_category = copy.deepcopy(self.manifest)
        missing_category["seams"] = [
            seam for seam in missing_category["seams"] if seam["category"] != "egress"
        ]
        with self.assertRaises(ManifestError):
            validate_manifest(missing_category, ROOT)

        missing_command = copy.deepcopy(self.manifest)
        missing_command["requalification_commands"] = ["true"]
        with self.assertRaises(ManifestError):
            validate_manifest(missing_command, ROOT)


if __name__ == "__main__":
    unittest.main()
