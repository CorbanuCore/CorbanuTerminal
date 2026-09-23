# Local verification — 2026-09-22

Outcome: local debug candidate installed; exact Opus 5.5 Claude Plan selection, response, restart persistence, and shell tool round trip passed. Not a public release or full release qualification.

## Artifact and installation

- Base main: `0f18b63404`; local additions on `fix/opus-5-5-claude-plan-20260922`.
- Binary: `/mnt/HC_Volume_101713660/pfrpc/build-cache/corbanu-debug-installs/0f18b63404-opus55-plan/bin/corbanu-debug`.
- SHA-256: `44cca34ccfa81581c9b18c31fa75922a65e0113771ac17fe26b4d6f886f3c696`.
- Launcher updated: `/home/pfrpc/.local/bin/corbanu-debug`. Previous launcher backed up outside the repository.
- Version remains main's `0.1.42`; identify this candidate by source and binary hash, not the inherited version alone.
- Stable launcher, stable binary, stable config, and original debug config match their pre-test hashes. Test profile was isolated; no user default model was changed.

## Checks

- `just fmt` and `git diff --check`: passed.
- `just test -p codex-model-provider-info -p codex-models-manager --lib -j24`: 132 passed, 0 skipped.
- Targeted core tests `opus_5_5_plan_request_preserves_exact_model_and_subscription_identity` and `claude_plan_versions_use_exact_upstream_slugs`: 2 passed; 2,416 other tests filtered out.
- `cargo build --locked --manifest-path codex-rs/Cargo.toml -p codex-cli --bin corbanu-debug -j24`: passed.
- Actual TUI launched with `--yolo` and a dedicated QA profile. Ambiguous Claude Account credentials initially blocked model use; explicitly selecting the existing Claude Code login enabled the provider.
- `/model` displayed and selected **Claude Opus 5.5 Plan** with High effort; live reply was `CORBANU_OPUS55_PLAN_OK`.
- Persisted selection: provider `claude-plan`, model `claude-opus-5-5-plan`, effort `high`. Restart without model override retained selection and returned `OPUS55_RESTART_OK`.
- Captured Anthropic request bodies used exactly `claude-opus-5-5`, adaptive thinking, and high output effort.
- Live shell round trip ran `printf OPUS55_TOOL_OK`, returned that output, and completed the model response. Follow-up request retained the exact upstream model and included the tool result.
- Local Messages error fixture recorded one exact-model request, exit 1, and unchanged model/provider selection. No downgrade observed. Fixture used a custom provider and fake credential; it does not simulate an actual subscription authentication outage.
- Exact-byte scan of evidence text/JSON/log files against native access/refresh credentials found no matches. This is a limited leak check, not a general credential-storage security audit.

## Evidence and review

Evidence directory: `/home/pfrpc/corbanu-debug-evidence` (symlink to mounted-volume storage). Key files: `model-tests.log`, `core-tests.log`, `installed-binary.sha256`, `tui-plan-picker.txt`, `tui-plan-success.txt`, `tui-restart.txt`, `tui-tool-loop.txt`, `plan-request.json`, `plan-request-restart.json`, `rejection-result.json`, `stable-before.sha256`, `stable-after-check.txt`.

Independent code-blind test design was frozen before final testing in `TEST_DESIGN.md`. Independent evidence review supported the local Plan handoff and identified the limitations below. Review is not release qualification.

## Deliberately unverified

No live GPT-6 Sol or Anthropic API Opus 5.5 calls; no configured API/Plan switching test; no real expired-subscription test; no post-test stable launch/auth exercise. The GPT-6 Sol and API Opus 5.5 catalogue updates from main are included in the compiled candidate, but live availability on those routes is not claimed. Stable file integrity, not runtime stable-app behavior, was checked.

No public release, push, default-model change, or benchmark run was performed.
