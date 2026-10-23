"""Disposable preparation evidence; deliberately stops before real transport.

No real-worker success is claimed. An approved isolated inference profile and
non-live binary/transport are not supplied by this allocation.
"""
import copy
import json
from pathlib import Path
import tempfile

from coordinator import Coordinator, digest
import fable_launcher as f
import owner_daemon as owner
from owner_tmux import freeze_worker_inputs, worker_runtime
from test_coordinator import seed


def main():
    base = Path(tempfile.mkdtemp(prefix="os93-", dir="/private/tmp"))
    worktree = base / "worktree"
    worktree.mkdir(mode=0o700)
    brief = base / "brief.json"
    f.write_json(brief, {"task": "Reply RETURN followed by disposable probe complete. Do not use tools."})
    cycle = dict(allocation="cycle", base_commit="dc8dbf5cffce83ac773c92571901f704cfa0366d",
                 brief_file=str(brief), brief_sha256=f.file_digest(brief),
                 model="gpt-6-astra", reasoning_effort="high",
                 task="Read brief_file (verify sha256) and do exactly what it says; return what it lists.",
                 worktree=str(worktree))
    f.write_json(base / "original-inputs.json", cycle)
    coordinator = Coordinator(base / "coordinator")
    coordinator.initialize(*seed())
    coordinator.set_enabled(True, {"authority": "owner-shape-93 disposable preparation"})

    def prepare(inputs, key, replace):
        allocation = copy.deepcopy(seed()[2]["bootstrap"])
        allocation["inputs"] = {k: v for k, v in inputs.items() if k != "allocation"}
        coordinator.put_allocation("cycle", allocation, replace,
                                   coordinator.snapshot()["revision"], {"disposable": True})
        coordinator.event({"id": key})
        packet = coordinator.begin_manager()
        action = dict(id=key, kind="repair", workstream="delivery", sprint="PF80",
                      rationale="disposable preparation", inputs=inputs, timeout_seconds=60,
                      expected_revision=packet["state_revision"])
        coordinator.accept_decision(packet["manager_run"],
                                    dict(state_revision=packet["state_revision"], actions=[action]),
                                    {"disposable": True})
        return coordinator.snapshot()["actions"][key]

    original = prepare(cycle, "original-flat", False)
    try:
        worker_runtime(original["inputs"])
    except f.LaunchError as exc:
        flat_refusal = str(exc)
    else:
        raise AssertionError("unrecorded runtime was accepted")
    bound = freeze_worker_inputs(cycle, provider="openai", policy="--yolo")
    corrected = prepare(bound, "corrected-bound", True)
    state = coordinator.snapshot()
    assert state["actions"]["original-flat"]["status"] == "cancelled"
    assert corrected["inputs"] == bound
    assert corrected["allocation_digest"] == digest(state["allocations"]["cycle"])
    assert {k: v for k, v in bound.items() if k != "worker"} == cycle
    assert worker_runtime(bound)["provider"] == "openai"

    config_path = base / "config.json"
    config = dict(coordinator=str(coordinator.directory), worktrees=[str(worktree)],
                  package_digest=owner.package_digest(), manager_enabled=False)
    f.write_json(config_path, config)
    owner.setup(config_path)
    owner.arm_owner(config_path, dict(decision_id="shape93-disposable", revision=1,
                    authority="owner-shape-93 disposable preparation", scope="fixture-only",
                    generation=1, config_digest=digest(config), package_digest=owner.package_digest()))
    status = owner.activation_status(config_path)
    assert status["complete"] and status["scope"] == "fixture-only" and status["state"] == "armed"
    owner.disarm_owner(config_path, 1)
    evidence = dict(root=str(base), original_input_keys=sorted(cycle),
                    flat_refusal=flat_refusal, original_status="cancelled",
                    corrected_status=corrected["status"], frozen_runtime=worker_runtime(bound),
                    original_digest=original["allocation_digest"],
                    corrected_digest=corrected["allocation_digest"],
                    preserved_cycle_fields=True, fixture_activation_observed=True,
                    final_activation=owner.activation_status(config_path)["state"],
                    real_worker="BLOCKED",
                    stop="before tmux transport validation/promotion/launch: no supplied approved "
                         "isolated inference profile or non-live binary transport",
                    real_promotions=0, real_launches=0, real_acks=0, real_starts=0, real_returns=0,
                    live_access=False)
    print(json.dumps(evidence, sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
