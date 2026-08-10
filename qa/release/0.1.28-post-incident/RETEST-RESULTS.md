# 0.1.28 quarantine re-test results

Date: 2026-08-09 UTC

Result: **HARD STOP — candidate is not releasable**

## Binary identity

| Artifact | Commit | Reported version | SHA-256 |
|---|---|---|---|
| Packaged baseline `pfterminal` | `827ad686965042a474fe4146471cf71b54ab440d` | `0.1.27` | `229e80f85be5a54d5ffcb2d41a079853969819926c865191f7fd19a4f52eadc3` |
| Locally built quarantine `pfterminal` | `4a5dce18d0401a603ef5532cdb7a1934ce43ff97` | `0.1.28` | `2e6cd66c8a53f11d9279331b9b535fca778b041139ecd603a211935ff27c2c4f` |

The quarantine candidate was built from a detached worktree with:

```text
cargo build --release -p codex-cli --bin pfterminal --bin pfterminal-debug --bin pfterminal-acp
```

Build result: exit 0 in 11m50s. This candidate was never installed, published, or pointed at Latest.

## #83/#86 provider-routing reproduction

Exact explicit input pair: provider `vercel-anthropic`, model `claude-fable-5`.

| Binary | Observed resolved pair | Exit | Elapsed | Observable result | Output SHA-256 |
|---|---|---:|---:|---|---|
| 0.1.27 baseline | `vercel-anthropic` + `zai/glm-5.2` | 0 | 4,941 ms | Exact marker `OK-BASE-VERCEL-ANTHROPIC` | `bd38d3b46fb389177b2441f4e14a1ca3cc67484a9a4baf6e5457d2dac8572265` |
| quarantine candidate | `anthropic` + `claude-fable-5` | 1 | 1,543 ms | Direct Anthropic rejected the request for low credit | `3a5bca5d4358395348a7ae33923589676e2ec8b7af80c149f67d7ceef4c6f6ed` |

This proves that the quarantine build silently changes an explicitly requested gateway, endpoint,
credential, model mapping, and billable route. The historical 5m00s timeout did **not** reproduce
under current external conditions: direct Anthropic now returns the low-balance error immediately.
The route mutation and resulting failure are observed, not inferred.

Two alternate baselines were also checked before selecting the working reproduction pair:

- `openrouter-anthropic` was preserved by 0.1.27 but the external key currently returns HTTP 403
  `Key limit exceeded`, so it cannot demonstrate a successful baseline today.
- `zai-anthropic` was preserved by 0.1.27 and resolved to `glm-5.2`, but 0.1.27 rejected that model
  locally because it lacked a catalogued maximum-output limit.

## Live provider matrix

Every row used the locally built candidate, an explicit current model/provider pair, a 120-second
outer timeout, and an exact-output marker. Raw responses were not committed; hashes are over the
captured combined output.

| Provider | Model | Exit | Elapsed | Result | Output SHA-256 |
|---|---|---:|---:|---|---|
| `anthropic` | `claude-fable-5` | 1 | 1,833 ms | **FAIL:** direct account credit is too low | `e05a388edc6678f2fc66f3f35ec41db08e3a0383a4a02d9d36c1034083fd2602` |
| `claude-plan` | `claude-fable-5-plan` | 0 | 3,470 ms | PASS: `OK-CLAUDE-PLAN` | `c04e78f7f45dce7cb46e67ef51c347cadaf20706aafc0b860a050df75bfb5b26` |
| `zai` | `glm-5.2` | 0 | 5,769 ms | PASS: `OK-ZAI` | `0d0df937ff81876ae6c3dc1b68311b383594e3db102ad464e62b1d076cb2f4e8` |
| `kimi-code` | `k3` | 0 | 7,795 ms | PASS: first turn `OK-KIMI-FIRST` | `fc65899b57a2dcdad02fec08e85b1bd574636f7603fdbfced700742b55c729de` |
| `kimi-code` resumed same session | `k3` | 0 | 9,353 ms | PASS: `OK-KIMI-RESUME` | `4104681354fd084ec9e8430e3a22dc0dbda71a5e062ca231685d557ec46f6000` |
| `openai` | `gpt-5.6-terra` | 0 | 3,313 ms | PASS: `OK-OPENAI`; local build warns packaged code-mode host is absent | `35f515e3ac2ba149936302773fb1c78b5a26bf94080a3cb4de0dcd36c7676689` |
| `ambient` | `z-ai/glm-5.2` | 0 | 4,514 ms | PASS: `OK-AMBIENT` | `0370c69e8c63e67a1723edac19150d95b449e283e4b7ebbae6b1f7db2d81f6db` |
| `openrouter` | `z-ai/glm-5.2` | 1 | 1,205 ms | **FAIL:** external key total limit exceeded | `6e6d1e628dd2f5c72c087444f024b84228992e75192a2280267b5f206aa4c8c4` |

The incident plan's model labels had drifted from the candidate catalog. The actual candidate uses
Kimi Code `k3`, not “Kimi k2.7”, and Ambient's default is `z-ai/glm-5.2`, not `ambient/large`.
The current catalog/config pairs were tested and the drift is recorded rather than silently using
non-current labels.

No release can pass while the required Anthropic and OpenRouter rows are red, regardless of whether
the cause is code, credentials, balance, or provider state.

## Vanilla configuration

`pfterminal` overwrites `CODEX_HOME` at entry. Therefore the plan's `CODEX_HOME=$(mktemp -d)` command
does not create a vanilla PFTerminal home. The corrected stable-entrypoint variable is
`PFTERMINAL_HOME`; debug uses `PFTERMINAL_DEBUG_HOME`.

