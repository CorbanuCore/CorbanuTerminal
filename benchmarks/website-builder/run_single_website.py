#!/usr/bin/env python3
"""Single-lane website-build benchmark for one provider/model.

Runs corbanu-debug against the frozen task_prompt.md in a clean workspace,
captures wall time and token usage from the JSON stream, then runs the repo
verify_site.py checker.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

WB = Path(__file__).resolve().parent
REPO = WB.parent.parent
BASELINE = WB / "baseline"
PROMPT = WB / "task_prompt.md"
VERIFIER = WB / "verify_site.py"


def utc() -> str:
    import datetime
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: run_single_website.py <run-root>", file=sys.stderr)
        return 2
    run_root = Path(sys.argv[1]).resolve()
    workspace = run_root / "workspace"

    # Fresh baseline copy
    if workspace.exists():
        shutil.rmtree(workspace)
    shutil.copytree(BASELINE, workspace)

    binary = str(Path(os.environ.get("CORBANU_BIN", "/home/pfrpc/.local/bin/corbanu-debug")))
    provider = os.environ.get("BENCH_PROVIDER", "openrouter")
    model = os.environ.get("BENCH_MODEL", "z-ai/glm-5.3-flash")
    timeout = float(os.environ.get("BENCH_TIMEOUT", "1800"))

    cmd = [
        binary, "exec", "--json",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "-C", str(workspace),
        "-c", f'model_provider="{provider}"',
        "-m", model,
        "-",
    ]
    prompt_text = PROMPT.read_text(encoding="utf-8")

    started_at = utc()
    t0 = time.monotonic()
    stdout_file = run_root / "corbanu.stdout"
    stderr_file = run_root / "corbanu.stderr"
    timed_out = False
    rc = None
    try:
        with open(stdout_file, "wb") as so, open(stderr_file, "wb") as se:
            proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=so, stderr=se)
            proc.communicate(prompt_text.encode(), timeout=timeout)
            rc = proc.returncode
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
        rc = proc.returncode
        timed_out = True
    wall = round(time.monotonic() - t0, 3)
    ended_at = utc()

    # Parse usage totals from JSON stream
    usage_totals: dict[str, float] = {}
    text = stdout_file.read_text(errors="replace")
    for line in text.splitlines():
        line = line.strip()
        if not line or not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except Exception:
            continue
        for key in ("token_usage", "usage"):
            val = obj.get(key) if isinstance(obj, dict) else None
            if not isinstance(val, dict):
                continue
            for k, v in val.items():
                if isinstance(v, (int, float)) and v > 0:
                    usage_totals[k] = max(usage_totals.get(k, 0), v)

    # Run verifier
    verifier_rc = None
    try:
        vr = subprocess.run(
            [sys.executable, str(VERIFIER), str(workspace), "--result-dir", str(run_root)],
            capture_output=True, text=True, timeout=300,
        )
        verifier_rc = vr.returncode
        (run_root / "verifier.stdout").write_text(vr.stdout)
        (run_root / "verifier.stderr").write_text(vr.stderr)
    except Exception as exc:
        (run_root / "verify_error.txt").write_text(str(exc))

    passed = bool(verifier_rc == 0)

    summary = {
        "agent": "corbanu",
        "provider": provider,
        "model": model,
        "binary": binary,
        "started_at": started_at,
        "ended_at": ended_at,
        "wall_seconds": wall,
        "timed_out": timed_out,
        "returncode": rc,
        "verifier_passed": passed,
        "verifier_returncode": verifier_rc,
        "token_usage": usage_totals,
    }
    (run_root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
