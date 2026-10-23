You are the TUI lane worker (round 5). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding, including the new disk hygiene rule), then sec-tui4.result.md in the same folder.

1. Issue #218: bring Claude panes (and other external CLI agents Corbanu launches in panes) under the PF-27-S02
   secretless-launch contract so they can run under protected levels instead of being refused: clean environment,
   every provider behind the bridge/broker (no raw key via `corbanu vault auth-helper`), a sandbox around the
   external CLI that still reaches the bridge (Unix-socket bridge on Linux), and a replacement for
   `--permission-mode bypassPermissions` that routes the CLI's tool approvals through Corbanu's approval flow. If the
   whole thing is too big for one PR, split it into bounded PRs that each keep the current refusal in place until the
   last one lands. Behind a default-off flag.
2. Smaller open items from sec-tui4 (each its own small PR): make the Aggressive-homes registry read-only inside the
   Aggressive sandbox; run the nested-launch check in the standalone `codex-exec`/`codex-tui` binaries too.
Do not change the Aggressive sandbox design itself (still with Travis). Gate per fix per sec-common.md. PR to main;
merge when green (stale CodeQL checks can be ignored). Report as in sec-common.md.
