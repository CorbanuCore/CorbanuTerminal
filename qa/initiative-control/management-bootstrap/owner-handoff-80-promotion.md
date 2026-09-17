# owner-handoff-80 — live promotion commands (NOT EXECUTED)

This recipe changes the existing installation in place, preserving owner history,
coordinator history, activation generations, schedule ticks and recovery evidence.
Run it only after receiving this candidate and coordinating with the hand
dispatcher. This worker did not inspect or access the live installation.

Round 85 correction: the concurrent rehearsal proves real tmux/PTY and journal
coordination with synthetic workers. It does **not** qualify real Corbanu rollout
provenance, auth-link startup, model latency, ACK/START/RETURN or RETURN after pane
death under two dispatchers. Those remain a separate real-binary, disposable-root
qualification, or explicitly limited evidence from the first authorized promoted
action. Do not call this recipe a fully qualified real-worker promotion.

The deliberate watchdog policy is ownership-local: scheduled owner passes cover
owner-owned actions; the manager must arrange stall detection for hand-owned
actions, including every future default action. Manual `--hand-run` reports their
stalls when admitted, but there is no scheduled hand watchdog. Disarm disables
both lanes. This preserves the accepted hand ownership boundary and avoids
mutating hand claims from the owner lane.

Read the computed `coordinator.watchdog_coverage.summary` before executing the
cutover using the received candidate's status reader. Round 89 corrects counts:
only unreported overdue dispatching/dispatched/running actions count. Prepared,
returned, uncertain, terminal, already-reported and future-deadline actions do
not. The separate ownership map includes those actions.

The exact line formats are below; `C` and `E` stand for observed integer counts,
not literal text to type or invented live measurements:

```text
BEFORE: Owner watchdog on admitted ticks (unreported overdue actions): covered=C, excluded=0; manager=covered; new default actions=covered; hand stall detection=covered by this watchdog.
AFTER: Owner watchdog on admitted ticks (unreported overdue actions): covered=C, excluded=E; manager=covered; new default actions=excluded; hand stall detection=manager responsibility (--hand-run is manual; no scheduled hand watchdog).
```

BEFORE ARM uses the AFTER format with its own observed counts. The recipe prints
all three computed lines and checks the same-snapshot counts against report
predictions. Manager coverage and hand guidance come from the watchdog's own
lane predicate. The expected change is fixture-wide coverage to owner-lane
coverage; count equality across time is not expected. Existing actions can
expire, report or finish between reads. Predictions require an admitted tick
reaching its watchdog; status alone does not prove admission or service health.

If the fields, line or action predictions disagree within a snapshot, or the
routing change differs from these expectations, stop the cutover. Before arm,
leave admission OFF; after arm, revoke admission with the current generation
using the disarm command below. Keep both raw status JSONs, timestamps, revisions,
pins and the refusal transcript. Quiesce cooperating dispatchers and compare
each changed action's status, deadline, stall flag and owner; re-read with the
received candidate before proceeding. Do not edit SQLite, retry arming blindly,
or assume disarm kills existing workers. If ownership, holds or pins remain
uncertain, escalate to the integration owner with that evidence.

Before the command, stop **new hand dispatch and raw key delivery** and let any
in-progress delivery finish. Leave existing workers running. Finish/reconcile
any outstanding manager cycle. Both sides must use the **same existing
coordinator directory**, with its existing resource reservations. Do not seed or
clone a second coordinator for the same worktrees.

Prepare a private (0600), reviewed transport JSON using the already-supported
transport schema. Its values are installation inputs, not values this worker can
discover without violating the no-live-access instruction:

```json
{
  "kind": "tmux",
  "binary": "/absolute/path/to/reviewed/CorbanuTerminal",
  "binary_sha256": "SHA256_OF_THAT_BINARY",
  "tmux": "/absolute/path/to/tmux",
  "runs_dir": "/private/0700/short/worker-runs",
  "auth_link": "/absolute/path/to/operator-approved/auth.json"
}
```

The transport only creates the existing auth link; this recipe never reads,
prints or hashes auth contents. The short runs path must leave each generated
tmux socket path below 100 bytes. Preserve each selected allocation's exact
model/provider/effort/worktree and recorded `--yolo` policy. Do not substitute
the rehearsal's synthetic worker.

The following are exact commands; replace the six operator inputs with the
actual reviewed transport path, prepared action IDs and real activation
decision. Their values cannot be honestly invented by this worker.

