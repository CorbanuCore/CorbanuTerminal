#!/usr/bin/env python3
"""Portable multi-harness runner for Corbanu coding benchmarks."""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import threading
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Iterable


BENCH_ROOT = Path(__file__).resolve().parent
DEFAULT_CONFIG = BENCH_ROOT / "configs" / "example.json"
RUNNER_PATH = Path(__file__).resolve()

# Environment variables copied from the parent environment into the isolated
# candidate environment. Everything else must be requested explicitly through
# `required_env` or `env_passthrough`.
BASE_ENV_PASSTHROUGH = ("PATH", "TERM", "LANG", "LC_ALL", "TMPDIR")
DEFAULT_MAX_AGENT_COMMANDS = 120
DEFAULT_MAX_IDENTICAL_COMMANDS = 12


@dataclass(frozen=True)
class TaskSpec:
    name: str
    baseline: Path
    prompt: Path
    verifier: Path
    timeout_seconds: int
    visible_command: tuple[str, ...]
    core_rel: str | None = None


@dataclass(frozen=True)
class AgentSpec:
    name: str
    kind: str
    binary: str
    provider: str
    model: str
    lane: str
    required_env: tuple[str, ...]
    command: tuple[str, ...] | None = None
    stdin_prompt: bool | None = None
    reasoning_effort: str | None = None
    sandbox: str | None = None
    config_overrides: tuple[str, ...] = ()
    env_passthrough: tuple[str, ...] = ()
    env: tuple[tuple[str, str], ...] = ()
    isolate_home: bool = True


@dataclass(frozen=True)
class RunSpec:
    task: TaskSpec
    agent: AgentSpec
    wave: int
    workspace: Path
    result_dir: Path


def utc_now() -> str:
    return datetime.now(UTC).isoformat(timespec="seconds").replace("+00:00", "Z")


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    if not root.exists():
        return ""
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        if "__pycache__" in path.parts or path.suffix == ".pyc":
            continue
        rel = path.relative_to(root).as_posix()
        digest.update(rel.encode())
        digest.update(b"\0")
        digest.update(sha256(path).encode())
        digest.update(b"\0")
    return digest.hexdigest()


def source_tree_digest() -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in BENCH_ROOT.rglob("*") if item.is_file()):
        rel = path.relative_to(BENCH_ROOT)
        if "runs" in rel.parts or "__pycache__" in rel.parts or path.suffix == ".pyc":
            continue
        digest.update(rel.as_posix().encode())
        digest.update(b"\0")
        digest.update(sha256(path).encode())
        digest.update(b"\0")
    return digest.hexdigest()


def expand_path(raw: str, base: Path) -> Path:
    expanded = os.path.expandvars(os.path.expanduser(raw))
    if "$" in expanded:
        raise ValueError(f"unresolved environment variable in path: {raw}")
    path = Path(expanded)
    return path.resolve() if path.is_absolute() else (base / path).resolve()


def resolve_binary(raw: str) -> str | None:
    expanded = os.path.expandvars(os.path.expanduser(raw))
    if "$" in expanded:
        return None
    if "/" in expanded or "\\" in expanded:
        path = Path(expanded)
        return str(path.resolve()) if path.is_file() else None
    return shutil.which(expanded)


def binary_is_script(path: str) -> bool:
    try:
        with open(path, "rb") as handle:
            return handle.read(2) == b"#!"
    except OSError:
        return False


def git_repo_root(path: Path) -> Path | None:
    for candidate in (path, *path.parents):
        if (candidate / ".git").exists():
            return candidate
    return None


def git_provenance(repo_dir: Path) -> dict[str, Any]:
    def run_git(*args: str) -> str | None:
        try:
            completed = subprocess.run(
                ["git", "-C", str(repo_dir), *args],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                timeout=30,
            )
        except (OSError, subprocess.TimeoutExpired):
            return None
        return completed.stdout.strip() if completed.returncode == 0 else None

    status = run_git("status", "--porcelain")
    return {
        "commit": run_git("rev-parse", "HEAD"),
        "branch": run_git("rev-parse", "--abbrev-ref", "HEAD"),
        "dirty": bool(status) if status is not None else None,
    }


def binary_provenance(binary: str | None) -> dict[str, Any]:
    if not binary or not Path(binary).is_file():
        return {"path": binary, "sha256": None, "bytes": None, "is_script_wrapper": None}
    path = Path(binary)
    return {
        "path": str(path),
        "sha256": sha256(path),
        "bytes": path.stat().st_size,
        "mtime": datetime.fromtimestamp(path.stat().st_mtime, UTC).isoformat(timespec="seconds"),
        "is_script_wrapper": binary_is_script(binary),
    }


