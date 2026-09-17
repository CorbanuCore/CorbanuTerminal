# RETURN

Action `pf83-windows-90`; model `gpt-6-astra`; effort `high`.
Allocation digest `c59ea1ac5e6da20a650b672134ebfe460cf4be531201861cb507a7306517a6f1`.
Claim `e30ed8bb-27d0-4add-b31a-c2312dcacccc`.
Brief SHA-256 verified before work with `shasum -a 256`:
`64e4c35c368f16a90d024abac1165fe5eec4bfaa2ff8430e3f1b941ebc591899`.
Base/HEAD `29d8f67572333bab08de7236faec07beaf2d64d2`;
worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`.

Routine internal QA-contract revision supporting product heading **Permission
selection confirmation — TO BUILD**: “A submitted selection is not a confirmed
change” and “Existing active-turn approval/sandbox snapshots and pending approvals
are not retroactively changed.” Related plan `p0-security-levels.md`, sprint
`PF-83-S01`. No product implementation or sprint completion claimed.
Independent functional design/execution is N/A for this internal control revision,
subject to integrator acceptance; the later mandatory gate is admitted,
independent exact-package execution and independent evidence review. No human-test
readiness, human acceptance, live-repository qualification, benchmark or release
qualification is claimed. No guest contact, live-profile access, native prompt,
commit or push was performed/observed.

## Continuous F07 window

The distinct late turn accepts observation, first-effect and approval-decision
times in `[turn_ended_ns, observation_ended_ns)`. Decisions/effects must also
be at or before their row's observation. This continuously covers the former
turn-end-to-dwell gap, the dwell instant and the remaining interval to cutoff.
At least one late row must still be strictly after the frozen dwell, so accepting
early evidence cannot weaken late coverage. Settled observations remain in
`(settlement_ns, turn_ended_ns)` on the settled turn.

Controls accept silent and approved observations at times 15, 16, 17, 18 and 19
for turn-end 15, dwell 18 and cutoff 25, alongside the mandatory post-dwell row.
They also accept a time-20 observation reporting a time-15/16/17/18 effect and
approval, and reject gap-only coverage. The exact turn-boundary instant requires
independently established distinct-turn identity; uncertain attribution stays
raw evidence with a blocked/inconclusive timing claim.

## Bound F09 outer rows

All non-null effect and approval-decision times now share their phase bounds
and cannot postdate the observation:

- `pre_change`: old turn, `[0, selection_ack_ns)`, at/before observation.
- `in_continuation`: old turn, `(selection_ack_ns, turn_ended_ns)`, at/before observation.
- `new_turn`: distinct new turn, `(turn_ended_ns, observation_ns]`.

Negative clock values are schema-invalid. Fixed controls reject pre-change
events after observation or at/after selection, and new-turn events at/before
turn-end or after observation. Valid boundary-adjacent events, null absent
decisions/effects, and contradictory effect-before-approval evidence remain
recordable. F09 exact selection/end ties retain their existing strict-boundary
ambiguity; retain raw evidence and mark the affected claim inconclusive/blocked.

## Extended weakening controls

Round-86 hand-written fixtures remain unchanged. New [test_strength.py](test_strength.py)
adds recursive omission checks over fixed RESULTS fixtures for all six statuses,
including nested evidence, coverage tuples and product-authority decision records.
It adds literal invalid range/enum/type/pattern/empty-string/duplicate values.
Expectations come from fixture literals, never the schemas under test.
[source-freeze.json](source-freeze.json) binds fixtures, controls, checker and schemas.

Each demonstration runs all 29 tests against one in-memory weakening. Both
schemas remain meta-valid. On-disk schemas and fixtures remain unchanged.
Every red result is `AssertionError: ValidationError not raised`, exposing
acceptance of the deliberately invalid fixed input.

| Mutation | Concrete weakening | Failure method | Actual result |
| --- | --- | --- | --- |
| CAPTURE required | Remove F09 `pre_state.captured_authority` from required | `test_frozen_observations_cannot_be_omitted` | 29 tests; 1 failing subtest; exit 1 |
| CAPTURE range | `observer_sequence.minimum` 1 → 0 | `test_capture_frozen_value_constraints` | 29 tests; 1 failing subtest; exit 1 |
| CAPTURE enum | Add `unknown` to `claimed_effective` | `test_capture_frozen_value_constraints` | 29 tests; 1 failing subtest; exit 1 |
| RESULTS required | Make `last_checkpoint` optional | `test_results_frozen_fields_cannot_be_omitted` | 29 tests; 90 failing subtests; exit 1 |
| RESULTS range | `last_checkpoint.minLength` 1 → 0 | `test_results_frozen_value_constraints` | 29 tests; 1 failing subtest; exit 1 |
| RESULTS enum | Add `unknown` to `missing_artifacts.items` | `test_results_frozen_value_constraints` | 29 tests; 1 failing subtest; exit 1 |

Each deliberate run has 28 passing test methods and one failing method; the
RESULTS-required method has 90 failing subtests (15 records × six fixture
variants), not 90 separately dispatched tests. Raw logs and exact command/exit
receipts are the adjacent `weakened-*-command.json` and `weakened-*.log` files.
The unmutated run is **29/29 passed**, exit 0.

## Five refutability lines

Re-read original F05–F09; source SHA-256 still
`c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.

