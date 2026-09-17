"""Run the exact promotion recipe against a disposable launchd installation."""
from argparse import Namespace
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

import activate
from coordinator import Coordinator, digest
import fable_launcher as f
import owner_daemon as owner
from owner_tmux import Worker, freeze_worker_inputs
from test_coordinator import seed
from owner_handoff_80_rehearsal import FIXTURE, emit


def main():
    base = Path(tempfile.mkdtemp(prefix="op80-", dir="/private/tmp"))
    schedule, runtime, state, runs, worktree, publish = (
        base / name for name in ("schedule", "runtime", "state", "runs", "worktree", "publish"))
    for path in (schedule, runtime, runs, worktree, publish):
        path.mkdir(mode=0o700)
    for path in Path(owner.__file__).parent.glob("*.py"):
        if not path.name.startswith("test_"):
            target = runtime / path.name
            shutil.copyfile(path, target)
            target.chmod(0o600)
    c = Coordinator(state)
    c.initialize(*seed())
    c.set_enabled(True, {"fixture": True})
    allocation = seed()[2]["bootstrap"]
    allocation["timeout_seconds"] = 180
    brief = base / "brief.json"
    f.write_json(brief, {"task": "synthetic promotion probe"})
    cycle_inputs = dict(allocation="promotion", base_commit="a" * 40, brief_file=str(brief),
                        brief_sha256=f.file_digest(brief), model="fixture-model",
                        reasoning_effort="high", task="Read the frozen brief", worktree=str(worktree))
    allocation["inputs"] = freeze_worker_inputs(cycle_inputs, provider="fixture", policy="--yolo")
    allocation["inputs"].pop("allocation")
    c.put_allocation("promotion", allocation, False, c.snapshot()["revision"], {"fixture": True})
    c.event({"id": "promotion"})
    packet = c.begin_manager()
    action = dict(id="promotion-work", kind="repair", workstream="delivery", sprint="PF80",
                  rationale="synthetic promotion rehearsal", timeout_seconds=180,
                  inputs={"allocation": "promotion", **allocation["inputs"]},
                  expected_revision=packet["state_revision"])
    c.accept_decision(packet["manager_run"], dict(state_revision=packet["state_revision"], actions=[action]),
                      {"fixture": True})
    config_path = schedule / "config.json"
    config = dict(coordinator=str(state), worktrees=[str(worktree)],
                  package_digest=owner.package_digest(), manager_enabled=False)
    f.write_json(config_path, config)
    owner.setup(config_path)
    owner.arm_owner(config_path, dict(decision_id="synthetic-initial", revision=1,
                    authority="disposable rehearsal", scope="fixture-only", generation=1,
                    config_digest=digest(config), package_digest=owner.package_digest()))
    binary = base / "synthetic-worker"
    f.write_file(binary, FIXTURE.replace("PYTHON", str(Path(sys.executable).resolve()), 1), mode=0o700)
    transport = base / "transport.json"
    f.write_json(transport, dict(kind="tmux", binary=str(binary), binary_sha256=f.file_digest(binary),
                                tmux=str(Path(shutil.which("tmux")).resolve()), runs_dir=str(runs),
                                auth_link=str(base / "unused/auth.json")))
    f.write_file(worktree / "allow-ack", b"synthetic ACK allowed")
    python = Path(sys.executable).resolve()
    args = Namespace(owner="install", root=schedule, label="com.corbanu.initiative-owner.test-promotion80-" + str(os.getpid()),
                     python=python, python_sha256=f.file_digest(python), runtime=runtime,
                     config=config_path, interval=30, publish_state=publish, confirm_live=False, domain="user")
    try:
        activate.owner_activation(args)
        end = time.monotonic() + 20
        while owner.load(schedule / "tick.json")["last_success"] is None:
            assert time.monotonic() < end
            time.sleep(0.1)
        with c.connection() as db:
            before = db.execute("SELECT COUNT(*) FROM audit").fetchone()[0]
        recipe = Path(__file__).with_name("owner-handoff-80-promotion.md").read_text()
        code = recipe.split("-B - <<'PY'\n", 1)[1].split("\nPY\n", 1)[0]
        env = dict(PATH=f.SAFE_PATH, PYTHONPATH=str(Path(owner.__file__).parent), PYTHONDONTWRITEBYTECODE="1",
                   OWNER80_SCHEDULE=str(schedule), OWNER80_TRANSPORT=str(transport),
                   OWNER80_ACTIONS="promotion-work", OWNER80_DECISION_ID="synthetic-promotion",
                   OWNER80_DECISION_REVISION="1", OWNER80_AUTHORITY="disposable rehearsal only",
                   CORBANU_TEST_NO_NATIVE_KEYRING="1",
                   **{key: str(base) for key in ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
        result = subprocess.run([sys.executable, "-B", "-c", code], env=env, capture_output=True, text=True, timeout=90)
        emit("recipe", exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
        assert result.returncode == 0
        end = time.monotonic() + 100
        while c.snapshot()["actions"]["promotion-work"]["status"] != "returned":
            assert time.monotonic() < end, "promoted recurring owner did not finish work"
            time.sleep(0.2)
        # Returned is committed before the tick releases its owner lock. Status
        # is advisory and can be unavailable in that interval; preserve attempts.
        end = time.monotonic() + 5
        while True:
            status = owner.activation_status(config_path)
            emit("post_return_status", status=status)
            if status["complete"]:
                break
            assert time.monotonic() < end, "post-return status remained unavailable"
            time.sleep(0.1)
        assert status["scope"] == "tmux-workers" and status["generation"] == 3
        assert status["coordinator"]["ownership"]["promotion-work"]["owner"] == "owner"
        with c.connection() as db:
            assert db.execute("SELECT COUNT(*) FROM audit").fetchone()[0] > before
        assert (worktree / "performed.txt").read_text() == "4\n"
        emit("promoted_work_returned", status=status, tick=owner.load(schedule / "tick.json"))
        owner.disarm_owner(config_path, 3)
    finally:
        if (schedule / "installation.json").exists():
            args.owner = "uninstall"
            activate.owner_activation(args)
        for path in runs.glob("*/worker.json"):
            worker = Worker(path.parent)
            worker.tmux("kill-server", check=False)
        emit("cleanup", service=owner.service(args.label, "user/" + str(os.getuid()))[0])
    emit("PASS", root=str(base), exact_markdown_recipe=True, live_access=False, real_inference=False)


if __name__ == "__main__":
    main()
