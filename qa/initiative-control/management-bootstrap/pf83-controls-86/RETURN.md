# RETURN

Action `pf83-controls-86`; Astra `gpt-6-astra`, effort `high`.
Allocation digest `407adaff2bf04455c9ac9e7849d5877ae7a31835114ac3cf8f0dbd4f6f6789bb`.
Claim `2b64bb8c-bd62-4b6e-95a9-8086fa643329`.
Brief verified with `shasum -a 256`:
`9b89ede775c8ad4ea75941ec79ae543456a8fe4640da75066467be8e55d5fcdc`.
Base/HEAD: `c7b8a9b5336ad91a525d4643f725434e457b9bdc`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`.

Routine internal QA-contract revision supporting **Permission selection
confirmation — TO BUILD**: “A submitted selection is not a confirmed change”
and “Existing active-turn approval/sandbox snapshots and pending approvals are
not retroactively changed.” Related plan `p0-security-levels.md`, sprint
`PF-83-S01`; no product implementation or sprint completion claimed.
Independent functional design/execution is N/A for this internal control-only
revision, subject to integrator acceptance. The later gate remains admitted
independent exact-package execution and independent evidence review. No TUI
functional handoff, human acceptance, live-repository qualification, benchmark
or release qualification is claimed. No guest contact, live-profile access,
native prompt, commit or push was performed/observed.

## Frozen fixtures and deliberate red run

[frozen_fixtures.py](frozen_fixtures.py) contains hand-written F05–F09 observations,
typed F07/F09 rows, windows and all 15 original branch pairs. It imports no
contract/schema and never derives a fixture field from schema requirements.
The shared evidence-envelope helper and branch Cartesian expansion use only
literal synthetic data. Original F05–F09 text was inspected in
`pf83-handoff-70/artifacts/extracted/packet/original-F01-F11.md` and its SHA-256
matched `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`;
round-82/83 case checkpoint text supplies the coordinator capture field names.
[fixture-freeze.json](fixture-freeze.json) freezes fixture, binding, schema and
checker hashes; fixture SHA-256:
`d4f9b4f531867ebbc6cda8422697864b16606a2d17ce1b2dc77566f8a5e1c5c0`.

The unchanged fixtures require omission rejection for every literal observation
in all five cases, both typed windows and all authority rows. Controls also
require all 15 branches, indispensable artifacts and every mandatory phase.
In [weakened-schema-red.log](weakened-schema-red.log), `run_controls.py --weaken`
removes F09 `pre_state.required: captured_authority` from the loaded schema
in memory; it leaves the fixture and on-disk schema unchanged.
Result: **21 tests run, 20 pass, 1 expected failure, exit 1**:
`test_frozen_observations_cannot_be_omitted`,
subtest `case='F09', path=('pre_state', 'captured_authority')`,
`AssertionError: ValidationError not raised`.
Both schemas still meta-validate in that run: this is a weakened requirement,
not invalid JSON Schema. The subsequent unmutated run is **21/21 passed**.
Exact commands and exits are preserved in the adjacent `*-command.json` receipts.

## Window and branch requirements

- F07 now requires `late_window`: settled/distinct late turn IDs, settlement,
  turn-end, dwell deadline, observation cutoff and source. Settled rows belong
  to the settled turn strictly after settlement and before its end. Late rows
  belong to the distinct late turn strictly after both end and dwell deadline,
  before cutoff. Their effect/approval-decision times share those bounds and
  cannot postdate the observation. Different phases require different labels.
  The runbook requires a pre-frozen dwell/cutoff plan and raw boundary review;
  a checker cannot prove the freeze happened before execution.
- F09 `in_continuation` non-null effect/approval-decision times must each lie
  strictly inside `(selection_ack_ns, turn_ended_ns)` and at/before that row's
  observation. Withheld/unoffered approvals may still have null decision times;
  zero observed effects still require null first-effect time. Failing authority
  outcomes and effect-before-approval contradictions remain recordable.
- [branch-bindings.json](branch-bindings.json) pairs all 15 exact round-82 branch
  descriptions with round-83 identifiers. Its SHA-256 is pinned in the checker;
  it pins the untouched round-82 schema digest. Every capture/result check and
  meta-validation checks capture enums/annotations and both results enum
  locations against this mapping. Coverage requirements come from the mapping,
  not the current schema. Rename/drop controls and source/binding tamper controls
  pass; erasing both a result and its manifest branch still fails.

## Five refutability lines

- **F05:** A withheld/declined pending command that effects or is silently reissued is contradicted by mandatory pending/baseline, withheld interval, explicit resolution, original-effects and negative-control observations joined in `approval-effects.jsonl`; dropping any required observation now fails fixed controls.
- **F06:** A new-boundary probe that effects without required approval, false rollback claim, or selection after slow-work finish is exposed by mandatory running witness, overlapping change, separate probe and boundary result joined in `operation-boundary.jsonl`; honest continuation of old authorized work remains valid behavior.
- **F07:** An older completion that overwrites the latest accepted restriction is exposed by settled and distinct-turn late authority rows in `selection-order.jsonl`; wrong-turn, premature, post-cutoff or outside-window approval/effect evidence is rejected, while a silent effect inside the frozen window remains recordable failure evidence.
- **F08:** Cancellation/failure falsely labeled or behaving as success is exposed by mandatory exposed route, actual resolution, remaining authority, recovery and unsuccessful-request control joined in `resolution-authority.jsonl`; unavailable routes remain blocked/not exercised.
- **F09:** Retroactive restriction/full authority in the continuation is exposed by pre-change, SAME-turn post-selection and distinct-new-turn rows in `continuation-effects.jsonl`; correct new-turn behavior cannot erase a contradictory continuation, and outside-window effect/approval cannot qualify that probe.

**None of the five controls still adapts its expected observations to whatever
the schema supplies.** The demonstrated mutation goes red. These contracts do
not automatically determine a truthful verdict: arbitrary textual claims or
synthetic references are not authenticated raw evidence. Independent raw review
and admitted execution remain mandatory; the prior operational blockers for all
five cases remain. These five lines are refutability walks, not executed cases.

## Exact validation results

Prerequisites built first: `codex-cli`, `codex-rmcp-client`,
`codex-code-mode-host` binaries, exit 0. Both Rust lanes ran from `codex-rs`
via guarded `just test`, shared dedicated target
`qa/initiative-control/management-bootstrap/pf83-package-67/artifacts/test-target`,
`NEXTEST_TEST_THREADS=4`, `--locked --offline --retries 0`.
The wrapper reported disposable-profile/native-keyring denial for both lanes.

| Lane | Actual result | Failure names / caveats | Evidence |
| --- | --- | --- | --- |
| `just test -p codex-app-server thread_settings` | 26 run, 26 passed, 0 failed, 1082 skipped; exit 0 | None | [thread-settings.log](thread-settings.log) |
| `just test -p codex-tui permission_confirmation` | 12 run, 12 passed, 0 failed, 4161 skipped; exit 0 | 2 slow, 1 leaky; names below | [permission-confirmation.log](permission-confirmation.log) |
| Unmutated frozen controls | 21 run, 21 passed, 0 failed; exit 0 | None | [contract-controls.log](contract-controls.log) |
| Weakened-schema controls | 21 run, 20 passed, 1 expected failure; exit 1 | `test_frozen_observations_cannot_be_omitted`, F09 captured_authority | [weakened-schema-red.log](weakened-schema-red.log) |
| Draft 2020-12 meta-validation + branch binding | 2 schemas passed; exit 0 | None | [schema-meta-validation.log](schema-meta-validation.log) |

Nextest LEAK: `app::permission_confirmation::tests::permission_confirmation_held_input_resumes_only_after_matching_success`.
SLOW: `app::permission_confirmation::tests::permission_confirmation_f10_request_does_not_optimistically_apply_or_persist`
and `app::permission_confirmation::tests::permission_confirmation_f07_conflicts_are_refused_and_old_completion_cannot_win`.
These are actual gate caveats, not failures relabeled or silently retried. Rust
source repairs are outside this allocation. Existing compiler warnings remain.

## Changed lines and brief corrections

Existing tracked edits: `check_contract.py` +69/-8,
`evidence-contract.schema.json` +19/-3, `test_contract.py` +146/-42,
`runbook.md` +31/-1, all in `pf83-f09gap-83/`: **265 insertions, 54 deletions**.
New hand-written fixtures, pinned mapping, runners and immutable current receipts
are in this `pf83-controls-86/` directory; see [changed-lines.txt](changed-lines.txt).
Round-83 logs remain unchanged historical records at the base commit.
No workspace formatter was run. `git diff --check` passed and status contained
only paths under the assigned writable scope.

Nothing in the brief was found factually wrong. The round-82 descriptions and
round-83 identifiers differ by representation, so the tie is an explicit frozen
mapping rather than string equality between those two forms.
