# P0 security parallel handoff packet — round two

Status: allocated and ready. This packet follows PF-27-S03 integration at
`5521b681fff0ecb50b17c10bc1dd1356cbecc1b6` and replaces the initial three-lane
packet for new dispatches. Historical packet coordinates remain evidence only.

The three worktrees and branches already exist on CorbanuDrive. Each receiving
agent must set only its assigned sprint from `ready` to `in_progress` before its
first code change, remain within the literal front-matter scope, and update its
own sprint ledger as evidence changes. No additional owner decision, G0
bootstrap, branch creation, or Jim Ricketts action is required to begin these
three allocated sprints.

## Why these three tracks

The post-PF-27 graph has five dependency-complete drafts: PF-13-S06,
PF-19-S02, PF-20-S02, PF-21-S02, and PF-35-S01. The optimal disjoint frontier is:

| Track | Handoff | Ready sprint | Immediate unlock |
| --- | --- | --- | --- |
| Revocation/fence | [revocation-fence.md](revocation-fence.md) | PF-19-S02 | One half of PF-41-S03 |
| Authoritative state | [authoritative-state.md](authoritative-state.md) | PF-20-S02 | Other half of PF-41-S03; state input to PF-22-S02 |
| Compatibility/drift | [compatibility-drift.md](compatibility-drift.md) | PF-21-S02 | Independent compatibility input to PF-22-S02 |

PF-13-S06 waits because it overlaps security-policy/Core paths in the first two
tracks. PF-35-S01 waits despite a completed dependency because corpus licenses,
blind-evaluator custody, weakest-supported CPU, and artifact ownership are not
yet fixed. The prior browser/retrieval lane also waits: its first runtime sprint
requires PF-27-S02, then PF-33-S01/S02. Assigning idle thematic lanes would add
coordination without moving the critical path.

## Coordinates

| Track | Branch | Worktree | Build/cache root |
| --- | --- | --- | --- |
| Revocation/fence | `feat/p0-security-revocation-fence` | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-revocation-fence` | `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-revocation-fence/` |
| Authoritative state | `feat/p0-security-authoritative-state` | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-authoritative-state` | `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-authoritative-state/` |
| Compatibility/drift | `feat/p0-security-compatibility-drift` | `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift` | `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/` |

All use immutable allocation base
`5521b681fff0ecb50b17c10bc1dd1356cbecc1b6`. Before builds, put `CARGO_HOME`,
`CARGO_TARGET_DIR`, `TMPDIR`, `UV_CACHE_DIR`, `PIP_CACHE_DIR`, and any tool/model
caches beneath the listed CorbanuDrive root. Credentials remain only in the
gitignored `AgentCredentials.md` chain and must not enter evidence or review
transcripts.

## Integration order and allowance

The Codex ingress/classifier integration lane remains the plan's integration
owner. It merges PF-19-S02, then PF-20-S02, then PF-21-S02, auditing each diff
against its literal scope and rerunning combined-tree tests before archive.
Shared Cargo/Bazel/lock/schema, plan, current index, MkDocs, and archive edits
are integration-owner-only.

No sprint midpoint estimates exist, so the calculated provisional allowance is
35 of every 100 capacity units: 65 delivery and 35 integration/rebase/review
remediation/rerun/evidence. Reviewer availability is free; fixing findings is
charged. Once estimates exist, replace this with the active plan's per-sprint
formula and reforecast at every convergence gate.

## Next rounds

```text
PF-19-S02 + PF-20-S02 -> PF-41-S03
PF-19-S02 + PF-20-S02 + PF-21-S02 + PF-41-S03 -> PF-22-S02
PF-41-S03 + completed PF-27 foundations -> PF-27-S04 -> PF-27-S02
PF-27-S02 + PF-33-S03 -> PF-33-S01 -> PF-33-S02
PF-27-S02 + PF-33-S02 + PF-31-S04 -> PF-31-S01
PF-31-S01 + PF-30-S01 -> PF-31-S02
PF-31-S02 + PF-30-S01 + PF-34-S04 -> PF-34-S01
```

After round two, PF-41-S03 can run alongside PF-13-S06 and, only after its
external decisions are recorded, PF-35-S01. PF-22-S02 is a single convergence
sprint after PF-41-S03 and all three round-two inputs archive. Recompute the
frontier after each handback; do not reserve later sprint files early.

## Common review and handback

All independent evaluations use a real TMUX session running Corbanu Terminal
with Claude Opus 5.0 and Max effort, read-only. Record exact model/effort,
base/candidate commits, transcript hash, findings and dispositions. Fix verified
in-scope findings, rerun deterministic tests, and repeat until clean. TUI work is
not present in this round.

Each lane hands back its candidate commit, scope audit, changed paths, contract
identity, exact commands/counts, limitations, and immutable clean-review
evidence. It must not merge itself, touch another lane, change shared manifests,
or archive its sprint.