- **F05 — Yes, conditionally:** mandatory pending/withheld/resolution and original-effect observations in `approval-effects.jsonl` can refute silent acceptance, declined effects or reissue outside communicated scope.
- **F06 — Yes, conditionally:** overlap, displayed boundary and separate-probe approval/effect records in `operation-boundary.jsonl` can refute false rollback or excess new-boundary authority without inventing retroactive revocation.
- **F07 — Yes, within the frozen interval:** `selection-order.jsonl` and settled/late authority rows can refute stale visible authority or covert unapproved effects, including the former turn-end-to-dwell gap.
- **F08 — Yes, for exposed routes:** `resolution-authority.jsonl` can refute false success or silently changed authority after cancellation/failure; unavailable routes remain not exercised/blocked.
- **F09 — Yes, with the same-turn probe:** `continuation-effects.jsonl` can refute retroactive continuation authority, misleading phase claims or replay; all three phases now bind their approval/effect times.

These are contract refutability conclusions, not executed functional cases.

## Named residuals

- **R1 — Admitted execution remains unavailable:** prior preflight-only harness,
  missing admitted case/mediator/capture interface and unproven routes still
  block all five functional cases; no operational blocker is closed here.
- **R2 — Raw truth and clock attribution require independent review:** the
  checker cannot authenticate source bytes, prove continuous coverage or
  pre-freezing, resolve clock uncertainty, or itself judge arbitrary textual
  verdicts. Finite F07 cutoff cannot refute events beyond it. F09 records no
  distinct outer turn-start/cutoff timestamp beyond its phase/observation bounds.
- **R3 — Mutation coverage is finite:** the six demonstrated weakenings and
  fixed negative corpus are detected; this is not a proof that every possible
  schema weakening is detected. Newly introduced constraints need independent
  positive/negative fixtures, never expectations derived from the live schema.
- **R4 — TUI leak recurred:** nextest reported LEAK for
  `app::permission_confirmation::tests::permission_confirmation_held_input_resumes_only_after_matching_success`.
  Exit 0 does not erase this caveat; Rust repair is outside this allocation.

## Gate evidence

Prerequisites built first: `codex-cli`, `codex-rmcp-client`,
`codex-code-mode-host` binaries; exit 0. Rust lanes run from `codex-rs` using
guarded `just test`, `--locked --offline --retries 0`,
`NEXTEST_TEST_THREADS=4`, and shared dedicated target
`qa/initiative-control/management-bootstrap/pf83-package-67/artifacts/test-target`.
Test-isolation instructions were read first. No workspace formatter was run.

- App server `thread_settings`: **26 run, 26 passed, 0 failed, 1082 skipped**;
  exit 0. Disposable-profile/native-keyring denial reported.
- TUI `permission_confirmation`: **12 run, 12 passed, 0 failed, 4161 skipped**;
  **1 slow, 1 leaky**, exit 0. Disposable-profile/native-keyring denial reported.
  LEAK: `app::permission_confirmation::tests::permission_confirmation_held_input_resumes_only_after_matching_success`.
  SLOW: `app::permission_confirmation::tests::permission_confirmation_f10_request_does_not_optimistically_apply_or_persist`.
  No retry was dispatched.
- Contract controls: **29 run, 29 passed, 0 failed**; exit 0.
- Draft 2020-12 meta-validation and frozen branch bindings: **2 schemas passed**;
  exit 0.
- Deliberate weakening controls: **6 separate runs × 29 tests**, all expected
  exit 1, with exact failure counts above.

## Changed lines and brief corrections

Changed existing files are `pf83-f09gap-83/check_contract.py`,
`evidence-contract.schema.json`, `test_contract.py` and `runbook.md`.
Fresh controls, runners and receipts are under `pf83-windows-90/`.
Existing tracked edits total **50 insertions, 25 deletions**: checker +19/-13,
capture schema +2/-2, runbook +24/-9, prior controls +5/-1.
The final [changed-lines.txt](changed-lines.txt) records per-file changes and
new-file line counts. Prior-round logs remain untouched. `git diff --check`
passed; final status contains only the assigned writable scope.

Nothing in the brief was found factually wrong. Its round-86 receipt and reviewer
status are supplied manager context, not a new approval asserted by this worker.
