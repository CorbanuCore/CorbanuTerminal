import io
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory

from select_linux_bazel_shard import assign_shard, main


ALL = [f"//codex-rs/crate{i}:test" for i in range(40)] + [
    "//codex-rs/core:core-all-test",
    "//codex-rs/app-server:app-server-all-test",
    "//codex-rs/code-mode-runtime:code-mode-runtime",
]
V8 = [label for label in ALL if not label.startswith("//codex-rs/crate")] + [
    f"//codex-rs/crate{i}:test" for i in range(5)
]


class SelectLinuxBazelShardTest(unittest.TestCase):
    def test_shards_partition_the_complete_target_set(self):
        for shard_count in (2, 3, 4):
            with self.subTest(shard_count=shard_count):
                shards = [
                    assign_shard(ALL, V8, shard, shard_count)
                    for shard in range(1, shard_count + 1)
                ]
                selected = [label for shard in shards for label in shard]
                self.assertEqual(sorted(selected), sorted(ALL))
                self.assertEqual(len(selected), len(set(selected)))

    def test_only_the_last_shard_receives_non_v8_targets(self):
        last = assign_shard(ALL, V8, 3, 3)
        self.assertEqual(set(last), set(ALL) - set(V8))
        earlier = assign_shard(ALL, V8, 1, 3) + assign_shard(ALL, V8, 2, 3)
        self.assertEqual(set(earlier), set(V8))

    def test_assignment_is_deterministic_and_order_independent(self):
        first = assign_shard(ALL, V8, 1, 3)
        self.assertEqual(first, assign_shard(list(reversed(ALL)), sorted(V8), 1, 3))
        self.assertEqual(first, sorted(first))

    def test_invalid_selection_is_rejected(self):
        for all_labels, v8_labels, shard, shard_count in (
            (ALL, V8, 1, 1),
            (ALL, V8, 0, 3),
            (ALL, V8, 4, 3),
            ([], V8, 1, 3),
            (ALL, [], 1, 3),
            (ALL, V8 + ["//codex-rs/other:missing"], 1, 3),
            (V8, V8, 3, 3),
        ):
            with self.subTest(shard=shard, shard_count=shard_count), self.assertRaises(
                ValueError
            ):
                assign_shard(all_labels, v8_labels, shard, shard_count)

    def test_cli_prints_labels_and_rejects_bad_files(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "all.txt").write_text("\n".join(ALL) + "\n", encoding="utf-8")
            (root / "v8.txt").write_text("\n".join(V8) + "\n", encoding="utf-8")
            args = ["--all", str(root / "all.txt"), "--v8", str(root / "v8.txt")]
            stdout = io.StringIO()
            with redirect_stdout(stdout):
                self.assertEqual(main(args + ["--shard", "3", "--shard-count", "3"]), 0)
            self.assertEqual(
                stdout.getvalue().splitlines(), sorted(set(ALL) - set(V8))
            )

            (root / "dup.txt").write_text("//a:b\n//a:b\n", encoding="utf-8")
            (root / "ext.txt").write_text("@repo//a:b\n", encoding="utf-8")
            for bad in ("dup.txt", "ext.txt"):
                with self.subTest(bad=bad):
                    stdout, stderr = io.StringIO(), io.StringIO()
                    with redirect_stdout(stdout), redirect_stderr(stderr):
                        status = main(
                            ["--all", str(root / bad), "--v8", str(root / "v8.txt")]
                            + ["--shard", "1", "--shard-count", "3"]
                        )
                    self.assertEqual(status, 1)
                    self.assertEqual(stdout.getvalue(), "")
                    self.assertIn("::error::", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
