#!/usr/bin/env python3

from __future__ import annotations

import re
import unittest

import rust_toolchain_bazel_sha256s as pins


class ArchiveKeyTest(unittest.TestCase):
    def test_keys_match_rules_rust_names(self) -> None:
        # The names rules_rust looks up in `sha256s` (produce_tool_suburl).
        triple = "x86_64-pc-windows-gnullvm"
        self.assertEqual(
            pins.archive("rustc", "1.99.1", triple),
            "rustc-1.99.1-x86_64-pc-windows-gnullvm.tar.xz",
        )
        self.assertEqual(
            pins.archive("rustc-dev", "nightly/2025-09-18", triple),
            "2025-09-18/rustc-dev-nightly-x86_64-pc-windows-gnullvm.tar.xz",
        )

    def test_toolchain_repository_archives(self) -> None:
        repo = {
            "rule": "rust_toolchain_tools_repository",
            "exec_triple": "x86_64-pc-windows-msvc",
            "target_triple": "x86_64-pc-windows-gnullvm",
            "version": "nightly/2025-09-18",
            "rustfmt_version": "nightly/2026-07-16",
            "dev_components": True,
        }
        self.assertEqual(
            pins.archives(repo),
            {
                "2025-09-18/rustc-nightly-x86_64-pc-windows-msvc.tar.xz",
                "2025-09-18/clippy-nightly-x86_64-pc-windows-msvc.tar.xz",
                "2025-09-18/cargo-nightly-x86_64-pc-windows-msvc.tar.xz",
                "2025-09-18/llvm-tools-nightly-x86_64-pc-windows-msvc.tar.xz",
                "2026-07-16/rustfmt-nightly-x86_64-pc-windows-msvc.tar.xz",
                # Target-side components use the target triple.
                "2025-09-18/rust-std-nightly-x86_64-pc-windows-gnullvm.tar.xz",
                "2025-09-18/rustc-dev-nightly-x86_64-pc-windows-gnullvm.tar.xz",
            },
        )
        bare = {**repo, "rustfmt_version": "nightly"}
        self.assertIn(
            "2025-09-18/rustfmt-nightly-x86_64-pc-windows-msvc.tar.xz",
            pins.archives(bare),
        )
        stable = {**repo, "version": "1.99.1", "dev_components": False}
        self.assertNotIn(
            "rustc-dev-1.99.1-x86_64-pc-windows-gnullvm.tar.xz", pins.archives(stable)
        )

    def test_release_of_key(self) -> None:
        self.assertEqual(
            pins.release("2025-09-18/rustc-nightly-x86_64-unknown-linux-gnu.tar.xz"),
            "nightly/2025-09-18",
        )
        self.assertEqual(
            pins.release("rust-std-1.99.1-x86_64-pc-windows-gnullvm.tar.xz"), "1.99.1"
        )

    def test_module_bazel_block_is_well_formed(self) -> None:
        # The block the script rewrites exists once and every toolchain
        # declaration of the rules_rust extension uses it.
        text = pins.MODULE.read_text()
        self.assertEqual(text.count(pins.BEGIN), 1)
        self.assertEqual(text.count(pins.END), 1)
        block = text.split(pins.BEGIN)[1].split(pins.END)[0]
        entries = re.findall(r'^    "([^"]+)": "([0-9a-f]{64})",$', block, re.MULTILINE)
        self.assertTrue(entries)
        self.assertEqual(len(entries), len(block.strip().splitlines()) - 2)
        self.assertEqual([key for key, _ in entries], sorted(key for key, _ in entries))
        toolchain = re.search(r"nightly_rust\.toolchain\(\n(.*?)\n\)", text, re.DOTALL)
        self.assertIsNotNone(toolchain)
        self.assertIn("sha256s = RUST_TOOLCHAIN_SHA256S,", toolchain.group(1))
        for name in (
            "rust_windows_x86_64",
            "rust_windows_x86_64_gnullvm",
            "rust_windows_x86_64_gnullvm_stable",
        ):
            first_call = re.search(
                r'nightly_rust\.repository_set\(\n    name = "'
                + name
                + r'",\n(.*?)\n\)',
                text,
                re.DOTALL,
            )
            self.assertIsNotNone(first_call, name)
            self.assertIn(
                "sha256s = RUST_TOOLCHAIN_SHA256S,", first_call.group(1), name
            )

    def test_every_declared_release_is_pinned(self) -> None:
        # A cheap pull-request guard: a Rust version changed in MODULE.bazel
        # without rerunning the script has no pins. It can't see a new exec or
        # target triple, or a new rules_rust default rustfmt nightly after a
        # rules_rs bump; the nightly Bazel job's full `--check` catches those.
        text = pins.MODULE.read_text()
        declared = set()
        for call in re.findall(
            r"^nightly_rust\.\w+\((.*?)^\)", text, re.DOTALL | re.MULTILINE
        ):
            for versions in re.findall(r"versions = \[([^\]]*)\]", call):
                declared.update(re.findall(r'"([^"]+)"', versions))
        self.assertTrue(declared)
        block = text.split(pins.BEGIN)[1].split(pins.END)[0]
        pinned = {
            pins.release(key)
            for key in re.findall(r'^    "([^"]+)":', block, re.MULTILINE)
        }
        self.assertLessEqual(
            declared, pinned, "rerun .github/scripts/rust_toolchain_bazel_sha256s.py"
        )


if __name__ == "__main__":
    unittest.main()
