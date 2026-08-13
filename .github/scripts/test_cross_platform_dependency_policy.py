#!/usr/bin/env python3

import tomllib
import unittest
from pathlib import Path


class CrossPlatformDependencyPolicyTest(unittest.TestCase):
    def test_ctor_does_not_enable_unused_destructor_support(self) -> None:
        workspace_manifest = Path(__file__).parents[2] / "codex-rs" / "Cargo.toml"
        manifest = tomllib.loads(workspace_manifest.read_text(encoding="utf-8"))
        ctor = manifest["workspace"]["dependencies"]["ctor"]

        self.assertFalse(ctor["default-features"])
        self.assertEqual(ctor["features"], ["proc_macro"])


if __name__ == "__main__":
    unittest.main()