def load_specs(config_path: Path) -> tuple[dict[str, Any], list[TaskSpec], list[AgentSpec], Path]:
    config_path = config_path.resolve()
    config = read_json(config_path)
    base = config_path.parent
    caps = config.get("caps") if isinstance(config.get("caps"), dict) else {}
    default_timeout = int(caps.get("timeout_seconds") or 1200)
    run_root = expand_path(str(config["run_dir"]), base)

    tasks = []
    for raw in config.get("tasks") or []:
        visible = raw.get("visible_command") or [
            "{python}",
            "-m",
            "unittest",
            "discover",
            "-s",
            "tests",
            "-v",
        ]
        tasks.append(
            TaskSpec(
                name=str(raw["name"]),
                baseline=expand_path(str(raw["baseline"]), base),
                prompt=expand_path(str(raw["prompt"]), base),
                verifier=expand_path(str(raw["verifier"]), base),
                timeout_seconds=int(raw.get("timeout_seconds") or default_timeout),
                visible_command=tuple(str(item) for item in visible),
                core_rel=str(raw["core_rel"]) if raw.get("core_rel") else None,
            )
        )

    agents = []
    for raw in config.get("agents") or []:
        env_map = raw.get("env") or {}
        agents.append(
            AgentSpec(
                name=str(raw["name"]),
                kind=str(raw["kind"]),
                binary=str(raw["binary"]),
                provider=str(raw.get("provider") or ""),
                model=str(raw.get("model") or ""),
                lane=str(raw.get("lane") or raw.get("provider") or raw["name"]),
                required_env=tuple(str(item) for item in raw.get("required_env") or []),
                command=tuple(str(item) for item in raw["command"]) if raw.get("command") else None,
                stdin_prompt=bool(raw["stdin_prompt"]) if "stdin_prompt" in raw else None,
                reasoning_effort=str(raw["reasoning_effort"]) if raw.get("reasoning_effort") else None,
                sandbox=str(raw["sandbox"]) if raw.get("sandbox") else None,
                config_overrides=tuple(str(item) for item in raw.get("config_overrides") or []),
                env_passthrough=tuple(str(item) for item in raw.get("env_passthrough") or []),
                env=tuple((str(key), str(value)) for key, value in env_map.items()),
                isolate_home=bool(raw.get("isolate_home", True)),
            )
        )

    if not tasks:
        raise ValueError("config must define at least one task")
    if not agents:
        raise ValueError("config must define at least one agent")
    return config, tasks, agents, run_root


def validate_inputs(
    tasks: Iterable[TaskSpec],
    agents: Iterable[AgentSpec],
    paid: bool,
    require_binaries: bool,
) -> list[str]:
    errors: list[str] = []
    for task in tasks:
        for label, path in (
            ("baseline", task.baseline),
            ("prompt", task.prompt),
            ("verifier", task.verifier),
        ):
            if not path.exists():
                errors.append(f"task {task.name}: missing {label}: {path}")
    for agent in agents:
        if require_binaries and resolve_binary(agent.binary) is None:
            errors.append(f"agent {agent.name}: binary not found: {agent.binary}")
        if agent.kind in {"corbanu", "codex"} and agent.command is None:
            expanded = os.path.expandvars(os.path.expanduser(agent.binary))
            if not os.path.isabs(expanded):
                errors.append(
                    f"agent {agent.name}: {agent.kind} binary must be an absolute path "
                    f"(PATH lookup can resolve to an env-overriding wrapper): {agent.binary}"
                )
            elif require_binaries:
                resolved = resolve_binary(agent.binary)
                if resolved and binary_is_script(resolved):
                    errors.append(
                        f"agent {agent.name}: {agent.kind} binary is a script wrapper, "
                        f"which can override CODEX_HOME isolation: {resolved}"
                    )
        if agent.kind == "corbanu" and agent.command is None and not agent.reasoning_effort:
            errors.append(
                f"agent {agent.name}: corbanu agents must pin reasoning_effort explicitly "
                "(models such as glm-5.3 silently default to max)"
            )
        if paid:
            missing = [name for name in agent.required_env if not os.environ.get(name, "").strip()]
            if missing:
                errors.append(f"agent {agent.name}: missing required environment: {', '.join(missing)}")
    return errors


def schedule(tasks: list[TaskSpec], agents: list[AgentSpec], waves: int) -> dict[str, list[tuple[TaskSpec, AgentSpec, int]]]:
    lanes: dict[str, list[AgentSpec]] = {}
    for agent in agents:
        lanes.setdefault(agent.lane, []).append(agent)
    plan: dict[str, list[tuple[TaskSpec, AgentSpec, int]]] = {}
    for lane, lane_agents in lanes.items():
        steps: list[tuple[TaskSpec, AgentSpec, int]] = []
        for task in tasks:
            for wave in range(1, waves + 1):
                for agent in lane_agents:
                    steps.append((task, agent, wave))
        plan[lane] = steps
    return plan


