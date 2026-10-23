# PF-84 follow-up gate: token usage per account in `/status`

Travis's decision of 2026-10-11 (option b) on S05's open question: release builds get a light per-account
token summary in `/status`, with no cost records. Cost tracking (`/cost`) stays developer-accounting only.
PR [#528](https://github.com/CorbanuCore/CorbanuTerminal/pull/528), branch `pf-84-status-account-tokens`;
behind `named_accounts`.

## What shipped

- **Core.** With named accounts on, each request's provider-reported usage (the `record_token_usage_info`
  path that feeds **Token usage**, compaction and the completion check included) is added to the
  (provider id, account name) that served it and sent with the token count as `account_usage`, which is
  omitted otherwise, so other events and rollouts keep their bytes. Names only: no key, no fingerprint.
  - Requests the ledger records under the conversation but made outside its turn loop count too: guardian and
    `/review` delegates add each increase of their running total to the parent (sent on the parent's active
    turn, else with its next count); stage-one memory extraction adds to its owner.
  - A resumed thread restores its counts from its rollout, only with the flag on. A fork (spawned agent with
    forked context, side conversation) starts at zero and, with the flag on, writes a marker after its copied
    prefix, so resuming it before its first reply does not count the parent's usage again.
- **App server.** `thread/tokenUsage/updated` has an optional `accountUsage`, also on the resume replay.
- **TUI.** `/status` adds **Account usage** for the active provider, summed over this session's threads
  (coordinator, spawned agents, side conversations; a report from an earlier session's thread is ignored):
  one account on one line (wrapped to the card), several as an aligned table with the session's account marked
  `(active)`; `no usage yet` before the first reply. Input is the uncached part, as on **Token usage**; cached
  input and reasoning only when reported. A new session starts from zero; a thread switch keeps the totals.

## Tests (macOS, `just test`, isolated profile)

| Command | Result |
| --- | --- |
| `just test -p codex-tui -E 'test(/account_usage\|status\|thread_account_usage\|token\|account\|provider_health\|named/)'` | all pass except 4 environmental (below) |
| `just test -p codex-tui --features developer-accounting -E` same `+ cost` | same, plus the `cost_scope` tmux flake (below) |
| `just test -p codex-core -E 'test(/account\|token_usage\|token_info\|token_count\|accounting\|named_account\|resume\|throttle\|provider_request\|forward_events\|delegate/)'` | all pass |
| same with `--features developer-accounting` | all pass except `request_user_input_interrupt_emits_deferred_token_count`, which fails identically on `origin/main` with the feature (checked) |
| `just test -p codex-app-server-protocol` (schema fixtures) | 287/287 |
| `just test -p codex-app-server -E 'test(/token_usage\|thread_resume\|thread_fork\|token_count/)'` | 80/80 |
| `just test -p codex-exec -E 'test(/token_usage/)'`; `-p codex-thread-store -p codex-external-agent-migration` | 1/1; 307/307 |

Environmental TUI failures: `multi_provider_onboarding::tmux_corbanu_env_*` (3, "path must be shorter than
SUN_LEN", as in S05); `claude_auth::tmux_first_run_…` waits for the footer's `TPS:`, which this worktree's long
path truncates; `cost_scope::…` typed `/cost` before its session started (the S05 flake). None touches `/status`.

New tests: `status/account_usage_tests.rs`; insta snapshots `status_snapshot_account_usage_{flag_off,
no_usage_yet,one_account,several_accounts}`; `status_account_usage_flag_off_leaves_status_unchanged` (reports
recorded, flag off: byte-identical to no reports) and `status_account_usage_lists_the_active_providers_accounts`;
`thread_account_usage_reaches_status_and_resets_with_the_session` (through `handle_app_server_event`: unknown
thread and flag-off reports ignored, a thread that left still counts, a thread switch keeps totals, a new session
clears); core `token_usage_is_attributed_per_account_only_with_named_accounts`,
`resume_restores_account_usage_and_fork_does_not` (flag off restores nothing),
`resuming_a_fork_before_its_first_reply_has_no_account_usage` (real fork rollout),
`delegated_usage_counts_toward_the_parent_only_with_named_accounts`,
`forward_events_counts_delegate_usage_toward_the_parent_once`, `account_usage_increase_is_per_account_and_skips_unchanged`
and `accounting_chat_named_account_summary_agrees_with_the_ledger` (two turns on a named account: input, cached,
output, reasoning and total equal the ledger's `DayTotals` that `/cost` sums).

Clippy `-D warnings`: clean on macOS and on glitch (`-j10`) for every touched crate, with and without
developer-accounting.

## Live tmux check (GLM 5.3 Flash, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`)

Script: [`s05-status-tokens/scripts/tmux-check.sh`](s05-status-tokens/scripts/tmux-check.sh). Two named Z.AI
accounts, `work` and `personal`, both enrolled with the real key from `provider/zai_api_key` (read only inside
the consuming command). The session ran with `--account work`; a spawned agent ran on `personal`; then the
conversation was resumed. Captures are from the final tree (`20528c9d78`; later commits change only a doc comment and docs).

- **Release-path build** (debug, no developer-accounting) ([captures](s05-status-tokens/captures/release/)):
  `work · no usage yet` → after a turn `work · 7,432 input · 19 output (16 reasoning)` → after the spawned
  agent the table `work (active) 7,768 / 22,464 / 138 / 57`, `personal 4,434 / 0 / 3 / 0` (input / cached input
  / output / reasoning). These equal the provider-reported usage summed from each thread's rollout
  ([reconcile.txt](s05-status-tokens/captures/reconcile.txt)). After `resume --last --account work`, `/status`
  shows the same table again, the spawned agent's row included (`5-resumed.txt`).
- **Developer-accounting build** ([captures](s05-status-tokens/captures/developer-accounting/)): `/status`
  showed `work (active) 14,589 / 15,552 / 112` and `personal 3,847 / 576 / 16`; `/cost` at the same moment showed
  `account work … 30,253 tokens` (14,589 + 15,552 + 112) and `account personal … 4,439 tokens`
  (3,847 + 576 + 16), equal to the rollouts too; the resumed `/status` matched. (A run before the review fixes
  differed only because the coordinator kept working after the capture.)
- Observed, outside this change: resuming a thread recorded on a named account without `--account`, in a home
  with no default key, opens the API-key screen before the thread's account applies.
- Leak scan: no capture, log or disposable home holds the key in plain text.

Video (leak-scanned, 0 hits): [`pf84-status-account-tokens`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s05-pf84-status-account-tokens-20528c9d785e-2026-10-11.mp4),
session on `default` and a spawned agent on `work` ([demo index](../../demos/index/PF-84-S05.md)).

## Review (Opus 5.5 High, read-only, installed `corbanu exec`)

Round 1: REQUEST CHANGES (flag-off resume still sent `account_usage`; guardian, `/review` and stage-one memory
usage missing from `/status`). Round 2: REQUEST CHANGES (gate evidence predated the fixes). Round 3 on
`1a7c976a47`: **APPROVE**; its doc-comment nit is fixed, its other two notes are listed below.

## Known limits

- `/cost` is per UTC day; the summary covers the session. Background memory consolidation is recorded by `/cost`
  under the conversation but not shown in `/status`. Stage-one memory extraction outside a turn shows with the
  next count.
- A remote app server older than this change sends no `accountUsage`, so the line reads `no usage yet`.
- A side conversation discarded in the background before it ever reported loses a request that finishes in
  that gap (the ledger keeps it).
- Review round 3 (APPROVE) suggested a test that delegated usage is sent on the active turn; not added.
