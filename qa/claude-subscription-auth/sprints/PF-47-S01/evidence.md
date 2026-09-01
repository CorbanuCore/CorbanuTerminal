# PF-47-S01 first-run Anthropic-account evidence

## Recovered candidate boundary

- The original external-volume worktree remains preserved at commit
  `1055f423bee68d616a8271d87598ad982926b6bd`; no destructive reset or cleanup
  was performed after its shared Git object store and filesystem became
  unreliable.
- Recovery found missing commit `f4ffa9fc461a011cb7d9ad83b87a5aefa4f66fb8`
  and missing helper blob
  `3fb2108ec54e22e86f2f48a6b866c657451adca5`.
- `codex-rs/tui/src/internal_cli_helper.rs` was reconstructed from the persisted
  task edit record and verified to hash exactly to the missing blob ID.
- The final candidate was reconstructed in
  `/Users/Neo/.codex/worktrees/claude-subscription-auth-recovered` on branch
  `feat/claude-subscription-auth-recovered`, based on remote `main`
  `9ec532ed144ff041cae32592414e9e21873df6fe`.
- Final binary: `codex-rs/target/debug/corbanu`, version `0.1.35`, SHA-256
  `c7aaec9431a96db7478672a9d4a4312796814a0c3dc3a9f0560ddc0d06ced86a`.

## Final-review dispositions

The artifact-backed PF-47 history contains twelve formal review runs: nine
structured Codex autoreviews, two completed Opus reviews, and one interrupted
Opus attempt. Ten produced completed verdicts and two were attempted or in
progress. This feature was not sent through a thirteenth review.

- The original Corbanu Terminal Opus artifact is
  `.codex-work/claude-subscription-auth/pf-47/opus-review/logs/codex-tui.log`
  in the preserved worktree, SHA-256
  `4b09638a6424f972b58f5ce5e2296801f0552736525798de424d9b90906b0f95`.
- The last completed Opus artifact is
  `.codex-work/claude-subscription-auth/pf-47/opus-review-final-2/logs/codex-tui.log`,
  SHA-256
  `9de4e4b679738ade4f9515684875d13ba06d2de1c218bf46a909a621f6febdb5`.
- Its accepted findings were fixed: cancellation now wins while waiting for
  the child and during verification; stale completion cannot persist a
  provider after cancellation; forced API-key onboarding cannot highlight the
  hidden Anthropic option; and metadata documentation matches `/providers`.
- The clean structured artifact
  `autoreview-final-clean-6.txt` reported no actionable findings and a 0.62
  probability that the patch was correct; SHA-256
  `f2ad1aa28add9ec529ade4875c00ed00953f660332e3c96cea8e04fa35811486`.

## Retry-disabled final-tree tests

| Surface | Result | Nextest run |
| --- | --- | --- |
| Cancel after code submission | 1/1 passed | `7d2366e1-9308-4c51-897f-c5777f5aa931` |
| Stale completion after cancel | 1/1 passed | `58580151-7ed2-4903-843c-4fdf17cfb036` |
| Forced API-key default | 1/1 passed | `8ad8b82c-89d0-4f96-97f1-c2cbe13c6bc2` |
| Onboarding affected suite | 46/46 passed | `ca1a51b7-7893-4697-888b-3c6942c47bd1` |
| Claude login affected suite | 27/27 passed | `5b308f0e-81b1-4610-a916-d1970095946e` |
| Vault Claude-auth custody and rollback | 21/21 passed | `3fe3368f-30ab-4cf0-b7af-1abb81175f9c` |
| CLI platform discovery, selection, and refresh | 135/135 passed across all CLI binaries | `481f128e-137c-4dd6-a36c-bf0155b770e2` |
| External-bearer cache and revision behavior | 7/7 passed | `d05698e8-2758-49bc-aff3-a44a09b879a4` |
| Claude Plan provider policy | 3/3 passed | `6914c988-0934-4a89-ae3c-a1ed282a533e` |

`just fix -p codex-tui`, `just fmt`, `cargo build -p codex-cli --bin corbanu`,
and `git diff --check` also passed. The build emitted only existing dead-code
warnings outside this feature and a non-failing test-helper argument-count lint.

## Required typed-Tmux qualification

All cases used the real recovered binary, unique private Tmux sockets, typed
Down/Enter/Escape keys, text and Enter as separate actions, `RUST_LOG=trace`,
isolated homes/log directories, `CORBANU_TMUX_REQUIRED=1`, serial execution, and
`--retries 0`.

| Flow | Result | Nextest run |
| --- | --- | --- |
| First-run Anthropic account to Claude Code login, persistence, and `/providers` | 1/1 passed in 29.143s | `168ecd09-47e4-4e18-ac1c-ac02869c3df5` |
| Managed success, cancel, failure, recovery, masking, and restart | 1/1 passed in 56.502s | `304458c3-836e-43aa-8adb-7f285606ea1f` |
| Existing Claude Code compatibility login | 1/1 passed in 12.185s | `b252db0d-b4cb-4042-8fa1-bce34cda4f83` |

Successful runs emitted no failure bundles under
`.codex-work/claude-subscription-auth/pf-47-final/tmux`. The managed canary
checks found no raw token in viewport, scrollback, isolated home, trace log, or
retained artifacts.

## Human evidence and deliberately open gates

Neo visually tested the first-run screen, identified that Anthropic account
enrollment was missing, then tested managed-token entry and identified pasted
line-break handling. Both issues are implemented and covered by automated
qualification. No claim is made that a live eligible Anthropic request passed.

Live eligible-account requests, TensorCash and Isometric Game qualification,
physical Linux/Windows release-host evidence, and target release/tag work remain
open on the active plan. Main integration is not itself a tagged release.