def plan_payload(
    config_path: Path,
    tasks: list[TaskSpec],
    agents: list[AgentSpec],
    run_root: Path,
    waves: int,
) -> dict[str, Any]:
    lanes = schedule(tasks, agents, waves)
    return {
        "config": str(config_path.resolve()),
        "run_root": str(run_root),
        "waves": waves,
        "agents": [
            {
                **asdict(agent),
                "binary_resolved": resolve_binary(agent.binary),
                "required_env": list(agent.required_env),
                "command": list(agent.command) if agent.command else None,
            }
            for agent in agents
        ],
        "tasks": [
            {
                **asdict(task),
                "baseline": str(task.baseline),
                "prompt": str(task.prompt),
                "verifier": str(task.verifier),
                "visible_command": list(task.visible_command),
            }
            for task in tasks
        ],
        "lanes": {
            lane: [
                {"task": task.name, "agent": agent.name, "wave": wave}
                for task, agent, wave in steps
            ]
            for lane, steps in lanes.items()
        },
    }


def prepare_workspace(task: TaskSpec, workspace: Path) -> dict[str, Any]:
    if workspace.exists():
        raise RuntimeError(f"refusing to reuse benchmark workspace: {workspace}")
    workspace.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(
        task.baseline,
        workspace,
        ignore=shutil.ignore_patterns("__pycache__", "*.pyc", ".pytest_cache"),
    )
    shutil.copy2(task.prompt, workspace / "BENCHMARK_TASK.md")
    core_path = workspace / task.core_rel if task.core_rel else None
    return {
        "baseline_tree_sha256": tree_digest(task.baseline),
        "workspace_tree_sha256": tree_digest(workspace),
        "core_sha256": sha256(core_path) if core_path and core_path.is_file() else None,
    }


def template_values(run: RunSpec, binary: str) -> dict[str, str]:
    return {
        "binary": binary,
        "workspace": str(run.workspace),
        "result_dir": str(run.result_dir),
        "model": run.agent.model,
        "provider": run.agent.provider,
        "python": sys.executable,
        "prompt_path": str(run.workspace / "BENCHMARK_TASK.md"),
    }


def isolated_env(run: RunSpec) -> dict[str, str]:
    """Build a minimal, isolated environment for a candidate run.

    The previous behaviour (`os.environ.copy()`) leaked the operator's HOME,
    XDG dirs, PYTHONPATH, and Python user site-packages into every candidate,
    which allowed cross-run contamination (see MODEL_EVAL_HANDOFF_2026-08-28).
    """
    env: dict[str, str] = {}
    for name in (*BASE_ENV_PASSTHROUGH, *run.agent.env_passthrough, *run.agent.required_env):
        value = os.environ.get(name)
        if value is not None:
            env[name] = value
    if run.agent.isolate_home:
        run_home = run.result_dir / "home"
        run_home.mkdir(parents=True, exist_ok=True)
        env["HOME"] = str(run_home)
        env["XDG_CONFIG_HOME"] = str(run_home / ".config")
        env["XDG_DATA_HOME"] = str(run_home / ".local" / "share")
        env["XDG_CACHE_HOME"] = str(run_home / ".cache")
        env["XDG_STATE_HOME"] = str(run_home / ".local" / "state")
        real_home = os.environ.get("HOME")
        if real_home and env.get("PATH"):
            kept = [
                part
                for part in env["PATH"].split(os.pathsep)
                if part and not part.startswith(real_home.rstrip("/") + "/") and part != real_home
            ]
            env["PATH"] = os.pathsep.join(kept)
    env["PYTHONNOUSERSITE"] = "1"
    env["NO_COLOR"] = "1"
    env.pop("PYTHONPATH", None)
    values = template_values(run, resolve_binary(run.agent.binary) or run.agent.binary)
    for key, value in run.agent.env:
        env[key] = value.format_map(values)
    return env


