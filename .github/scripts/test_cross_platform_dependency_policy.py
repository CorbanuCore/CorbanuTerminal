#!/usr/bin/env python3

import re
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

    def test_bazel_proc_macros_are_partitioned_into_exec_dependencies(self) -> None:
        root = Path(__file__).parents[2]
        resolver_patch = (root / "patches" / "rules_rs_proc_macro_deps.patch").read_text(
            encoding="utf-8"
        )
        manifest_patch = (
            root / "patches" / "rules_rs_proc_macro_manifest.patch"
        ).read_text(encoding="utf-8")
        module = (root / "MODULE.bazel").read_text(encoding="utf-8")

        self.assertIn("def _split_proc_macro_deps", resolver_patch)
        self.assertIn("proc_macro_deps_select", resolver_patch)
        self.assertIn("proc_macro_deps = proc_macro_deps", resolver_patch)

        declared = re.findall(r'^\+    "([^"]+)": True,$', manifest_patch, re.MULTILINE)
        self.assertEqual(len(declared), len(set(declared)))
        self.assertGreaterEqual(len(declared), 90)
        for crate in (
            "include_dir_macros",
            "linktime-proc-macro",
            "serde_derive",
            "thiserror-impl",
            "tokio-macros",
        ):
            self.assertIn(crate, declared)

        resolver_index = module.index('"//patches:rules_rs_proc_macro_deps.patch"')
        manifest_index = module.index('"//patches:rules_rs_proc_macro_manifest.patch"')
        self.assertLess(resolver_index, manifest_index)


if __name__ == "__main__":
    unittest.main()
