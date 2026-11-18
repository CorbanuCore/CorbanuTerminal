# 0.1.49 release-note coverage audit

Baseline: main `d3fe32513ee65deb397fa228c786212fb5afacd3`, compared with
`rust-v0.1.48` (`e4e8d9b223215ffa9a7764ff2de32fded42b1e9b`).
`release_notes.py check` passed for **123** existing fragments; `preview`
succeeded. The requested first-parent merge inventory contains **1,266** rows.
The assignment's estimate of 124 fragments differs from this exact baseline.

Method: enumerate first-parent merges, extract PR numbers/titles, compare with
fragment filenames and referenced PR/issue numbers, then inspect relevant code
and the release-tag diff before adding prose. Only 73 merge PR numbers are
literally referenced by a fragment filename or `#number`; that is **not** a
count of covered changes. Many fragments combine multiple PRs under an issue
number, and many merges are tests, CI, documentation or dependency work.
The raw inventory and machine-readable cross-reference are preserved in the
worker evidence directory as `astra-rel149-merges.txt` and
`astra-rel149-coverage.json`.

## Verified missing areas filled

| Area | Verified code / merge | Fragment |
| --- | --- | --- |
| Approval typing protection | `tui/src/bottom_pane/approval_overlay.rs`, typing guard; #187 | `187-approval-typing-summary.md` |
| Secret arguments in CLI errors | `cli/src/main.rs::secret_argument_error`; #1301 | `1301-secret-argument-errors.md` |
| Named-account CLI and resume identity | `cli/src/account_cmd.rs`, `cli/src/named_login.rs`, named-account/resume paths; #409/#430/#566/#1299 | `409-named-account-command-summary.md` |
| Home selection / launcher compatibility | `cli/src/pfterminal_home.rs`, `scripts/install/install.sh`; #408/#422 | `408-home-selection.md` |
| Retired Claude model pin recovery | `model-provider-info/src/lib.rs::retired_model_replacement`, config caller; #698 | `698-retired-claude-pins.md` |
| Provider throttle salt / active legacy state | migration 53, `state/src/runtime/provider_requests.rs`, `core/src/provider_throttle.rs`; #485/#569 | `485-provider-state-upgrade.md` |
| Exec recovery from unreadable home | `exec/src/cli.rs::home_wait_secs`, `core/src/home_access/mod.rs`; #861 | `861-cli-home-access-recovery.md` |
| Resumed settings saved as defaults | `tui/src/app/resume_defaults.rs`; integration #138; observed in isolated upgrade | `138-resume-default-settings.md` |
| Downgrade limitation | old binary actually rejects migration 53; restored backup succeeds | `0.1.49-upgrade-and-rollback.md` |

Paths in the first column's code references are relative to `codex-rs/` unless
explicitly rooted at `scripts/`. These notes describe verified behavior; they
do not claim fresh qualification of every historical security fix.

## Existing coverage and no-change findings

- Security: existing fragments cover Moderate/Aggressive post-untrusted-content
  command parsing, protected paths, Windows ACL/sandbox changes, secretless
  executor/MCP launch, credential-bearing shell snapshots and nested scripts.
  Those entries already combine many first-parent merges; do not duplicate each
  parser hardening PR as a separate release claim.
- Providers/auth: existing fragments cover named-account status, login-status
  fingerprint redaction, regional OpenAI hosts, restored provider picker rows,
  Anthropic pre-output stream recovery and plan-entitlement behavior. The missing
  account-command introduction and provider-state migration are added above.
- Removals: existing fragments explicitly retire OpenRouter Owl Alpha and Tencent
  Hy3 Free. Add the verified Claude Opus 4.8 pin recovery, not an unsupported claim
  that all older models were removed. No new product-file deletion appeared in
  the inspected code/CLI/installer diff. Keep existing model visibility intact;
  the upcoming new-models PR must be additive per D703/D704.
- Doctor: `cli/src/doctor.rs` differs only in its HEAD-timeout/GET-fallback test
  synchronization and timeout budget (10 additions, 6 deletions). No production
  doctor behavior change is claimed. The old binary's downgrade error suggesting
  doctor is recorded as an observation, not a new doctor feature.
- Release tooling: fragment validation/assembly is maintainer process, not a
  product feature. Installer test fixes are not described as runtime changes.

## Final refresh

Release base `ad2b22f0b287132dbc4dad48575614403c758fc6` adds ten first-parent
merges to the audited baseline (1,276 since rust-v0.1.48). Main now has 132
fragments. Nine added/updated fragments cover quoting, trusted PowerShell parsing,
Windows allow spelling, cooldowns including named-account clearing, PowerShell
command-line parsing, Telegram polling, Windows label diagnostics, ChatGPT model
access and the model additions. All eight gate PRs and #2589 are in the base.

Our nine verified additions bring the assembly to 141 fragments. Check and
preview pass; `assemble --version 0.1.49` transferred every fragment. The
versioned notes body exactly matches that preview, and no unreleased fragments
remain. This is coverage evidence, not certification of every historical merge's
product classification or fresh qualification of every described feature.
