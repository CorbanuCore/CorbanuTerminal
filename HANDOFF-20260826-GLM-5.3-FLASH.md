# Handoff — GLM-5.3-Flash, 2026-08-26

## Resume here

| Field | Value |
| --- | --- |
| Worktree | `/home/pfrpc/repos/CorbanuTerminal-glm53-flash` |
| Branch | `feat/glm-5-3-flash-vast-preset` |
| Base / upstream | `origin/main` at `1bdc515bff48a4d9048dae7d06c6214e884265bc` |
| Implementation head before this handoff | `0a90e5652b` |
| Branch state before this handoff | clean; 19 commits ahead of `origin/main`, 0 behind |
| Remote branch | none; the local branch tracks `origin/main` |
| Machine launcher | `/home/pfrpc/.local/bin/corbanu-debug` points to this worktree's debug binary |
| Tmux scrollback | global `history-limit` is `10000000` |

Read the root `AGENTS.md`, then use the repository
`corbanu-terminal-development` and `test-tui` skills for further product or TUI
work. Preserve unrelated dirty state in other worktrees.

## Do not conflate the two GLM paths

This branch contains two separate, completed paths for GLM-5.3-Flash:

| Path | Entry point | Runtime and credential | Current state |
| --- | --- | --- | --- |
| Self-hosted GPU | `/gpu` | Open weights on a rented Vast instance; Vault-backed Vast key plus a distinct generated per-rental endpoint token | Four-H200 and two-B300 vLLM presets implemented; B300 live-qualified and terminated |
| Hosted Coding Plan | `/model` in `corbanu-debug --yolo` | Z.AI-hosted API at `https://api.z.ai/api/coding/paas/v4`; existing Vault-backed `ZAI_API_KEY` | `glm-5.3-flash` catalog entry implemented and live-qualified |

Selecting the Coding Plan model does not rent GPUs. Conversely, the completed
Vast benchmark authorization is not authorization for another rental.

## User requests and authority already exercised

The user requested a preconfigured GLM-5.3-Flash `/gpu` setting, then approved
a live two-B300 benchmark with maximums of $16/hour, $125 total, and 480
minutes. That one rental completed at an estimated $28.3518 and was terminated
with provider confirmation. Treat those limits as exhausted authorization for
that completed run, not as standing permission to create another billable
resource.

The latest request was to look up GLM-5.3-Flash and enable it in
`corbanu-debug --yolo` under the existing Z.AI GLM Coding Plan. That hosted-plan
work is complete and committed.

## Committed GPU preset and benchmark work

The branch's first 17 commits, `5ba70072b1` through `b5ce6ed96d`, implement and
qualify the `/gpu` work. The durable sources are:

- `docs/plans/active/glm-5-3-flash-vast-preset.md` — active initiative record;
- `docs/sprints/archive/glm-5-3-flash-vast-preset/pf-27-s01-curated-recipe-and-qualified-endpoint.md`
  — completed execution sprint;
- `qa/gpu-rentals/sprints/PF-27-S01/evidence.md` — full implementation, live
  endpoint, spend, credential-rotation, and cleanup evidence;
- `qa/gpu-rentals/benchmarks/glm53-b300/README.md` — reproducible workload;
- `qa/gpu-rentals/benchmarks/glm53-b300/results/20260826-vast-48809614/` —
  secret-free 4–256 stream results.

Key accepted deployment facts:

- H200 preset: `glm-5.3-flash-4xh200`, four connected H200s, TP4, 65,536
  context, BF16 KV cache, maximum four sequences.
- B300 preset: `glm-5.3-flash-fp8-2xb300`, two connected B300s, TP2, 131,072
  context, FP8 KV cache, maximum 256 sequences.
- Both pin model revision
  `3f1971b7b5f7a528c9c4ef6212c8785298a8c24a` and image digest
  `sha256:2c6da6c6f16ed15c91e412d896dba13701f25fe1861eaec9ddaa4db34d1d21c4`.
- vLLM was selected because its model-specific recipe was current. The launch
  is text-only even though the checkpoint contains multimodal components.
- The B300 sweep completed 1,016/1,016 requests with exactly 6,000 mean
  requested output tokens and zero failures. At 256 streams it measured
  2,662.88 aggregate output tok/s and 10.40 tok/s per stream. This is a stress
  ceiling, not a recommended no-headroom production setting.
- Vast resource `48809614`, endpoint transport, and rental token were cleaned
  up. Unrelated resources `48790553` and `48790554` were intentionally untouched.

PF-27-S01 is complete and archived. The plan remains active only because
product-owner evidence review and later release linkage are still pending.

## Committed hosted Z.AI Coding Plan work

### `5da7926796` — `feat(models): enable GLM 5.3 Flash on Z.AI plan`

Implemented:

- model code `glm-5.3-flash` and direct provider `zai`;
- 1,000,000-token catalog context, 128,000 output-token bound, text and image
  inputs, required preserved thinking, and `low`/`high`/`max` efforts;
