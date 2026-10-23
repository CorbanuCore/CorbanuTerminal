PF-30-S01 is merged to main (PR #178, merge commit `b96b23344b`), behind a new default-off feature flag `source_envelopes`. PF-30-S02 has not started.

**What existed before:** all the earlier PF-30 work was already on main, including the `feat/security-round5-provenance` branch and the frozen handoff commits (`2a4fb5857`, `e890ae4a9`, `0266c2db9`). It had the source envelopes, admission checks, chunked screening and the three provider request paths. I reused all of it. The gap was that Moderate and Aggressive failed closed on every request: no model call could go out at all.

**What shipped:**
- **Flag on, Moderate or Aggressive:** requests go through. Everything that didn't come from the user, from Corbanu itself, or from the model's own output stream reaches the model as labelled untrusted data, marked `authority=none`. That covers tool, MCP and hook output, shell output, agent messages, and anything injected or restored from a saved session.
- **Forgeries are defused:** fake wrapper closes, `<system>`-style tags, chat special tokens, look-alike characters and hidden Unicode in that data can't pose as a user approval.
- **Ordinary use still works:** normal multi-step tool use completes as before.
- **Flag off or Permissive:** behaviour is unchanged. Live voice and memory summarisation still fail closed.

**Gate results:**
- **Focused tests:** `just test -p codex-core pf_30_s01` passes 39/39 (12 new). The features, protocol and content-security suites pass 349/349.
- **Full core suite:** 3,740 pass and 2 fail. Both failures happen identically on unmodified main; they pick up skills installed on this machine.
- **CI:** all 32 checks passed.
- **Independent review (Opus 5.5 High via `corbanu exec`):** four rounds. Rounds 1–3 requested changes and I fixed all of them. The main ones:
  - Restored sessions were wrongly given trusted standing.
  - Trust was decided by the message's role label instead of where it was recorded.
  - Some look-alike characters got past the filter.
  - Converted tool calls kept provider IDs the API would reject.

  Round 4 approved commit `61b75ec43a`. Reviews are in `.codex-work/workers-20261002/pf30-review{1..4}/`.
- **GLM 5.2 tmux test and videos:** all three ran in the real TUI and pass; I checked frames from each. The `61b75ec43a` recordings are published on the `demos` prerelease and indexed in `qa/demos/index/PF-30-S01.md`:
  - [Labelled tool output](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s01-pf30-labelled-tool-output-61b75ec43ad5-2026-10-05.mp4): GLM sees the label, refuses a forged approval, and never creates the planted file.
  - [Normal tool use](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s01-pf30-normal-tool-use-61b75ec43ad5-2026-10-05.mp4): a multi-step edit-and-run task completes.
  - [Flag off](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s01-pf30-flag-off-fails-closed-61b75ec43ad5-2026-10-05.mp4): Moderate still fails closed.

**Docs:** the gate evidence is in `qa/security-levels/sprints/PF-30-S01-typed-source-envelope/labelled-ingress-gate.md`. On the sprint file and index:
- I left the status as `draft` because the sprint checker blocks the alternatives:
  - `ready` or `in_progress` needs the worktree registered in the plan's front matter, which the plan worker owns.
  - Those statuses also count toward the 3-sprint global limit, which other sprints already fill.
- Archiving the record as completed is the plan worker's step.
- In the merge with main I took the plan worker's rewritten index as-is, which has no status column.

**Open:**
- **PF-30-S02 not started.** Until it lands, resumed sessions treat everything restored as labelled data. The first slice I'd do is saving each message's origin in the session file so resumed history keeps correct standing. That touches about 15 production files plus the app-server schemas, so it's a full sprint under the gate.
- **Credential in debug logs:** the demo tool's scan found the GLM provider API key written into the TUI log and its log database when `RUST_LOG=trace` is set. The tool redacted it in place. It isn't caused by this change and needs an owner.
- **Misleading error with the flag off:** Moderate stops with "source admission registry is full or poisoned". It still fails closed safely, but the message is wrong. This predates the change.
- **Known gaps, recorded in the QA file:**
  - Images, audio and encrypted agent content pass through unwrapped.
  - MCP tool-search descriptions aren't labelled.
  - The PR merge commit isn't recorded in the QA file yet; that needs a small follow-up docs PR.