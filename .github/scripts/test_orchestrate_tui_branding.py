#!/usr/bin/env python3

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MATRIX_SCRIPT = REPO_ROOT / "qa" / "orchestrate_tui_matrix.sh"


class OrchestrateTuiBrandingTest(unittest.TestCase):
    def test_default_product_label_matches_corbanu_ui(self) -> None:
        script = MATRIX_SCRIPT.read_text()
        match = re.search(
            r"^TERMINAL_LABEL=\$\{PFTERMINAL_QA_PRODUCT_LABEL:-(.+)\}$",
            script,
            re.MULTILINE,
        )
        self.assertIsNotNone(match, "TUI matrix must retain an overridable product label")
        self.assertEqual(match.group(1), "Corbanu Terminal")


if __name__ == "__main__":
    unittest.main()