def build_command(run: RunSpec, prompt: str) -> tuple[list[str], dict[str, str], str | None]:
    binary = resolve_binary(run.agent.binary)
    if binary is None:
        raise RuntimeError(f"binary not found: {run.agent.binary}")
    env = isolated_env(run)
    values = template_values(run, binary)

    if run.agent.command:
        values["prompt"] = prompt
        command = [item.format_map(values) for item in run.agent.command]
        stdin = prompt if run.agent.stdin_prompt else None
        return command, env, stdin

    kind = run.agent.kind
    if kind == "corbanu":
        if binary_is_script(binary):
            raise RuntimeError(
                f"refusing script wrapper as corbanu binary (it can override CODEX_HOME): {binary}"
            )
        codex_home = run.result_dir / "corbanu-home"
        codex_home.mkdir(parents=True, exist_ok=True)
        env["CODEX_HOME"] = str(codex_home)
        env["CORBANU_TRACE_STREAM_TIMING"] = "1"
        env["CORBANU_DUMP_CHAT_REQUEST"] = str(run.result_dir / "corbanu.chat.request.json")
        env["CORBANU_DUMP_ANTHROPIC_REQUEST"] = str(
            run.result_dir / "corbanu.anthropic.request.json"
        )
        env["CORBANU_DUMP_RESPONSES_REQUEST"] = str(
            run.result_dir / "corbanu.responses.request.json"
        )
        sandbox = run.agent.sandbox or "workspace-write"
        command = [
            binary,
            "exec",
            "--json",
            "--skip-git-repo-check",
            "--ignore-user-config",
            "-C",
            str(run.workspace),
        ]
        if sandbox == "danger-bypass":
            command.append("--dangerously-bypass-approvals-and-sandbox")
        else:
            command.extend(["--sandbox", sandbox])
        if run.agent.provider:
            command.extend(["-c", f'model_provider="{run.agent.provider}"'])
        if run.agent.model:
            command.extend(["-m", run.agent.model])
        if run.agent.reasoning_effort:
            command.extend(["-c", f'model_reasoning_effort="{run.agent.reasoning_effort}"'])
        for override in run.agent.config_overrides:
            command.extend(["-c", override])
        command.append("-")
        return command, env, prompt

    if kind == "kilo":
        return (
            [
                binary,
                "run",
                "--model",
                run.agent.model,
                "--dir",
                str(run.workspace),
                "--format",
                "json",
                "--auto",
                "--dangerously-skip-permissions",
                prompt,
            ],
            env,
            None,
        )

    if kind == "hermes":
        hermes_home = run.result_dir / "hermes-home"
        hermes_home.mkdir(parents=True, exist_ok=True)
        env["HERMES_HOME"] = str(hermes_home)
        return (
            [
                binary,
                "--provider",
                run.agent.provider,
                "-m",
                run.agent.model,
                "--yolo",
                "--accept-hooks",
                "-z",
                prompt,
            ],
            env,
            None,
        )

    if kind == "codex":
        codex_home = run.result_dir / "codex-home"
        codex_home.mkdir(parents=True, exist_ok=True)
        env["CODEX_HOME"] = str(codex_home)
        sandbox = run.agent.sandbox or "workspace-write"
        command = [
            binary,
            "exec",
            "--json",
            "--skip-git-repo-check",
            "-C",
            str(run.workspace),
        ]
        if sandbox == "danger-bypass":
            command.append("--dangerously-bypass-approvals-and-sandbox")
        else:
            command.extend(["--sandbox", sandbox])
        command.extend(["-m", run.agent.model])
        if run.agent.reasoning_effort:
            command.extend(["-c", f'model_reasoning_effort="{run.agent.reasoning_effort}"'])
        for override in run.agent.config_overrides:
            command.extend(["-c", override])
        command.append(prompt)
        return command, env, None

    if kind == "claude-code":
        claude_home = run.result_dir / "claude-home"
        claude_home.mkdir(parents=True, exist_ok=True)
        env["CLAUDE_CONFIG_DIR"] = str(claude_home)
        return (
            [
                binary,
                "-p",
                "--verbose",
                "--output-format",
                "stream-json",
                "--model",
                run.agent.model,
                "--permission-mode",
                "bypassPermissions",
                "--dangerously-skip-permissions",
                prompt,
            ],
            env,
            None,
        )

    raise ValueError(f"unsupported agent kind: {kind}")


