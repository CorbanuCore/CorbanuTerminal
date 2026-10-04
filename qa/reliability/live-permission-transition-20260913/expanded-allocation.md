# PF-83 expanded confirmation allocation

## Authority

Travis answered **"yes, scope is expanded"** to app-server confirmation while
retaining next-turn effectiveness. This resolves the previous TUI-only gate;
do not request the same scope permission again. Classified **product initiative**
because request completion/compatibility is expanded. Active parent plan is
`docs/plans/active/p0-security-levels.md`; new feature PF-83 is separate from
PF22 and PF27. Sprint `PF-83-S01` owns one end-to-end confirmation unit.
Product citation: **Permission selection confirmation — TO BUILD**, "A submitted
selection is not a confirmed change." Prior broad quiescence proposal is rejected.

## Reservation handoff

Security owner task `01a04a32-b01b-7ad2-91b1-0f7ff2b11456` explicitly confirmed
returning PF27-S04's paused executable slot, keeping its original owner,
worktree, branch, base, Done/Remaining and all evidence. PF27 is draft/non-executable,
not cancelled/completed or resumed. Retained qualified source:
`19f6624938c53e0c9709e1b25ada4a44fd13267e`, Rust
`09f72396304d80a82cf466e220378f763ea8313b`; final owner proof
`899041c914f82f31a3097a34801209e5f20874f4` received `855ab3382`.
Review history and private A-D failures/interventions remain untouched. Owner
reported no active workers and no literal collision with app-server/TUI paths.
Accounting retains only state golden tests/QA; PF80 retains initiative_control
scripts/bootstrap docs/QA. Their recorded scopes are disjoint from PF83.

## Fresh Fable decision and validation

Cycle `/private/tmp/perm-manager3.8SShht/f-gjd205mp/receipt.json` completed at
`2026-09-14T05:23:29.192428Z`, model `claude-fable-5-1-plan`, provider
`claude-plan`, effort `high`. Correlated session/turn/final and clean normal
owned shutdown recorded, no force. Parent accepted PF83 allocation and one fresh
Astra High implementation worker. The requested reservation ACK already exists;
do not repeat coordination. Fable's generic "separate reviewer" wording does
not combine execution and evidence-review roles: those remain distinct agents.
Its specific proposed implementation is advisory, not proof of race safety.

## Execution

Manager path refinement after worker ACK: verified native AppEvent lives at
`codex-rs/tui/src/app_event.rs`, not the erroneous `app/app_event.rs`.
Corrected literal scope and added planned sibling
`codex-rs/tui/src/app/permission_confirmation_tests.rs` for the already
allocated asynchronous confirmation module. No product or authorization scope
expansion; TUI stop/approval controls must remain responsive while confirmation
is pending. These paths remain disjoint from the other two active lanes.

Manager fixture refinement: the first focused TUI compile identified the existing
`codex-rs/tui/src/app/test_support.rs` App initializer. Parent inspected it and
added this literal path solely for `pending_permission_confirmation: None`.
This is a one-field test-fixture repair, not additional runtime scope. Preserve
the first failed attempt in `tui-attempt-1.log` and rerun after formatting.

Dispatched worker: Peirce, Astra High, `01a09e63-91ce-71d1-9ee9-0d661f5de6c2`.
Both governance checkers passed before dispatch: active3/3, current116/archive126;
initial PF27 note length109 then101 exceeded100 and was compacted preserving
all original text/evidence. No runtime test result is inferred from validation.

- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913`.
- Branch: `fix/live-permission-transition-20260913`.
- Base: `005cc644f59b1e762e5497b329e106c67925d4ed`.
- Upstream ancestor: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, canonical API/ancestry verified in manager-checkpoint.
- Parent owns planning/registration coordination; worker owns only sprint literal source/test scope and repair QA.
- Existing target: `.codex-work/live-permission-transition-target` in this worktree, pinned Rust1.95/locked/offline; do not use active-user targets.
- Initial target500 non-test/800 hand-authored source+test changed lines; revised by manager below. Generated fixtures separately accounted. Stage at real overage with parent.

Prefer a narrow opt-in confirmation request/explicit applied result so old
clients retain acceptance semantics and new clients cannot mistake an old
server's empty success for application. Minimal v2 schema/doc/test changes are
authorized. Do not assume a response snapshot obtained later is the original
applied snapshot. Bind outcome to submitted request and listener generation;
register before submit, handle unchanged success, failure and terminal cleanup.
Uncertain transport effects are not proof of rollback; no blind retry. Keep
Core authorization and current-turn/MCP behavior unchanged. UI must distinguish
session next-turn selection from running command authority, not blanket-claim
every permission consumer defers.

## Test and handoff gates

### Manager size refinement during implementation

Worker reported about540 non-test/~720 hand-authored source+test changed lines,
with formatting/docs/test completion remaining, and requested650/950 caps.
Parent inspected actual tracked diff/numstat, untracked leaf modules and source
paths: changes remain the one allocated server-confirmation/TUI-consumption unit.
Approved **<=650 non-test / <=950 total hand-authored source+test changed lines**
for this increment, with generated schemas separately mechanical. This is an
explicit scoped exception to the initial800-line guidance, not a new behavior,
file scope, reviewer waiver or permission to add unrelated refactoring. The
extra allowance covers async completion/lifetime cleanup, compatibility, all
picker paths and real concurrency/no-op/failure tests. Prefer compaction of
repeated plumbing without obscuring ownership. Functional and independent
review gates unchanged; do not replace behavioral evidence with string tests.

Subsequent regression migration exception: parent inspected existing picker
tests at `codex-rs/tui/src/chatwidget/tests/permissions.rs` (including disabled
choices, both directions, same-state choice, reviewer selection and Full Access
confirmation). They assert the old optimistic history/mutation protocol.
Approved this literal test path and the three existing permission-selection
history snapshots named in sprint scope, with total hand-authored source/test
allowance **<=1250**, retaining **<=650 non-test**. The extra300 lines are for
preserving and migrating existing behavioral regressions, not new runtime scope.
Preserve each case's intent; assert typed request payloads and no optimistic
mutation/history. New App-level tests must cover confirmed outcome/history.
Do not erase Full Access consent, disabled-choice or reviewer coverage. Snapshot
migration/deletion requires an explicit replacement mapping; Windows evidence
must not be presented as executed without a Windows run. Initial RPC timeouts
remain failed attempts pending serial retry, not assumed infrastructure passes.

Frozen original F01-F11 hash remains `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.
Attach explicit user-approved next-turn interpretation without rewriting cases.
Public RPC success/no-op/error/concurrency/disconnect/shutdown and old/new
client/server matrix, TUI both directions and continuation/pending approvals,
unchanged relevant Core/MCP regressions, formatting then final affected tests.
Fresh Fable code review, enforced code-blind exact-package real-key execution,
separate independent evidence check and both disposable live-repository workflows
remain mandatory. No code or functional acceptance claimed at allocation.

Canonical integration is untouched and Fable-owned; no worker commit/push/merge,
install, release, live app/profile/shortcut change or product-lane resumption.
The repair coordinator will arrange receiving/installation handoff separately.
