I found four blocking problems. Two of them can silently run work on an account nobody chose, and one lets a model switch accounts under Aggressive without approval. I didn't build or run tests; everything below comes from reading `git diff origin/main -- codex-rs` and the code around it.

**Blocking**

1. **Children spawned with a role lose the parent's account.** (`core/src/agent/role.rs:174`, `reload_overrides` at `:383`)
   - **What happens:** both spawn handlers call `apply_spawn_agent_role` before `apply_spawn_agent_account`. For any role with a config file (the built-in explorer, nazgul, troll and orc), the role step reloads config from the config layers. Its `reload_overrides` doesn't carry the account, and the reload replaces `config.model_provider`. The one exception is the copy-back that only runs when accounting is on, and accounting is `Disabled` by default.
   - **Result:** if the parent's account came from `--account` or from resume, a Nazgul, Troll, Orc or explorer child quietly runs on the `[provider_accounts]` account or on `default`. It's also the wrong baseline for the D3 check.
   - **Fix:** after `build_next_config`, if `next_config.model_provider_id == config.model_provider_id`, copy `config.model_provider.account` into both `next_config.model_provider` and the `next_config.model_providers` entry. Add a test that spawns with `agent_type: "explorer"` from a parent on a named account.

2. **D3 can be bypassed because it checks the configured security level, not the level actually in force.** (`core/src/tools/handlers/multi_agents_account.rs:59`)
   - **What happens:** the check uses `turn.config.security_level`. But the session can be under a stricter policy level than its config says; `session/mod.rs:4380-4389` records the max of the two for exactly that reason, and `client.rs:1070` falls back to Aggressive when the policy can't be read.
   - **Result:** if the level was raised to Aggressive during the session, or the child inherited Aggressive from its parent while its config says Moderate, a model-chosen account switch goes through with no approval.
   - **Fix:** compare against `config.security_level.max(agent_control.effective_security_policy().snapshot_for_agent(thread_id)?.level)`, and treat a failed snapshot as Aggressive. Add a test where the config says Moderate and the policy says Aggressive.

3. **An `--account` name without a provider prefix gets applied to whatever provider the thread ends up on.** (`core/src/config/mod.rs:4193,4777`)
   - **What happens:** `provider_account_override` is stored as the user typed it (`work`) and forwarded as-is by `tui/src/app_server_session.rs:2108,2174` and `exec/src/lib.rs:1212,1245`. The app-server then matches it against the thread's provider, not the provider it was validated against.
   - **Result:**
     - A TUI native worker started with a provider override (`app_server_session.rs:826`) runs on `kimi:work` even though the user chose `zai:work`.
     - The preflight at `spawn_orchestration.rs:2374-2400` checks `self.config.model_providers["kimi"]`, which has kimi's configured account, not `work`. So it tests a different account from the one the worker uses, which is the exact requirement it was meant to meet.
     - Resuming a thread whose saved provider differs from the config default has the same problem.
   - **Fix:** in `Config::load`, after `split_account_selection`, store the qualified form `"{provider_id}:{name}"` in `provider_account_override`. Add a test for a TUI native spawn with a provider override.

4. **TUI native workers don't inherit the parent's account.** (`app-server/src/request_processors/thread_processor.rs:671`, `thread_spawn_agent`)
   - **What happens:** `thread/spawnAgent` goes through the normal thread-start path. The account comes only from the CLI override or from `[provider_accounts]`.
   - **Result:** a parent running on an account it got from resume, or from a spawn `account`, starts workers on the configured or default account instead.
   - **Fix:** when `thread.provider_account` is `None` and the child's provider matches the parent's, fill it from the parent thread's live account. Add a test.

**Non-blocking**

5. **Resume can pick up an old account.** `recorded_provider_account` (`thread_processor.rs:232`) skips newer turn contexts that have no account and returns an older one. Your own test expects `kimi:default` when the newest entry is `zai` with no account. Then `thread_processor.rs:3941` applies it even if the resume ends up on another provider. If that provider has been removed, or the feature is now off, the resume is refused with "unknown provider" or "needs feature", although that account isn't in use. Suggested fix: read only the newest turn context, apply the recorded account only when its provider matches the resolved session provider, and mention `--account default` in the feature-off error.
6. **Resuming a running thread ignores `--account`.** `collect_resume_override_mismatches` (`:63`) has no `provider_account` entry, so an explicit account on a running thread is dropped silently. Add a mismatch entry.
7. **`resume_agent` (v1) ignores the child's own account.** `build_agent_resume_config` (`multi_agents_common.rs:198`) uses the parent's provider and account, not the account the child recorded. This doesn't widen access, but it doesn't follow "resume uses the recorded account" for a child that was spawned with `account`.
8. **`--account` without a prefix on resume fails early.** `corbanu resume --account work` and `exec resume` check the name against the local default provider before the thread's saved provider is known. If the default is `openai`, they fail with "does not support named accounts". Either document that `<provider>:<name>` is needed or validate later.
9. **CLI errors use the config-table prefix.** Errors from `--account` start with `provider_accounts.<id>:`, which points users at the config table rather than the flag.
10. **Tests missing for important behaviour:**
    - A spawn with no `account` inheriting the parent's account.
    - An approved and a declined switch under Aggressive.
    - A switch under a non-Aggressive level going through without a prompt.
    - An end-to-end app-server resume where the recorded account beats `[provider_accounts]` and an explicit account beats the recorded one.
    - `Config::load` rejecting a named `--account` with the feature off.
    - Parsing `--account` in exec and the TUI.

I found no credential exposure. The model only ever sees account names, no environment variable carries a credential, and the new rollout field and app-server parameters are backward-compatible.

VERDICT: changes requested
1. Role reload drops the parent's account for role-based children.
2. D3 checks the configured level instead of the level in force, so it can be bypassed.
3. A prefix-less `--account` is forwarded as typed and applied to the wrong provider, and the TUI preflight tests a different account from the worker's.
4. TUI native workers (`thread/spawnAgent`) don't inherit the parent's account.

Non-blocking: items 5–10 above.