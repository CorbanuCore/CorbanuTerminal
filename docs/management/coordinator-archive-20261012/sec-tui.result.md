A, C and D are fixed and merged to main. B was already fixed on main, and E behaves per the PF-83 contract. The fix for A changes how approval prompts take input, which needs your sign-off (decision 1 below).

## Reproduction on main `cb78550a31` (tmux, GLM 5.2)
| | Result | Issue / PR |
|---|---|---|
| **A** typed text approves (security) | Still present: typing `/permissions` approved with "don't ask again" | #174 (labelled security) → PR #187, merged `d2ac064b02` |
| **B** declined request shows "Ran … (no output)" | Already fixed on main by `f655e60d7d`; a decline now shows only "✗ Did not run" | No issue |
| **C** "Enable full access?" Cancel is silent | Still present | #175 → PR #185, merged |
| **D** `zai-anthropic` route fails every turn | Still present: `glm-5.2` had no output limit in the model catalogue | #176 → PR #182, merged `697b833ae3` |
| **E** Full Access chosen while an approval is pending | Works per contract. Choosing it doesn't answer the request, which is asked again. Accept runs only that command, decline runs nothing, and later commands run under Full Access. | No defect, so no PR. Evidence is in #187. |

#182 and #185 were merged by someone else once their checks passed; I merged #187 myself (`--merge --match-head-commit`).

## What changed
- **A:** typed or pasted text can no longer answer an approval prompt. A letter or digit key only highlights its option; Enter confirms it. Anything else typed (`/`, a second character, a paste, Backspace, Ctrl shortcuts) makes Enter do nothing, and the prompt explains how to answer until the user presses an arrow key. Enter is also ignored until the user makes a choice on:
  - the next queued request;
  - a prompt that appeared while the user was typing;
  - the prompt right after the open-thread key.

  My first attempt used a 250 ms timing window; the reviewer rejected it because a slow typist could still approve, so the final version uses no timing at all.
- **C:** Cancel stays the default. Cancelling now adds "Full access was not enabled; permissions are unchanged. To enable it, choose Full Access, then 'Yes, continue anyway'."
- **D:** `glm-5.2` now has a 128,000-token output limit (the figure Z.AI publishes), with a regression test.

## Gate evidence
- **Tests:**
  - A: `just test -p codex-tui` ran 4,224 of 4,229 tests with 22 failures. They are the same environmental failures that also happen on main plus the C change: Unix-socket path too long, the command-popup snapshot, and onboarding tmux tests. The tmux server dropped before the last 5 tests; I re-ran that group and all 20 passed. The focused approval set passed 329/329.
  - C: 78/78.
  - D: models-manager 73/73.
  - `just fix` and `just fmt` were clean.
- **Review:** Opus 5.5 High, read-only.
  - A took four rounds: two asked for changes and the last two approved. I fixed all of the reviewer's should-fix items.
  - C: the only should-fix was a missing UI snapshot test; I added it, plus the nits. There was no re-review.
  - D was approved.
- **Videos** (all under 90 s, scanned for the Z.AI key): on the `demos` prerelease, linked from each PR. Assets are `bug-174-*`, `bug-175-*`, `bug-176-*` and `pf83-f05-*`. The `.cast` files are kept in `.codex-work/tui-lane-20261006/casts/`.
- **Picker sprint:** the plan PR #173 has landed the new picker sprint, **PF-24-S03 "Flagged /security picker"**. It heads the TUI lane, ahead of PF-24-S02, PF-25-S01 and PF-25-S02.

## Decisions for Travis (from A)
1. **Approving now takes two keys.** `y` then Enter, instead of `y` alone. `j`/`k` and Ctrl+P/N no longer move through the approval list; the arrow keys do. I recommend keeping this, since a single key can't be told apart from typing.
2. **A single key plus Enter can still grant "don't ask again" or session scope.** For example, a paste of `p` plus a newline on a terminal without bracketed paste. The reviewer recommends requiring the arrow keys, not a letter, to pick those persistent options. I agree.
3. **Enter on a first prompt that appears while you're idle still accepts "Yes, proceed".** This is unchanged from before. Options: leave it, or require a choice first on every prompt.

## Still open
- **#181:** the Baseten and OpenRouter Anthropic-compatible routes can hit the same missing-output-limit error. The D reviewer found this; I haven't reproduced it live.
- **A known limit:** on terminals that don't report key repeats, holding a custom key-combination approve shortcut can still answer queued requests one by one. Default keys are unaffected.
- **Shared build folder warning for other lanes:** sharing one `CARGO_TARGET_DIR` across worktrees produced a binary that silently contained another worktree's code. I caught it, rebuilt, and checked every demo binary by searching it for its fix's text.
- **Stray folder:** I wrote two files to `/Volumes/CorbanuDrive/Corbanu/codex-rs/` by mistake and moved them into the worktree. The now-empty folder is still there.