def stop_process(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    if os.name == "posix":
        os.killpg(process.pid, signal.SIGTERM)
    else:
        process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        if os.name == "posix":
            os.killpg(process.pid, signal.SIGKILL)
        else:
            process.kill()
        process.wait()


def scan_stdout_for_loops(
    stdout_path: Path,
    max_commands: int,
    max_identical: int,
) -> str | None:
    """Return a kill reason when the agent JSONL stream shows a pathological loop."""
    try:
        text = stdout_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None
    commands: dict[str, int] = {}
    total = 0
    for line in text.splitlines():
        if '"command_execution"' not in line or '"item.started"' not in line:
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        item = event.get("item") or {}
        if item.get("type") != "command_execution":
            continue
        total += 1
        key = str(item.get("command") or "")
        commands[key] = commands.get(key, 0) + 1
    if max_commands and total > max_commands:
        return f"command cap exceeded: {total} > {max_commands}"
    if max_identical:
        for key, count in commands.items():
            if count > max_identical:
                return f"identical command repeated {count} times (> {max_identical}): {key[:200]}"
    return None


def run_process(
    command: list[str],
    cwd: Path,
    env: dict[str, str],
    stdin_payload: str | None,
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
    max_commands: int = DEFAULT_MAX_AGENT_COMMANDS,
    max_identical_commands: int = DEFAULT_MAX_IDENTICAL_COMMANDS,
) -> dict[str, Any]:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    started_at = utc_now()
    started = time.monotonic()
    loop_kill_reason: list[str] = []
    monitor_stop = threading.Event()
    with stdout_path.open("w", encoding="utf-8") as stdout, stderr_path.open(
        "w", encoding="utf-8"
    ) as stderr:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdin=subprocess.PIPE if stdin_payload is not None else subprocess.DEVNULL,
            stdout=stdout,
            stderr=stderr,
            text=True,
            start_new_session=True,
        )

        def monitor() -> None:
            while not monitor_stop.wait(10):
                reason = scan_stdout_for_loops(stdout_path, max_commands, max_identical_commands)
                if reason:
                    loop_kill_reason.append(reason)
                    stop_process(process)
                    return

        monitor_thread = threading.Thread(target=monitor, daemon=True)
        monitor_thread.start()
        timed_out = False
        try:
            if stdin_payload is None:
                returncode = process.wait(timeout=timeout_seconds)
            else:
                process.communicate(stdin_payload, timeout=timeout_seconds)
                returncode = int(process.returncode or 0)
        except subprocess.TimeoutExpired:
            timed_out = True
            stop_process(process)
            returncode = int(process.returncode or 124)
        finally:
            monitor_stop.set()
            monitor_thread.join(timeout=15)
    return {
        "command": command,
        "started_at": started_at,
        "ended_at": utc_now(),
        "wall_seconds": round(time.monotonic() - started, 3),
        "returncode": returncode,
        "timed_out": timed_out,
        "loop_capped": bool(loop_kill_reason),
        "loop_cap_reason": loop_kill_reason[0] if loop_kill_reason else None,
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
    }


def file_map(root: Path) -> dict[str, str]:
    if not root.exists():
        return {}
    return {
        path.relative_to(root).as_posix(): sha256(path)
        for path in sorted(item for item in root.rglob("*") if item.is_file())
        if "__pycache__" not in path.parts and path.suffix != ".pyc"
    }


def test_integrity(task: TaskSpec, workspace: Path) -> dict[str, Any]:
    expected = file_map(task.baseline / "tests")
    actual = file_map(workspace / "tests")
    modified = sorted(path for path in expected.keys() & actual.keys() if expected[path] != actual[path])
    missing = sorted(expected.keys() - actual.keys())
    extra = sorted(actual.keys() - expected.keys())
    return {
        "ok": not modified and not missing and not extra,
        "modified": modified,
        "missing": missing,
        "extra": extra,
    }


def render_command(parts: tuple[str, ...], values: dict[str, str]) -> list[str]:
    return [part.format_map(values) for part in parts]


def run_check(command: list[str], cwd: Path, output: Path, timeout_seconds: int) -> dict[str, Any]:
    env = {
        name: value
        for name in BASE_ENV_PASSTHROUGH
        if (value := os.environ.get(name)) is not None
    }
    check_home = output.parent / "check-home"
    check_home.mkdir(parents=True, exist_ok=True)
    env["HOME"] = str(check_home)
    env["PYTHONNOUSERSITE"] = "1"
    env["PYTHONPATH"] = str(cwd / "src")
    started = time.monotonic()
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout_seconds,
        )
        text = completed.stdout
        returncode = completed.returncode
        timed_out = False
    except subprocess.TimeoutExpired as error:
        text = error.stdout if isinstance(error.stdout, str) else ""
        returncode = 124
        timed_out = True
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8")
    return {
        "command": command,
        "returncode": returncode,
        "timed_out": timed_out,
        "wall_seconds": round(time.monotonic() - started, 3),
        "output": str(output),
    }


def verify_workspace(
    run: RunSpec,
    integrity: dict[str, Any],
    source_integrity: dict[str, Any],
) -> dict[str, Any]:
    values = {
        "python": sys.executable,
        "workspace": str(run.workspace),
        "verifier": str(run.task.verifier),
    }
    visible = run_check(
        render_command(run.task.visible_command, values),
        run.workspace,
        run.result_dir / "visible.stdout",
        min(run.task.timeout_seconds, 300),
    )
    hidden = (
        run_check(
            [sys.executable, str(run.task.verifier), str(run.workspace)],
            run.workspace,
            run.result_dir / "hidden.stdout",
            min(run.task.timeout_seconds, 300),
        )
        if source_integrity["ok"]
        else {
            "returncode": None,
            "timed_out": False,
            "skipped": "benchmark source changed during the agent run",
        }
    )
    result = {
        "ok": (
            visible["returncode"] == 0
            and hidden["returncode"] == 0
            and integrity["ok"]
            and source_integrity["ok"]
        ),
        "visible": visible,
        "hidden": hidden,
        "test_integrity": integrity,
        "source_integrity": source_integrity,
    }
    write_json(run.result_dir / "verification.json", result)
    return result


