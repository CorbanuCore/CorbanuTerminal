from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCANNER_PATH = Path(__file__).resolve().parents[1] / "scan_exact_keys.py"
SPEC = importlib.util.spec_from_file_location("benchmark_key_scanner", SCANNER_PATH)
assert SPEC and SPEC.loader
scanner = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = scanner
SPEC.loader.exec_module(scanner)


class ExactKeyScannerTests(unittest.TestCase):
    def test_detects_secret_across_chunk_boundary(self) -> None:
        canary = b"benchmark-canary-value"
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "artifact.bin"
            path.write_bytes(b"x" * (1024 * 1024 - 5) + canary + b"tail")
            found, scanned = scanner.contains_secret(path, [canary])
            self.assertTrue(found)
            self.assertGreaterEqual(scanned, 1024 * 1024)

    def test_clean_file_has_no_hit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "artifact.txt"
            path.write_text("safe benchmark artifact", encoding="utf-8")
            found, scanned = scanner.contains_secret(path, [b"not-present"])
            self.assertFalse(found)
            self.assertEqual(scanned, path.stat().st_size)


if __name__ == "__main__":
    unittest.main()