```bash
export OWNER80_TRANSPORT=/absolute/path/to/reviewed-owner-transport.json
export OWNER80_ACTIONS='prepared-action-id-one prepared-action-id-two'
export OWNER80_DECISION_ID='actual-tmux-promotion-decision-id'
export OWNER80_DECISION_REVISION=1
export OWNER80_AUTHORITY='actual named authority and authorization reference'
export OWNER80_SCHEDULE='/absolute/path/to/existing-owner-schedule'

# Run from the received candidate checkout. Build the administrative interpreter
# with only the pinned requirements; owner ticks keep the installed interpreter.
OWNER80_ADMIN="$(mktemp -d /private/tmp/owner80-admin.XXXXXX)"
env -i HOME="$OWNER80_ADMIN" PATH=/opt/homebrew/bin:/usr/bin:/bin \
  python3 -m venv "$OWNER80_ADMIN/venv"
env -i HOME="$OWNER80_ADMIN" PATH=/opt/homebrew/bin:/usr/bin:/bin \
  "$OWNER80_ADMIN/venv/bin/python" -m pip install -r scripts/initiative_control/requirements.txt
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control "$OWNER80_ADMIN/venv/bin/python" -B - <<'PY'
import json, os, shutil, subprocess, sys, time
from pathlib import Path
import fable_launcher as f
import owner_daemon as owner
from coordinator import TERMINAL, digest
from manager_cycle import ExistingCoordinator

root = f.private_dir(os.environ["OWNER80_SCHEDULE"])
receipt = owner.load(root / "installation.json")
assert receipt["phase"] == "installed"
pins = receipt["pins"]
python, runtime, config_path = map(Path, (pins["python"], pins["runtime"], pins["config"]))
config = owner.load(config_path)
assert "transport" not in config, "This recipe expects the currently armed fixture-only installation"
transport = owner.load(Path(os.environ["OWNER80_TRANSPORT"]))
from owner_tmux import validate
validate(transport)
selected = set(os.environ["OWNER80_ACTIONS"].split())
assert selected, "Name at least one already-authorized prepared worker action"
coordinator = ExistingCoordinator(config["coordinator"])
snapshot = coordinator.snapshot()
assert snapshot["manager"] is None, "Finish/reconcile the existing hand manager cycle first"
for key in selected:
    action = snapshot["actions"][key]
    assert action["status"] == "prepared" and action["kind"] in owner.WORKER_KINDS
    assert action["inputs"]["worker"]["worktree"] in config["worktrees"]
    assert action["inputs"]["worker"]["policy"] == "--yolo"
assert snapshot["enabled"], "Dispatch must already be authorized and enabled"

env = dict(PATH=f.SAFE_PATH, PYTHONDONTWRITEBYTECODE="1",
           **{key: str(root) for key in ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
def run(script, *args, expected=0):
    # activate imports the pinned markdown dependency; use this administrative
    # venv with site packages. The owner runtime keeps its installed -S pin.
    interpreter = sys.executable if script == "activate.py" else str(python)
    flags = ["-E", "-s", "-B"] if script == "activate.py" else ["-E", "-s", "-S", "-B"]
    argv = [interpreter, *flags, str(runtime / script), *map(str, args)]
    result = subprocess.run(argv, env=env, capture_output=True, text=True, timeout=60)
    print(json.dumps(dict(argv=argv, exit=result.returncode,
                          stdout=result.stdout, stderr=result.stderr)), flush=True)
    assert result.returncode == expected, "Stop here; retain the transcript and inspect the named refusal"
    return json.loads(result.stdout) if result.stdout.strip() else None

def check_coverage(status, *, partitioned):
    assert status["complete"], "Coverage unavailable; stop and preserve the raw status"
    impact = status["coordinator"]
    coverage = impact["watchdog_coverage"]
    assert coverage["manager_covered"], "Unexpected manager coverage; stop"
    assert coverage["future_default_covered"] is (not partitioned)
    assert coverage["hand_stall_detection"] == (
        "manager responsibility (--hand-run is manual; no scheduled hand watchdog)"
        if partitioned else "covered by this watchdog")
    assert set(coverage["covered_actions"]) == {
        row["id"] for row in impact["in_flight"] if row["watchdog_will_report"]}
    expected_excluded = {
        row["id"] for row in impact["in_flight"]
        if row["overdue"] and not row["stall_reported"]
        and row["status"] in {"dispatching", "dispatched", "running"}
        and not row["watchdog_covered"]}
    assert set(coverage["excluded_actions"]) == expected_excluded
    if not partitioned:
        assert not coverage["excluded_actions"]
    return coverage["summary"]

# Existing runtime can disarm despite a later candidate/package change.
status = run("owner_daemon.py", "--activation-status", "--config", config_path)
assert status["complete"] and status["state"] == "armed"
# Inspect with the received candidate too: the old installed status may predate
# computed watchdog coverage. This read does not repin, arm or change history.
before = owner.activation_status(config_path)
assert before["complete"] and before["scope"] == "fixture-only"
print("BEFORE: " + check_coverage(before, partitioned=False), flush=True)
print(json.dumps(dict(before_status=before)), flush=True)
run("owner_daemon.py", "--disarm", "--config", config_path, "--generation", status["generation"])
run("activate.py", "--owner", "uninstall", "--root", root)

# The job is absent before replacing source. Never delete SQLite or receipts.
source = Path(owner.__file__).parent
for path in source.glob("*.py"):
    if not path.name.startswith("test_"):
        target = runtime / path.name
        shutil.copyfile(path, target)
        target.chmod(0o600)
# activate.py reads this adjacent installation asset; owner ticks use only .py.
template = runtime / "com.corbanu.initiative-owner.plist.in"
shutil.copyfile(source / template.name, template)
template.chmod(0o600)

evidence = root / ("promotion-80-" + str(time.time_ns()))
evidence.mkdir(mode=0o700)
replacement = evidence / "config.json"
config.update(transport=transport, package_digest=owner.package_digest())
f.write_json(replacement, config)
run("owner_daemon.py", "--reconfigure", replacement, "--config", config_path, "--schedule", root)

# Freeze the current exact action/claim/allocation/status map. Initial partition
# includes EVERY pending action: selected prepared work to owner; all else hand.
snapshot = coordinator.snapshot()
assert snapshot["manager"] is None
for key in selected:
    assert snapshot["actions"][key]["status"] == "prepared"
assignments = {
    key: {**{field: action.get(field) for field in ("claim", "allocation_digest", "status")},
          "from": coordinator.dispatch_owner(snapshot, action),
          "to": "owner" if key in selected else "hand"}
    for key, action in snapshot["actions"].items() if action["status"] not in TERMINAL
}
request = evidence / "handoff.json"
f.write_json(request, dict(expected_revision=snapshot["revision"], assignments=assignments,
                          evidence=dict(decision=os.environ["OWNER80_DECISION_ID"],
                                        authority=os.environ["OWNER80_AUTHORITY"],
                                        manual_deliveries_quiesced=True,
                                        single_coordinator=str(coordinator.directory))))
result = run("owner_daemon.py", "--handoff", request, "--config", config_path)
assert result["state"] == "HANDED_OFF"
status = run("owner_daemon.py", "--activation-status", "--config", config_path)
assert status["complete"] and status["state"] == "off"
assert {key for key, row in status["coordinator"]["ownership"].items()
        if row["owner"] == "owner"} == selected
coverage = status["coordinator"]["watchdog_coverage"]
print("BEFORE ARM: " + check_coverage(status, partitioned=True), flush=True)
# Prepared selected actions are in ownership, not the overdue coverage count.
# Every excluded action and all future default hand work require manager-owned
# stall detection. --hand-run is one pass, not a recurring hand watchdog.
print(json.dumps(dict(excluded=coverage["excluded_actions"],
                      unresolved_holds=status["unresolved_holds"])), flush=True)
decision = evidence / "activation.json"
f.write_json(decision, dict(decision_id=os.environ["OWNER80_DECISION_ID"],
                           revision=int(os.environ["OWNER80_DECISION_REVISION"]),
                           authority=os.environ["OWNER80_AUTHORITY"], scope="tmux-workers",
                           generation=status["next_generation"], config_digest=digest(config),
                           package_digest=owner.package_digest()))

# Repin only the OFF, uninstalled job. Preserve label/domain/cadence/publication.
run("activate.py", "--owner", "install", "--repin", "--confirm-live",
    "--root", root, "--label", receipt["label"], "--domain", receipt["domain"].split("/")[0],
    "--interval", receipt["interval"], "--publish-state", receipt["publish_state"],
    "--python", python, "--python-sha256", pins["python_sha256"],
    "--runtime", runtime, "--config", config_path)

# Establish/observe the OFF hold before arming; the installed kickstart may
# already have done so. This does not dispatch a worker.
run("owner_daemon.py", "--run", "--schedule", root, expected=2)
run("owner_daemon.py", "--arm", "--config", config_path, "--authority", decision)
run("owner_daemon.py", "--schedule", root, "--recover",
    "tmux promotion: shared ownership map inspected; manual raw delivery stopped; pins and allocations verified")
after = run("owner_daemon.py", "--activation-status", "--config", config_path)
assert after["complete"] and after["state"] == "armed" and after["scope"] == "tmux-workers"
print("AFTER: " + check_coverage(after, partitioned=True), flush=True)
print(json.dumps(dict(promotion_evidence=str(evidence), runtime=str(runtime),
                      python=str(python), config=str(config_path))))
PY
```

