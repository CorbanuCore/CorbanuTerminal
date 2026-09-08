#!/usr/bin/env python3
"""Keep provider refusal handling independent of regular expressions."""

from pathlib import Path
import unittest


class ProviderPolicyContractTest(unittest.TestCase):
    def test_provider_error_paths_have_no_regex_dependency(self):
        root = Path(__file__).resolve().parents[1]
        for relative in (
            "codex-rs/codex-api/src/api_bridge.rs",
            "codex-rs/codex-api/src/sse/responses.rs",
            "codex-rs/codex-api/src/endpoint/responses_websocket.rs",
        ):
            source = (root / relative).read_text()
            for dependency in ("regex::", "regex_lite::", "Regex::"):
                self.assertNotIn(dependency, source, relative)


if __name__ == "__main__":
    unittest.main()
