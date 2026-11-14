#!/usr/bin/env python3
"""N concurrent `corbanu exec` sessions on one home against a mock provider.

Usage: load.py --bin <corbanu> --db <large state DB copy> --out <dir>
               [--sessions 10] [--turns 3] [--tool-calls 4]

Each session runs `--turns` turns (the first `exec`, then `exec resume`), and
every turn makes `--tool-calls` + 1 model requests. The home is disposable and
holds a copy of `--db`; developer accounting records every request in it.

Reports, per run:
- client gap: from the end of a response to the conversation's next request
  (tool run + whatever the client waited on, including the state DB);
- state DB lock waits and holds logged by `codex_state::db_lock`;
- every busy/locked/fatal line on stderr and every failed exec.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import socket
import sqlite3
import statistics
import subprocess
import sys
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
BAD = re.compile(
    r"database is locked|state database is busy|developer accounting failed|"
    r"Fatal error|could not record|request throttle state",
    re.IGNORECASE,
)
LOCK_EVENT = re.compile(r"codex_state::db_lock: (?P<msg>[^\n]*)")
FIELD = re.compile(r"(\w+)=(\"[^\"]*\"|\S+)")


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def pct(values: list[float], q: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(round(q * (len(ordered) - 1))))]


def summary(values: list[float]) -> dict[str, float]:
    return {
        "n": len(values),
        "p50": round(pct(values, 0.50), 1),
        "p90": round(pct(values, 0.90), 1),
        "p99": round(pct(values, 0.99), 1),
        "max": round(max(values), 1) if values else 0.0,
        "mean": round(statistics.fmean(values), 1) if values else 0.0,
    }


def ledger_attempts(home: Path) -> int:
    """Attempts the mock provider's requests left in the ledger."""
    con = sqlite3.connect(home / "pfterminal_state_5.sqlite", timeout=60)
    try:
        has_ledger = con.execute(
            "SELECT count(*) FROM sqlite_schema WHERE name = 'draft_accounting_attempts'"
        ).fetchone()[0]
        if not has_ledger:
            return 0
        return con.execute(
            "SELECT count(*) FROM draft_accounting_attempts "
            "WHERE json_extract(payload, '$.provider') = 'mock'"
        ).fetchone()[0]
    finally:
        con.close()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin", required=True)
    parser.add_argument("--db", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--sessions", type=int, default=10)
    parser.add_argument("--turns", type=int, default=3)
    parser.add_argument("--tool-calls", type=int, default=4)
    parser.add_argument("--latency", type=float, default=0.2)
    parser.add_argument(
        "--hold-seconds",
        type=float,
        default=0.0,
        help="hold the state DB's write lock this long, 5 s into the run, as a "
        "long maintenance or a backup in another process would",
    )
    parser.add_argument(
        "--warmup",
        action="store_true",
        help="run one exec first, alone, so the home's other runtime DBs exist "
        "(ten first launches at once race creating them; a separate issue) and "
        "the ledger's hourly validation is done, as on a home in use",
    )
    args = parser.parse_args()

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=False)
    home = out / "home"
    work = out / "work"
    home.mkdir()
    work.mkdir()
    started = time.time()
    shutil.copyfile(args.db, home / "pfterminal_state_5.sqlite")
    port = free_port()
    (home / "config.toml").write_text(
        f"""model = "claude-opus-5-5"
model_provider = "mock"
approval_policy = "never"
sandbox_mode = "danger-full-access"

[model_providers.mock]
name = "Mock Anthropic"
base_url = "http://127.0.0.1:{port}/v1"
wire_api = "anthropic"
env_key = "MOCK_ANTHROPIC_KEY"
request_max_retries = 0
stream_max_retries = 0

[projects.{json.dumps(str(work))}]
trust_level = "trusted"
"""
    )
    mock_log = out / "mock.jsonl"
    mock = subprocess.Popen(
        [
            sys.executable,
            str(HERE / "mock_anthropic.py"),
            "--port",
            str(port),
            "--log",
            str(mock_log),
            "--tool-calls",
            str(args.tool_calls),
            "--latency",
            str(args.latency),
        ],
        stderr=subprocess.DEVNULL,
    )
    time.sleep(0.5)
    env = dict(os.environ)
    for name in ("CORBANU_HOME", "PFTERMINAL_HOME", "CODEX_HOME"):
        env[name] = str(home)
    env["CORBANU_TEST_NO_NATIVE_KEYRING"] = "1"
    env["MOCK_ANTHROPIC_KEY"] = "mock"
    env["RUST_LOG"] = "warn,codex_state::db_lock=debug,codex_core::accounting=info"
    results: list[dict] = []
    lock = threading.Lock()
    warmup_seconds = 0.0
    if args.warmup:
        t0 = time.time()
        subprocess.run(
            [args.bin, "exec", "--skip-git-repo-check", "warm-up: reply"],
            cwd=work,
            env=env,
            capture_output=True,
            timeout=900,
        )
        warmup_seconds = round(time.time() - t0, 1)
        mock_log.rename(out / "mock-warmup.jsonl")
        started = time.time()
    recorded_before = ledger_attempts(home) if args.warmup else 0

    def session(index: int) -> None:
        session_id = None
        for turn in range(args.turns):
            prompt = f"load session {index}: run the tool, turn {turn}"
            if session_id is None:
                command = [args.bin, "exec", "--skip-git-repo-check", "--json", prompt]
            else:
                command = [
                    args.bin,
                    "exec",
                    "--skip-git-repo-check",
                    "--json",
                    "resume",
                    session_id,
                    prompt,
                ]
            t0 = time.time()
            proc = subprocess.run(
                command,
                cwd=work,
                env=env,
                capture_output=True,
                text=True,
                timeout=900,
            )
            elapsed = time.time() - t0
            (out / f"s{index}-t{turn}.stderr").write_text(proc.stderr)
            (out / f"s{index}-t{turn}.stdout").write_text(proc.stdout)
            for line in proc.stdout.splitlines():
                try:
                    event = json.loads(line)
                except ValueError:
                    continue
                if event.get("type") == "thread.started":
                    session_id = session_id or event.get("thread_id")
            with lock:
                results.append(
                    {
                        "session": index,
                        "turn": turn,
                        "exit": proc.returncode,
                        "seconds": round(elapsed, 2),
                        "bad": [line for line in proc.stderr.splitlines() if BAD.search(line)],
                    }
                )
            if session_id is None:
                return

    holder: dict[str, float] = {}

    def hold() -> None:
        time.sleep(5)
        con = sqlite3.connect(home / "pfterminal_state_5.sqlite", timeout=60)
        con.execute("BEGIN IMMEDIATE")
        holder["from"] = time.time()
        time.sleep(args.hold_seconds)
        con.rollback()
        holder["to"] = time.time()
        con.close()

    threads = [threading.Thread(target=session, args=(i,)) for i in range(args.sessions)]
    if args.hold_seconds > 0:
        threads.append(threading.Thread(target=hold))
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    mock.terminate()
    mock.wait()

    requests = [json.loads(line) for line in mock_log.read_text().splitlines()]
    by_session: dict[str, list[dict]] = {}
    for request in requests:
        by_session.setdefault(request["session"], []).append(request)
    gaps = []
    for items in by_session.values():
        items.sort(key=lambda r: r["arrived"])
        for previous, current in zip(items, items[1:]):
            # Only gaps inside one turn: the next request after a tool call.
            if previous["stop"] == "tool_use" and current["calls"] == previous["calls"] + 1:
                gaps.append((current["arrived"] - previous["done"]) * 1000)

    waits: list[float] = []
    holds: list[float] = []
    slow_holds: dict[str, list[float]] = {}
    for path in out.glob("s*-t*.stderr"):
        for match in LOCK_EVENT.finditer(path.read_text()):
            fields = {k: v.strip('"') for k, v in FIELD.findall(match.group("msg"))}
            if "wait_ms" in fields:
                waits.append(float(fields["wait_ms"]))
            if "held_ms" in fields:
                held = float(fields["held_ms"])
                holds.append(held)
                if held > 250:
                    slow_holds.setdefault(fields.get("site", "?"), []).append(held)

    failed = [r for r in results if r["exit"] != 0]
    bad_lines = [line for r in results for line in r["bad"]]
    report = {
        "bin": args.bin,
        "sessions": args.sessions,
        "turns": args.turns,
        "tool_calls": args.tool_calls,
        "wall_seconds": round(time.time() - started, 1),
        "warmup_seconds": warmup_seconds,
        "execs": len(results),
        "failed_execs": len(failed),
        "requests": len(requests),
        "client_gap_ms": summary(gaps),
        "lock_wait_ms": summary(waits),
        "lock_hold_ms": summary(holds),
        "holds_over_250ms_by_site": {
            site: summary(values) for site, values in sorted(slow_holds.items())
        },
        "bad_stderr_lines": len(bad_lines),
        "bad_examples": sorted(set(line[:300] for line in bad_lines))[:10],
        "exec_seconds": summary([r["seconds"] for r in results]),
        "lock_held_by_another_process_s": round(holder["to"] - holder["from"], 1) if holder else 0,
        "ledger_attempts_recorded": ledger_attempts(home) - recorded_before,
        "records_still_saved_for_a_later_writer": len(
            list((home / "accounting-pending").glob("*.json"))
        ),
    }
    (out / "report.json").write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
