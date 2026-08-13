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

    def test_bazel_uses_upstream_proc_macro_exec_routing(self) -> None:
        root = Path(__file__).parents[2]
        module = (root / "MODULE.bazel").read_text(encoding="utf-8")

        self.assertIn('bazel_dep(name = "rules_rs", version = "0.0.102")', module)
        self.assertIn('module_name = "rules_rs"', module)
        self.assertIn('version = "0.0.102"', module)
        self.assertNotIn("rules_rs_proc_macro_deps.patch", module)
        self.assertNotIn("rules_rs_proc_macro_manifest.patch", module)
        self.assertNotIn("rules_rust_build_script_tools_transition.patch", module)

        for obsolete_patch in (
            "rules_rs_proc_macro_deps.patch",
            "rules_rs_proc_macro_manifest.patch",
            "rules_rust_build_script_tools_transition.patch",
        ):
            self.assertFalse((root / "patches" / obsolete_patch).exists())


if __name__ == "__main__":
    unittest.main()
