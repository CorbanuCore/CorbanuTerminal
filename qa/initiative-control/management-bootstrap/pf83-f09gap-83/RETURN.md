# RETURN

Allocation `pf83-f09gap-83`; claim `a6577840-a09c-4ee8-a9d3-014ccb3ba204`.
Base `a35700c9bcbfcd730237ca99ea9ea5bae583b62b`; brief SHA-256 verified:
`9f75cac4fb2312cf7f6085dee478356d4d2d24e9a56624cb1959d11c07511d82`.
Routine internal QA-contract revision; product citation, N/A limitations and
later independent functional gate are in [runbook.md](runbook.md).

- **F09 observation:** mandatory typed pre-change, same-turn in-continuation and
  distinct-new-turn probes, with command approval ID/disposition/scope/decision
  time, marker, effect count/first-effect time, raw references and turn/window
  binding. The continuation observation must follow selection acknowledgement
  and precede the old turn's end. Missing observation blocks; a contradictory
  observation remains recordable failed evidence. Raw current/next UI explanation
  is also mandatory. See the runbook's mandatory-observation section.
- **F05 refutability — yes conditionally:** `approval-effects.jsonl` exposes
  pending/declined effects before acceptance or outside accepted scope.
- **F06 refutability — yes conditionally:** `operation-boundary.jsonl` exposes
  boundary/rollback claims contradicted by started/finished markers and fresh
  probe approval/effects, preserving actual overlap and ordering uncertainty.
- **F07 refutability — yes within the frozen interval conditionally:**
  `selection-order.jsonl` exposes stale-selection reversion or unapproved effects.
  A scoped command accept preceding first effect distinguishes an approved
  restricted effect from silent Full Access; setting confirmation is insufficient.
- **F08 refutability — yes conditionally:** `resolution-authority.jsonl` exposes
  success labeling or authority changes following observed cancel/failure;
  missing exposed routes cannot be treated as tested.
- **F09 refutability — yes conditionally:** `continuation-effects.jsonl` exposes
  retroactive restriction/elevation on the same captured turn even when the new
  turn is correct; also preserve replay/context/deferred-state/UI contradictions.
- **Failing runs:** the runbook has one concrete failing-run walkthrough per case,
  with its indispensable artifact and the observations that distinguish failure.
  All five remain operationally blocked pending an admitted executor/mediator/
  capture seam. No actual functional refutation, passing case or handoff is claimed.
- **Coverage/blocked format:** case-specific closed branch enums; frozen expected
  case/branch/queue variant/packet hash/profile/repository tuples;
  `results.schema.json` requires structured status, missing-artifact/prerequisite/
  gap enums, last checkpoint, retained evidence and nullable capture/authority
  records. Missing tuples/original branches and duplicate attempts are rejected.
  Accepted scope dispositions require an actual named-authority decision ref;
  no acceptance is fabricated. External queue completeness still needs review.
- **Retention rule:** verified synthetic data and isolated guest identity may be
  retained verbatim; incidental personal identifiers are redacted before any
  publication and re-encoding, with a value-free transformation ledger. Real
  credentials/live profiles/private payloads must never be captured. Unexpected
  protected data stops capture without printing, persisting or hashing it.
  The old unrestricted publisher is blocked for real capture until admitted
  source isolation and publication filtering enforce the rule. No guest contact.
- **Meta-validation:** `jsonschema==4.25.1`,
  `Draft202012Validator.check_schema`: **2 schemas passed**, exit 0.
  [Raw result](schema-meta-validation.log) and command receipt retained.
- **Contract controls:** **12 run, 12 passed, 0 failed**, exit 0; failure names:
  none. Negative tests cover missing/mistimed/wrong-turn continuation, F07 approval
  ambiguity, invalid branch IDs, missing coverage and unsupported dispositions.
  [Raw result](contract-controls.log). These are synthetic structural tests.
- **Rust gates:** prerequisites built first from `codex-rs`; same dedicated
  `CARGO_TARGET_DIR`, `NEXTEST_TEST_THREADS=4`, isolated environment and guarded
  `just test`, `--locked --offline --retries 0`. App-server `thread_settings`:
  **26 run, 26 passed, 0 failed, 1082 skipped**. TUI `permission_confirmation`:
  **12 run, 12 passed, 0 failed, 4161 skipped**, 2 slow. Both exit 0; failure
  names: **none**. Command/environment receipts and complete logs retained here.
- **Brief correction:** round 82's F09 pre_state did not actually require an
  explicit authority probe; it required completed output and selected change.
  This revision adds the missing pre-change probe as well as the requested
  continuation probe. The reviewer correctly identified the continuation gap.
  Existing identity-mismatch detection remains unchanged; this round makes no
  contrary claim and does not modify the historical round-82 evidence.
- **Scope:** new round-83 contract/results schema, structural checker, synthetic
  tests, runbook and gate evidence; round-70/73 links point future capture to the
  amendment. No runtime source, credentials, live profiles, workspace formatter,
  external messaging, staging, guest execution, commit or push. Changed-line
  accounting is reported by the final worker response; local caches are ignored.

Capture schema SHA-256:
`d07bb3e72b0c339038b0a6dae155097eb42aa85dc64fdd3f407423083bc1031b`.
Results schema SHA-256:
`3e47eb22cb1603322075dbc30d48b7c2bdc4d8e3127330f87e1095cfe813a5ce`.
