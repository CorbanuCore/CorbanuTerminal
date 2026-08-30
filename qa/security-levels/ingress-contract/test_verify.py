#!/usr/bin/env python3
import hashlib
import json
import re
import shutil
import tempfile
import unittest
from pathlib import Path

import verify


SOURCE = Path(__file__).resolve().parent


class IngressContractVerifierTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name) / "ingress-contract"
        shutil.copytree(SOURCE, self.root, ignore=shutil.ignore_patterns("__pycache__"))
        self.original_root = verify.ROOT
        verify.ROOT = self.root

    def tearDown(self) -> None:
        verify.ROOT = self.original_root
        self.temporary.cleanup()

    def load(self, relative: str) -> dict:
        return json.loads((self.root / relative).read_text(encoding="utf-8"))

    def save(self, relative: str, value: dict) -> None:
        (self.root / relative).write_text(json.dumps(value), encoding="utf-8")

    def assert_verification_fails(self, message: str) -> None:
        with self.assertRaisesRegex(SystemExit, message):
            verify.main()

    def test_expected_verdict_drift_fails(self) -> None:
        manifest = self.load("manifest.json")
        manifest["cases"][1]["expected_verdict"] = "allow"
        self.save("manifest.json", manifest)
        self.assert_verification_fails("expected verdict mismatch")

    def test_quarantine_transition_drift_fails(self) -> None:
        relative = "fixtures/quarantine-v1/transitions.json"
        transitions = self.load(relative)
        transitions["transitions"][-1]["to"] = "screened_untrusted"
        self.save(relative, transitions)
        manifest = self.load("manifest.json")
        fixture = next(
            item
            for item in manifest["fixtures"]
            if item["id"] == "quarantine-v1-transitions"
        )
        fixture["sha256"] = hashlib.sha256((self.root / relative).read_bytes()).hexdigest()
        self.save("manifest.json", manifest)
        self.assert_verification_fails("quarantine v1 semantics changed")

    def test_schema_rejects_parent_traversal(self) -> None:
        schema = self.load("schema.json")
        pattern = schema["properties"]["fixtures"]["items"]["properties"]["path"][
            "pattern"
        ]
        self.assertIsNone(re.fullmatch(pattern, "fixtures/../manifest.json"))
        self.assertIsNotNone(re.fullmatch(pattern, "fixtures/benign-v1/raw.txt"))


if __name__ == "__main__":
    unittest.main()
