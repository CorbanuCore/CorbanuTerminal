Fixes postmerge-ci on main. These test expectations predate intended product changes. No product code changes.

## 1. PF-30-S03 vault test vs #443 (red since #453)
`suite::provenance::pf_30_s03_vault_command_after_untrusted_content_needs_the_human` ran for Moderate and Aggressive and expected the PF-30-S03 post-taint prompt at both. Since #443, Aggressive's vault rule refuses `corbanu vault ...` before that prompt (Travis, 2026-10-11: protections apply from confirmation).
- The existing test is now Moderate only, with its assertions unchanged. It asks the human, offers no "don't ask again" amendment, and is refused with approvals off.
- New `pf_30_s03_aggressive_refuses_a_vault_command_after_untrusted_content_outright` covers Aggressive after tainting tool output, with approvals on-request and never. There is no approval request; the output carries the Aggressive vault refusal. A fake `corbanu`, called by absolute path, never runs.

## 2. App-server verification test vs #394 (red since #394, 2026-10-10)
`suite::v2::safety_check_downgrade::model_verification_emits_typed_notification_and_warning_v2` sets `OPENAI_API_KEY` and fails on any `warning`. Since #394, that turn also carries the one-time notice "Using OPENAI_API_KEY from your environment…". The collector now ignores exactly that notice (`OPENAI_API_KEY_ENV_FALLBACK_NOTICE`). Any other warning still fails, and the message is included.

## 3. exec trace-log test: diagnostics only
`suite::trace_log_credentials::http_turn_keeps_the_key_out_of_trace_logs` has failed on postmerge since #469 because a home file vanished during the walk. The panic now names the file. The failure is tracked in #477; this PR doesn't change the test's guarantee.

## Evidence (Linux, glitch)
- Focused tests: core `pf_30_s03|issue_443` 30/30; app-server `safety_check_downgrade` 5/5; exec `trace_log_credentials` 2/2.
- Full `just test -p codex-core` at 863da24358 plus this change: 4067/4069. The two failures are glitch-only and also fail at c7e6e07cbc, before the 2026-10-11 merges; CI passes both:
  - `pf_30_s01_source_envelopes…`: bwrap fails when `~/.claude` exists without `.credentials.json`. This is a real product bug, filed as #473.
  - `network_approval::ambiguous_unattributed…`: interrupt → `TurnAborted` times out on glitch only.
- `cargo clippy --locked --tests -p codex-core -p codex-app-server -p codex-exec -- -D warnings`: clean.
