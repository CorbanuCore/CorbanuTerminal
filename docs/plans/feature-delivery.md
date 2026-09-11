# Feature-delivery contract

Main integration status: this contract is imported from the reviewed operations
source. The referenced `scripts/initiative_control/` tooling is not yet ported
to remote main; it is integrated and tested on the local receiving branch at
`0415a00dc3d3ee55a96662920c3eedbc0f0d4838`. PF-80-S01 owns the remaining
native and human/live qualification. Until a separately authorized main landing,
then, references below are a contract, not runnable-main evidence or approval.

Travis Good's 2026-09-10 delivery decision supplements, but never replaces,
root release gates. A merge, QA enablement and user release are different events.

## Default-OFF incremental delivery

Reuse `codex-rs/features/src/lib.rs`: a new incomplete user feature normally
has `Stage::UnderDevelopment` and `default_enabled: false`. Do not invent a
second flag store. Use the native `[features]` configuration only in disposable
QA homes until qualification; an under-development menu entry is hidden, but
that alone does not gate commands, tools or persisted-session execution.

Before an implementation sprint becomes ready, its manager records a delivery
contract using `scripts/initiative_control/delivery.example.json`. Register the
real native flag only when implementing its feature; do not add pretend flags
for every draft idea. An internal-only tool may justify a non-runtime boundary.
Existing security and large-image qualification records are historical, not
retroactive proof that their features have an adequate flag boundary.

## Required proof

| Gate | Required evidence |
| --- | --- |
| Merge hidden increments | OFF discovery and execution tests; regression compatibility; exact candidate tree hash; default false and under-development registry entry; reviewer-approved boundary contract |
| QA enablement | Success, cancel, failure and recovery tests with ON; separate disposable QA profile; no implicit authorization or credentials |
| Disable after use | Work/data preserved or explicitly paused; jobs stop accepting new work; restart/resume stays safely OFF; user-visible re-enable/recovery route |
| User enablement / release | All applicable automated, actual-key TUI, human, live-repository and release gates; explicit product enablement decision; planned flag removal |

Enumerate every entry: menus, slash commands, tool registration/handlers, APIs,
background jobs, restored sessions and migrations. State explicitly which are
not applicable. A feature flag is neither an authorization check nor a rollback
of an irreversible migration, a credential change or a financial action.

## Evidence checker

Run `python3 scripts/initiative_control/delivery.py --repo . --contract PATH
--phase merge` (or `enable`). The JSON contract names the feature, native flag,
boundaries, candidate tree digest, and repository-relative evidence artifacts
with SHA-256. Test cases must have passing results tied to the same candidate.
The checker catches missing proof, unsafe flag defaults and stale artifacts;
it cannot establish that an agent's claim faithfully describes its tests.
An independent tester must run the documented human-style plan, and a named
human must accept the final affected experience. No automated check is bulletproof.

## Sequential ownership

Each of up to three active initiatives has one reserved sprint and one named
integration owner. Builders may not self-approve their own changes. Independent
review/testing supports the same sprint; shared-file writes are serialized.
Blocked work retains a slot until the manager records a safe handoff.