def json_values(path: Path) -> list[Any]:
    if not path.is_file():
        return []
    text = path.read_text(encoding="utf-8", errors="replace")
    values: list[Any] = []
    try:
        values.append(json.loads(text))
    except json.JSONDecodeError:
        for line in text.splitlines():
            try:
                values.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return values


def collect_named_values(value: Any, names: set[str]) -> list[Any]:
    found: list[Any] = []
    if isinstance(value, dict):
        for key, child in value.items():
            if str(key).lower() in names:
                found.append(child)
            found.extend(collect_named_values(child, names))
    elif isinstance(value, list):
        for child in value:
            found.extend(collect_named_values(child, names))
    return found


def route_and_usage(run: RunSpec) -> dict[str, Any]:
    sources = [
        run.result_dir / f"{run.agent.name}.stdout",
        run.result_dir / "corbanu.chat.request.json",
        run.result_dir / "corbanu.anthropic.request.json",
        run.result_dir / "corbanu.responses.request.json",
    ]
    values = [value for path in sources for value in json_values(path)]
    models = sorted(
        {
            str(value)
            for item in values
            for value in collect_named_values(item, {"model", "model_id"})
            if isinstance(value, str)
        }
    )
    costs = [
        float(value)
        for item in values
        for value in collect_named_values(item, {"total_cost_usd", "cost_usd", "sumcost"})
        if isinstance(value, (int, float))
    ]
    reasoning_efforts = sorted(
        {
            str(value)
            for item in values
            for value in collect_named_values(item, {"reasoning_effort"})
            if isinstance(value, str)
        }
    )
    thinking_flags = sorted(
        {
            bool(value)
            for item in values
            for value in collect_named_values(item, {"enable_thinking"})
            if isinstance(value, bool)
        },
        key=str,
    )
    expected = run.agent.model
    return {
        "provider_expected": run.agent.provider,
        "model_expected": expected,
        "models_observed": models,
        "route_verified": expected in models if models else None,
        "reasoning_effort_expected": run.agent.reasoning_effort,
        "reasoning_effort_observed": reasoning_efforts,
        "reasoning_effort_verified": (
            reasoning_efforts == [run.agent.reasoning_effort]
            if run.agent.reasoning_effort and reasoning_efforts
            else None
        ),
        "enable_thinking_observed": thinking_flags,
        "native_cost_usd": max(costs) if costs else None,
        "sources": [str(path) for path in sources if path.is_file()],
    }


def agent_telemetry(run: RunSpec) -> dict[str, Any]:
    """Summarize the agent's JSONL event stream for loop/usage diagnostics."""
    stdout_path = run.result_dir / f"{run.agent.name}.stdout"
    if not stdout_path.is_file():
        return {}
    turns = 0
    commands = 0
    file_changes = 0
    errors = 0
    agent_messages = 0
    command_counts: dict[str, int] = {}
    usage_totals: dict[str, int] = {}
    for line in stdout_path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.startswith("{"):
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        kind = event.get("type")
        if kind == "turn.started":
            turns += 1
        elif kind == "turn.completed":
            usage = event.get("usage") or {}
            for key, value in usage.items():
                if isinstance(value, (int, float)):
                    usage_totals[key] = usage_totals.get(key, 0) + int(value)
        elif kind == "item.started":
            item = event.get("item") or {}
            if item.get("type") == "command_execution":
                commands += 1
                key = str(item.get("command") or "")
                command_counts[key] = command_counts.get(key, 0) + 1
        elif kind == "item.completed":
            item = event.get("item") or {}
            if item.get("type") == "file_change":
                file_changes += 1
            elif item.get("type") == "error":
                errors += 1
            elif item.get("type") == "agent_message":
                agent_messages += 1
    repeated = {key: count for key, count in command_counts.items() if count >= 3}
    return {
        "turns": turns,
        "commands": commands,
        "file_changes": file_changes,
        "errors": errors,
        "agent_messages": agent_messages,
        "usage": usage_totals,
        "repeated_commands": {key[:200]: count for key, count in sorted(repeated.items())},
    }