A busy command, stale revision, unresolved operation, drift, active owner claim
or wrong state is a refusal to inspect, not a reason to edit SQLite. If a step
fails after disarm, retain its output; admission stays OFF until the explicit
arm step. The initial installation kickstart's OFF hold is cleared only by the
last explicit recovery command. A post-arm failure may leave admission armed:
use the disarm command below to revoke it. This recipe is a promotion procedure,
not an automatic retry/recovery utility.

## First five minutes

Read the printed exact `python`, `runtime` and `config` paths into shell variables
`OWNER80_PYTHON`, `OWNER80_RUNTIME`, `OWNER80_CONFIG`, then run:

```bash
"$OWNER80_PYTHON" -E -s -S -B "$OWNER80_RUNTIME/owner_daemon.py" \
  --activation-status --config "$OWNER80_CONFIG"
"$OWNER80_PYTHON" -E -s -S -B "$OWNER80_RUNTIME/owner_daemon.py" \
  --observe --schedule "$OWNER80_SCHEDULE"
```

At the cutover, inspect `coordinator.dispatch_control.revision/at` and every
`coordinator.ownership` row: selected prepared work is owner-owned; existing
hand claims keep their exact claim and allocation digest. Unassigned future
actions default to hand. Both hand and daemon command paths use this same config.
Inspect `coordinator.watchdog_coverage.excluded_actions`: their stall detection
belongs to the manager. Check top-level `unresolved_holds`, including
`excluded_from_owner_lane`; an excluded hold remains unresolved and blocks
reconfiguration. Moving ownership or running schedule `--recover` does not
resolve it. No hold-resolution command is implemented; do not edit SQLite to
manufacture resolution or describe disarm as a complete scope rollback.

