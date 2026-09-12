# PF-30-S04 independent review scope

Review the complete branch diff from allocation `db141e9cb`, not only the last
checkpoint. Parent authorized Astra High followed by Fable5.1High after final
qualification; maximum five invocations, never nested reviewers.

Original task: replace the detached stage-one memory ModelClient constructor
with one opaque CodexThread-owned facade. Deny all protected raw-rollout input,
including inherited/live policy strengthening and per-attempt HTTP/post-connect
WebSocket races. Preserve successful Permissive payload, metadata, auth, proxy
and cache shaping. Prevent denied, incomplete or owner-cancelled output from
being marked successful; retain finite existing retry/backoff.

Ownership boundaries: Core memory facade and private session snapshot; minimal
ModelClient transport hooks; memories-write runtime/phase1/start; dedicated
Core and true-TUI test files. Root authorized lib/factory/suite declarations.
No new dependency, migration, setting, activation or privileged setup. No
positive admission, persisted source lineage, phase-two redesign or claim of
complete protected memory. Historical source IDs are data, not runtime owners.

Inspect the real path from app-server turn_processor into memories startup,
fresh per-job host routing, facade construction, lower HTTP attempts, WebSocket
connection/frame boundary, response completion and DB persistence. Distinguish
new scoped bugs from explicitly deferred historical-policy/phase-two concerns.
Cancellation here is dropping extraction or terminating its owning thread;
ordinary foreground-turn interrupt semantics are not redesigned.

Read `qualification.md`, the adopted design and sprint for evidence/limitations.
Do not infer passed human or cross-platform acceptance. Verify useful findings
against actual reachable behavior and report exact small ownership-bound fixes.
Read-only inspection only: do not modify files, run builds/tests locally, access
credentials, launch nested reviewers or perform privileged/system operations.
Builds and tests for this work run only on the RTX machine.

The implementation is intentionally staged in recovery commits: host facade,
worker routing, and qualification. Review the complete boundary. Production and
tests are separated; formatting expands the combined diff beyond ordinary
800-line guidance, with the coherent stages recorded in qualification.md.
