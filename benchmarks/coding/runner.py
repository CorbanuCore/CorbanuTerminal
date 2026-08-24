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
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Iterable


BENCH_ROOT = Path(__file__).resolve().parent
DEFAULT_CONFIG = BENCH_ROOT / "configs" / "example.json"


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


def build_command(run: RunSpec, prompt: str) -> tuple[list[str], dict[str, str], str | None]:
    binary = resolve_binary(run.agent.binary)
    if binary is None:
        raise RuntimeError(f"binary not found: {run.agent.binary}")
    env = os.environ.copy()
    env["NO_COLOR"] = "1"
    values = template_values(run, binary)

    if run.agent.command:
        values["prompt"] = prompt
        command = [item.format_map(values) for item in run.agent.command]
        stdin = prompt if run.agent.stdin_prompt else None
        return command, env, stdin

    kind = run.agent.kind
    if kind == "corbanu":
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
        command = [
            binary,
            "exec",
            "--json",
            "--skip-git-repo-check",
            "--dangerously-bypass-approvals-and-sandbox",
            "-C",
            str(run.workspace),
        ]
        if run.agent.provider:
            command.extend(["-c", f'model_provider="{run.agent.provider}"'])
        if run.agent.model:
            command.extend(["-m", run.agent.model])
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
        return (
            [
                binary,
                "exec",
                "--json",
                "--skip-git-repo-check",
                "--dangerously-bypass-approvals-and-sandbox",
                "-C",
                str(run.workspace),
                "-m",
                run.agent.model,
                prompt,
            ],
            env,
            None,
        )

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


def run_process(
    command: list[str],
    cwd: Path,
    env: dict[str, str],
    stdin_payload: str | None,
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
) -> dict[str, Any]:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    started_at = utc_now()
    started = time.monotonic()
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
    return {
        "command": command,
        "started_at": started_at,
        "ended_at": utc_now(),
        "wall_seconds": round(time.monotonic() - started, 3),
        "returncode": returncode,
        "timed_out": timed_out,
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
    env = os.environ.copy()
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
    expected = run.agent.model
    return {
        "provider_expected": run.agent.provider,
        "model_expected": expected,
        "models_observed": models,
        "route_verified": expected in models if models else None,
        "native_cost_usd": max(costs) if costs else None,
        "sources": [str(path) for path in sources if path.is_file()],
    }


def run_one(
    task: TaskSpec,
    agent: AgentSpec,
    wave: int,
    run_root: Path,
    expected_source_digest: str | None = None,
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
    agent_run = run_process(
        command,
        workspace,
        env,
        stdin_payload,
        result_dir / f"{agent.name}.stdout",
        result_dir / f"{agent.name}.stderr",
        task.timeout_seconds,
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
    summary = {
        "task": task.name,
        "agent": agent.name,
        "agent_kind": agent.kind,
        "wave": wave,
        "workspace": str(workspace),
        "result_dir": str(result_dir),
        "baseline": baseline,
        "agent_run": agent_run,
        "verification": verification,
        "source_integrity": source_integrity,
        "route_and_usage": route,
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
) -> int:
    max_runs = int((config.get("caps") or {}).get("max_total_runs") or 100)
    total = len(tasks) * len(agents) * waves
    if total > max_runs:
        raise RuntimeError(f"planned runs {total} exceed caps.max_total_runs {max_runs}")
    if run_root.exists() and any(run_root.iterdir()):
        raise RuntimeError(f"refusing to reuse nonempty run root: {run_root}")
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
            run_one(task, agent, wave, run_root, expected_source_digest)
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
    return run_campaign(config, tasks, agents, run_root, waves)


if __name__ == "__main__":
    raise SystemExit(main())
