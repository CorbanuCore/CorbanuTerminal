#!/usr/bin/env python3
"""Run the frozen Corbanu versus Claude Code website benchmark."""

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
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


HARNESS_ROOT = Path(__file__).resolve().parent
BASELINE = HARNESS_ROOT / "baseline"
PROMPT_PATH = HARNESS_ROOT / "task_prompt.md"
VERIFIER = HARNESS_ROOT / "verify_site.py"


def utc_now() -> str:
    return datetime.now(UTC).isoformat(timespec="seconds").replace("+00:00", "Z")


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tree_sha256(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        if "__pycache__" in path.parts or path.suffix == ".pyc":
            continue
        rel = path.relative_to(root).as_posix()
        digest.update(rel.encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def harness_source_sha256() -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in HARNESS_ROOT.rglob("*") if item.is_file()):
        rel = path.relative_to(HARNESS_ROOT)
        if "runs" in rel.parts or "__pycache__" in rel.parts or path.suffix == ".pyc":
            continue
        digest.update(rel.as_posix().encode())
        digest.update(b"\0")
        digest.update(file_sha256(path).encode())
        digest.update(b"\0")
    return digest.hexdigest()


def secret_value(path: Path) -> str:
    value = path.read_text(encoding="utf-8").strip()
    if not value:
        raise RuntimeError(f"empty key file: {path}")
    return value


def resolve_binary(raw: str) -> Path:
    if "/" in raw or "\\" in raw:
        path = Path(raw).expanduser().resolve()
    else:
        found = shutil.which(raw)
        if found is None:
            raise RuntimeError(f"binary not found: {raw}")
        path = Path(found).resolve()
    if not path.is_file():
        raise RuntimeError(f"binary not found: {raw}")
    return path


def prepare_run_root(run_root: Path) -> None:
    if run_root.exists() and any(run_root.iterdir()):
        raise RuntimeError(f"refusing to reuse nonempty run root: {run_root}")
    run_root.mkdir(parents=True, exist_ok=True)
    shutil.copytree(BASELINE, run_root / "frozen" / "baseline")
    frozen_prompt = run_root / "frozen" / "task_prompt.md"
    frozen_prompt.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(PROMPT_PATH, frozen_prompt)


def prepare_workspace(run_root: Path, lane: str, wave: int) -> Path:
    workspace = run_root / "workspaces" / lane / f"wave-{wave:03d}"
    if workspace.exists():
        raise RuntimeError(f"refusing to reuse workspace: {workspace}")
    workspace.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(BASELINE, workspace)
    shutil.copy2(PROMPT_PATH, workspace / "BENCHMARK_TASK.md")
    return workspace


def base_env(anthropic_key: str, openai_key: str) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "ANTHROPIC_API_KEY": anthropic_key,
            "OPENAI_API_KEY": openai_key,
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
            "NO_COLOR": "1",
        }
    )
    return env


def stop_process(process: subprocess.Popen[bytes]) -> None:
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
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
) -> dict[str, Any]:
    started_at = utc_now()
    started = time.monotonic()
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=stdout,
            stderr=stderr,
            start_new_session=True,
        )
        timed_out = False
        try:
            returncode = process.wait(timeout=timeout_seconds)
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


