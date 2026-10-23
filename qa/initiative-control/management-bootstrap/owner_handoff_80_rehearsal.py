"""Concurrent real TMUX/PTY rehearsal; synthetic workers, no inference or auth."""
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

from coordinator import Coordinator, digest
import fable_launcher as f
import owner_daemon as owner
from owner_tmux import Worker
from test_coordinator import seed

FIXTURE = r'''#!PYTHON
import hashlib, json, os, pathlib, sys, time, tty, uuid
tty.setcbreak(sys.stdin.fileno())
home = pathlib.Path(os.environ["HOME"])
meta = json.loads((home.parent / "worker.json").read_text())
b = meta["binding"]
sessions = home / "sessions"
sessions.mkdir(mode=0o700)
path = sessions / "fixture.jsonl"
session = str(uuid.uuid4())
def record(kind, **payload):
    return dict(type=kind, payload=payload)
def append(rows):
    with path.open("a") as stream:
        stream.write("".join(json.dumps(row) + "\n" for row in rows))
        stream.flush()
        os.fsync(stream.fileno())
    path.chmod(0o600)
append([record("session_meta", id=session, session_id=session, cwd=b["worktree"],
               source="cli", model_provider=b["provider"])])
print("\033[?2004hCorbanu Terminal model: fixture-model high READY", flush=True)
buffer = ""
while True:
    char = sys.stdin.read(1)
    if not char:
        break
    buffer += char
    value = buffer.replace("\x1b[200~", "").replace("\x1b[201~", "").rstrip("\r\n")
    if char not in "\r\n" or value not in (meta["prompt"], "START", "/quit"):
        continue
    buffer = ""
    if value == "/quit":
        break
    with (home / "received.jsonl").open("a") as stream:
        stream.write(json.dumps(dict(kind="prompt" if value == meta["prompt"] else "start",
                                     digest=hashlib.sha256(value.encode()).hexdigest(), at=time.time())) + "\n")
    if value == meta["prompt"]:
        while not (pathlib.Path(b["worktree"]) / "allow-ack").exists():
            time.sleep(0.05)
        turn, final = "ack-turn", meta["ack"]
    else:
        turn, final = "work-turn", "RETURN\nSynthetic calculation: 2 + 2 = 4"
        (pathlib.Path(b["worktree"]) / "performed.txt").write_text("4\n")
    append([record("event_msg", type="task_started", turn_id=turn),
            record("turn_context", turn_id=turn, cwd=b["worktree"], model=b["model"],
                   model_provider=b["provider"], effort=b["effort"], approval_policy=b["approval"],
                   sandbox_policy=dict(type=b["sandbox"])),
            record("event_msg", type="user_message", message=value),
            record("event_msg", type="model_response_completed", turn_id=turn,
                   model=b["model"], model_provider_id=b["provider"], response_id=turn),
            record("event_msg", type="task_complete", turn_id=turn, last_agent_message=final)])
    print(final, flush=True)
'''


def emit(kind, **data):
    print(json.dumps(dict(kind=kind, at=time.time(), **data), sort_keys=True), flush=True)


def dispatch_loop(config, side, base):
    end = time.monotonic() + 90
    while time.monotonic() < end and not (base / "stop").exists():
        try:
            result = owner.Kernel(config, dispatcher=side).tick()
        except BlockingIOError:
            result = dict(state="BUSY")
        except Exception as exc:
            emit("error", dispatcher=side, error=type(exc).__name__, detail=str(exc))
            raise
        emit("tick", dispatcher=side, result=result)
        time.sleep(0.4)


