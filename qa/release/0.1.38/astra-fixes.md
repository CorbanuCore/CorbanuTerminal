# Astra reconciliation repairs — 2026-09-04

Status: scoped repair complete; not a release authorization or production fix claim.

## Identity and authority

- Classification: product initiative under the repository's credential-boundary rule.
- Product citation: **Shipping MVP — LIVE**, “Encrypted `/vault`, masked entry,
  metadata-only inspection, and operational credential use without placing raw
  values in chat”; existing profile-owned Task Node identity and scoped wallet signing.
- Plan: `docs/plans/active/unified-provider-auth.md`; feature PF-57; sprint PF-57-S02.
- User authorized repair of three previously reported Terminal regressions and
  separately requested site API-key-generation review.
- Worktree: `/home/pfrpc/repos/worktrees/corbanu-reconcile-release-fixes`.
- Branch: `fix/reconcile-release-0.1.37-review`.
- Reviewed source: `integration/reconcile-release-0.1.37`, exact head
  `f03e95f7a65609bb442764d6306682d5fe43f6bb` (version 0.1.38 internally).
- Tested implementation: `41794c3ae7de689594b21837c18899945ee75cf5`.
  Subsequent handoff changes are documentation/ledger only.
- Original main checkout and its unrelated dirty files are untouched. No push,
  deployment, installed-binary replacement or live financial mutation was requested.

## Repairs and limits

1. **Daemon compatibility.** Ping proves liveness, not compatible operations.
   Clients negotiate the wire version before status, passcode delivery or an
   operational request. Legacy/older/newer mismatches receive
   `daemon_upgrade_required`, with payment-safe, home-specific restart guidance.
   Lock remains compatible and the TUI invokes it directly, without a failing
   status preflight. Existing socket and ownership-lock semantics are unchanged.
   **Legacy recovery is intentionally manual:** those daemons cannot safely
   acknowledge drained payment work. No automatic killing, socket unlinking,
   versioned second owner or payment retry is attempted.
2. **Task Node logout.** Durable per-scope link/unlink state prevents matching
   legacy authority from being imported again after logout/restart or cancelled
   relinking. Unlink commits its marker before deleting both credential records,
   propagates failures, and keeps residual credentials suppressed. Explicit
   relink cannot resurrect residual active state. Other profiles and the default
   legacy session are preserved.
3. **Corbanu key aliases.** A shared ordered alias list and non-blank resolver
   align transport, shared provider status and read-only account/usage requests.
   Order: `CORBANU_API_KEY`, `CORBANU_PLAN_API_KEY`, `PFTERMINAL_PLAN_API_KEY`, then
   managed storage. Environment credentials are not persisted. The wallet's
   stored-key disconnect action still refers only to stored credentials.

Astra reviewed the repairs. Its additional TUI lock-preflight finding was fixed
and re-reviewed. No additional P1/P2 was identified in the bounded repair review.
This does not claim an exhaustive security audit.

## Verification ledger

Formatting/fix tools ran before the final affected tests:

```bash
export CARGO_TARGET_DIR=/tmp/corbanu-astra-review-phGuUE/codex-rs/target
just fix -p codex-tasknode-session -p codex-wallet-daemon -p codex-model-provider-info -p codex-tui --locked --offline
just fmt
just test -p codex-wallet -p codex-wallet-daemon -p codex-tasknode-session -p codex-model-provider-info -p codex-provider-auth --locked --offline
just test -p codex-tui --locked --offline -E 'test(provider_status_host) | test(chatwidget::wallet_) | test(onboarding::provider_setup)'
CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all --locked --offline -E 'test(tmux_corbanu_env_)'
```

- Domain suites: **175 passed, 0 skipped**, run
  `5f571ba1-7e7a-4f31-b0af-6a8074704614`,
  `/tmp/corbanu-reconcile-fix-unit-final.log`.
- Focused TUI: **82 passed**; 3,955 unrelated cases filtered out, run
  `37849b53-4863-4efc-85cc-ec6d45d8d217`,
  `/tmp/corbanu-reconcile-fix-tui-unit-final.log`.
- True TMUX: **3 passed**, run `c14258d5-91e1-482f-831a-3ffe292ed154`,
  `/tmp/corbanu-reconcile-fix-tmux-final.log`. Each alias is checked alone, with
  higher-priority aliases blank, across two app processes. Actual text and Enter
  are sent separately; the account endpoint requires the expected synthetic
  bearer credential and the view shows `$12.20 available`. The public-alias case
  additionally verifies incompatible-daemon guidance, `/wallet lock` reaching
  the legacy wire shape, and successful retry after compatibility is restored.
