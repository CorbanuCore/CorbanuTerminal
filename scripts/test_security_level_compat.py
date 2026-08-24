import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_level_compat as compat


class SecurityLevelCompatTests(unittest.TestCase):
    def test_extract_test_source_stops_at_next_test(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "tests.rs"
            source.write_text(
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n\n"
                "#[tokio::test]\nasync fn adjacent() {\n    assert!(false);\n}\n",
                encoding="utf-8",
            )
            self.assertEqual(
                compat.extract_test_source(source, "frozen_probe"),
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n",
            )

    def test_manifest_requires_immutable_probe_digest_and_full_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "probe.rs"
            source.write_text(
                "#[test]\nfn frozen_probe() {\n    assert!(true);\n}\n",
                encoding="utf-8",
            )
            probe = {
                "id": "frozen-probe",
                "package": "codex-example",
                "test_filter": "frozen_probe",
                "source": "probe.rs",
                "function": "frozen_probe",
                "source_sha256": compat.sha256_bytes(
                    compat.extract_test_source(source, "frozen_probe").encode("utf-8")
                ),
                "covers": ["surface"],
            }
            manifest = {
                "schema_version": 1,
                "captured_from_commit": "a" * 40,
                "composition_contract": {
                    "rule": "final_allow = existing_allow && security_layer_allow",
                    "permissive_security_layer_allow": True,
                },
                "surfaces": [{"id": "surface"}],
                "probes": [probe],
            }
            self.assertEqual(
                compat.validate_manifest(manifest, root, "a" * 40), [probe]
            )

            source.write_text(
                "#[test]\nfn frozen_probe() {\n    assert!(false);\n}\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                compat.CompatibilityError, "probe source drift"
            ):
                compat.validate_manifest(manifest, root, "a" * 40)

    @unittest.skipIf(os.name == "nt", "executable fixture uses a POSIX shebang")
    def test_candidate_identity_records_binary_hash_and_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            candidate = root / "candidate"
            candidate.write_text(
                '#!/bin/sh\n[ "$1" = "--version" ] && echo \'corbanu 1.2.3\'\n',
                encoding="utf-8",
            )
            candidate.chmod(0o755)
            identity, result = compat.candidate_identity(candidate, root)
            self.assertEqual(result.returncode, 0)
            self.assertEqual(identity["version"], "corbanu 1.2.3")
            self.assertEqual(identity["sha256"], compat.sha256_file(candidate))

    def test_write_report_replaces_complete_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            destination = compat.write_report(Path(directory), {"status": "passed"})
            self.assertEqual(
                json.loads(destination.read_text(encoding="utf-8")),
                {"status": "passed"},
            )


if __name__ == "__main__":
    unittest.main()
