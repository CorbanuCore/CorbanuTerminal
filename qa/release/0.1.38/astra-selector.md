# GPT-6 Astra selector — September 5, 2026

## Scope and source

- Class: product initiative, explicitly requested by the user; feature PF-55 in
  the [unified provider-auth plan](../../../docs/plans/active/unified-provider-auth.md),
  sprint PF-55-S02.
- Product citation: **Shipping MVP — LIVE**, “OpenAI, Anthropic/Claude Plan, Kimi,
  Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama,
  LM Studio, Corbanu Plan, and custom providers.”
- Version: untagged 0.1.38. Implementation:
  `6b17a2630f31f5447d2c53fa8f6a29b60407b42a` on
  `integration/reconcile-release-0.1.38`.
- Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile`;
  reconciled baseline `90e29701f26704225f31cee03234dc05e65bc484`;
  allocation `2da9662db2`. Upstream baseline remains
  `ba6cf9c69277caec51a4c12c5b7401a9920930e0`; no upstream merge.

## Delivered behavior

`/model` offers **GPT-6 Astra** under OpenAI, using exact model ID `gpt-6-astra`
and the existing native Responses path. Low, Medium, High, Extra high and Max
are available; unsupported None, Minimal and Ultra are not offered. Medium is
Astra's picker default, not a change to the existing Sol application default.
Cancel preserves the previous selection. Confirmation persists the exact
model, OpenAI provider and effort through process restart.

Catalog metadata comes from the [official Astra model reference](https://developers.openai.com/api/docs/models/gpt-6-astra)
and [Astra API migration guidance](https://developers.openai.com/api/docs/guides/latest-model#gpt-6-astra-update-api-and-model-parameters),
fetched September 5: 1,050,000 context tokens, 922,000 maximum input tokens,
128,000 maximum output tokens, text/image input and text output. Auto-compaction
uses the documented input ceiling instead of the larger derived default.
No fast-service tier, pricing, unsupported sampling parameters or special
provider adapter was invented.

Automatic agent allocation remains disabled for Astra because account-usage
economics have not been configured. Explicit selection remains enabled. The
catalog merge preserves this policy and supported efforts, keeps the bundled
entry when remote catalogs omit it, and honors explicit remote hiding.
Actual access still depends on OpenAI's rollout and account permissions.

## Qualification

- `just fix -p codex-models-manager -p codex-tui --locked --offline`, `just fmt`,
  `just fmt-check` and `git diff --check` pass. Existing unrelated lint warnings
  remain; scoped fix made no additional production changes.
- Selected model-manager and TUI suite: **87/87 passed**, no retries, run
  `9806e4fa-3dcb-488f-b788-03500c51eef9`. This includes all 64 model-manager
  tests, 22 focused TUI unit/snapshot tests and one true-TMUX journey.
- After the final fix/format check, the same suite is rerun with ID
  `a6f3821f-aa09-49d0-a435-210c7c210586`: **87/87 passed**, no retries,
  33.264 seconds, including the true-TMUX journey in 32.025 seconds.
- New TMUX journey selects Astra, cancels effort selection and verifies the
  config is unchanged, selects High, submits one synthetic prompt, exits,
  restarts from the same temporary home, and submits again. Both loopback
  `/v1/responses` JSON bodies must contain `model=gpt-6-astra` and
  `reasoning.effort=high`, without temperature, top_p, logprobs,
  top_logprobs or the retired prompt_cache_retention parameter.
- New model/reasoning snapshots and the two existing changed picker snapshots
  were visually reviewed before acceptance. No pending `.snap.new` files remain.
- Manual Linux PTY run used actual `just codex`, a private home, synthetic auth,
  `RUST_LOG=trace` and an explicit temporary log directory. Keys exercised
  `/model`, Astra, More reasoning → Max, Escape/back, then Astra → High.
  Visible confirmation and saved configuration both showed Astra/OpenAI/high;
  `/exit` ended normally. This manual run submitted no inference prompt.

Reproduce from `codex-rs` with the debug application binary built and available
at `codex-rs/target/debug/codex` (the recorded run used a shared target symlink):

```sh
CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_TMUX_REQUIRED=1 just test \
  -p codex-wallet -p codex-wallet-daemon -p codex-tasknode-session \
  -p codex-model-provider-info -p codex-provider-auth -p codex-keyring-store \
  -p codex-models-manager -p codex-tui -p codex-cli \
  --locked --offline --retries 0 --test-threads 4 \
  -E 'package(codex-models-manager) | (package(codex-tui) & kind(lib) & (test(model_selection_popup) | test(model_picker) | test(astra) | test(model_reasoning))) | test(tmux_astra_selection_cancel_restart_and_request)'
```

## Evidence identities and limitations

- Linux automated-TMUX binary SHA-256:
  `8a0a5027c6b429c06828688263dcda1eb47a496520eab74bb9f93f1889b71136`.
- Manual `just codex` executable SHA-256, read from its running `/proc` image:
  `eef15e199b6b08bfd89f4ac6782423a92e28fb1bb332a387551ab7e90850375a`.
  The manual and test builds use different Cargo feature graphs, but the same
  production source. Shared build cache:
  `/tmp/corbanu-astra-review-phGuUE/codex-rs/target`.
- Manual artifacts: `/tmp/corbanu-astra-selector-manual-eO3wwZ/`;
  `astra-selected.txt` SHA-256
  `73c9d1a9028cd0222661402db302ece247c0a214523eb50f0f9451ac5176d6be`;
  `astra-max.txt` SHA-256
  `19a8d34d116acc806e65d28330df63b55ac365e7a9fe8d5645e3d62d171f0c4d`.
- Final command logs: `/tmp/corbanu-astra-selector-final-fix.log`,
  `/tmp/corbanu-astra-selector-final-fmt-check.log`, and
  `/tmp/corbanu-astra-selector-post-fix-tests.log`.
- Preserved final TMUX captures and binary metadata:
  `/tmp/corbanu-astra-selector-evidence-I87SvC/target/tmux-artifacts/`,
  scenarios `pf54-astra-reasoning-cancel`, `pf54-astra-selected-restart-false`
  and `pf54-astra-selected-restart-true`.
- Plan, sprint and portable-skill checks pass; PF-55-S02 is archived complete.
- Earlier runs were not qualification: expected snapshot updates, a missing
  fixture TOML key, and a wrong test endpoint environment variable were fixed.
  The wrong override sent synthetic credentials to OpenAI and returned 401;
  no real credential or paid model response was used. The corrected test uses
  explicit `openai_base_url` configuration and validates two loopback requests.
- Linux debug selection/routing proof only, not proof of real OpenAI account
  entitlement or live Astra inference. No real credentials were entered.
  The pre-existing debug key-event logging concern remains separately scoped.
- TensorCash and Isometric Game are not applicable to this feature's
  selection/persistence boundary; no coding-logic changes were made. No new
  live-repository, benchmark, cross-platform, or named-human acceptance run is
  claimed. Broader release gates remain as disclosed in the candidate record.
- [User documentation](../../../docs/features/model-providers.md#gpt-6-astra)
  describes the verified flow and limitations. This change creates no tag,
  release artifact, publication or default-model migration.
