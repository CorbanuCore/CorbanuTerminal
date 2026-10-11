# PF-84-S04 gate: account management in /providers and onboarding

Candidate: branch `feat/pf-84-s04-provider-accounts-ui`, code at `7a1ab9bc9d` (later commits are
docs only). Change class: product initiative (plan PF-84). Product spec: "Unified provider
onboarding and management" in [the plan](../../../docs/plans/active/unified-provider-auth.md),
acceptance rows "Add second account", "Session account switch", "One row per account".

## What shipped

Behind the default-off `named_accounts` feature, except the onboarding duplicate-row fix:

- **Onboarding, one row per configured provider.** A configured provider used to show its setup
  row and a "use existing" row (Claude Plan: "Anthropic Claude Account" and "Claude Account",
  both configured). It now shows one row. Choosing it offers *Use configured credentials*,
  *Add another account* (feature on) and *Replace with ...*. Focus stays on that row after setup.
- **/providers, one row per (provider, account)** with kind, a salted 12-hex fingerprint and
  "this session" / "default for new sessions" markers. Account actions: *Use for this session*
  (starts a new session on that account; a live thread never changes account), *Make default*
  (`[provider_accounts]` in config.toml, verified after reload), *Rename* (refused for this
  session's account; a default follows), *Remove* (only that account's labels; an account in
  use needs a replacement first). Provider actions add *Add another account* and moves back to
  `default`. Adding never replaces an existing account (`Vault::create_provider_account`).
- **/status** names the session's account (`Z.AI - <url> · account fake`) for providers that can
  hold accounts. **Spawn line** shows `(<model> <effort> · account <name>)` from a new
  `provider_account` field on the collab spawn item and its legacy end event.
- **S03 follow-up tests:** D3 after a mid-session raise to Aggressive; a `thread/spawnAgent`
  worker inherits the parent's account (app-server integration test through a mock provider);
  running-thread resume reports an account mismatch.

## Tests (final code tree, macOS, `just test`, isolated profile)

| Command | Result |
| --- | --- |
| `just test -p codex-core -E 'test(/provider_accounts\|multi_agents\|thread_manager\|schema\|config::\|turn_context\|rollout_reconstruction/)'` | 731 passed |
| `just test -p codex-app-server -E 'test(/thread_processor\|thread_resume\|thread_start\|thread_spawn\|schema\|provider_account\|turn_start\|thread_agent_message\|feedback/)'` | 215 passed |
| `just test -p codex-vault -p codex-provider-auth -p codex-protocol -p codex-app-server-protocol -p codex-exec -p codex-analytics` | 970 passed, 1 failed (experimental schema export, regenerated); rerun of `-p codex-app-server-protocol -p codex-protocol`: 589 passed |
| `just test -p codex-tui` | 4448 passed, 23 failed; all failures environmental, below |
| `just test -p codex-tui -E 'test(/onboarding\|provider_\|named_account\|multi_agents\|status::/)'` after the focus fix | 295 passed, 10 environmental failures |
| `just test -p codex-tui -p codex-core --features codex-core/developer-accounting,codex-tui/developer-accounting -E 'test(/provider_\|multi_agents\|onboarding\|status\|named_account\|accounting/)'` | see PR comment (run on the final tree) |

Environmental TUI failures, all outside this change and recorded by earlier lanes on this drive:
Unix socket paths longer than `SUN_LEN` under the drive's temp path (canary scan, env-alias,
`ide_context`, `wallet_menu`), the wallet daemon not starting in the test archive (6
`tmux_*wallet*` onboarding tests), a long-path wrap in a security-picker test, and
`default_command_popup_items_snapshot`, which does not match current main. One real
regression was found and fixed: `tmux_configure_many_preserves_first_default_restart_and_request`
(focus left the newly configured row); it now passes.

Clippy `-D warnings`: macOS clean on the touched crates; Linux on glitch
(`~/corbanu-rtx/pf84-s04`, `-j10`) clean with and without developer-accounting at `38cc946c1b`
and rerun at `7a1ab9bc9d` (PR comment).

## Live runs (tmux, debug build of `7a1ab9bc9d`, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`)

Z.AI on GLM 5.3 Flash, real default key from `provider/zai_api_key` (env only), text and Enter sent
separately:

1. Default account replies `pong`.
2. `/providers` → Z.AI → *Add another account* → `fake` → masked key (0 occurrences of the
   value on screen) → "Saved zai account `fake`"; rows `Z.AI · default` and `Z.AI · fake`.
3. Esc in the account actions is inert. *Use for this session* on `fake` → new session; `/status`
   shows `account fake`; the request gets a 401 with "account `fake` was rejected".
4. Claude Plan: *Add another account* → `work` → token (masked) → one row per account.
5. *Make default* on `fake`; rename `fake` while in use is refused; restart → `/status` shows
   `account fake` and the request 401s (durable). *Use default account for this session* →
   `pong`. Rename `fake` → `spare` carries the default; *Remove* `spare` asks for a replacement,
   then removes it and clears `[provider_accounts]`; `corbanu account list` shows only
   `claude-plan work`.
6. Second provider, Kimi (`kimi-code`, `k3`, real key from `provider/kimi_api_key`): default
   `pong`; added `fake`, used it for the session, `/status` `account fake`, 401.
7. Spawn line: `Spawned … (glm-5.3-flash max · account fake)`.
8. Onboarding (fresh home): Claude token saved → exactly one `Provider: Claude Account` row,
   chooser, *Add another account* saved `work`.

Canary scan: synthetic keys and the real Z.AI key absent from every file in the homes (outside
the encrypted vault). The homes were deleted after the run.

## Videos (SOP, leak-scanned: 0 literal, 0 pattern hits)

- [no duplicate rows (onboarding)](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-no-duplicate-rows-7a1ab9bc9ddb-2026-10-10.mp4)
- [add account (/providers)](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-add-account-7a1ab9bc9ddb-2026-10-10.mp4)
- [switch account (GLM 5.3 Flash, 401)](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-switch-account-7a1ab9bc9ddb-2026-10-10.mp4)

## Independent review (Opus 5.5 High, read-only `corbanu exec`)

Pass 1 (`5db40ab11f`): changes requested, 2 blocking. Fixed: adding could overwrite an existing
account (now refused under the vault lock); a failed session switch reported success and removal
went ahead (now restored and skipped). Non-blocking, fixed: one session selection could drop
another provider's; in-use checks without `/providers` open; partial rename/remove messages and a
default outranked by `-c` or a project layer; spawn account lost through legacy events; hard-coded
onboarding status; onboarding value not zeroized; `/status` account for OpenAI; PF tags in protocol
docs. Missing tests added (app-level selection, rename-with-default, higher-layer default).

Pass 2 (scoped check of the fixes, integrator-authorized extension, the one review over budget):
**approve**. Its two non-blocking notes are fixed in `7a1ab9bc9d` (accurate failed-switch message;
another provider's selection is kept unless it equals that provider's configured default).

## Known limits

- One session selection covers one provider at a time (refused with an explanation otherwise).
- *Use for this session* starts a new session; the running thread keeps its account.
- Onboarding adds API-key and Claude-token accounts; a Claude Code login directory is added in
  `/providers`. `corbanu account add` still replaces an existing account (S02 behaviour).
- Named ChatGPT sign-ins and AWS profile accounts stay refused; Travis's D4 moves them to PF-84-S06.
- Multi-agent v2 spawns (`SubAgentActivity`) do not show the account.
- Code-blind functional design/execution is the independent acceptance step, not this lane.
