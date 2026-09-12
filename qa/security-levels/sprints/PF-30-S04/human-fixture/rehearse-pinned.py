#!/usr/bin/env python3
"""Rehearse the ignored fixture entry using a frozen runner; synthetic input only."""
import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import time


def until(predicate, seconds=90):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        result = predicate()
        if result:
            return result
        time.sleep(0.1)
    raise TimeoutError("bounded pinned-fixture checkpoint timed out")


def read_json(path):
    try:
        return json.loads(path.read_text())
    except FileNotFoundError:
        return None


def validate_completion(code, case, finished):
    expected = "Ok(Complete)" if case == "startup" else "Ok(Cancelled)"
    assert (code == 0) == (case == "startup"), code
    assert finished == {"outcome": expected, "human_acceptance": False}, finished


def run_case(args, case):
    evidence = args.evidence / case
    env = dict(os.environ, CORBANU_MEMORY_HUMAN_OPT_IN="1",
               CORBANU_MEMORY_HUMAN_CASE="startup",
               CORBANU_MEMORY_HUMAN_EVIDENCE=str(evidence),
               CORBANU_MEMORY_CANDIDATE_SHA256=args.sha256,
               CARGO_BIN_EXE_codex=str(args.candidate), CORBANU_TMUX_REQUIRED="1")
    with (args.evidence / f"{case}.log").open("x") as log:
        process = subprocess.Popen(
            [str(args.runner), "--ignored", "--exact",
             "suite::memory_human_fixture::human_memory_fixture", "--nocapture"],
            env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        try:
            def ready_or_error():
                if process.poll() is not None:
                    raise RuntimeError(f"fixture exited early: {process.returncode}")
                return read_json(evidence / "ready.json")

            ready = until(ready_or_error)
            # Only the attachment command emitted by this child is usable.
            lines = (evidence / "attach.sh").read_text().splitlines()
            socket_env = shlex.split(lines[1])
            command = shlex.split(lines[2])
            assert socket_env[0] == "export"
            assert socket_env[1] == "TMUX_TMPDIR=" + ready["socket_dir"]
            assert command[:3] == ["exec", "tmux", "-L"]
            assert command[4:6] == ["attach-session", "-t"]
            assert len(command) == 7
            tmux_env = dict(os.environ, TMUX_TMPDIR=ready["socket_dir"])
            prefix = command[1:4]
            target = command[6]

            def tmux(*arguments):
                return subprocess.check_output(
                    prefix + list(arguments), env=tmux_env, text=True,
                    stderr=subprocess.STDOUT,
                )

            def send(text):
                tmux("send-keys", "-t", target, "-l", "--", text)
                until(lambda: text in tmux("capture-pane", "-p", "-t", target), 10)
                tmux("send-keys", "-t", target, "Enter")

            if case == "startup":
                send("HUMAN_FOREGROUND synthetic fixture")
                until(lambda: (read_json(evidence / "status.json") or {}).get("source_outputs") == 1)
                send("/exit")
            else:
                (evidence / "cancel").touch(exist_ok=False)
            code = process.wait(timeout=45)
            validate_completion(code, case, read_json(evidence / "finished.json"))
            assert not Path(ready["home"]).exists(), "disposable home leaked"
            assert not Path(ready["socket_dir"]).exists(), "owned socket leaked"
        finally:
            if process.poll() is None:
                if evidence.exists():
                    (evidence / "cancel").touch()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.terminate()
                    process.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--sha256", required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    args.runner = args.runner.resolve(strict=True)
    args.candidate = args.candidate.resolve(strict=True)
    args.evidence.mkdir()
    for case in ("startup", "cancel"):
        run_case(args, case)
    print("Pinned ignored entry: startup and cancellation passed; no human acceptance claimed.")


if __name__ == "__main__":
    main()