def parse_json_file(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def route_evidence(lane: str, result_dir: Path, expected_model: str) -> dict[str, Any]:
    if lane == "corbanu":
        path = result_dir / "corbanu.anthropic.request.json"
        payload = parse_json_file(path)
        model = payload.get("model") if isinstance(payload, dict) else None
        cost = None
    else:
        path = result_dir / "claude.stdout"
        payload = parse_json_file(path)
        usage = payload.get("modelUsage") if isinstance(payload, dict) else {}
        models = sorted(usage) if isinstance(usage, dict) else []
        model = models[-1] if models else None
        cost = payload.get("total_cost_usd") if isinstance(payload, dict) else None
    return {
        "provider": "anthropic-direct",
        "model": model,
        "route_verified": model == expected_model,
        "client_total_cost_usd": cost,
        "evidence_path": str(path),
    }


def verify_workspace(workspace: Path, result_dir: Path, timeout_seconds: int) -> dict[str, Any]:
    capture_dir = result_dir / "captures"
    stdout_path = result_dir / "verifier.stdout"
    stderr_path = result_dir / "verifier.stderr"
    metadata = run_process(
        [
            sys.executable,
            str(VERIFIER),
            str(workspace),
            "--result-dir",
            str(capture_dir),
        ],
        workspace,
        os.environ.copy(),
        stdout_path,
        stderr_path,
        min(timeout_seconds, 900),
    )
    metadata["verification_path"] = str(capture_dir / "verification.json")
    return metadata


def verification_allowed(
    agent_run: dict[str, Any],
    route: dict[str, Any],
    source_integrity: dict[str, Any],
) -> bool:
    return (
        agent_run.get("returncode") == 0
        and route.get("route_verified") is True
        and source_integrity.get("ok") is True
    )


def run_lane(
    args: argparse.Namespace,
    lane: str,
    wave: int,
    anthropic_key: str,
    openai_key: str,
    expected_source_digest: str,
) -> dict[str, Any]:
    workspace = prepare_workspace(args.run_root, lane, wave)
    result_dir = args.run_root / "results" / lane / f"wave-{wave:03d}"
    result_dir.mkdir(parents=True, exist_ok=True)
    prompt = PROMPT_PATH.read_text(encoding="utf-8")
    env = base_env(anthropic_key, openai_key)
    env["CORBANU_BENCHMARK_VERIFIER"] = str(VERIFIER)

    if lane == "corbanu":
        home = result_dir / "corbanu-home"
        home.mkdir()
        env["CODEX_HOME"] = str(home)
        env["CORBANU_TRACE_STREAM_TIMING"] = "1"
        env["CORBANU_DUMP_ANTHROPIC_REQUEST"] = str(
            result_dir / "corbanu.anthropic.request.json"
        )
        env["CORBANU_DUMP_CHAT_REQUEST"] = str(
            result_dir / "corbanu.chat.request.json"
        )
        command = [
            str(args.corbanu_bin),
            "exec",
            "--json",
            "--skip-git-repo-check",
            "-C",
            str(workspace),
            "-c",
            'model_provider="anthropic"',
            "-m",
            args.model,
            "--dangerously-bypass-approvals-and-sandbox",
            prompt,
        ]
        stdout_path = result_dir / "corbanu.stdout"
        stderr_path = result_dir / "corbanu.stderr"
    else:
        home = result_dir / "claude-home"
        home.mkdir()
        env["CLAUDE_CONFIG_DIR"] = str(home)
        command = [
            str(args.claude_bin),
            "--bare",
            "--print",
            "--output-format",
            "json",
            "--model",
            args.model,
            "--dangerously-skip-permissions",
            "--max-budget-usd",
            str(args.claude_max_budget_usd),
            prompt,
        ]
        stdout_path = result_dir / "claude.stdout"
        stderr_path = result_dir / "claude.stderr"

    agent_run = run_process(
        command,
        workspace,
        env,
        stdout_path,
        stderr_path,
        args.timeout_seconds,
    )
    route = route_evidence(lane, result_dir, args.model)
    source_after = harness_source_sha256()
    source_integrity = {
        "ok": source_after == expected_source_digest,
        "before_sha256": expected_source_digest,
        "after_sha256": source_after,
    }
    verification = (
        verify_workspace(workspace, result_dir, args.timeout_seconds)
        if verification_allowed(agent_run, route, source_integrity)
        else {
            "returncode": None,
            "skipped": "agent failure, route mismatch, or benchmark source mutation",
        }
    )
    result = {
        "lane": lane,
        "wave": wave,
        "workspace": str(workspace),
        "result_dir": str(result_dir),
        "prompt_sha256": file_sha256(PROMPT_PATH),
        "workspace_sha256_after": tree_sha256(workspace),
        "agent_run": agent_run,
        "route": route,
        "verification": verification,
        "source_integrity": source_integrity,
        "passed": (
            agent_run["returncode"] == 0
            and route["route_verified"]
            and source_integrity["ok"]
            and verification.get("returncode") == 0
        ),
    }
    (result_dir / "summary.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(
        json.dumps(
            {
                "lane": lane,
                "wave": wave,
                "passed": result["passed"],
                "wall_seconds": agent_run["wall_seconds"],
            },
            sort_keys=True,
        ),
        flush=True,
    )
    return result


def run_lane_series(
    args: argparse.Namespace,
    lane: str,
    key: str,
    openai_key: str,
    expected_source_digest: str,
) -> list[dict[str, Any]]:
    results = []
    for wave in args.waves:
        result = run_lane(
            args,
            lane,
            wave,
            key,
            openai_key,
            expected_source_digest,
        )
        results.append(result)
        if not result["passed"]:
            break
    return results


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-root", type=Path, required=True)
    parser.add_argument("--corbanu-bin", default="corbanu")
    parser.add_argument("--claude-bin", default="claude")
    parser.add_argument("--corbanu-anthropic-key-file", type=Path, required=True)
    parser.add_argument("--claude-anthropic-key-file", type=Path, required=True)
    parser.add_argument("--openai-key-file", type=Path, required=True)
    parser.add_argument("--model", default="claude-opus-5")
    parser.add_argument("--waves", nargs="+", type=int, default=[1, 2, 3])
    parser.add_argument("--timeout-seconds", type=int, default=45 * 60)
    parser.add_argument("--claude-max-budget-usd", type=float, default=6.0)
    parser.add_argument("--confirm-paid-run", action="store_true")
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if not args.confirm_paid_run:
        raise SystemExit("live website benchmark requires --confirm-paid-run")
    if not args.waves or len(args.waves) != len(set(args.waves)):
        raise SystemExit("waves must be a nonempty unique list")
    if min(args.waves) < 1:
        raise SystemExit("wave numbers must be positive")

    args.run_root = args.run_root.resolve()
    args.corbanu_bin = resolve_binary(args.corbanu_bin)
    args.claude_bin = resolve_binary(args.claude_bin)
    corbanu_key = secret_value(args.corbanu_anthropic_key_file)
    claude_key = secret_value(args.claude_anthropic_key_file)
    openai_key = secret_value(args.openai_key_file)
    if corbanu_key == claude_key:
        raise SystemExit("contestant Anthropic keys must be distinct")

    prepare_run_root(args.run_root)
    expected_source_digest = harness_source_sha256()
    manifest = {
        "created_at": utc_now(),
        "model": args.model,
        "waves": args.waves,
        "timeout_seconds": args.timeout_seconds,
        "prompt_sha256": file_sha256(PROMPT_PATH),
        "baseline_sha256": tree_sha256(BASELINE),
        "benchmark_source_sha256": expected_source_digest,
        "contestant_keys_distinct": True,
        "corbanu": {
            "binary": str(args.corbanu_bin),
            "sha256": file_sha256(args.corbanu_bin),
        },
        "claude_code": {
            "binary": str(args.claude_bin),
            "sha256": file_sha256(args.claude_bin),
        },
    }
    (args.run_root / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    lane_keys = {"corbanu": corbanu_key, "claude-code": claude_key}
    results: dict[str, list[dict[str, Any]]] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        futures = {
            pool.submit(
                run_lane_series,
                args,
                lane,
                key,
                openai_key,
                expected_source_digest,
            ): lane
            for lane, key in lane_keys.items()
        }
        for future in concurrent.futures.as_completed(futures):
            lane = futures[future]
            results[lane] = future.result()

    (args.run_root / "pair_run.json").write_text(
        json.dumps(results, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    complete = all(len(results.get(lane, [])) == len(args.waves) for lane in lane_keys)
    passed = complete and all(
        result["passed"] for lane_results in results.values() for result in lane_results
    )
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
