# PF83 round 83 — refutable captures

Round-86 revision: use the hardened schema/checker in this directory and the
[frozen controls and current receipts](../pf83-controls-86/RETURN.md).
Round-83 logs/command receipts remain historical evidence at base commit
`c7b8a9b5336ad91a525d4643f725434e457b9bdc`; they do not qualify this revision.

Routine internal QA-contract revision supporting product heading **Permission
selection confirmation — TO BUILD**: “A submitted selection is not a confirmed
change” and “Existing active-turn approval/sandbox snapshots and pending approvals
are not retroactively changed.” Plan `p0-security-levels.md`, sprint `PF-83-S01`.
No product behavior changes. Independent functional work is N/A for this internal
revision, subject to integrator acceptance; admitted exact-package code-blind
execution and independent evidence review remain the later gate. No functional
handoff, human acceptance, guest contact or staging is claimed here.

This supplement replaces round 82's contract for FUTURE captures, preserving its
historical schema, hashes and results. Follow its ordered capture runbook with
the amendments here; freeze the round-83 schema hash before keys. Original cases
remain frozen at `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.
This source-informed contract is coordinator-only, never a blind-executor brief.

## Mandatory authority observations

F09 requires `pre_state.captured_authority`, `observable.continuation_authority`,
and `post_state.new_turn_authority`, plus typed `authority_observations` for
`pre_change`, `in_continuation` and `new_turn`. Their `source` must cite actual
rows in `continuation-effects.jsonl`. Each row includes command-probe label,
actual turn/session identity, monotonic time, claimed level, command approval ID,
disposition, scope and decision time, independent marker existence, effect count,
first-effect time and observer sequence. Null approval ID means no prompt was
observed throughout continuous PTY coverage; it does not mean unknown. Missing
observations go to a blocked record. Every row cites original raw PTY/key and
observer records with digests and exact locators. Copies in capture.json must
match the referenced timeline rows field for field.

Freeze `continuation_window`: captured turn ID, distinct new turn ID, selection
acknowledgement/deferment time, current-turn end time and raw boundary reference.
Observe the calibrated command probe before selection; after acknowledgement,
trigger a distinct probe WITHIN THE SAME continuation, before that turn ends;
then probe a distinct new turn. Never manufacture this by submitting a new user
turn or merely watching an already-started command finish. Probe current-turn
command authority; shared-service policy is not the same boundary. Unknown or
ambiguous clock ordering blocks the claim. If the admitted interface cannot
produce this observation, mark `continuation_probe_route` blocked.

An `in_continuation` row's non-null `first_effect_ns` and
`approval_decision_ns` must each be strictly inside
`(selection_ack_ns, turn_ended_ns)`, and no later than its `monotonic_ns`.
Null still means observed absence under continuous coverage. Outside-window
events stay in raw evidence and make this probe insufficient; never drop an
event or relabel it to claim the required window was exercised.

Full→restricted: the continuation probe should retain captured Full Access and
effect without fresh approval, while a new-turn restricted probe remains
unapproved with no effect. Restricted→full: withhold approval for the continuation
probe and observe no effect; a distinct new-turn Full Access probe effects
without approval. Preserve the visible explanation of which authority governs
each phase: correct effects cannot excuse a missing or misleading explanation.
For deferred changes, retain the request through the boundary and its actual
resolution; do not silently treat an unavailable defer path as covered.

F07 requires typed `settled` and `late` authority observations, each pointing to
`selection-order.jsonl`. Its indispensable timeline now includes command
`approval_id`, `approval_disposition`, `approval_scope`, `approval_decision_ns`,
`first_effect_ns`, `marker_exists` and observer sequence. A restricted approved
effect has an observed matching scoped command approval and accept key BEFORE
its first effect. Silent Full Access has an effect with no such acceptance (or
before it). A permission-setting confirmation cannot stand in for command
approval. In the mandatory negative probe, explicitly withhold command approval;
any effect contradicts restricted authority. An additional approved control may
run separately with another label. Keep late probes and raw observation through
the frozen dwell and turn boundary; disclose the finite observation window.

F07 now requires typed `late_window` with `settled_turn_id`, distinct
`late_turn_id`, `settlement_ns`, `turn_ended_ns`, `dwell_until_ns`,
`observation_ended_ns` and a sealed `source` reference. Freeze the planned dwell,
cutoff and probe labels before keys; record actual turn IDs and boundary events
from raw capture, binding the window before the corresponding probes. The
reviewer must compare these with the frozen plan rather than accept a window
chosen retrospectively to fit results. Settled rows must belong to the settled
turn strictly between settlement and its end. Late rows must belong to the
distinct late turn strictly after BOTH the turn boundary and dwell deadline and
before observation cutoff. Row, effect and approval-decision times share the
phase window; decisions/effects cannot postdate the observation. Each phase has
a distinct probe label. A later timestamp alone cannot satisfy late coverage.
F06 and F08 timelines use the same command-approval disposition/scope/decision
and first-effect columns, so their approval IDs cannot stand for acceptance.
For every case, effect entries cite independent observer bytes and approval
entries cite actual prompt/keys; null denotes observed absence, never unknown.

## Failing-run walkthroughs and refutability

| Case | Concrete failing run and what its indispensable artifact exposes | Can this contract refute the failure? |
| --- | --- | --- |
| F05 | Full Access selection silently accepts/reissues a pending restricted command. `approval-effects.jsonl` joins original approval ID/scope, withheld keys and independent marker/start before accept; decline followed by an effect also fails. Reissue labels stay linked to original scope. | Yes, given admitted continuous capture: withheld/declined effects and excess accepted scope are observable. |
| F06 | After a selection during a slow operation, UI claims the new-turn boundary is restricted but a fresh command runs there without approval; or UI claims interruption/rollback while the old operation continues. `operation-boundary.jsonl` places start < selection < finish/interruption, preserves partial effects, separate probe admission/queue and actual approval/marker against the displayed boundary. Continuing under the honestly disclosed old snapshot alone is not a failure. Selection after finish invalidates coverage. | Yes, given admitted overlap and calibrated boundary probes: false boundary/rollback claims contradict observed order/effects. |
| F07 | Older Full Access completion overwrites a newer accepted restricted selection after settlement, with either a visible reversion or a covert unapproved effect. `selection-order.jsonl` retains all selection dispositions, later frames, settled/late probes and command acceptance-to-first-effect order. An approved restricted effect has a matching prior accept; silent Full Access lacks it. | Yes within the frozen observation interval, given actual conflict opportunity; serialized attempts cannot close overlap coverage. |
| F08 | Cancel/failure is labeled success, or silently applies requested Full Access while UI still reports restricted. `resolution-authority.jsonl` joins offered route, starting probe, actual cancellation/failure, remaining-level claim and fresh command approval/effect, then recovery. In the reverse direction a formerly Full Access probe unexpectedly requiring approval exposes silent restriction. | Yes, given an exposed route and calibrated probes; missing routes stay not_exercised/blocked. |
| F09 | Full→restricted selection retroactively re-governs the continuation while the new turn correctly becomes restricted. `continuation-effects.jsonl` shows pre-change Full Access effects and SAME-turn post-selection fresh approval/no effect, then correct new-turn restricted behavior. The inverse produces an unapproved effect in a still-restricted continuation. Raw phase explanations, deferred resolution, context frames and old-label counts/hashes also expose misleading level claims, lost state and replay. | Yes, with the mandatory same-turn probe and current/next UI evidence; correct pre/new probes alone are insufficient. |

These are evidence-design conclusions, not execution results. All five cases
remain operationally blocked by the previously recorded preflight-only harness,
missing admitted case/mediator/capture interface and unproven routes. No case can
CURRENTLY supply admitted refuting evidence; the amendments do not remove that
blocker. Actual attempts may fail and still be structurally valid evidence.

## Branch coverage and blocked-record format

Use `results.json` validated by `results.schema.json`. Freeze a coverage manifest
before dispatch, with one expected tuple per original branch × assigned queue
variant/profile/repository combination. Tuple fields are `case`, enumerated
`branch`, `queue_variant`, `queue_packet_sha256`, `profile`, `repository`.
The queue variant and packet digest must match the frozen queue manifest, not a
new alias. Preserve all original branches, including conditional exposed routes;
absence is a disposition, never deletion. The manifest includes all assigned
variants/profile/repository combinations and all 15 enumerated original branches;
independent review compares it with the frozen original and queue before keys.
If a variant cannot exercise a branch, record that tuple's limitation explicitly.

Each record adds `attempt_id`, `status`, `capture`, `missing_artifacts`,
`prerequisites`, `coverage_gaps`, `last_checkpoint`, `retained_evidence`, and
`product_authority`. All fields are required; unknown fields are rejected.
Status is exactly `passed`, `failed`, `inconclusive`, `blocked`, `not_exercised`,
or `accepted_disposition`. Artifact, prerequisite and gap arrays have closed
enumerations in the schema. Last checkpoint explains the specific observed stop,
not a substitute for the structured gap. Blocked requires a named prerequisite;
blocked/not_exercised/disposition requires a coverage gap and null capture.
`retained_evidence` may be empty if no observation occurred; never invent a ref.
Usable outcomes require a capture reference. Passed requires empty gap, missing
artifact and prerequisite arrays. Scope disposition requires the named product
authority, explicit acceptance decision, reason and actual decision-record ref;
otherwise `product_authority` is null. No approval is supplied by this contract.

`check_contract.py` meta-validates both schemas, validates documents, rejects
missing/unexpected coverage tuples and duplicate attempt IDs, requires all
original branches, and checks F07/F09 turn/window ordering. The hash-pinned
round-86 `branch-bindings.json` pairs every exact round-82 descriptive branch
with its round-83 identifier. The checker verifies the frozen round-82 digest,
all 15 mappings, capture enums/annotations and both results enum locations
before accepting captures/results; renamed or dropped branches fail. Required
coverage comes from that frozen mapping, not from the current schema. Preserve multiple
attempts per tuple; a later pass never deletes a failure. The checker cannot
verify external files, true observation, full queue coverage, accepted scope or
independence. The independent reviewer must verify these against frozen inputs,
raw sources, timeline columns/phase rows, clocks, calibrations and quiescence.
A missing branch blocks whole-case pass unless product authority accepts its
explicit scope disposition; acceptance of a limitation is never an execution pass.

## Retention policy: pf83-synthetic-capture-v1

Decide before any real guest/functional capture. Base64 is encoding, not
redaction. For those captures this rule applies equally to decoded text, base64,
raw logs, screenshots, error reports, check detail copies, hashes and committed
artifacts. Local build/test receipts are separate internal gate metadata.

- **Verbatim permitted:** synthetic fixture command/key/PTY and observer data,
  package hashes and admitted isolated guest identity fields (OS, architecture,
  synthetic account/UID, guest VM UUID), after verification that the source was
  restricted to synthetic state. The committed round-82 mock bytes are synthetic
  controls, not real guest observations. Preserve mismatches and timeouts from
  this allowed data, including exact bytes when invalid UTF-8 matters.
- **Redact before publication:** incidental non-secret operator/account/path/
  host identifiers in transport diagnostics, using stable per-attempt tokens.
  Decode first; apply the same transformation to every duplicate and regenerate
  base64 from sanitized bytes. Record source artifact, field/byte span, category,
  replacement token and reason in `redactions.jsonl`. Do not retain the original
  value or its hash there. Hash/seal the published bytes; references target those
  bytes. Raw-to-published provenance records the transformation, not false claims
  of exact raw retention. If redaction removes identity/order/approval evidence
  needed for a verdict, block that claim; never substitute expected values.
- **Never capture:** real credentials/tokens/private keys, native credential
  store contents or prompts, live profile files, environment dumps, real user
  workspace/conversation data or protected payloads. Exclude these at source with
  the admitted OS/tool/process/network boundary; a later regex is insufficient.
  Any unexpected protected bytes in memory stop capture and successor dispatch;
  do not print, serialize, hash, base64, commit or forward them. Emit only a
  content-free incident/category and mark the attempt contaminated. Preserve
  already-safe checkpoints, not the protected content. Do not read credentials
  to classify them. Unexpected unclassified diagnostics are withheld in full
  from publication, with a content-free gap record, until safely classifiable.

The round-82 preflight serializes unrestricted stdout/stderr; it is NOT an
approved real-capture publisher under this rule. No guest-contact capture may
use it until the admitted owner supplies and verifies source isolation and a
publication filter implementing this policy. This revision states the rule and
blocks unsafe real capture; it does not implement or authorize a new transport.
Existing synthetic-only tests may still exercise exact-byte retention.
