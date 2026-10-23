# PF-80-S01 — Slack sender/reply recovery allocation

Product initiative within existing PF-80-S01 reservation. Product heading
**Internal delivery control — TO BUILD**: “Show blockers, rendered sprints,
human test plans, machines, run logs and freshness.” Travis requested contextual
Slack questions, logged replies and routing to the appropriate agent. Existing
approved target is AmbientCrypto's private The Corbanu Project; no new recipient.

Base `0368ed402a9dac4276b427313b9d7335670ee91c`; worker
`/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`, branch
`workstream/tasknode-pf80-s01-20260911`. Parent records clean actual launch after
allocation review/integration, checks disjoint scope and dispatches Astra High.
No implementation before that dispatch. Same sprint, not a fourth initiative.

## Output and literal five-file ownership

- `scripts/initiative_control/decision_alerts.py`
- `scripts/initiative_control/decision_replies.py`
- `scripts/initiative_control/test_decision_alerts.py`
- `scripts/initiative_control/test_decision_replies.py`
- `qa/initiative-control/pf-80-s01/decision-projection/slack-contract-receipt.md`

Implement one complete offline injected-transport sender/intake/resolution/handoff
workflow with real local durability and restart tests, not just a schema or a
mock-only interface proposal. No runtime registration, scheduler, real auth/token
reads, native source edits, export change, live network or shared state mutation.
All fixtures use disposable synthetic directories. Existing Task Node outbox,
PF76 history and native credential contracts remain untouched.

The earlier400/250 estimate was not credible for complete two-way recovery proof.
Manager explicitly allocates target1200total/650non-test, counting test-gated
implementation and receipt as non-test. This one coherent send/reply recovery unit
justifies above800; it is not inherited F01/C2 exception authority. Report measured
overage before expanding, preserve tests and avoid another prerequisite chain.
One new code review plus necessary correction is authorized, without resetting
earlier code/design/evidence usage. This allocation's own review is separate from
the future new-code pass. No worker model/review/child calls or commits/pushes.

## Existing contracts to reuse

Read `decisions.py` fully: validate/canonical/digest, append-only advance, original
raised_at and full question/context revision binding, private no-symlink storage,
locked CAS save_fixture, directory fsync and documented post-replace uncertainty.
Do not loosen/copy that validator or change its whole-feed rejection policy.
Read `tasknode.py` for stable IDs/locks, but do not reuse its batch flush/retry
for uncertain Slack sends. `control.atomic_json` alone is not directory-durable.
Native SessionScope/validity machinery is not a Slack client or credential source.

## Complete recovery contract

1. Bind immutable alert intent to feed/decision/question revision, exact presented
   context/payload digest, event kind, team/channel/app/bot identity and supplied
   credential generation. Validate identity and safe approved remote permalink;
   escape Slack markup/mentions and link every sprint to approved exact context.
   A summary plus threaded details/question is the Slack equivalent, not HTML
   accordion support. New/revised/resolved notifications remain distinguishable.
2. Separate transport states pending/sending/sent/failed/uncertain from decision
   status. Lock send ownership; durably consume a stable per-phase attempt before
   exchange. Parent message and threaded details are separate phases. Persist
   exact returned parent timestamp before sending details. Never fabricate receipt
   success; validate returned identity. Crash/timeout after intent is uncertain
   across restart, never a blind resend. Exact retained receipt evidence can
   reconcile only the same identity/payload. No exactly-once transport promise.
3. Intake accepts injected already-verified envelopes only for pinned team/channel,
   exact alert thread and immutable allowed human ID. Verification is a future
   transport precondition, not merely a boolean from untrusted JSON. Bound input
   bytes and validate types. Dedup event ID and message identity durably before
   acknowledging processing. Preserve edit/delete audit identity and timestamps;
   no silent replacement, invented sender or reply-text execution.
4. A received reply is data, not operational approval or an accepted interpretation.
   Manager-supplied answer/scope must bind the exact presented question/context
   revision and current owner/allocation. Old/conflicting/ambiguous/resolved
   answers remain inspectable and require clarification, not automatic dispatch.
   Acknowledgment is not resolution. Keep received/needs-clarification/recorded/
   queued/delivered/agent-acknowledged distinct, including stopped/stale owners.
5. Resolution and delivery ledger are different durable files, not one transaction.
   Persist exact resolution intent, apply existing feed CAS, then queue a stable
   handoff ID. On restart compare retained intent with actual feed/revision/digest
   to resume or report conflict. Handle save_fixture post-replace uncertainty by
   reload/exact comparison. Never strand a saved answer without its handoff or
   overwrite a newer question. Do not claim cross-file atomicity.
6. Handoff uses explicit current-owner/allocation identity and injected receiver;
   no Codex tool calls from this code. Retain stable IDs across restart. Uncertain
   dispatch requires reconciliation, not duplicate logical assignment. Only a
   matching actual agent acknowledgment marks acknowledged. Owner changes or
   stopped agents stay pending manager action, never silently routed elsewhere.
7. Keep only approved sanitized decision/answer context in private durable records;
   no token values/raw private logs in errors, payloads, receipt or repository.
   Owner-only roots/files, no symlink/hardlink surprises, bounded parsing and
   predictable sanitized failure errors are required. Missing/corrupt state is
   unknown/held, not an empty ledger that permits duplicate sends.

## Required synthetic tests and final evidence

Exercise actual on-disk restart, concurrent lock contenders and injected failures:
new/revised/resolved parent+thread payloads and safe links; duplicates; partial
thread send; timeout/crash after intent and after parent receipt; identity rotation,
cancellation and exact-evidence recovery; wrong actor/team/channel/thread;
duplicate/out-of-order/edit/delete replies; ambiguous and stale revisions;
CAS conflicts; resolution/handoff crash window; stopped/reallocated owners;
dispatch uncertainty and matching/mismatching agent acknowledgment. Include storage
permission/link/corruption/size failures and secret/mention/unsafe-URL negatives.
Expected states must be literal, not generated by the operation under test.

Run pinned dashboard venv Python `-m unittest discover -s scripts/initiative_control -p 'test_*.py'`,
standalone Facilities Node regression, both governance checkers and git diff check.
Report nonzero counts, actual commands/exits, candidate/diff hashes, original
failures and limitations in the allocated receipt; return uncommitted. Parent
reviews new code once, resolves real findings, integrates locally and tests the
combined tree. No TUI/browser/live proof is claimed for this backend-only slice.
Original DEC001..026/F01 evidence and pending DEC021/DEC025 dispositions remain;
they do not prohibit independent offline Slack implementation.

## First live gate and immediate successor

This code is not an operational Slack connection. Parent's immediate next seam
is real supported Slack transport/registration, not another generic contract.
Record actual callable interfaces and needed integration files in the receipt.
Freshly verify bot/app scopes, membership/identity, immutable Travis user ID,
supported credential storage/generation and authenticated remotely accessible
dashboard URL. Recorded proposed setup: teamT074X5KNENT, channelC0C0X2ELFKR,
appA0C1CC2P4SE; not a current consent/credential audit. No scraping session tokens
or accessing wallet seeds. Surface a precise missing credential/consent only
when established; do not reask the approved recipient or ordinary progress authority.
After qualified transport, one pinned harmless question and real reply must
demonstrate restart, dashboard resolution and actual agent acknowledgment before
ongoing delivery. No bulk flush, Task Node acceptance/rewards, public beta or
financial/signing side effects. PF79/PF81 remain dependent drafts.