A fresh `PFTERMINAL_HOME`, `--ignore-user-config`, provider credentials supplied only through
environment variables, and no model/provider override produced the actual defaults
`ambient` + `z-ai/glm-5.2` and returned `VANILLA-OK`:

| Exit | Elapsed | Output SHA-256 | Result |
|---:|---:|---|---|
| 0 | 3,056 ms | `cf8671d1d7e0f621df6bfd1bed5b8b000ed9ae69faa126ce83a71ef7f949ffb5` | PASS |

## Task Node

| Surface | Exit | Elapsed | Result | Output SHA-256 |
|---|---:|---:|---|---|
| Existing stable home `tasknode status --json` | 1 | 185 ms | **BLOCKED:** local state has a pending GitHub link, not an active linked session | `a96403783381d20220ddc50e6c4264be651fbbb7862bd1eaa316f7308a1dd55b` |
| Fresh `PFTERMINAL_HOME` status | 1 | 22 ms | PASS behavior: concise “not linked” instruction, no stack trace | `8fff3369dca02f9c9d5faa524d1d99f4d5283c5b82a4b7a172cdcfd021455672` |
| Linked chat round-trip | — | — | **NOT RUN:** there is no active linked session; finishing the pending user GitHub authorization is outside this retest | — |

The raw pending-link URL is deliberately not included in this committed artifact.

## ACP launcher

| Surface | Exit | Elapsed | Result | Output SHA-256 |
|---|---:|---:|---|---|
| `pfterminal-acp --version` | 0 | 17 ms | Reports launcher 0.1.28, resolved `pfterminal`, and that `codex-acp` is not installed | `7d07ec8307906d716b8c8eb609da7d27c74965dce6d1888ea6a05fbb067d6515` |
| Controlled handoff with `CODEX_ACP_PATH=/bin/true` | 0 | 18 ms | PASS: zero stdout bytes before handoff | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Telegram

Candidate debug-home `telegram --health` passed against the live Bot API in 927 ms and identified
one authorized chat/user and the configured workspace. Output SHA-256:
`6d2195336adcbf74e8fcc69d17a9011fa754fa3799f5a955421dac90cf7718d4`.

The two user-message staging interactions were not represented as passed: this host has the bot
credential but no Telegram user-session credential capable of sending `/status` to the bot as an
authorized user. Focused automated Telegram tests are recorded separately. A future release still
requires the actual staging-chat interaction.

## Installer and updater

| Command/check | Result | Evidence |
|---|---|---|
| `python3 scripts/install/test_pfterminal_release_contract.py` | PASS, 2 tests | exit 0; output SHA-256 `91c646601811acf203ebaa2e3139defe262a93b5c94ed2fa6f86c465734e7d56` |
| `python3 scripts/install/test_install_sh.py` | PASS, 17 tests | exit 0; output SHA-256 `e3d0d46a90a01a3ab7e992dfd81922624b250af303f776dd6e6bd110f3066c8e` |
| PowerShell installer tests | NOT RUN | `pwsh` is not installed on this Linux host |
| Cached target release was deleted | **KNOWN FAIL** | `get_upgrade_version` consumes the cached version immediately and only refreshes in a background task; no pre-prompt release revalidation exists |
| Installed release was deleted / recall downgrade | **KNOWN FAIL** | prompt selection is gated only by `is_newer(latest, installed)`; no installed-release existence check or recall path exists |

The updater requirements need an implementation and deterministic HTTP-mock tests before any release.

## Focused source tests

The repository instruction forbids substituting direct `cargo test`; focused checks use the
repository's `just test` recipe.

| Command | Result |
|---|---|
| `just test -p codex-model-provider-info` | PASS: 53/53 |
| `just test -p codex-tasknode-session` | **FAIL:** 7/10 passed; `legacy_pending_only_record_migrates`, `promotion_replaces_active_and_clears_pending`, and `clear_all_unlinks_everything` each timed out twice at 60 seconds |
| N=1 `legacy_pending_only_record_migrates` retry | **FAIL:** timed out twice at 60 seconds |
| `just test --release -p codex-tasknode-session` | PASS: 10/10 in 2.771 seconds after the optimized build completed |
| `just test -p codex-telegram` | PASS: 125/125 |
| `just test -p codex-core load_config_k3_with_explicit_incompatible_provider_repairs_pair` | PASS: 1/1, but this test codifies the rejected automatic provider-repair policy |
| `just test -p codex-cli --test pfterminal_acp` | PASS: 5/5 |

Tracing the N=1 test showed it acquired the temporary vault lock immediately and then spent seconds
inside the debug vault/encryption work; this is not contention with the user's live vault. Regardless
of cause, the ordinary required focused command is red and the Task Node commits are therefore
`readmit-after-fix`. The optimized Task Node suite passing shows the state-machine assertions work,
but it does not turn a timing-out normal-profile release gate green.

The provider/config regression test also needs replacement: its passing assertion expects the
candidate to rewrite an explicit incompatible pair. That is the exact policy this incident review
rejects. The replacement should assert a typed, immediate incompatibility error with the explicit
pair left intact.

## Gate disposition

The candidate is rejected because:

1. the explicit provider regression is reproduced;
2. two required provider rows are red;
3. the normal Task Node focused suite is red, and linked/chat plus Telegram user-message staging are
   not qualified;
4. both deleted-release updater behaviors are absent.

No merge, tag, prerelease, Latest change, install, or production deployment was performed.
