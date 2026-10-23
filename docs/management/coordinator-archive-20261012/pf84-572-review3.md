You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI). Round 3.

Worktree /Volumes/CorbanuDrive/Corbanu/worktrees/fix-572-bedrock-named-model, branch fix/572-bedrock-named-model. Your round-2 review:
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/pf84-572-review2.result.md. Round-3 changes: `git -C <worktree> diff HEAD~1..HEAD`;
whole change `git diff origin/main...HEAD`.

For R1: the blocking job only computes Option<bool> (None = vault unreadable -> no change) and sends AppEvent::NamedBedrockAccountChecked
with a generation from ModelCatalog::begin_provider_models_check; App::named_bedrock_account_checked applies only the newest, then runs
sync_provider_manager_model_policy (sync_runtime_models with the live config + refresh_provider_policy on the UI thread), which puts the
current model back. For R2: core/src/model_info_for_provider.rs model_info_for_config used by new_turn, /review, the four spawn sites and
both guardian sites, with a unit test. Nits: policy refresh now on the UI thread; vault error leaves the list unchanged; N2 still open.
Re-check and look for regressions; rank blocking / should-fix / nit with file:line; verdict approve / approve with fixes / request changes.