def isolation_evidence(run: RunSpec, env: dict[str, str]) -> dict[str, Any]:
    """Post-run proof that per-run isolation actually took effect."""
    evidence: dict[str, Any] = {
        "env_keys": sorted(env.keys()),
        "home_isolated": env.get("HOME", "").startswith(str(run.result_dir)),
        "pythonnousersite": env.get("PYTHONNOUSERSITE") == "1",
    }
    codex_home = env.get("CODEX_HOME")
    if codex_home:
        sessions = Path(codex_home) / "sessions"
        session_files = (
            [str(p) for p in sessions.rglob("*.jsonl")] if sessions.exists() else []
        )
        evidence["codex_home"] = codex_home
        evidence["codex_home_isolated"] = codex_home.startswith(str(run.result_dir))
        evidence["codex_home_session_count"] = len(session_files)
        # If the per-run CODEX_HOME never received a session, the binary most
        # likely re-exported CODEX_HOME (for example via a wrapper script).
        evidence["codex_home_used"] = bool(session_files)
    return evidence


def run_one(
    task: TaskSpec,
    agent: AgentSpec,
    wave: int,
    run_root: Path,
    expected_source_digest: str | None = None,
    caps: dict[str, Any] | None = None,
) -> dict[str, Any]:
    workspace = run_root / "workspaces" / task.name / agent.name / f"wave-{wave:03d}"
    result_dir = run_root / "results" / task.name / agent.name / f"wave-{wave:03d}"
    if result_dir.exists():
        raise RuntimeError(f"refusing to reuse benchmark result directory: {result_dir}")
    result_dir.mkdir(parents=True)
    run = RunSpec(task=task, agent=agent, wave=wave, workspace=workspace, result_dir=result_dir)
    source_before = expected_source_digest or source_tree_digest()
    baseline = prepare_workspace(task, workspace)
    prompt = task.prompt.read_text(encoding="utf-8")
    command, env, stdin_payload = build_command(run, prompt)
    caps = caps or {}
    agent_run = run_process(
        command,
        workspace,
        env,
        stdin_payload,
        result_dir / f"{agent.name}.stdout",
        result_dir / f"{agent.name}.stderr",
        task.timeout_seconds,
        max_commands=int(caps.get("max_agent_commands") or DEFAULT_MAX_AGENT_COMMANDS),
        max_identical_commands=int(
            caps.get("max_identical_commands") or DEFAULT_MAX_IDENTICAL_COMMANDS
        ),
    )
    integrity = test_integrity(task, workspace)
    source_after = source_tree_digest()
    source_integrity = {
        "ok": source_after == source_before,
        "before_sha256": source_before,
        "after_sha256": source_after,
    }
    verification = verify_workspace(run, integrity, source_integrity)
    route = route_and_usage(run)
    telemetry = agent_telemetry(run)
    isolation = isolation_evidence(run, env)
    summary = {
        "task": task.name,
        "agent": agent.name,
        "agent_kind": agent.kind,
        "wave": wave,
        "workspace": str(workspace),
        "result_dir": str(result_dir),
        "baseline": baseline,
        "binary": binary_provenance(resolve_binary(agent.binary)),
        "reasoning_effort": agent.reasoning_effort,
        "sandbox": agent.sandbox or ("workspace-write" if agent.kind in {"corbanu", "codex"} else None),
        "agent_run": agent_run,
        "verification": verification,
        "source_integrity": source_integrity,
        "route_and_usage": route,
        "telemetry": telemetry,
        "isolation": isolation,
        "passed": bool(verification["ok"]) and agent_run["returncode"] == 0,
        "workspace_tree_sha256_after": tree_digest(workspace),
    }
    write_json(result_dir / "summary.json", summary)
    print(
        json.dumps(
            {
                "task": task.name,
                "agent": agent.name,
                "wave": wave,
                "passed": summary["passed"],
                "wall_seconds": agent_run["wall_seconds"],
                "loop_capped": agent_run.get("loop_capped", False),
            },
            sort_keys=True,
        ),
        flush=True,
    )
    return summary


def collect_summaries(run_root: Path) -> list[dict[str, Any]]:
    summaries = []
    for path in sorted((run_root / "results").glob("*/*/wave-*/summary.json")):
        value = read_json(path)
        if isinstance(value, dict):
            summaries.append(value)
    return summaries


