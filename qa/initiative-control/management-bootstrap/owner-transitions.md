# Executable owner lifecycle transitions — September 13, 2026

Internal product-initiative implementation under PF-80-S01 (`in_progress`) and
the active initiative-delivery-control plan. Product heading: **Internal delivery
control — TO BUILD**, requirement: “Use sequential sprints per initiative”.
The Corbanu development skill routed governance and scope checks; parent owns
plan/sprint ledgers, independent Fable review and receiving integration.

## Allocation and provenance

- Action: `owner-transitions-01`.
- Allocation digest: `2f6260d33bd7d64554cea4f0fb64f140d03021108fc70fdc8229d975774e892e`.
- Claim: `be470463-8f5d-491c-8be0-f17fff7c79de`.
- Host-issued worker: `01a09c8a-de49-7480-9641-db3494b843fd`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-transitions-20260913`.
- Branch: `bootstrap/owner-transitions-20260913`.
- Implementation base: `b004bb06c1b7aad432c0633ffdf319b2d247d8df`.
- Clean allocation-only startup HEAD: `5b1d343dfac37d877d740bba9dec244e99b08991`.
- Exactly `coordinator.py`, `coordinator_cli.py`, `test_coordinator.py` under
  `scripts/initiative_control/`, and this receipt are authored.
- Targets: 600 changed / 250 implementation lines; stop limits: 850 / 350.

## Owner contract

The existing owner-only JSON CLI now accepts an unclaimed `prepared`
`complete_sprint` proposal through `complete_sprint`, with exact current
`expected_revision`, owner `evidence`, and mandatory `gates`. Receiving action
verification, exact assignment/commit, each required successful receiving test,
dependencies, reservation counts, pause and pending-action checks still apply.

For a prepared `prepare_successor` proposal, `activate_successor` additionally
requires a separate nonempty `activation_authority` object supplied by the trusted
owner. Manager preparation or ordinary evidence cannot supply that argument.
The predecessor must already have verified completion and owner-recorded archival,
and the frozen successor base must match its verified receiving commit.
`activate_successor` action kind is not treated as a `prepare_successor` proposal.

Each successful operation commits its SQLite lifecycle change, actual
`owner_effect` evidence reference, terminal action status, audit and event in one
transaction. The CLI returns that evidence reference. Prepared actions become
`accepted` with this reference as verification. Existing accepted actions remain
supported, including actions in durable history; their earlier verification is
preserved and the actual effect is added. No owner action needs a native claim,
agent ID, dispatch, ACK or return. Active manager ownership and shared resources
are checked, and claimed/uncertain/dispatched/running/returned actions cannot
enter this owner path. The current proposal alone is excluded from pending checks.

`archive_sprint` remains the separate owner-confirmed database operation; it does
not move repository documents. All failed calls roll back state, evidence, events,
audit, history and SQLite sequence changes. Restart never automatically retries.

## Actual verification

- Focused coordinator run: 50 tests passed in 2.274 seconds.
- Required combined command: `env PYTHONPATH=scripts/initiative_control python3 -B -m unittest test_coordinator test_integration test_native_owner test_manager_cycle`.
- Combined result: 113 tests passed in 8.821 seconds; zero failures/errors.
- `python3 docs/plans/check.py`: passed, active 3/3.
- `python3 docs/sprints/check.py`: passed, current 115 / archived 126.
- `git diff --check`: passed.
- New tests cover a real disposable Git merge and passing receiving test through
  completion/archive/activation/first successor claim, without native lifecycle
  fixtures for the two owner actions. Receiving-worker transport remains a labelled
  fixture. Other gates are synthetic and do not assert human acceptance.
- Actual subprocess exits before commit preserve every database row; exits after
  commit preserve the complete effect and reject stale replay after reopening.
  Negative cases cover receiving/gate failures, all other action kinds, owned and
  terminal states, pauses, revision/evidence/allocation errors, manager ownership,
  cross-stream resources, pending actions, preparation-only authority and bad base.

## Remaining limits

No actual sprint completion, repository archival, product resumption, recurring
enablement or functional acceptance is claimed. True-TUI, live repositories and
browser acceptance are not applicable to this internal SQLite increment; the later
combined dashboard/Slack workflow retains independent isolated functional gates.
Parent must accept this N/A, perform the allocated Fable review, integrate and
rerun receiving checks, then qualify the actual owner/full-loop workflow. Human
sign-off, live Slack/native proof and release/benchmark qualification are not
provided by this worker. Prior failures/reviews remain unchanged; no review was
launched or consumed here. No service, credentials, private state or network used.

## Parent receiving acceptance

Worker ca0474fe43ec868c9171c3b671dfe7161d97c2c5 passed its separate Fable High
review with no findings. After the retained manager-packet failures and reviewed
batching correction, an actual fresh Fable proposed its exact integration.
Fresh Astra High Heisenberg performed the sole receiving operation, producing
ee93206c0f8b070bd9cc767defb3d364ba4fd06b with parents
5b2ac8ea620fc8a01053316f2adadb3efc89555a and the worker commit above.
119 combined tests passed in 8.997 seconds; plans/sprints/diff checks passed.
Parent independently checked exact assignment, scope, all test argv/exit codes,
log hashes, clean tree and absent integration marker. Actual executor was closed
and subsequently reported not_found; coordinator accepted receipt at revision 253.
The internal-only N/A is accepted for this increment. Actual product completion,
document archival and successor launch still require the full lifecycle rehearsal
and all applicable gates; no sprint has been completed by this merge.
