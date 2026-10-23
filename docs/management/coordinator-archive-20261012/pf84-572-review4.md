You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI). Round 4.

Worktree /Volumes/CorbanuDrive/Corbanu/worktrees/fix-572-bedrock-named-model, branch fix/572-bedrock-named-model. Your round-3 review:
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/pf84-572-review3.result.md. Changes since: `git -C <worktree> diff HEAD~1..HEAD`;
whole change `git diff origin/main...HEAD`.

R1: set_provider_models takes current_model and keeps that entry (with its Bedrock metadata, still recorded as added) when withdrawing;
App::named_bedrock_account_checked passes chat_widget.current_model() when config.model_provider_id is amazon-bedrock and no longer runs
sync_runtime_models from the stale config.model (only refresh_provider_policy). Test selects the model the way the picker does (provider in
config, model in the chat widget) and asserts the kept entry still has reasoning efforts; catalog test covers keep + later removal.
Nits: spawn metadata now looked up after the child's provider correction; with_model uses model_info_for_config.
Give a final verdict (approve / approve with fixes / request changes) with any remaining findings ranked.
