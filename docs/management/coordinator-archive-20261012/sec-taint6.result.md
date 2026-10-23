PF-23-S01 is finished and archived: PR #233 (slices 2 and 3) is merged to main as `64137b7189` with all checks green. I didn't start PF-23-S02. One issue should be fixed before `source_envelopes` is turned on (see "Still open").

Everything below is behind the default-off `source_envelopes` flag and only applies under Moderate or Aggressive. With the flag off, under Permissive, or before any untrusted content arrives, nothing changes.

## What shipped
- **Slice 2, memory summaries under Moderate:** the background memory job may now summarise a past session under Moderate, but only from text Core builds itself.
  - **How Core builds it:** it reads the claimed session's own rollout and refuses it unless the first record belongs to that session. It removes secrets, keeps text as trusted only where this home's signed records say so, and labels the rest as untrusted data. If the text is too long, it drops whole entries from the middle, so a label is never cut.
  - **What gets refused:** any request that isn't exactly that message is refused before it is sent, including when the level rises mid-job. Aggressive still refuses, and the follow-up merging step is skipped above Permissive.
- **Slice 3, the OS sandbox blocks reads after untrusted content:**
  - **What's covered:** agent commands, the in-process file tools (patch pre-check, edits, image view, extension tools) and Codex Apps uploads.
  - **What's denied:** the Corbanu home (everything except the folders commands need to run, skills and plugins), other Corbanu homes, and fixed credential paths in `$HOME` such as `~/.ssh` and `~/.config/solana`.
  - **What stays readable:** only the turn's own folder and workspace, never a folder the model picks for a command.
  - **Approvals:** under Moderate, approving the exact flagged shell command lets that one run read through. Under Aggressive nothing lifts the block.
  - **Readiness:** `qa/security-levels/sprints/PF-23-S01/activation-readiness.md` concludes Moderate isn't ready to offer yet.
- **PF-29 split:** recorded in the archived PF-23-S01 record and in the PF-29-S02 file. It isn't in PF-29-S01 because that file is at the 100-line limit. No file is written by both.

## Gate evidence
- **Tests:**
  - Focused core tests: 145 pass. `codex-memories-write`: 45 pass.
  - Full `just test -p codex-core` on the tree merged with main: 3,881 of 3,884. The 3 failures are the known baseline ones.
- **Lint:** `just fmt` and `just fix` are clean, and Linux clippy with warnings as errors is clean on the RTX box at the final commit. The RTX run caught three issues and CI caught two (one of which broke the Windows build); all are fixed.
- **Review (Opus 5.5 High):** three rounds; rounds 1 and 2 asked for changes, round 3 approved. Main fixes:
  - A command's own working folder could switch a denial off.
  - File tools and Apps uploads weren't protected.
  - Long sessions were always refused for summaries.
  - Secret removal could cut a label.
  - Outputs are in `.codex-work/workers-20261002/pf23s01-review{7,8,9}/`.
- **GLM 5.2 videos:**
  - [A hidden home-folder read is blocked by the sandbox](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-hidden-home-read-denied-a58a5e18e699-2026-10-06.mp4)
  - [Approving the command lets that read through under Moderate](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-approval-lifts-under-moderate-a58a5e18e699-2026-10-06.mp4)
  - [The memory job sends a labelled session to GLM under Moderate](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-moderate-memory-labelled-a58a5e18e699-2026-10-06.mp4)
- **Gate record:** `qa/security-levels/sprints/PF-23-S01/slices-2-3-gate.md`.

## Still open
- **Fix before turning the flag on:** a Codex Apps upload checks the file, then reads it a moment later outside the sandbox. A background command could swap the file in between. The read should go through the sandbox; this is moved to PF-23-S02.
- **Memory summaries don't actually complete with GLM 5.2:** GLM wraps its JSON answer in a code fence and the existing parser rejects it at every level, Permissive included. The memory video shows the request going out and then the parse error. This needs an owner.
- **Moved to PF-23-S02:** Aggressive grants, gaps in the command-text check for writes, and processes started before untrusted content (their sandbox is fixed when they start).
- **Moved to PF-23-S03:** the merging step and summaries under Aggressive, plus the level a past session ran under.
- **Known limits (listed in the gate record):**
  - External sandboxes and remote environments don't get the extra rules.
  - Tainted full-access commands fail closed where no sandbox can start.
  - Extension tools keep permissions granted earlier in the session.
  - Hard links that already existed aren't covered.
  - The shell snapshot stays readable.
- **Other findings:**
  - The Chat Completions and Anthropic connections still drop MCP tools.
  - The memory job only picks up sessions idle for at least 1 hour (default 6), so the memory demo backdates the session in a visible setup step.

**Cleanup:** my worktree, its 134 GB target folder and the RTX worktree, target and clippy script are removed. The report is saved as `.codex-work/workers-20261002/sec-taint6.result.md`.