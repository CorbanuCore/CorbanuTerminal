You are the broker lane worker (round 5). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding), then sec-broker4.result.md and sec-url2.result.md in the same folder.

1. Small bug fix first (own PR): PF-33-S01 sets `url_destination_policy` in one config path in `core/src/config/mod.rs`
   but not the second one used when the permission profile changes (sec-broker4 found it; PF-28-S02 set its own flag
   in both places). Fix it with a regression test that changes the permission profile mid-session and asserts the
   policy still applies.
2. Brokered credentials are not pinned (from PF-33-S02): requests that go through the credential broker skip
   connection pinning and the broker looks the name up again. Pass the checked DNS answers through to the broker so
   the broker connects to the pinned address. Tests with fake DNS as in PF-33-S02. Record it in PF-28-S02's or
   PF-27-S05's record, whichever owns it.
3. Then PF-27-S05 "Model-client auth in the broker"
   (docs/sprints/current/p0-security-levels/pf-27-s05-model-client-auth-broker.md): move Core's own model-provider keys
   (and vault-label credentials) into the broker so Core and agents only hold opaque references.
Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo videos).
Run affected crates' tests locally and Linux clippy on the RTX box before merging (PR CI doesn't run them). Note PR
#214 (docs bookkeeping) may touch the same sprint files; merge origin/main before pushing. Fresh worktree off
origin/main; own CARGO_TARGET_DIR. PR to main; merge when green (stale CodeQL checks can be ignored). Report as in
sec-common.md.