def write_report(run_root: Path) -> Path:
    rows = collect_summaries(run_root)
    lines = [
        "# Coding benchmark report",
        "",
        "| Task | Agent | Wave | Pass | Route | Wall seconds | Native cost USD | Evidence |",
        "| --- | --- | ---: | --- | --- | ---: | ---: | --- |",
    ]
    for row in rows:
        route = row.get("route_and_usage") or {}
        agent_run = row.get("agent_run") or {}
        lines.append(
            "| {task} | {agent} | {wave} | {passed} | {route} | {wall} | {cost} | `{evidence}` |".format(
                task=row.get("task"),
                agent=row.get("agent"),
                wave=row.get("wave"),
                passed=row.get("passed"),
                route=route.get("route_verified"),
                wall=agent_run.get("wall_seconds"),
                cost=route.get("native_cost_usd"),
                evidence=Path(str(row.get("result_dir"))) / "summary.json",
            )
        )
    report = run_root / "report.md"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return report


def run_campaign(
    config: dict[str, Any],
    tasks: list[TaskSpec],
    agents: list[AgentSpec],
    run_root: Path,
    waves: int,
    config_path: Path | None = None,
) -> int:
    caps = config.get("caps") if isinstance(config.get("caps"), dict) else {}
    max_runs = int(caps.get("max_total_runs") or 100)
    total = len(tasks) * len(agents) * waves
    if total > max_runs:
        raise RuntimeError(f"planned runs {total} exceed caps.max_total_runs {max_runs}")
    if run_root.exists() and any(run_root.iterdir()):
        raise RuntimeError(f"refusing to reuse nonempty run root: {run_root}")
    enclosing_repo = git_repo_root(run_root.parent if not run_root.exists() else run_root)
    if enclosing_repo is not None and not config.get("allow_run_root_in_repo"):
        raise RuntimeError(
            "run root is inside a git repository "
            f"({enclosing_repo}); candidates would inherit that repository's AGENTS.md "
            "and .codex skills. Point run_dir outside any repo or set "
            '"allow_run_root_in_repo": true to accept the contamination.'
        )
    run_root.mkdir(parents=True, exist_ok=True)
    lane_plan = schedule(tasks, agents, waves)
    expected_source_digest = source_tree_digest()
    write_json(
        run_root / "manifest.json",
        {
            "created_at": utc_now(),
            "waves": waves,
            "tasks": [task.name for task in tasks],
            "agents": [agent.name for agent in agents],
            "benchmark_source_sha256": expected_source_digest,
            "benchmark_git": git_provenance(BENCH_ROOT),
            "runner_sha256": sha256(RUNNER_PATH),
            "config_path": str(config_path) if config_path else None,
            "config_sha256": sha256(config_path) if config_path and config_path.is_file() else None,
            "caps": caps,
            "binaries": {
                agent.name: binary_provenance(resolve_binary(agent.binary)) for agent in agents
            },
            "agent_settings": [
                {
                    "name": agent.name,
                    "kind": agent.kind,
                    "provider": agent.provider,
                    "model": agent.model,
                    "reasoning_effort": agent.reasoning_effort,
                    "sandbox": agent.sandbox,
                    "isolate_home": agent.isolate_home,
                    "config_overrides": list(agent.config_overrides),
                    "required_env": list(agent.required_env),
                    "env_passthrough": list(agent.env_passthrough),
                }
                for agent in agents
            ],
            "lanes": {
                lane: [
                    {"task": task.name, "agent": agent.name, "wave": wave}
                    for task, agent, wave in steps
                ]
                for lane, steps in lane_plan.items()
            },
        },
    )

    def run_lane(steps: list[tuple[TaskSpec, AgentSpec, int]]) -> list[dict[str, Any]]:
        return [
            run_one(task, agent, wave, run_root, expected_source_digest, caps=caps)
            for task, agent, wave in steps
        ]

    results: list[dict[str, Any]] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(lane_plan)) as pool:
        futures = [pool.submit(run_lane, steps) for steps in lane_plan.values()]
        for future in concurrent.futures.as_completed(futures):
            results.extend(future.result())
    report = write_report(run_root)
    print(str(report))
    return 0 if results and all(result["passed"] for result in results) else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("plan")
    run = subparsers.add_parser("run")
    run.add_argument("--confirm-paid-run", action="store_true")
    subparsers.add_parser("report")
    args = parser.parse_args()

    config, tasks, agents, run_root = load_specs(args.config)
    waves = int(config.get("waves") or 1)
    errors = validate_inputs(
        tasks,
        agents,
        paid=args.command == "run",
        require_binaries=args.command == "run",
    )
    if errors:
        raise SystemExit("\n".join(errors))

    if args.command == "plan":
        print(json.dumps(plan_payload(args.config, tasks, agents, run_root, waves), indent=2, default=str))
        return 0
    if args.command == "report":
        print(write_report(run_root))
        return 0
    if not args.confirm_paid_run:
        raise SystemExit("live benchmark execution requires --confirm-paid-run")
    return run_campaign(config, tasks, agents, run_root, waves, config_path=args.config)


if __name__ == "__main__":
    raise SystemExit(main())