- New wallet upgrade snapshot reviewed and explicitly accepted. It includes the
  entire wrapped restart/payment warning and Retry action at width 100.
- Real encrypted-vault reopen probe: `scoped_labels_immediately_after_unlink=1`,
  `active_session_restored_on_reload=false`. The remaining scoped record is the
  non-secret lifecycle marker, not a token. Probe uses a temp directory and an
  in-memory keyring; source/result are under `/tmp/corbanu-review-probes-DXFkxZ/`.
- Clippy completed with warnings and no errors; log
  `/tmp/corbanu-reconcile-fix-clippy-qualified.log`. Plan, sprint, portable-skill
  and `git diff --check` checks passed.

The first TUI run had 81 passes and one new snapshot awaiting acceptance. The
initial aggregate TMUX attempt timed out: large debug-binary evidence hashing
added overhead and its mock daemon address did not match the mock account. The
ownership guard correctly refused that account. The fixtures now agree, alias
cases are independent, and evidence hashing uses a streaming system SHA-256 tool
with the prior Rust implementation as fallback. No production ownership check or
timeout was weakened. Only the two test-owned orphaned TMUX servers were stopped.

### Interactive artifacts and binary identity

Artifacts were preserved outside the worktree at
`/tmp/corbanu-reconcile-tui-artifacts-gobo9D/target/tmux-artifacts/pf53-env-alias-{0,1,2}-restart-{false,true}/`.
Each directory
contains captured viewport/scrollback and `binary.sha256`. Canaries were checked
against visible output, request bodies, isolated-home files and saved evidence;
they appear only in their intended authorization path, not in those outputs.

Branded binary in the first successful TMUX run:
`fdbb0e8a95fbb104b7384e9b50f7b77e9109b65f1fecd139b88712e6d438ce22`.
Matching daemon: `8f719035c87ff573967f0d610563fb752413686444182c91226f11950c0a4a48`.

The repository skill's additional `just codex` run used the formatted source,
`RUST_LOG=trace`, explicit temporary `log_dir`, and home
`/tmp/corbanu-just-tui-gZ9RIN`. Only the public Corbanu key alias was populated with
a synthetic value; API origin was loopback. After confirming trust in that empty
temporary directory, the app entered chat without provider onboarding. Sending
`/providers` and Enter separately displayed `Corbanu API — Active · current`.
Escape and `/exit` exited cleanly. No inference request, signing or payment was
performed. Its home/log files contained no synthetic-key canary.

That freshly built `codex` binary has SHA-256
`05828c06c683bd87cd326c185766ed5d9e9a74b0aafb1720995d74f7af6fe6b6`.
All three TMUX cases were rerun successfully against this freshly built binary
by setting `CARGO_BIN_EXE_corbanu` to its exact path. Run
`3d31549a-e1d1-40ad-8010-e5130ddc13b5` passed 3/3 in 10.564 seconds;
`/tmp/corbanu-reconcile-fix-tmux-qualified.log` records the result. Preserved
viewport/scrollback/hash artifacts now correspond to this final rerun.

Build/fix logs are under `/tmp/corbanu-reconcile-fix-*.log`. Tests use the external
build cache `/tmp/corbanu-astra-review-phGuUE/codex-rs/target`, never an installed
user binary. The new interactive fixture uses synthetic canaries, private sockets
and loopback HTTP only; no real provider credentials, wallet signatures or funds.

## Site/API review

See [Astra site API-key review](astra-site-api-key-review.md): no demonstrated P0
key-generation bypass, but a P1 post-payment Solana error/recovery bug and a P2
funded-wallet replacement-key checkout bug remain in the site. The site/backend
were reviewed, not modified.

## Applicability and release gates

Linux host verification only; physical macOS/Windows, real OS-keyring behavior,
browser wallets, live settlement and production deployment are not qualified here.
The bounded home-level auth repair does not depend on TensorCash or Isometric
Game project contents; rationale is in the active plan. Prior plan evidence is
not relabeled as final-tree evidence. Named-human acceptance is pending. No
benchmark rerun or full 0.1.38 release qualification is claimed. Site findings and
the one-time legacy-daemon restart requirement must be disclosed before shipping.
