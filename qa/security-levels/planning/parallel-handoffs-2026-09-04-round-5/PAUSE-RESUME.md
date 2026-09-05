# Security pipeline — account-switch checkpoint and resume

## Resumed by user

The user explicitly requested resumption after the account switch. The five-minute
heartbeat is ACTIVE again. The human-memory fixture agent resumed its preserved
worktree; the reviewed PF20 dependency was merged without conflicts into the
integration branch at `b12e32db398c83854271e2e70f29e5290278af8b` for combined RTX
qualification. This is not a main merge or a qualification claim. RTX SSH works;
approximately 1.5 TiB disk and 82 GiB memory were available at the resume check.
Existing review budgets, allocations and pending authority decisions are unchanged.

The historical pause instructions below are retained as a recovery checkpoint;
the explicit resume request satisfies their stop condition.

## Historical pause record

User requested a full pause on 2026-09-04 (Phoenix; 2026-09-05 UTC) to switch
accounts. **Do not resume implementation, tests, reviews or the heartbeat until
the user explicitly asks to resume.** This is a pause, not completion/cancellation.

## Monitor and agent identities

- Heartbeat automation: monitor-three-security-lanes, PAUSED; previous cadence
  five minutes. Preserve its prompt and pending decisions on explicit resume.
- Current task: 01a04a32-b01b-7ad2-91b1-0f7ff2b11456.
- /root/broker: completed/stopped; /root/provenance: completed/stopped.
- /root/security_ui: stopped with clean pushed WIP2b6e9a082; last remote test
  exited100. Agent verified no owned tests/reviews/fixtures or build.lock remain.
  Preexisting unrelated TMUX was left untouched. See its detailed pause record
  at /Volumes/CorbanuDrive/Corbanu/.codex-work/security-human-memory-fixture/PAUSE-HANDOFF.md
  for final process/lock and artifact status. Do not assume an old PID is live.
- If agent identities do not survive the account switch, recreate only the
  required bounded agents from these existing worktrees after user resume.

## Recoverable branches and worktrees

All worktrees are under /Volumes/CorbanuDrive/Corbanu/worktrees.
These GitHub branch heads were verified during pause preparation:

| Worktree | Branch | Checkpoint | State |
| --- | --- | --- | --- |
| security-round5-20260904 | integration/security-round5-20260904 | be9bdaacba08704d7060b98b0d0beac9bb26355e before this pause document | Clean staged integration, not main |
| security-local-anchor | feat/security-local-anchor | ee07e07009a312e386b960fbf909bb57270c2c2a | Clean, pushed, reviewed; awaits combined integration |
| security-round5-broker | feat/security-round5-broker | cd7457da743660fe36213816fcdb7bebd91ba1ce | Pushed service stage; external review budget pending |
| security-human-memory-fixture | test/security-human-memory-fixture | 2b6e9a082ca64933d9719ac7d5c24445aa383499 | Pushed WIP, not qualified; consult agent pause record |

Origin: https://github.com/CorbanuCore/CorbanuTerminal.git.
Some local upstream tracking refs are absent; use git ls-remote for exact branch
heads. Preserve dirty files if any appear; never reset/clean a worktree to resume.
Earlier provenance is frozen incomplete on feat/security-round5-provenance;
its source2a4fb5857/evidencee890ae4a9 is already integrated.

## Last fully qualified human candidate

- Runtime6a6bb029d8f3e0c16653ce335d252f45b4d7326f, Corbanu0.1.38, RTX Linux.
- Binary /home/travis/security-round5/evidence/integration/6a6bb029d/candidate/codex
- SHA25690d6a1f7f72c5397ff858583c038b2615c8fb034f57a890d6595d6b98afccd4f
- Core110/110, memory-write44/44, read3/3, UI235/235, actual-key TMUX4/4.
- PF30S04 archived;27/78 security sprints archived. humanTest.html has23 checks;
  checks21–23 explicitly QUEUED until operator fixture is qualified.
- Mac Apps shortcut unchanged. No new human acceptance, main merge or release.

## Resume order

1. Read root AGENTS.md, corbanu-terminal-development skill, active P0 plan,
   current sprint records and this packet. Inspect actual branch/process state.
2. Integrate PF20 final ee07e0700 only after scoped diff/lock audit. Its
   qa/security-levels/sprints/PF-20-S03/integration-handoff.md contains exact
   combined RTX commands. Runtime8d6967179 has leaf18/Core17/audit46/config229,
   Bazel parity and actual-key profiles/restart proof. Astra P2 fixed; Fable no
   code blockers with two documented P3 notes. Root ledger/dashboard corrected.
3. Finish human-memory fixture test-only WIP in its own worktree. Product code
   must remain unchanged. Consult its pause record and versioned fixture recipe.
   Actual provider-switch rehearsal still failing/not qualified. Adjacent
   product findings: duplicate custom-model labels/current markers and the
   single-effort picker path losing explicit provider identity. Do not silently
   implement those product changes under the routine fixture allocation.
4. After combined qualification, update humanTest/progress/evidence and archive
   only genuinely complete bounded work. PF27 launcher/bootstrap follows its
   completed dependency and explicit scope; do not waive missing isolation.

## Reviews and authority limits

- Broker5/5 used. Sixth Fable service review awaits already-asked user approval.
- Frozen provenance5/5; completed security UI3/5.
- Memory/runtime plus manual fixture2/5. Fixture#3 Astra and#4 Fable authorized
  only after final fixture proof. No reviewer invoked for fixture yet.
- PF20 controller2/5; no further review needed for documentation-only closure.
- Astra High autoreview; Fable5.1 High through Corbanu/private TMUX external review.
- Main merge choice remains unanswered because integration also contains
  previously unmerged provider reconciliation. Do not infer permission.
- User approved data rollback protection using a protected local checkpoint;
  whole-machine rollback/TPM/off-host witness is out of scope.
- Accepted staged layout: minimal root anchor/launcher, separate journal/policy
  roots and distinct trusted child roles/PIDs. Not installed or privileged-qualified.
- No elevated principal/ACL/service setup, TPM changes, real Vault migration or
  automatic lost-state reset without explicit authority. PF35 remains external.

## RTX and storage rules

Build only on100.99.88.49 as travis. All local artifacts stay on CorbanuDrive.
Do not write passwords/tokens into this packet. Credentials remain in existing
local authorized files; inspect only needed paths and never echo secret values.
SSH master path:
/Volumes/CorbanuDrive/Corbanu/.codex-work/ssh/rtx-security-control

Remote shared target:
/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
Shared flock: /home/travis/security-round5/locks/build.lock
Use fresh per-run TMPDIR on a supported ext/XFS filesystem for PF20, jobs8 and
scoped just fix/full just fmt before final just test/TMUX. No local builds.
Bazel must shut down before lock release. Inspect only owned process identities;
do not kill unrelated services or use a stale PID after resume.

Other exact evidence and review wrappers are recorded in this packet's README
and the per-lane QA. Private TMUX review credentials are loaded by the existing
wrapper, not copied into prompts. Account changes may require reauthentication;
ask rather than assuming credentials or review sessions carried over.
