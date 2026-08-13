# PfTerminal 0.1.29 release candidate

Date: 2026-08-12 UTC

Branch: `release/0.1.29`

Candidate commits:

- `a6109561e6` — audited 0.1.28 remediation base and evidence
- `d41a57722d` — Grok 4.6 and DeepSeek V4 Pro model integration
- `1cdaeeafc4` — 0.1.29 version and release preparation
- `997dcc719b` — release-validation and explicit provider-fallback corrections

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
| Blob-size policy regressions | PASS, 5/5; pre-existing oversized blobs pass only when they do not grow |
| Cargo workspace manifest policy | PASS |
| `cargo shear --deny-warnings` | PASS |
| `cargo deny check advisories bans licenses sources` | PASS |
| `just test -p codex-utils-cache -p codex-mcp` | PASS, 145/145 |
| `just test -p codex-utils-home-dir` | PASS, 5/5 |
| PFTerminal runtime-entrypoint home isolation regressions | PASS, 9/9 across all three CLI binaries |
| TUI/core dependency-boundary verifier | PASS; GPU runtime access is behind the app-server-client legacy seam |
| TUI GPU runtime catalog regression | PASS, 1/1 |
| `pnpm install --frozen-lockfile` with pinned pnpm 10.33.0 | PASS, including TypeScript SDK build |
| `just bazel-lock-check` | PASS |
| `bazel build //codex-rs/tasknode-session:tasknode-session //codex-rs/cli:codex` | PASS |
| `just fix -p` for core, provider info, models manager, and TUI | PASS |
| `just fmt` | PASS |

The first clean-worktree code-mode test attempt failed because
`codex-code-mode-host` had not yet been built. After building the host included
by the release workflow, the exact regression and its broader filter passed.
This was a test-environment prerequisite, not a product behavior failure.

The first pull-request CI pass also exposed repository-policy drift accumulated
across the recovery branch. The candidate repairs the affected boundaries:

- the blob-size gate now blocks new or growing oversized blobs while allowing an
  already-oversized file to be edited only when its size does not increase;
- Cargo feature exceptions follow the code-mode sandbox feature to its current
  crate, and the unused home-directory feature was removed;
- the Task Node session crate is represented in the Bazel graph and the module
  lock is current;
- the CLI derives its entrypoint from runtime argv rather than a Cargo-only
  compile-time variable, preserving state isolation in Bazel builds;
- the TUI no longer imports `codex-core` directly; its remaining embedded GPU
  startup uses are isolated behind the app-server-client legacy seam;
- stale unlinked quarantine test residue and unused dependencies were removed;
  and
- newly disclosed `webbrowser` and `lru` advisories were resolved by upgrading
  to fixed versions, with the direct-HTTP migration debt explicitly enumerated
  in the existing dependency-policy ratchet; and
- the workspace pnpm lock was regenerated to restore the complete frozen
  dependency graph used by repository and SDK CI; and
- CI no longer targets repository-scoped Windows runners or paid macOS runner
  labels that are not provisioned for this repository. Windows validation now
  uses `windows-2025` and macOS validation uses `macos-15`, so the checks remain
  runnable after the planned repository transfer and rename; and
- the GPU market library explicitly disables an empty doctest target, keeping
  the workspace manifest warning-free under the CI `cargo shear
  --deny-warnings` policy; and
- the README's agent-hierarchy notation now follows the repository's enforced
  ASCII-only policy; and
- the repository's own pinned Prettier formatter was applied to the 32 tracked
  Markdown files that had accumulated formatting drift, restoring the
  all-repository `pnpm run format` gate without changing their content.

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
| `claude-plan` + `claude-fable-5-plan` | PASS, exit 0, exact `PFTERMINAL_129_FABLE_FINAL_OK` in 4 seconds | `d9b85a67f5a0d32d6b704ef6f5dbac4770148e70fd2895ef0d14ba496df30bd4` |
| `vercel-anthropic` + `claude-fable-5` | PASS fail-closed behavior, exit 1, no model output and message confirms no request was sent | `d63c6809654b27d4ffb180402489e183c18600c9c5dc0ef8f0613e5c5e0093d3` |

The earlier Fable overloads were provider capacity, not silent fallback: the
candidate kept the authenticated `claude-plan` route on every attempt and the
later packaged-binary marker completed successfully. Raw live output is
intentionally uncommitted and remains under the recorded `/tmp` evidence paths
on this host.

## Workspace-wide regression run

The approved complete workspace run used isolated state under `/var/tmp`, two
Cargo build jobs, and two nextest threads. It ran 15,202 tests: 15,169 passed,
one flaky prompt-caching test passed on retry, 31 failed, two timed out, and 28
were skipped. The run exposed and led to fixes for:

- app-server provider/model fallback now remains explicitly opt-in while the
  documented fallback request selects the provider's own catalog default;
- OpenAI-specific stream fixtures no longer inherit PFTerminal's Ambient
  product default;
- provider-specific remote model tests assert the OpenAI default rather than
  the product-wide Ambient default;
- websocket retry tests now return a retryable handshake status instead of an
  implicit non-retryable 404; and
- external-provider websocket tests follow the current no-synthetic-`Continue`
  protocol contract.

Focused reruns for these corrections passed, including all four provider/model
fallback cases and seven remote-model/websocket regressions. The remaining full
run failures reproduce in untouched upstream-derived areas (multi-agent timing,
Guardian/network approval fixtures, skill prompt accounting, and vault timing)
and are not changes introduced by the 0.1.29 provider/model release diff. They
remain visible evidence rather than being hidden or converted to skips.

## Remaining release gates

This is not yet a publish recommendation:

1. The release CI matrix must pass against the final post-remediation candidate
   SHA; earlier non-publishing matrices cannot qualify a later SHA.
2. PowerShell installer tests were not run because `pwsh` is unavailable on this
   Linux host.
3. No tag, GitHub release, installer target, package pointer, or production
   installation should move until the remaining gates are green and publishing
   is explicitly authorized.