During minutes 0–1, expect the first interval tick, a new owner boot and selected
actions moving prepared → dispatching. During minutes 1–3, check actual ACK,
START and working evidence in the private worker rollout/journal. Pane text or
sent keys alone do not establish ACK or working. Every action should have one
claim, one prepare, one launch, one prompt and one START; pending ACK/readiness
is not a successful worker execution.

During minutes 3–5, require continued interval successes and forward progress
or a specific truthful failure. Returned work remains unaccepted until the
existing manager verifies it. Hand claims must keep their state except for
their own dispatcher activity. `BUSY` is an explicit skipped pass, not an
admitted success; repeated BUSY or stale last-success is contention to resolve.
Watch for `prior_activation`, `effect_uncertain`, `lease_expired`,
`missing_ack`, runtime/credential prompts, or wrong ownership. A recurring
dashboard mark alone does not prove worker supervision.

A cooperating manual TMUX pass uses:

```bash
"$OWNER80_PYTHON" -E -s -S -B "$OWNER80_RUNTIME/owner_daemon.py" \
  --hand-run --config "$OWNER80_CONFIG"
```

It only operates hand-owned claims with the shared journal. Legacy claims
without that journal are reported `external_hand_claim`; their existing hand
dispatcher remains responsible for them. Do not send raw keys into daemon
panes or run another coordinator against overlapping work. These fences govern
cooperating tools, not a hostile process with the same OS account.

For subsequent transfer, write a private JSON request with the current
coordinator revision and each selected action's exact `from/to/claim/status/
allocation_digest`, then use the same command in either direction:

```bash
"$OWNER80_PYTHON" -E -s -S -B "$OWNER80_RUNTIME/owner_daemon.py" \
  --handoff /absolute/path/to/frozen-transfer.json --config "$OWNER80_CONFIG"
```

The successful SQLite commit is the transfer instant; its revision/time and
evidence are returned and audited. An active journal-backed claim preserves its
claim ID and completed deliveries. Legacy hand claims cannot be silently
adopted: finish or reconcile them through the existing protocol and hand over
new prepared work.

To revoke admission, use the generation from the latest activation-status:

```bash
"$OWNER80_PYTHON" -E -s -S -B "$OWNER80_RUNTIME/owner_daemon.py" \
  --disarm --config "$OWNER80_CONFIG" --generation CURRENT_GENERATION
```

Disarm revokes both shared transport dispatchers; it does not terminate workers.
Retain raw evidence and let the existing owner reconciliation handle them.
