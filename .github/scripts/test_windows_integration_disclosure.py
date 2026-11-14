#!/usr/bin/env python3
"""#1856: the Windows gnullvm cross shard says plainly, when it skips, that its
bazel test step did not run, and docs/ci.md documents the cross shards.
Runs the detection step's own script with and without a key: its outputs (and
so every later `if:` and the job outcome) are unchanged."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github" / "workflows" / "bazel.yml"
DOCS = ROOT / "docs" / "ci.md"


def detect_script() -> str:
    """The `run: |` body of "Detect authenticated Windows gnullvm execution"."""
    lines = WORKFLOW.read_text(encoding="utf-8").splitlines()
    start = lines.index("      - name: Detect authenticated Windows gnullvm execution")
    run = next(i for i in range(start, len(lines)) if lines[i].strip() == "run: |")
    body = []
    for line in lines[run + 1 :]:
        if line.strip() and not line.startswith("          "):
            break
        body.append(line[10:])
    return "\n".join(body) + "\n"


def run_detect(key: str) -> tuple[str, str, str]:
    """(GITHUB_OUTPUT, step summary, stdout) of the step for this key."""
    with tempfile.TemporaryDirectory() as tmp:
        output = Path(tmp, "output")
        summary = Path(tmp, "summary")
        env = {
            **os.environ,
            "BUILDBUDDY_API_KEY": key,
            "GITHUB_OUTPUT": str(output),
            "GITHUB_STEP_SUMMARY": str(summary),
            "SHARD": "3",
        }
        done = subprocess.run(
            ["bash", "-c", detect_script()],
            env=env,
            check=True,
            capture_output=True,
            text=True,
        )
        read = lambda path: path.read_text(encoding="utf-8") if path.exists() else ""
        return read(output), read(summary), done.stdout


class WindowsIntegrationDisclosureTest(unittest.TestCase):
    def test_skipped_shard_discloses_its_tests_did_not_run(self) -> None:
        output, summary, stdout = run_detect("")
        self.assertEqual(output, "enabled=false\n")
        for text in (summary, stdout):
            self.assertIn("Windows gnullvm cross shard 3/4", text)
            self.assertIn("bazel test step did NOT run", text)
            self.assertIn("BuildBuddy", text)
            self.assertIn("#1856", text)
            # Only what this shard knows: no claim about other lanes' tests.
            self.assertNotIn("unit tests only", text)
        self.assertIn("::notice::", stdout)

    def test_authenticated_shard_is_unchanged(self) -> None:
        output, summary, stdout = run_detect("key")
        self.assertEqual((output, summary, stdout), ("enabled=true\n", "", ""))

    def test_later_steps_still_gate_on_the_same_output(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        gate = "steps.windows_gnullvm.outputs.enabled == 'true'"
        self.assertEqual(text.count(f"if: {gate}"), 2)
        self.assertIn(f"if: matrix.shard == 1 && {gate}", text)
        self.assertIn(f"if: always() && !cancelled() && {gate}", text)

    def test_docs_document_the_cross_shards(self) -> None:
        docs = DOCS.read_text(encoding="utf-8")
        self.assertIn(
            "(`Bazel test on windows-latest for x86_64-pc-windows-gnullvm shard N/4`,",
            docs,
        )
        self.assertIn(
            "Without it every shard skips that step and still\n  reports success", docs
        )
        # The nightly line is unchanged.
        self.assertIn("native Windows for every crate", docs)


if __name__ == "__main__":
    unittest.main(verbosity=2)