def main():
    base = Path(tempfile.mkdtemp(prefix="oh80-", dir="/private/tmp"))
    state, runs = base / "state", base / "runs"
    runs.mkdir(mode=0o700)
    c = Coordinator(state)
    c.initialize(*seed())
    c.set_enabled(True, {"authority": "disposable synthetic rehearsal"})
    binary = base / "synthetic-worker"
    f.write_file(binary, FIXTURE.replace("PYTHON", str(Path(sys.executable).resolve()), 1), mode=0o700)
    config_path = base / "config.json"
    worktrees = {}
    for key in ("owner-work", "hand-work"):
        worktrees[key] = base / key
        worktrees[key].mkdir(mode=0o700)
        allocation = seed()[2]["bootstrap"]
        allocation["resources"] = [key]
        allocation["timeout_seconds"] = 180
        allocation["inputs"]["worker"] = dict(model="fixture-model", provider="fixture",
                                               effort="high", worktree=str(worktrees[key]), policy="--yolo")
        c.put_allocation(key, allocation, False, c.snapshot()["revision"], {"fixture": True})
        c.event({"id": "prepare-" + key})
        packet = c.begin_manager()
        action = dict(id=key, kind="repair", workstream="delivery", sprint="PF80", rationale="synthetic work",
                      inputs={"allocation": key, **allocation["inputs"]}, timeout_seconds=180,
                      expected_revision=packet["state_revision"])
        c.accept_decision(packet["manager_run"], dict(state_revision=packet["state_revision"], actions=[action]),
                          {"fixture": True})
    config = dict(coordinator=str(state), worktrees=list(map(str, worktrees.values())),
                  package_digest=owner.package_digest(), manager_enabled=False,
                  transport=dict(kind="tmux", binary=str(binary), binary_sha256=f.file_digest(binary),
                                 tmux=str(Path(shutil.which("tmux")).resolve()), runs_dir=str(runs),
                                 auth_link=str(base / "unused/auth.json")))
    f.write_json(config_path, config)
    owner.setup(config_path)

    def transfer(choices):
        end = time.monotonic() + 40
        while True:
            snapshot = c.snapshot()
            request = dict(expected_revision=snapshot["revision"], evidence={"rehearsal": "cooperating dispatchers"},
                           assignments={key: {**{field: snapshot["actions"][key].get(field)
                                                  for field in ("claim", "allocation_digest", "status")},
                                              "from": c.dispatch_owner(snapshot, snapshot["actions"][key]), "to": side}
                                        for key, side in choices.items()})
            try:
                result = owner.handoff(config_path, request)
                emit("handoff", request=request, result=result)
                return
            except BlockingIOError:
                if time.monotonic() >= end:
                    raise
                time.sleep(0.05)

    def wait(description, predicate):
        end = time.monotonic() + 65
        while time.monotonic() < end:
            if predicate():
                emit(description, actions=c.snapshot()["actions"])
                return
            time.sleep(0.1)
        raise AssertionError("timeout: " + description)

    transfer({"owner-work": "owner", "hand-work": "hand"})
    authority = dict(decision_id="owner-handoff-80-disposable", revision=1, authority="synthetic rehearsal only",
                     scope="tmux-workers", generation=1, config_digest=digest(config),
                     package_digest=owner.package_digest())
    emit("armed", result=owner.arm_owner(config_path, authority), root=str(base))
    children, logs = [], []
    env = dict(PATH=f.SAFE_PATH, PYTHONPATH=str(Path(owner.__file__).parent), PYTHONDONTWRITEBYTECODE="1",
               CORBANU_TEST_NO_NATIVE_KEYRING="1",
               **{key: str(base) for key in ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
    try:
        for side in ("owner", "hand"):
            log = (base / (side + ".jsonl")).open("w")
            logs.append(log)
            argv = [sys.executable, "-B", str(Path(__file__).resolve()), "--loop", str(config_path), side, str(base)]
            children.append(subprocess.Popen(argv, env=env, stdout=log, stderr=log))
            emit("dispatcher_started", dispatcher=side, pid=children[-1].pid, argv=argv)
        wait("both_claimed_and_prompted", lambda: len(list(runs.glob("*/home/received.jsonl"))) == 2)
        assert all(child.poll() is None for child in children)
        claims = {key: a["claim"] for key, a in c.snapshot()["actions"].items()}
        # Both actual processes remain running. Transfer the same pending claim
        # out and back before allowing ACK, proving continuation not relaunch.
        transfer({"owner-work": "hand"})
        time.sleep(1)
        transfer({"owner-work": "owner"})
        for path in worktrees.values():
            f.write_file(path / "allow-ack", b"allow synthetic ACK")
        wait("both_returned", lambda: all(a["status"] == "returned" for a in c.snapshot()["actions"].values()))
        assert claims == {key: a["claim"] for key, a in c.snapshot()["actions"].items()}
        for path in worktrees.values():
            assert (path / "performed.txt").read_text() == "4\n"
        with sqlite3.connect(state / "owner.sqlite3") as db:
            holds = db.execute("SELECT * FROM holds").fetchall()
            effects = db.execute("SELECT action_id,effect,phase,COUNT(*) FROM operations "
                                 "GROUP BY action_id,effect,phase ORDER BY action_id,effect").fetchall()
        assert not holds
        assert len(effects) == 22 and all(row[2:] == ("applied", 1) for row in effects)
        emit("journal", effects=effects, holds=holds, claims=claims)
    finally:
        f.write_file(base / "stop", b"stop loops")
        for child in children:
            try:
                child.wait(timeout=15)
            except subprocess.TimeoutExpired:
                child.terminate()
                child.wait(timeout=10)
        for log in logs:
            log.close()
        for path in sorted(runs.glob("*/worker.json")):
            worker = Worker(path.parent)
            try:
                capture = worker.tmux("capture-pane", "-p", "-t", worker.meta["session"], check=False).stdout
                received = [json.loads(line) for line in (worker.run / "home/received.jsonl").read_text().splitlines()]
                emit("pane", action=worker.binding["action_id"], received=received, capture=capture)
                assert [row["kind"] for row in received] == ["prompt", "start"]
            finally:
                worker.tmux("kill-server", check=False)
        for side in ("owner", "hand"):
            for line in (base / (side + ".jsonl")).read_text().splitlines():
                emit("dispatcher_log", dispatcher=side, line=line)
    assert all(child.returncode == 0 for child in children)
    emit("status", result=owner.activation_status(config_path))
    emit("disarmed", result=owner.disarm_owner(config_path, 1))
    try:
        owner.Kernel(config_path).tick()
        raise AssertionError("disarmed owner admitted")
    except f.LaunchError as exc:
        assert str(exc) == "owner_off"
        emit("refused_after_disarm", refusal=str(exc))
    emit("PASS", root=str(base), package_digest=owner.package_digest(),
         real_tmux=True, real_inference=False, synthetic_workers=2, live_access=False)


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--loop":
        dispatch_loop(Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4]))
    else:
        main()
