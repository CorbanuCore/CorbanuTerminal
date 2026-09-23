# Selector cleanup verification — 2026-09-22

Installed local debug candidate includes GPT-5.5 selector removal and new GPT-6 Luna entry. Claude Opus 5.5 Plan and other configured providers remain selectable. No public release, push, stable-binary change, or user-default change.

## Final package

`/mnt/HC_Volume_101713660/pfrpc/build-cache/corbanu-debug-installs/0f18b63404-opus55-plan-luna/bin/` contains `corbanu-debug` and `codex-code-mode-host`, built from the same worktree. Launcher retains the corrected shared normal profile and respects explicit CODEX_HOME. Binary hashes: `/home/pfrpc/corbanu-debug-evidence/selector-binary.sha256`.

`just fmt` and `git diff --check` passed. `just test -p codex-models-manager --lib -j24`: **68 passed, zero skipped**. New generalized regression covers every bundled-hidden model against remote and cached overlays, while retaining explicit-ID metadata. GPT-6 Sol/Luna added to existing missing-from-remote visibility coverage. Debug build passed in 2m19s; companion code-mode-host build passed in 35.62s. No full workspace test claim.

## Actual keys and outcomes

- Real existing shared profile, launcher with CODEX_HOME unset: `/model` shows Astra, GPT-6 Sol, GPT-6 Luna, retained GPT-5.6 tiers, and no GPT-5.5. Existing cache advertised GPT-5.5 as `list` and lacked Luna, proving the reported stale-cache starting state. Evidence: `selector-openai.txt`.
- Highlighted Luna then Escape, reopened picker, six Right keys to Claude Plan: exact Opus 5.5 Plan remains with all configured provider tabs. Evidence: `selector-claude-retained.txt`. No credential copying or eligibility edits.
- Isolated fake-key fixture solely for destructive-to-default persistence cases: saved GPT-5.5 remains accepted at startup but absent from picker. Highlight/cancel/restart retained GPT-5.5. Selected Luna, Medium, restarted same fixture without model override: persisted exact `gpt-6-luna`/`openai`. No real API call from fixture. Evidence: `selector-existing-gpt55-hidden.txt`, `selector-cancel-gpt55-restart.txt`, `luna-selected-fixture.txt`, `luna-restart-fixture.txt`.
- Search-box negative case is not applicable: this picker has no search input.
- User profile config, stable launcher/executable, and provider eligibility hashes unchanged after final probes: `selector-after-check.txt`.

## Live-account limitation (not a pass)

Exact live GPT-6 Luna and GPT-6 Sol requests through the user's existing ChatGPT account both received HTTP 400: model not supported when using Codex with a ChatGPT account. No alias substitution/fallback. Official rollout announcement is confirmed, but this account's inference access is **not** confirmed and was observed unavailable. Evidence: `luna-live-account-rejection.txt`, `sol-live-account-rejection.txt`. Separate API-key access not tested.

The first Luna probe additionally exposed an omitted `codex-code-mode-host` companion in the prior single-binary debug installation. Built and installed the same-tree companion; final Sol probe no longer showed the missing-host warning. No successful new-model tool loop is claimed because account access remains rejected.

Independent evidence review completed by the original blind-design agent; it confirmed the selector, persistence, cache, and profile-preservation evidence and retained the live-account limitations. This is a verified selector/UI update with explicitly failed live account availability, not a fully functional new-model release qualification. No live-repository or benchmark tests were applicable to this catalogue-only local handoff.
