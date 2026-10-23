You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI).

Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/fix-572-bedrock-named-model, branch fix/572-bedrock-named-model, one commit on
origin/main; review `git -C <worktree> diff origin/main...HEAD`. Issue: `gh issue view 572` (repo CorbanuCore/CorbanuTerminal).
Feature `named_accounts` (off by default). docs/provider-accounts.md (Bedrock section).

Problem: a session on Z.AI selects a named Bedrock account (/providers -> Amazon Bedrock · work -> Use for this session) but /model
offers no Bedrock model (sync_runtime_models resolves no model for amazon-bedrock unless the session runs on it), so the account is
unreachable. Fix: model-provider exports amazon_bedrock_model_presets() (static catalog -> ModelPreset); TUI ModelCatalog::
include_provider_models adds them (id "amazon-bedrock:<model>", never default, no duplicates) when the vault holds an AWS-profile
Bedrock account (provider_named_accounts::holds_named_account, labels only) and named_accounts is on: at startup (app.rs), in
sync_provider_manager_model_policy (opening /providers) and after an account add/remove finishes.
Verified in tmux with a mock Bedrock endpoint: picking "GPT-5.4 via amazon-bedrock" signs the request with the work profile. Known:
the session's models manager (Z.AI) has no Bedrock metadata, so a "Model metadata for openai.gpt-5.4 not found" fallback warning shows.

Check with file:line evidence, rank blocking / should-fix / nit:
1. Correctness: duplicates with the Bedrock session's own list, current-model marking, is_default, provider policy (StatusOnly ->
   Ready), picker grouping/tabs, model switch path (provider id resolution for "openai.gpt-5.4"), resume/restart.
2. Feature off / no account / account removed / default Bedrock configured: behaviour unchanged? Stale presets after removal.
3. Vault access cost and safety at startup (no secret read, no keyring prompt, NotInitialized vault).
4. The fallback-metadata warning: is it worse than before (context window, reasoning efforts), and is there a cheap fix?
5. Overlap with another worker's pending change to tui/src/model_catalog.rs (listed_ids/replace_listed_models for model/list refetch
   on account switch, commit 288fcd9740 in worktrees/fix-pf84-switch-follow): would its replace_listed_models drop these presets?
6. Tests (model_catalog_tests, app/provider_accounts_tests named_bedrock_account_offers_bedrock_models): do they fail without the fix?
Note: tui test a_session_account_that_cannot_load_changes_nothing fails on origin/main already (stale; another worker fixes it).
Finish with a verdict: approve / approve with fixes / request changes.
