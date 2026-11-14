from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


RUNNER_PATH = Path(__file__).resolve().parents[1] / "runner.py"
SPEC = importlib.util.spec_from_file_location("corbanu_benchmark_runner", RUNNER_PATH)
assert SPEC and SPEC.loader
runner = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = runner
SPEC.loader.exec_module(runner)


class CodingBenchmarkRunnerTests(unittest.TestCase):
    def test_lane_schedule_serializes_agents_by_wave(self) -> None:
        task = runner.TaskSpec(
            name="demo",
            baseline=Path("/baseline"),
            prompt=Path("/prompt"),
            verifier=Path("/verifier"),
            timeout_seconds=10,
            visible_command=("python",),
        )
        agents = [
            runner.AgentSpec(
                name=name,
                kind=name,
                binary=name,
                provider="provider",
                model="model",
                lane="shared",
                required_env=(),
            )
            for name in ("corbanu", "hermes", "kilo")
        ]
        plan = runner.schedule([task], agents, waves=2)
        self.assertEqual(
            [(agent.name, wave) for _, agent, wave in plan["shared"]],
            [
                ("corbanu", 1),
                ("hermes", 1),
                ("kilo", 1),
                ("corbanu", 2),
                ("hermes", 2),
                ("kilo", 2),
            ],
        )

    def test_end_to_end_custom_agent_preserves_tests_and_passes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            baseline = root / "baseline"
            source = baseline / "src" / "demo"
            tests = baseline / "tests"
            source.mkdir(parents=True)
            tests.mkdir()
            (source / "__init__.py").write_text("VALUE = 0\n", encoding="utf-8")
            (tests / "test_visible.py").write_text(
                "import unittest\n"
                "from demo import VALUE\n\n"
                "class VisibleTests(unittest.TestCase):\n"
                "    def test_value(self):\n"
                "        self.assertEqual(VALUE, 1)\n",
                encoding="utf-8",
            )
            prompt = root / "task_prompt.md"
            prompt.write_text("Set demo.VALUE to 1.\n", encoding="utf-8")
            verifier = root / "verify.py"
            verifier.write_text(
                "import pathlib, sys\n"
                "text = (pathlib.Path(sys.argv[1]) / 'src/demo/__init__.py').read_text()\n"
                "raise SystemExit(0 if 'VALUE = 1' in text else 1)\n",
                encoding="utf-8",
            )
            task = runner.TaskSpec(
                name="demo",
                baseline=baseline,
                prompt=prompt,
                verifier=verifier,
                timeout_seconds=30,
                visible_command=(
                    "{python}",
                    "-m",
                    "unittest",
                    "discover",
                    "-s",
                    "tests",
                    "-v",
                ),
                core_rel="src/demo/__init__.py",
            )
            script = (
                "from pathlib import Path; "
                "Path('src/demo/__init__.py').write_text('VALUE = 1\\n', encoding='utf-8')"
            )
            agent = runner.AgentSpec(
                name="fake",
                kind="codex",
                binary=sys.executable,
                provider="local",
                model="fake-model",
                lane="local",
                required_env=(),
                command=("{binary}", "-c", script),
            )
            summary = runner.run_one(task, agent, 1, root / "run")
            self.assertTrue(summary["passed"])
            self.assertTrue(summary["verification"]["test_integrity"]["ok"])
            saved = json.loads(
                (root / "run/results/demo/fake/wave-001/summary.json").read_text()
            )
            self.assertTrue(saved["passed"])

    def test_verifier_is_not_executed_after_source_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            workspace = root / "workspace"
            tests = workspace / "tests"
            tests.mkdir(parents=True)
            (tests / "test_visible.py").write_text(
                "import unittest\n"
                "class VisibleTests(unittest.TestCase):\n"
                "    def test_ok(self): self.assertTrue(True)\n",
                encoding="utf-8",
            )
            marker = root / "verifier-ran"
            verifier = root / "verify.py"
            verifier.write_text(
                "from pathlib import Path\n"
                f"Path({str(marker)!r}).write_text('ran')\n",
                encoding="utf-8",
            )
            task = runner.TaskSpec(
                name="demo",
                baseline=workspace,
                prompt=root / "prompt.md",
                verifier=verifier,
                timeout_seconds=30,
                visible_command=(
                    "{python}",
                    "-m",
                    "unittest",
                    "discover",
                    "-s",
                    "tests",
                ),
            )
            agent = runner.AgentSpec(
                name="fake",
                kind="codex",
                binary=sys.executable,
                provider="local",
                model="fake-model",
                lane="local",
                required_env=(),
            )
            run = runner.RunSpec(
                task=task,
                agent=agent,
                wave=1,
                workspace=workspace,
                result_dir=root / "result",
            )
            result = runner.verify_workspace(
                run,
                {"ok": True, "modified": [], "missing": [], "extra": []},
                {
                    "ok": False,
                    "before_sha256": "before",
                    "after_sha256": "after",
                },
            )
            self.assertFalse(result["ok"])
            self.assertEqual(
                result["hidden"]["skipped"],
                "benchmark source changed during the agent run",
            )
            self.assertFalse(marker.exists())


if __name__ == "__main__":
    unittest.main()
