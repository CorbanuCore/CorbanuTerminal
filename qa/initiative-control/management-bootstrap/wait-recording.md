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

## Receiving and actual state

Fable5.1High review01 found only the incorrect documentation flag `--state-dir`;
accepted and corrected to `--state`, verified against actual CLI help. Review02
completed clean, exit0, no findings; no further review of unchanged code. Retained
private receipts: `.codex-work/wait-review.II2ijj/review01.json` and `review02.json`.
Source `910a302662e10cfbe67a89760c1df9d8c270a175` has160 changed lines/64 non-test.
Single-writer receiving commit `43e65299124d05ffec5dc76fa4085d8a986cf750` passed83
tests in6.044s plus plan/sprint governance and whitespace checks. Exact assignment,
merge and hashed test logs: `receiving/receive-wait-910a30266.json` in that directory.

Owner CLI actually recorded four existing Fable waits, revision54 to58, preserving
global/three-stream pauses, manager=None, all sprint/allocation records, rationales
and all event rows byte-for-byte. Each operation has a real CLI receipt and audit.
The original post-check raised KeyError because normal terminal retention archived
`rehearsal01-delivery-native-ack-readonly`. No settlement was retried. A separate
read-only recovery check compared live plus archived records and proved that action
unchanged, all four waits recorded and no native receipt manufactured. Original
script/receipts are retained alongside `actual-verification.json`; event digest
`5f68e581a1a69d5fe8bfec7c4356a9fad09633eaacda68c0d62f8b2160d6fef5` unchanged.
This is real owner bookkeeping proof, not autonomous dispatch or resolved blockers.
