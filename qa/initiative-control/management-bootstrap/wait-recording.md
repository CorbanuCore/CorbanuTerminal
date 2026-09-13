# Passive manager wait recording

PF-80-S01 internal coordinator unit; product heading **Internal delivery control
— TO BUILD**, requirement “Use sequential sprints per initiative”. Allocation:
`docs/research/tasknode-integration/coordinator-bootstrap-20260913.md` in the
receiving checkout. Source branch `bootstrap/coordinator-wait-20260913`, base
`9bafae46ba615768d9fff3e72c6beddf54771f95`; four-file scope, target200/100
changed total/non-test lines, hard300/150. Parent owns implementation and receiving.

## Contract

`coordinator_cli.py --state <private directory> record_wait` takes owner JSON
with `action_id`, `expected_revision` and a nonempty `evidence` object. Inspect
the current snapshot first. Record only a prepared `wait` from an already accepted
manager decision with an unchanged allocation. The audit and referenced evidence
mean “wait observed”, not “blocker resolved”, worker success or sprint completion.

Global/stream pauses, dependencies, human decisions and sprint lifecycle do not
change. No claim, native worker, dispatch, ACK or return is manufactured. There
is no meaningful event for passive settlement, preventing a wait-triggered manager
loop. An active manager, stale owner revision/allocation, non-wait kind or any
claimed/terminal wait rejects atomically. Existing history retention applies.

## Evidence and limits

Initial final-behavior suite: 83 tests passed in 6.429s with the pinned bootstrap
SDK environment, `PYTHONPATH=scripts/initiative_control python -B -m unittest
test_coordinator test_manager_cycle test_integration`. Six new tests include CLI
invocation, reopened state, all other kinds, cancelled/uncertain waits, duplicate
denial, two concurrent connections and 52 successive waits with no pending growth.
Concurrency uses two threads with separate SQLite connections, not two OS processes.
Fixtures do not establish actual native dispatch or unattended controller behavior.

Independent review uses Fable5.1High via the established Corbanu wrapper. The
review result, receiving commit/tests and actual private-state settlement will be
recorded by the integrator after completion; this source record claims none yet.
Review scope is the four-file diff and same-boundary invariants, not a new runtime.

Integrator accepts internal-only functional N/A: this owner CLI bookkeeping adds
no human-facing interactive workflow. The later mandatory independent isolated
Slack/dashboard and full manager/worker handoff qualification remains open.
TensorCash/Isometric, benchmarks, release and human acceptance are not qualified
by this change. No product dispatch, recurrence, main or release activation.
