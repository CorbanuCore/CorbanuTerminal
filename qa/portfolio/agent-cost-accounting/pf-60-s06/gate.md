# PF-60-S06 gate — side-conversation and temporary-fork attribution (#400, #401)

Candidate: PR #793, code head `604fe9c89d` (the later commits on the branch are docs and evidence only). Plan
activation: PR #790. Gate style as in PF-60-S04/S05: focused tests, clippy on both platforms, one independent review,
a real-key tmux check and an SOP video. Code-blind execution is PF-60-S10's job, under the enforced isolation gate.

## What changed

- **#400:** a guardian review inside `/side`, and a sub-agent spawned from `/side`, are now recorded once under the
  saved conversation, as `side:review:` and `side:agent:`. A parent that isn't saved is followed while it is live.
- **#401:** `side:` is used only for TUI `/side`, which the TUI marks with thread source `side`. Every other temporary
  fork gets `fork:`.
- **Other sub-agents and `/review` tasks** with no thread of their own record as `agent:` and `code-review:`.
- **`exec --ephemeral`** and work done for it stay excluded.
- The full list of session kinds is in the [coverage audit](coverage-audit.md).

## Evidence

| Check | Result |
| --- | --- |
| Fail-before | The 3 new integration tests fail on the old `accounting_owner` (`fork:` was `side:`; `side:review:` and `side:agent:` rows were missing) and pass after: [fail-before.txt](fail-before.txt). The `/side` test from PF-60-S04 passes before and after. |
| AC5 regression guard | `accounting_sub_agent_of_an_ephemeral_run_stays_uncollected`. It passes before and after, because such a child was already excluded. The one-warning-per-session behaviour is pinned by the existing `an_uncollected_session_is_named_once`, which passes. |
| Focused `just test -p codex-core -E 'test(accounting) \| test(guardian) \| test(side) \| test(spawn)'` | 523/523 passed with `developer-accounting`; 518/518 without. macOS, head `604fe9c89d`. |
| `just test -p codex-tui -E 'test(side)'` | 94/94 passed, including the fork thread-source assertion |
| Repeat runs of `accounting_side_forks` | 3 out of 3 green (8 tests each) |
| Clippy `-D warnings --tests` | macOS: `codex-core` with and without `developer-accounting`, plus `codex-protocol` and `codex-tui`: clean. Linux (glitch): the same three runs are clean on `1bb8e8d834`; the re-run on the final head is in the PR comments. |
| Independent review (Opus 5.5 High, read-only) | `VERDICT: APPROVE` on `1bb8e8d834` (3 minor findings and nits, all fixed or explained), then `VERDICT: APPROVE` on the re-review of `604fe9c89d` (nits only) |
| tmux, real keys, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1` | Main turn on GLM 5.3 Flash, then a `/side` request that needs escalation. The auto-reviewer approved it, then `/cost`. Ledger: all 5 attempts are under the main thread. The labels are `""`, `side:` ×3 and `side:review:` (GLM 5.2) ([ledger-turns.txt](tmux-20261014/ledger-turns.txt), [technical-page.txt](tmux-20261014/technical-page.txt)). The Z.AI key was found in 0 files of the run directory. |
| Video | [pf60s06-side-guardian-review-cost](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s06-pf60s06-side-guardian-review-cost-604fe9c89da3-2026-10-14.mp4) (45 s, GLM 5.3 Flash) |

GLM 5.2 appears only as the auto-reviewer. That is the product's own review model for Z.AI
(`approval_review_preferred_model`), so this sprint measures it; it cost about $0.007 per review.

## Not shown live, and why

- **A sub-agent from `/side` (`side:agent:`).** `/side` tells the model that sub-agents are off-limits
  (`tui/src/app/side.rs`). Asked to spawn one, GLM 5.3 Flash answered: "I can’t spawn or wait for agents in this side
  conversation." The path is covered by `accounting_sub_agent_spawned_from_side_is_recorded_under_the_conversation`,
  which uses a mocked model that does spawn one.
- **`fork:`.** The TUI makes no temporary fork other than `/side`. Other clients' ephemeral `thread/fork` requests
  are covered by `accounting_other_temporary_forks_are_labelled_fork`.

## Side effect

`/side` now sends the thread source `side` instead of `user`. It goes in the existing turn-metadata header sent to the
provider and in the analytics events. Nothing branches on it.
