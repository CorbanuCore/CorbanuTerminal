You are a small cleanup worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding).
sec-win3 found that origin/main fails Windows clippy (`-D warnings`) in codex-core (`model_broker_auth.rs`,
`exec_policy.rs`) and in the `protected-state` crate; CI does not catch this. Fix it.
Real Windows machine (toolchain under C:\CorbanuQA): ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes
user@desktop-ujs3hc4.tail36b615.ts.net. Another worker (sec-win4) is running a gate there using C:\CorbanuQA\target and
its own run folders: do NOT touch those; use your own checkout and CARGO_TARGET_DIR under C:\CorbanuQA\clippy1\ and keep
-j modest (8) so you don't starve the gate.
1. On Windows, run clippy with -D warnings for the affected crates (and the workspace if feasible) at current origin/main;
   list every warning. Fix them minimally and correctly (no blanket allows unless a lint is genuinely wrong, then a
   scoped allow with a reason). Don't change behaviour.
2. Check whether CI runs clippy on Windows for these crates at all; if not, propose (in the PR body) the smallest CI change
   that would catch it, and add it only if it's cheap (a single job or matrix entry that reuses existing caches).
3. Linux clippy on the RTX box (ssh ambient@100.99.88.49) and macOS `cargo clippy` stay clean; focused tests pass.
4. One independent Opus 5.5 High review via installed `corbanu exec` (read-only). PR to main; merge when green with
   `gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f.
Clean up C:\CorbanuQA\clippy1 and any Mac/RTX scratch dirs when done. Report as in sec-common.md.
