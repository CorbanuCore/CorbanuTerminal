"""Keep unapproved upstream publishing inert until a reviewed owner decision."""

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DISABLED = (
    "python-runtime-build.yml",
    "python-runtime-release.yml",
    "python-sdk-build.yml",
    "python-sdk-release.yml",
    "python-sdk-cli-release.yml",
    "rust-release-provisioned-macos.yml",
)
REMOVED = (
    "rust-release.yml",
    "r2-release.yml",
    "rust-release-windows.yml",
    "rust-release-zsh.yml",
)


class WorkflowReconciliationTest(unittest.TestCase):
    def test_every_candidate_job_is_unconditionally_disabled(self):
        for name in DISABLED:
            with self.subTest(workflow=name):
                text = (ROOT / ".github/workflows" / name).read_text()
                jobs = text.split("\njobs:\n", 1)[1]
                blocks = re.split(r"(?m)^  [\w-]+:\n", jobs)[1:]
                self.assertTrue(blocks)
                for block in blocks:
                    conditions = re.findall(r"(?m)^    if: (.*)$", block)
                    self.assertEqual(len(conditions), 1)
                    self.assertEqual(
                        conditions[0].split("#", 1)[0].strip(), "${{ false }}"
                    )

    def test_candidate_ownership_survives_sync_mode_removal(self):
        permanent = (ROOT / ".github/CODEOWNERS").read_text().split(
            "# BEGIN sync-branch security owners", 1
        )[0]
        for name in DISABLED:
            with self.subTest(workflow=name):
                self.assertIn(
                    f"/.github/workflows/{name} @corbanu-security\n", permanent
                )

    def test_d480_legacy_publishers_stay_removed(self):
        for name in REMOVED:
            with self.subTest(workflow=name):
                self.assertFalse((ROOT / ".github/workflows" / name).exists())
                for path in (ROOT / ".github/workflows").glob("*.yml"):
                    self.assertNotIn(
                        f"uses: ./.github/workflows/{name}", path.read_text()
                    )


if __name__ == "__main__":
    unittest.main()
