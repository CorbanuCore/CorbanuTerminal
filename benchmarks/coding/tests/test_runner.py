from __future__ import annotations

import importlib.util
import json
import os
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


class IsolationAndPinningTests(unittest.TestCase):
    def _agent(self, **overrides) -> "runner.AgentSpec":
        values = dict(
            name="corbanu",
            kind="corbanu",
            binary="corbanu-debug",
            provider="zai",
            model="glm-5.3",
            lane="lane",
            required_env=(),
        )
        values.update(overrides)
        return runner.AgentSpec(**values)

    def test_validate_rejects_relative_corbanu_binary(self) -> None:
        errors = runner.validate_inputs([], [self._agent(reasoning_effort="low")], paid=False, require_binaries=False)
        self.assertTrue(any("absolute path" in error for error in errors))

    def test_validate_requires_explicit_reasoning_effort(self) -> None:
        errors = runner.validate_inputs(
            [], [self._agent(binary="/usr/bin/true")], paid=False, require_binaries=False
        )
        self.assertTrue(any("reasoning_effort" in error for error in errors))

    def test_validate_rejects_script_wrapper_binary(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            wrapper = Path(temporary) / "corbanu-debug"
            wrapper.write_text("#!/bin/sh\nexport CODEX_HOME=/tmp\nexec real \"$@\"\n")
            wrapper.chmod(0o755)
            errors = runner.validate_inputs(
                [],
                [self._agent(binary=str(wrapper), reasoning_effort="low")],
                paid=False,
                require_binaries=True,
            )
            self.assertTrue(any("script wrapper" in error for error in errors))

    def test_isolated_env_scrubs_operator_state(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            run = runner.RunSpec(
                task=runner.TaskSpec(
                    name="demo",
                    baseline=root / "baseline",
                    prompt=root / "prompt.md",
                    verifier=root / "verify.py",
                    timeout_seconds=10,
                    visible_command=("true",),
                ),
                agent=self._agent(binary="/usr/bin/true", reasoning_effort="low"),
                wave=1,
                workspace=root / "workspace",
                result_dir=root / "result",
            )
            real_home = os.environ.get("HOME", "")
            env = runner.isolated_env(run)
            self.assertEqual(env.get("PYTHONNOUSERSITE"), "1")
            self.assertNotIn("PYTHONPATH", env)
            self.assertTrue(env["HOME"].startswith(str(root)))
            if real_home:
                for part in env.get("PATH", "").split(os.pathsep):
                    self.assertFalse(
                        part == real_home or part.startswith(real_home.rstrip("/") + "/"),
                        f"operator home leaked into PATH: {part}",
                    )

    def test_corbanu_command_pins_sandbox_and_reasoning(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fake_binary = root / "corbanu-debug"
            fake_binary.write_bytes(b"\x7fELF fake")
            fake_binary.chmod(0o755)
            run = runner.RunSpec(
                task=runner.TaskSpec(
                    name="demo",
                    baseline=root / "baseline",
                    prompt=root / "prompt.md",
                    verifier=root / "verify.py",
                    timeout_seconds=10,
                    visible_command=("true",),
                ),
                agent=self._agent(binary=str(fake_binary), reasoning_effort="low"),
                wave=1,
                workspace=root / "workspace",
                result_dir=root / "result",
            )
            command, env, stdin = runner.build_command(run, "prompt text")
            self.assertIn("--sandbox", command)
            self.assertIn("workspace-write", command)
            self.assertIn("--ignore-user-config", command)
            self.assertNotIn("--dangerously-bypass-approvals-and-sandbox", command)
            self.assertIn('model_reasoning_effort="low"', command)
            self.assertTrue(env["CODEX_HOME"].startswith(str(root / "result")))
            self.assertEqual(stdin, "prompt text")

    def test_corbanu_command_refuses_script_wrapper(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            wrapper = root / "corbanu-debug"
            wrapper.write_text("#!/bin/sh\nexec true\n")
            wrapper.chmod(0o755)
            run = runner.RunSpec(
                task=runner.TaskSpec(
                    name="demo",
                    baseline=root / "baseline",
                    prompt=root / "prompt.md",
                    verifier=root / "verify.py",
                    timeout_seconds=10,
                    visible_command=("true",),
                ),
                agent=self._agent(binary=str(wrapper), reasoning_effort="low"),
                wave=1,
                workspace=root / "workspace",
                result_dir=root / "result",
            )
            with self.assertRaisesRegex(RuntimeError, "script wrapper"):
                runner.build_command(run, "prompt")

    def test_loop_scan_flags_repeated_commands(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            stdout_path = Path(temporary) / "agent.stdout"
            lines = [
                json.dumps(
                    {
                        "type": "item.started",
                        "item": {"type": "command_execution", "command": "python -m unittest"},
                    }
                )
                for _ in range(6)
            ]
            stdout_path.write_text("\n".join(lines) + "\n")
            reason = runner.scan_stdout_for_loops(stdout_path, max_commands=100, max_identical=5)
            self.assertIsNotNone(reason)
            self.assertIn("identical command", reason)
            self.assertIsNone(
                runner.scan_stdout_for_loops(stdout_path, max_commands=100, max_identical=10)
            )


if __name__ == "__main__":
    unittest.main()