- `max` as the documented recommended default;
- three-times-GLM-5.3 quota represented by relative plan burn of 334 off-peak
  and 1,000 weekday peak, conservatively rounding one-third upward;
- generalized `glm-5.3` family routing tests, including correction away from an
  incompatible Ambient pairing;
- Z.AI picker snapshot and selected provider/model event assertion;
- provider, installation, configuration, and integration documentation.

Primary files:

- `codex-rs/model-provider-info/src/lib.rs`
- `codex-rs/model-provider-info/src/model_provider_info_tests.rs`
- `codex-rs/models-manager/models.json`
- `codex-rs/models-manager/src/manager_tests.rs`
- `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__model_selection_popup_zai_glm_5_3_flash.snap`
- `docs/integrations/zai-glm-52.md` — content is now plan-wide; the legacy path
  was retained to avoid breaking links.

The implementation was based on the official Z.AI sources:

- <https://docs.z.ai/guides/vlm/glm-5.3-flash>
- <https://docs.z.ai/devpack/overview>
- <https://docs.z.ai/guides/overview/pricing>
- <https://huggingface.co/zai-org/GLM-5.3-Flash>

### `0a90e5652b` — `docs(qa): record GLM 5.3 Flash qualification`

Durable evidence is in `qa/model-providers/glm-5-3-flash/RESULTS.md`.

Final automated results recorded there:

- `codex-model-provider-info`: 57 passed;
- `codex-models-manager`: 61 passed;
- focused GLM-5.3-Flash picker test: 1 passed;
- `cargo build -p codex-cli --bin corbanu-debug`: passed;
- plan and sprint governance checkers: passed;
- `git diff --check`: passed.

MkDocs was not installed on this host, so the documentation site did not get a
local `mkdocs build --strict` run.

## Live hosted-plan proof

The built candidate was launched in a private tmux server at 120x40 with a true
PTY and real keys:

```text
RUST_LOG=trace corbanu-debug --yolo --no-alt-screen -c log_dir=<temporary-directory>
```

Input text and Enter were sent as separate events. `/model` was opened, the
Z.AI tab selected, then `Z.AI GLM 5.3 Flash` and advanced `max` reasoning were
confirmed. Visible checkpoints were:

```text
Model changed to Z.AI GLM 5.3 Flash via Z.AI max
GLM53_FLASH_OK
```

`/status` showed the Z.AI Coding Plan endpoint and 950K usable context after
the application's safety margin. The tiny max-reasoning request reported 2.9
output tok/s; this was only a connectivity and routing proof. `/exit` shut the
session down cleanly. The temporary trace directory was moved to trash and no
raw log, account identifier, session identifier, provider credential, or
authorization header was committed.

The machine-local launcher currently contains:

```sh
#!/bin/sh
export CODEX_HOME='/home/pfrpc/.corbanu'
exec '/home/pfrpc/repos/CorbanuTerminal-glm53-flash/codex-rs/target/debug/corbanu-debug' "$@"
```

Startup still defaults to the user's normal OpenAI model. Use `/model` to choose
Z.AI GLM 5.3 Flash; the change adds and qualifies the choice, it does not force
it as the global default. Provider authentication is managed separately through
`/providers` and the encrypted vault. Never print or commit the Z.AI key.

## Current machine and repository cautions

- `/home/pfrpc/repos/CorbanuTerminal` is a different main worktree and had an
  unrelated untracked `codex-rs/core/non-existent/` directory. Do not remove or
  absorb it.
- `/home/pfrpc/repos/CorbanuPlan` had unrelated untracked documentation. Do not
  modify it as part of this branch.
- The general tmux harness guide is commit `1dabd93f62` on
  `feat/pf-13-s02-scoped-vault-resolver`, not on this branch. It can be read with
  `git show 1dabd93f62:docs/tmuxHarness.md`; do not cherry-pick it here without
  an explicit integration decision.
- Host tmux already has `history-limit 10000000`; no further change is needed.
- There is no active GLM Vast rental from this work. Perform read-only provider
  checks before any future cleanup, and require fresh financial authorization
  before creating another rental.

## Remaining work and safest next actions

1. If the user asks to publish this branch, push explicitly to a feature ref,
   for example `origin feat/glm-5-3-flash-vast-preset`, and set the upstream;
   do not assume that tracking `origin/main` means these commits are already on
   the remote.
2. Obtain product-owner review of PF-27 evidence if closing the active plan.
   Human acceptance and release linkage are the documented remaining gates.
3. Run `mkdocs build --strict` in an environment with MkDocs before release.
4. For any further model change, keep the two boundaries separate: hosted Z.AI
   catalog/routing belongs to `/model`; rented open-weights lifecycle belongs to
   `/gpu`.
5. Before handing off again, run:

   ```text
   python3 docs/plans/check.py
   python3 docs/sprints/check.py
   git diff --check
   git status --short --branch
   ```

At the time this document was written there was no implementation blocker and
no known uncommitted change in this worktree.
