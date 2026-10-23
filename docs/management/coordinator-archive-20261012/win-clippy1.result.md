I fixed every Windows clippy failure, and PR #317 is merged into main (merge commit `5933e2eb0d`). `cargo clippy --workspace --all-targets -- -D warnings` now passes on Windows, Linux and macOS.

**What was wrong (Windows, at origin/main):**
- **codex-core:** 12 warnings in `model_broker_auth.rs` and `strict_rules()` in `exec_policy.rs`. Fixing those exposed test-only ones: `LINUX_SANDBOX_SKIP_REASON` in the core test support crate, and imports in `pf_23_s01.rs` and `pf_29_s01_tests.rs`.
- **codex-protected-state:** `Checkpoint`, `Binding` and `hash_valid` are only used by Linux-only modules. This one also failed on macOS.
- **Blocking crates outside the three named:**
  - codex-windows-sandbox (a redundant closure) stopped codex-core from compiling at all.
  - The full workspace run found more in http-client, telegram, wallet, browser-isolation and tui.
  - The `wallet-daemon` example called `expect()`, which fails on every platform with `--all-targets`.

**How I fixed it:** almost everything is Unix- or Linux-only code that is dead on Windows, so I put it behind the right `cfg`. There are no crate-wide allows. Where code is deliberately shared, I used narrow `cfg_attr(not(unix), allow(..., reason = ...))`, mainly in `model_broker_auth.rs`, where the credential type has no values on Windows so code after registration can never run there. One tui variant that is never constructed on any platform now has `#[allow(dead_code)]`. The only behaviour change: the wallet-daemon example now exits with code 1 and a usage message instead of panicking when run with no argument. Its callers always pass one.

**CI:** nothing ran clippy for Windows. I added one Windows entry to the postmerge `rust-ci-full.yml` lint matrix, reusing the existing job and caches, with a 60-minute timeout. A test run on the branch passed cold in about 17 minutes. I updated `test_ci_timeout_budgets.py` to match. Two limits remain:
- It runs after merge, so it won't block a PR. The same entry could go into PR CI (`rust-ci.yml`) if you want that.
- The shared job uses `--tests`, not `--all-targets`, so examples like the wallet-daemon one still aren't linted in CI on any platform.

**Gate evidence:**
- **Clippy:** the workspace run is clean on Windows, Linux (RTX box) and macOS.
- **Focused tests:** all pass on macOS. On Windows they pass apart from the 5 `pf_29_s01` tests below.
- **PR #317 checks:** all green.
- **Review:** the Opus 5.5 High read-only review approved with no blockers. I acted on its two should-fix points (Windows timeout, and noting the `--tests` gap). The report is at `.codex-work/workers-20261002/winclippy1-review/review-output.txt`.

**Test failures not caused by this change:**
- **Windows:** 5 `pf_29_s01` preflight tests fail on the Windows machine. My only edit to that file was gating one import. Windows tests don't run in CI.
- **Linux:** `pf20_s03_torn_pending_and_symlink_replacements_latch_unavailable` failed 2 of 3 parallel runs with a lock `Conflict` and passes alone. The Linux code is unchanged by this PR. The other protected-state tests fail on the RTX box's tmpfs `/tmp` by design and pass with `TMPDIR` on ext4.

**Open follow-ups:** delete the never-constructed `UpdateAskForApprovalPolicy` variant instead of allowing dead code, and decide on the two CI points above.

**Cleanup:** `C:\CorbanuQA\clippy1`, the RTX `~/corbanu-rtx/wclippy1*` folders, my Mac worktree and build folder, and the `/tmp` scratch files are all removed. I didn't touch sec-win4's folders. The review folder is kept as evidence, and the remote branch `fix/windows-clippy-20261008` still exists.