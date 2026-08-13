#!/usr/bin/env python3

import tomllib
import unittest
from pathlib import Path


class CrossPlatformDependencyPolicyTest(unittest.TestCase):
    def test_ctor_uses_linktime_macro_implementation(self) -> None:
        workspace_manifest = Path(__file__).parents[2] / "codex-rs" / "Cargo.toml"
        manifest = tomllib.loads(workspace_manifest.read_text(encoding="utf-8"))
        ctor = manifest["workspace"]["dependencies"]["ctor"]

        self.assertEqual(ctor, "1.0.6")

        lockfile = (workspace_manifest.parent / "Cargo.lock").read_text(encoding="utf-8")
        self.assertNotIn('name = "ctor-proc-macro"', lockfile)
        self.assertNotIn('name = "dtor-proc-macro"', lockfile)


if __name__ == "__main__":
    unittest.main()
