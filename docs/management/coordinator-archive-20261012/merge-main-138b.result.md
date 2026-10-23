Main `c2d3c04823` is merged into `integrate/management-workstreams-20260911` and pushed fast-forward as `6a69cfa76c`. On #138, 41 checks pass and none fail; the four v8-canary builds were still running when I stopped.

**Pushes**
- `f045f36dba`: the merge commit, including one compile fix I folded in before pushing.
- `6a69cfa76c`: by push time the branch had picked up one new docs-only commit (`13934dc3dc`, the Task Node map). I merged it in and pushed fast-forward. No rebase, no force.

## How each conflict group was resolved
- **`core/src/client.rs`:** main deleted the chat tool-call replay helpers, but this branch still calls them, so I kept them.
- **`core/src/client_tests.rs` (merged without a conflict, but broke the test build):** main had deleted 4 test helpers. The branch's tests still call two of them (the Opus 5.5 plan-request test and a cache-marker test), so I restored those two. The other two were already gone on the branch.
- **`provider_credentials.rs`, `wallet_receipt.rs`:** main marked the provider-credential status refresh and `latest_plan_receipt` as dead code. This branch had already deleted them, so they stay deleted.
- **`app_event.rs`:**
  - `ProviderCredentialStatusesReady` had picked up an allow-dead-code annotation from each side; I kept one, with main's comment.
  - `OpenPlans` takes main's narrower annotation, because only tests construct it here.
  - The two wallet-plan variants take main's comments, which say which commit made them unreachable.
- **`pfterminal_plan_status.rs` (merged without a conflict):** main's item-level annotations were redundant, because this branch already allows dead code for the whole file. I kept the branch's version.
- **`claude_code_login.rs`, `update_action.rs`, `updates.rs`, `dispatch_integration.rs`:** kept the branch's annotations. They are narrower or more accurate than main's.
- **`wallet_menu.rs`, `terminal_ownership_tests.rs`:** took main's explanatory comments.
- **`Cargo.lock` and snapshots:** neither needed any change; cargo left the lock file alone and no snapshot changed.

## Test results
- **Linux (RTX box, fresh clone, deleted afterwards):**
  - codex-tui: 4200/4200 pass.
  - Secret-broker crates and codex-state: 487/487 pass, and clippy is clean apart from 2 warnings the branch already had.
  - The 487/487 needed `TMPDIR` on a normal disk. With the default `/tmp` (an in-memory filesystem on that box), four fixture tests fail, because the protected-state code deliberately refuses that filesystem type.
- **macOS:**
  - `cargo build -p codex-cli` passes.
  - protocol, api and state: 906/906 pass.
  - codex-core (accounting, multi-agent, exec_env, spawn, security, chat): 548/548 pass.
  - codex-tui: 4181/4203 pass. The 22 failures:
    - 14 are the known "Unix socket path too long" problem with the long temp paths here.
    - 6 are wallet-daemon tmux tests where the daemon timed out on startup. I believe this is the same socket-path problem, but I didn't confirm it from their logs. All 6 pass on Linux.
    - 2 are the command-popup snapshots that are already out of date on both main and the branch.
  - `just fmt` is clean. Clippy on tui, core, cli and git-attribution is clean.
  - GitHub-script tests: 54 run, with only the 4 known setup-telegram failures.

## #138 check status at `6a69cfa76c`
- **Passing (41):**
  - Builds and tests: real-tui, Ubuntu tmux smoke (no retries needed), all Linux, macOS and Windows credential canaries, both Build and package jobs, and Windows Bazel.
  - Repo policy: repo-checks, Blob size policy, cargo-deny and cargo shear.
  - Lint and analysis: all three argument-comment lint jobs, Format, Codespell, and every CodeQL job including Rust.
- **Still running:** the four v8-canary builds (release and ptrcomp-sandbox, gnu and musl). They build V8 from source and had run for over two hours.
- **CI rounds:** this was the first and only round; nothing needed a fix.
- **PR description:** I added a "Merged main c2d3c04823" section with the conflict summary and this status.

## What's left
- Someone needs to confirm the four v8-canary builds finish green; GitHub shows #138 as "unstable" only because they're still pending.
- These were already true on the branch, not caused by the merge:
  - The two macOS-only command-popup snapshots are still out of date.
  - codex-state still gets two clippy `duplicate_mod` warnings.