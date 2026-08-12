# PfTerminal 0.1.29 release candidate

Date: 2026-08-12 UTC

Branch: `release/0.1.29`

Candidate commits:

- `a6109561e6` — audited 0.1.28 remediation base and evidence
- `d41a57722d` — Grok 4.6 and DeepSeek V4 Pro model integration
- `1cdaeeafc4` — 0.1.29 version and release preparation

This candidate has not been installed, tagged, published, promoted to Latest,
or deployed to production.

## 0.1.28 quarantine disposition

The quarantined 0.1.28 candidate silently rewrote the explicit pair
`vercel-anthropic` + `claude-fable-5` to direct `anthropic` +
`claude-fable-5`. That changed the endpoint, credentials, billing route, and
failure behavior. Direct Anthropic then rejected the request for insufficient
credit.

The 0.1.29 candidate fixes the provider-selection boundary rather than
special-casing one command:

- bare Claude and Fable catalog selections resolve to the authenticated
  `claude-plan` route;
- an explicit incompatible model/provider pair remains explicit and fails
  locally before any provider request;
- resume restores the saved model and provider together unless the user supplies
  an explicit override;
- Kimi and Telegram use the provider canonically paired with the selected model;
- the updater revalidates deleted targets and supports recall downgrade prompts;
- Task Node state-machine tests avoid unrelated vault-crypto latency; and
- Telegram reconciliation uses a structured app-server error code rather than
  matching English error text.

The combined model change retains those invariants. A stale TUI test that still
expected bare `claude-opus-5` to select direct Anthropic was corrected to expect
`claude-plan`.

## New model routes

| Selection | Provider | Upstream model |
|---|---|---|
| Grok 4.6 | `openrouter` | `x-ai/grok-4.6` |
| DeepSeek V4 Pro | `deepseek` | `deepseek-v4-pro` |
| Fable 5 Plan | `claude-plan` | `claude-fable-5` |

The associated benchmark report is stored outside this repository at
`/home/pfrpc/repos/pfterminal-perf-probe/runs/grok46-vs-fable-20260812/REPORT.md`.

## Focused source gates

All Rust tests used the repository's `just test` recipe.

| Gate | Result |
|---|---|
| `just test -p codex-model-provider-info` | PASS, 53/53 |
| `just test -p codex-models-manager` | PASS, 60/60 |
| Explicit incompatible-pair core regression | PASS, 1/1 |
| Core integral-float exec regressions | PASS, 2/2 |
| Core `write_stdin_` regressions | PASS, 11/11 |
| Core code-mode parallel regression | PASS after building the packaged code-mode host |
| Focused TUI provider, picker, crew, and quick-start tests | PASS |
| `just test -p codex-tasknode-session` | PASS, 10/10 |
| `just test -p codex-exec` | PASS, 130/130 |
| `just test -p codex-telegram` | PASS, 125/125 |
| `just test -p codex-cli --test pfterminal_acp` | PASS, 5/5 |
| `just test -p codex-skills` | PASS, 3/3 |
| Structured unmaterialized-thread app-server regression | PASS, 1/1 |
| Updater deletion/revalidation tests in release profile | PASS, 3/3 |
| `python3 scripts/install/test_pfterminal_release_contract.py` | PASS, 2/2 |
| `python3 scripts/install/test_install_sh.py` | PASS, 17/17 |
| `just fix -p` for core, provider info, models manager, and TUI | PASS |
| `just fmt` | PASS |

The first clean-worktree code-mode test attempt failed because
`codex-code-mode-host` had not yet been built. After building the host included
by the release workflow, the exact regression and its broader filter passed.
This was a test-environment prerequisite, not a product behavior failure.

## Linux production artifact

The Linux x86_64 package was built with the release workflow's target, binary
set, profile overrides, and pinned rusty-v8 artifacts. Both rusty-v8 SHA-256
checks passed. The first build attempt was interrupted only by a transient
GitHub download disconnect; the workflow-equivalent retry passed in 7m19s.

| Artifact | Reported version | Size | SHA-256 |
|---|---:|---:|---|
| `/tmp/pfterminal-0.1.29-package.MOIQjL/pfterminal-package-x86_64-unknown-linux-gnu.tar.gz` | 0.1.29 | 248,230,064 bytes | `912730ac79c933471d25c5e761953149c4e568dc169d726d0722fca1c8dfc03b` |

A clean extraction passed these package checks:

- `pfterminal --version` reports `pfterminal 0.1.29`;
- `pfterminal-debug`, `pfterminal-walletd`, `codex-code-mode-host`, and `bwrap`
  are executable;
- the wallet daemon creates its socket;
- the packaged `rg` and Telegram resources are present; and
- no unintended Codex-named executable is exposed.

The extracted smoke tree is
`/tmp/pfterminal-0.1.29-smoke.UmTL3w`.

## Live packaged-binary route smokes

These were minimal exact-marker turns through the extracted release binary, not
benchmark reruns.

| Route | Result | Captured-output SHA-256 |
|---|---|---|
| `openrouter` + `x-ai/grok-4.6` | PASS, exit 0, exact `PFTERMINAL_129_GROK_OK` | `be2faca8cf2c714176743622deb677feaaa4fdeba50424726e7c563f4ad16914` |
| `deepseek` + `deepseek-v4-pro` | PASS, exit 0, exact `PFTERMINAL_129_DEEPSEEK_OK` | `e2442135f446b2ff7a6ee06a0efc488dc73c25701508d0369cb53a709b405a41` |
| `claude-plan` + `claude-fable-5-plan` | EXTERNAL RED: two attempts reached Claude Plan but Anthropic returned `overloaded_error` before completion | `1336671063fc06542963dee589f59ef307ae75af36b55795b8accb15b2e23c95`, `c22a2cf7efbfbfc7b8d99fee8a5a146d231b117e0bd893ca003126702b77923a` |
| `vercel-anthropic` + `claude-fable-5` | PASS fail-closed behavior, exit 1, no model output and message confirms no request was sent | `d63c6809654b27d4ffb180402489e183c18600c9c5dc0ef8f0613e5c5e0093d3` |

The Fable result is provider capacity, not silent fallback: the candidate kept
the authenticated `claude-plan` route on both attempts. The remediation evidence
also contains a prior successful exact-marker Claude Plan turn. Raw live output
is intentionally uncommitted and remains under
`/tmp/pfterminal-0.1.29-live.bQSorB` on this host.

## Remaining release gates

This is not yet a publish recommendation:

1. Repository policy requires explicit user approval before the complete
   workspace-wide `just test`; that approval is pending.
2. Linux ARM64, macOS, and Windows packaging still require the release CI matrix.
3. Claude Plan should receive one successful release-candidate marker when the
   upstream overload clears.
4. PowerShell installer tests were not run because `pwsh` is unavailable on this
   Linux host.
5. No tag, GitHub release, installer target, package pointer, or production
   installation should move until the remaining gates are green and publishing
   is explicitly authorized.
