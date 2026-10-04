# RETURN

Allocation `pf83-capture-82`; base `2b03d5f01e36ac2434f164fa9ab2b3fe40d65fd3`.
Brief SHA-256 verified:
`252d09f4fc6cf96853da0e1da809e51e43c68d87ab096cdfbfc14d55693ad2ac`.
Routine internal QA preparation; authority, N/A limits and later independent
functional gate are recorded in [runbook.md](runbook.md).

- **Observed identity:** preflight retains exact SSH stdout/stderr bytes in
  `guest_identity_observation`, plus exit status, timeout and decoded stdout when
  available. It checks returned bytes against frozen identity expectations.
  Mismatch/nonzero exit/invalid UTF-8/timeout fails while retaining available
  output. Success detail is the observation itself, not expected constants.
- **Receiving refusal:** missing/invalid `--receiving`, wrong bundle/packet
  locations or occupied outputs refuse all three artifact checks. The no-argument
  report is now **1/12 passed, 11 failed, exit 1**, no guest contact; it does not
  verify the default bundle as a moved artifact.
- **Evidence contract:** [Draft 2020-12 schema](evidence-contract.schema.json)
  requires pre-state, observed transition, post-state, negative control and
  refuting artifact for each original, with raw-source locators/digests.
  SHA-256: `90d917a82807c6caa1a4b4553eeb489ca0c3a69b31f189b4b6b4b91d0a5a3342`.
  It is coordinator-only and does not rewrite the frozen independent cases.
- **Indispensable artifact per case:** F05 `approval-effects.jsonl`;
  F06 `operation-boundary.jsonl`; F07 `selection-order.jsonl`;
  F08 `resolution-authority.jsonl`; F09 `continuation-effects.jsonl`.
  Missing artifact means unusable, even with screenshots. Other required evidence
  also remains mandatory.
- **Runbook changes:** round 70 now requires the contract before case dispatch;
  round 73 documents receiving refusal and real identity capture and requires
  admitted capture capability. The new supplement freezes timing/labels/observer
  controls before keys, joins raw events per case, requires post-quiescence
  snapshots and negative controls, and requires independent raw-evidence review.
  F06 must not wait for slow completion before submitting its separate probe;
  F07 must retain late-event observation; F08 cannot substitute hidden injection;
  F09 must compare old output counts/hashes across continuation and new turn.
- **Unavailable refuting evidence:** all F05–F09 are blocked under the current
  preflight-only harness (`native_dispatch=false`, preflight-only operation
  guards, no approved case/mediator/capture interface). Case-specific navigation,
  overlap, conflict, exposed failure and continuation opportunities remain
  unproven, not presumed impossible. No functional pass or handoff is claimed.
- **Exact gate counts:** prerequisites built first, shared dedicated target,
  `NEXTEST_TEST_THREADS=4`, isolated environment, guarded `just test`,
  `--locked --offline --retries 0`. App-server `thread_settings`: **26 run,
  26 passed, 0 failed, 1082 skipped**. TUI `permission_confirmation`: **12 run,
  12 passed, 0 failed, 4161 skipped** (one slow). Both exit 0; failure names:
  **none**. Seven local preflight controls passed, including six identity-field
  mismatch subcases. Raw logs and command/environment receipts are retained.
- **Checks and limits:** Python syntax and JSON parsing pass; no full JSON Schema
  meta-validator is installed in the host Python, so formal meta-validation was
  not run. Independent functional evidence review and integrator N/A acceptance
  remain pending. The schema/checks cannot establish the truth of future results.
- **Brief correction:** mismatch detection already existed via
  `stage.check_identity`; recording expected constants made the evidence
  misleading, but did not prevent that existing comparison from rejecting a
  wrong guest. The prior 3/12 diagnostic is historical; this fix intentionally
  reduces today's no-argument result to 1/12. The pending human decision was not
  queried or represented as approved.

All authored changes are under `qa/initiative-control/management-bootstrap/`.
No guest contact, staging, functional cases, live profile, credential read,
workspace formatter or push occurred. Raw gate logs are committed with this
return. Exact changed-line counts are available in the accompanying commit's
`git show --numstat`; the worker's final return reports the totals.
