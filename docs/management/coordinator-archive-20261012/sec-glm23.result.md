## PF-23-S03 qualification — final report

### What shipped

| PR | Title | Status | Merge commit |
|---|---|---|---|
| #256 | PF-23-S03: GLM 5.2 demo videos and gate completion | MERGED | `213698e607` |
| #257 | PF-23-S03: archive | MERGED | `2caeb12f0f` |

PF-23-S03 is now **archived** (`status: completed`) in `docs/sprints/archive/p0-security-levels/`. The code was already merged in #246–#249; the sprint stayed `in_progress` only because Z.AI had insufficient balance for the GLM 5.2 tmux run. The balance cleared on 2026-10-07.

### Gate evidence

- **Focused tests** (candidate `a230f2082141`, origin/main after the merge): `security_transition` + `security_recovery` **26/26**; `revocation` **11/11**.
- **Review (Opus 5.5 High):** three rounds, APPROVE (already recorded in gate.md).
- **Linux clippy:** clean at the final tree (already recorded).
- **Videos:** four GLM 5.2 (`zai`) runs, each under 90s with clean leak scans (0 published credential values, 0 key-shaped strings, 0 redacted private files):

| Demo | Length | Final frame | MP4 |
|---|---|---|---|
| stored-level-next-session | 46s | `Operation not permitted` | [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-stored-level-next-session-a230f2082141-2026-10-07.mp4) |
| unreadable-state | 44s | `Operation not permitted` + unreadable warning | [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-unreadable-state-a230f2082141-2026-10-07.mp4) |
| project-cannot-lower | 31s | `Operation not permitted` | [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-project-cannot-lower-a230f2082141-2026-10-07.mp4) |
| unreadable-state-warns (model-free, earlier) | 26s | kill switch enforced warning | [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-unreadable-state-warns-c60857fc5d47-2026-10-07.mp4) |

Index: `qa/demos/index/PF-23-S03.md`. Gate record: `qa/security-levels/sprints/PF-23-S03/gate.md`.

### PR #253 (PF-24-S02) — not touched

Neither PR touches any file in PR #253's scope. The only shared file is `docs/sprints/current/p0-security-levels/index.md`; my edits are on the PF-23-S03 row (line 42), PR #253's are on the PF-24-S02 row (line 43) — no conflict.

### Cleanup

- Worktree `worktrees/sec-pf23s03-qual` removed via `git worktree remove`.
- CARGO_TARGET_DIR `.codex-work/targets/sec-pf23s03-qual` removed.
- No leftover tmux sockets or processes.

### Open

- PF-24-S02 (PR #253, still open) is the first production caller of `commit_transition`; the handoff items are recorded in PF-23-S03's archived Remaining section.
- One nextest-reported LEAK on `security_transition_unsaved_kill_switch_survives_the_next_commit` (the test passed; the leak is a nextest resource-tracking note, not a failure — the gate already records 26/26).