# Phase-1 upgrade compatibility: 0.1.48 to main

Date: 2026-11-17 UTC. **Upgrade passed the exercised compatibility checks with
one documented config addition. In-place downgrade failed; backup rollback
passed. This is not final 0.1.49 release qualification.**

## Exact inputs and isolation

- Old: published `rust-v0.1.48`, source
  `e4e8d9b223215ffa9a7764ff2de32fded42b1e9b`; Linux x86_64 GNU package verified
  against the published `corbanu-terminal-package_SHA256SUMS`.
- Old executable SHA-256:
  `d0763b4a36fd2589fe826143cac231ebcfa3fc13e2c6c043f84d0e789feba725`.
- New: `git archive` of main `d3fe32513ee65deb397fa228c786212fb5afacd3`,
  Rust 1.99.1, `cargo build --locked -j10 -p codex-cli --bin corbanu`, private
  target, debug info disabled. Build succeeded in 2m30s. The binary still
  reports **0.1.48** because the version bump is correctly gated.
- New executable SHA-256:
  `5f436c90c201f50040d339f7e0b68863307401163031fd40a73009a08e9535bd`.
- Host: glitch. No build started while the separately checked machine-wide
  Cargo count was two; the actual launch followed a count of zero.
- Ubuntu 24.04 container, `--network none --cap-drop ALL`, no-new-privileges,
  4 GiB memory and 256-PID caps, unprivileged host UID. Only this worker's
  directory was mounted. Negative probes confirmed no host `.ssh`, Corbanu
  home, user service sockets or Docker socket; no native keyring was mounted.
- All profile aliases pointed to `/work/profile`; synthetic vault value only.
  Local mock Responses server ran on the container's loopback. No live provider
  authentication, model inference or coordinator-home access occurred.
- Package was extracted and installed in a private `install/current` plus
  `install/bin/corbanu` layout. The real installer was **not** used to install
  this fixture. Its separate Linux contract tests passed 5/5.
- Initial bubblewrap attempts failed at network namespace / UID-map setup.
  Those attempts ran no candidate. Host settings were not changed; Docker
  provided the isolated test environment.

## Actual TUI sequence

The 140x45 tmux PTY launched the old installed executable with `--no-alt-screen`,
trace logging, a trusted private workspace and a custom loopback provider.
Text and Enter were sent separately.

1. `/vault credential add`: enter label `release-upgrade-canary`, then a
   generated synthetic value; observe the added-credential confirmation.
2. Submit `Reply with UPGRADE_REPLY for baseline`; observe the completed
   `UPGRADE_REPLY` response; `/exit`.
3. Verify vault retrieval by comparing captured stdout in memory, without
   printing it. Checkpoint all SQLite databases and copy the stopped profile.
4. Seed five synthetic provider-state cases and create a second backup: idle
   unsalted row, active cooldown, active lease, 308,374-second cooldown, and an
   unrelated provider/key. These rows were fixture inserts into a database
   created by 0.1.48, not provider responses manufactured by that binary.
5. Run the main executable with `resume --last` on the same profile. Observe
   the existing transcript, send the upgrade prompt, receive a new mock
   response, then `/exit`.
6. Reopen a copied upgraded profile using 0.1.48: observe startup refusal for
   foreign migration 53. Do not modify migration records.
7. Mount a copy of the untouched pre-upgrade backup at the original profile
   path; run 0.1.48 `resume --last`, receive a response, verify vault again.

## Results

| Check | Result |
| --- | --- |
| Published old package checksum and version | Pass |
| Existing thread id and rollout | Pass; same thread, rollout prefix preserved and extended |
| History reopening | Pass; old history is an unchanged prefix of new history |
| New completed response after resume | Pass; local mock received a new request; persisted assistant messages and `task_complete` events increase from one to two (also two on backup rollback) |
| Vault value after upgrade | Pass; compared in memory; Linux file-backed key storage only |
| State, logs, goals and memories databases | Pass; `PRAGMA integrity_check = ok` on all four |
| State migrations | 1–52 become 1–53; 32-byte provider salt exists |
| Active cooldown/lease, count and unrelated-provider controls | Pass; preserved |
| Expired idle unsalted row | Removed as specified by migration 53 |
| Actual legacy credential-key adoption | Pass; one salted row replaces the old row; days-long cooldown, active lease and consecutive-429 count survive; attempted exec is blocked by that cooldown |
| Configuration byte identity | **Fail as a strict assertion**: `service_tier = "default"` was added |
| Existing config values | Pass; every original parsed entry unchanged; default service tier is the only addition |
| In-place downgrade to 0.1.48 | **Unsupported in this test**; startup rejects migration 53 |
| Rollback with pre-upgrade backup | Pass; config byte identity, conversation response and vault comparison |
| Synthetic vault-value scan | Pass; no matches in the initial 33 files or final 34-file scan |

The config addition is explained by
`codex-rs/tui/src/app/resume_defaults.rs::remember_resumed_model`: acknowledged
interactive resume remembers the session's selection and persists an absent
service tier as `default`. It is disclosed in the release notes. The original
failed byte-equality assertion is preserved in `upgrade-result.json` rather
than rewritten as a pass.

The downgrade UI says the database “appears to be damaged,” then its technical
detail identifies the actual foreign migration. All integrity checks passed;
this is compatibility refusal, not observed corruption. Restore the complete
matching backup and package to roll back; do not erase migrations or mix a new
database with old vault/session files. Post-backup turns are absent from that
backup.

## Limits and next execution

This exercise uses current-main debug code, a deterministic provider and Linux
file-backed vault storage. It does not qualify real providers, macOS Keychain,
Windows Credential Manager, native Windows/macOS UI, installer-in-place upgrade,
code-blind acceptance or final release packaging. No extra native-account
qualification is implied by the environment flag.

The known #2567 days-long cooldown is preserved and still blocks the request in
this baseline. Re-run its clamping/clear/reprobe cases after its fixing PR merges.
Repeat the full compatibility run on the final release-branch binary/package.

Raw evidence and synthetic fixture backups are retained under
`glitch:~/corbanu-rtx/astra-rel149/evidence/` and that worker's adjacent profile
folders. Harness sources are retained under `$W/astra-rel149-{upgrade,seed,
throttle,verify}.py`; the original harness is also archived beside the evidence.
Small, secret-scanned result records are checked in under `upgrade/`.
