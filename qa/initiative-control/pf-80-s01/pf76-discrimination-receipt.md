# PF-76 provenance discrimination — revise receipt

Allocation: `tasknode-pf76-discriminate-01`; worker: `gpt-6-astra`, effort `high`.
Digest: `c4fa129dfc47fc42342c3fec73e27e4c82fcdaf66c60ae94cd2e00a1133d9482`.
Claim: `0223ae38-bbab-4dc7-907d-46c5b29a0e52`.
Base: `f81a38a68dcc7725c193a1b0dc6ecddca6b90c2b`; branch: `bootstrap/tasknode-pf76-20260916`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf76-20260916`.
Brief SHA-256 verified: `f8c43b3a63cc78cd05c8148c22f486d9f877c8558bc580a0691531b9e5e7b59f`.
Bounded fix; product heading **Internal delivery control — TO BUILD**: “Task Node receives only explicitly mapped, supported progress”.
PF-80-S01 remains in progress; this receipt does not complete its live or functional gates.

Before: refuse when report `sprint_id` / event `turnId == "PF-76-S01"`, irrespective of provenance.
After: refuse when that ID is `PF-76-S01` and local `source_namespace != "main-provider-profile-persistence"`, including missing/unrecognized provenance.
The fixture namespace originally exists only on its wrapper; reports and outbox records had no namespace. The optional report field is now validated and copied to new local outbox records, never to the immutable wire payload.
All four Task Node guards and dashboard collection use the same predicate. Historical notices link to the history document; unknown provenance is unresolved, not attributed to the provider sprint.
The live PF-76-S01 mapping becomes code-reachable for explicitly tagged modern reports. Existing producers without provenance remain held. No live state or producer configuration was read or changed; live reachability was not exercised.
Original fixture, queued event payloads, IDs and receipts are preserved. No alias, migration, replay, deletion, enrollment change, live post or push occurred.

## Regression evidence

- `test_tasknode.SendTests.test_historical_pf76_provenance_is_refused_with_reason`: before FAIL (generic ID-only refusal lacked the provenance reason); after PASS. Also checks unknown/missing provenance, preview, retry and historical flush preservation.
- `test_tasknode.SendTests.test_modern_pf76_provenance_reaches_mapping`: before ERROR (report schema rejected namespace); after PASS. Checks report acceptance, mapping, local metadata, idempotence, disabled posting, retry and mocked flush.
- Extended existing attention test proves collection separates historical, unknown and modern reports; existing history/modern link cases remain passing.
- Final focused run: 100 tests, 3.616s, exit 0. Broad discovery: 518 tests, 156.336s, exit 1 solely for the five declared missing-`slack_sdk` errors: test_decision_feed, test_decision_inspection, test_decision_manager, test_slack_transport and test_slack_reply_poll's real-SDK case. No dependency repair attempted.

## Exact verification commands

Python commands ran from `scripts/initiative_control`; governance/hash commands from the worktree root.

```sh
shasum -a 256 /private/tmp/fmgr.Q1SIYZ/briefs/tasknode-pf76-discriminate-01.json
TMPDIR=/private/tmp /Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python -m unittest test_tasknode.SendTests.test_historical_pf76_provenance_is_refused_with_reason test_tasknode.SendTests.test_modern_pf76_provenance_reaches_mapping
TMPDIR=/private/tmp /Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python -m unittest test_tasknode test_attention test_preparation test_control
TMPDIR=/private/tmp /Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python -m unittest
python3 docs/plans/check.py && python3 docs/sprints/check.py
git diff --check
```

Exit codes in order: 0; 1 before / 0 after; 0; 1 (known dependency errors); 0; 0.
Internal-only code handoff: no runtime TUI change or built candidate. Proposed TUI/live-repository/code-blind N/A requires integrator acceptance; later PF-80 functional/live gates remain mandatory. No human-test readiness, human sign-off, benchmark or release qualification claimed.
