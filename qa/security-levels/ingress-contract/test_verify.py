#!/usr/bin/env python3
import hashlib
import json
import re
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.append(str(Path(__file__).resolve().parent))
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

    def test_schema_drift_fails(self) -> None:
        schema = self.load("schema.json")
        schema["title"] = "silently relaxed schema"
        self.save("schema.json", schema)
        self.assert_verification_fails("schema digest mismatch")

    def test_fixture_digest_drift_fails(self) -> None:
        path = self.root / "fixtures/benign-v1/raw.txt"
        path.write_bytes(path.read_bytes() + b"drift")
        self.assert_verification_fails("digest mismatch")

    def test_crlf_fixture_drift_fails(self) -> None:
        path = self.root / "fixtures/benign-v1/raw.txt"
        path.write_bytes(path.read_bytes().replace(b"\n", b"\r\n"))
        self.assert_verification_fails("digest mismatch")

    def test_unlisted_fixture_fails(self) -> None:
        (self.root / "fixtures/unlisted.txt").write_bytes(b"not in manifest")
        self.assert_verification_fails("inventory does not exactly match")

    def test_runtime_parent_traversal_fails(self) -> None:
        manifest = self.load("manifest.json")
        manifest["fixtures"][0]["path"] = "fixtures/../manifest.json"
        self.save("manifest.json", manifest)
        self.assert_verification_fails("invalid fixture entry")

    def test_duplicate_fixture_id_fails(self) -> None:
        manifest = self.load("manifest.json")
        duplicate = dict(manifest["fixtures"][0])
        duplicate["path"] = manifest["fixtures"][1]["path"]
        manifest["fixtures"].append(duplicate)
        self.save("manifest.json", manifest)
        self.assert_verification_fails("invalid fixture entry")

    def test_duplicate_fixture_path_fails(self) -> None:
        manifest = self.load("manifest.json")
        duplicate = dict(manifest["fixtures"][0])
        duplicate["id"] = "unique-id-with-duplicate-path"
        manifest["fixtures"].append(duplicate)
        self.save("manifest.json", manifest)
        self.assert_verification_fails("invalid fixture entry")

    def test_missing_fixture_fails_cleanly(self) -> None:
        (self.root / "fixtures/benign-v1/raw.txt").unlink()
        self.assert_verification_fails("missing fixture")

    def test_missing_required_case_fixture_fails_cleanly(self) -> None:
        manifest = self.load("manifest.json")
        manifest["fixtures"] = [
            item for item in manifest["fixtures"] if item["id"] != "benign-v1-raw"
        ]
        (self.root / "fixtures/benign-v1/raw.txt").unlink()
        self.save("manifest.json", manifest)
        self.assert_verification_fails("required fixture is missing")

    def test_fixture_identity_drift_fails(self) -> None:
        manifest = self.load("manifest.json")
        manifest["fixture_verdict_identity"]["artifact_sha256"] = "0" * 64
        self.save("manifest.json", manifest)
        self.assert_verification_fails("model or threshold identity changed")

    def test_fixture_symlink_fails(self) -> None:
        fixture = self.root / "fixtures/benign-v1/raw.txt"
        outside = Path(self.temporary.name) / "outside.txt"
        outside.write_bytes(fixture.read_bytes())
        fixture.unlink()
        try:
            fixture.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"symlinks unavailable: {error}")
        self.assert_verification_fails("fixture path contains symlink")

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
