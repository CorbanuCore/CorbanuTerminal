# 0.1.28 remediation results

Date: 2026-08-10 UTC

Branch: `fix/0.1.28-remediation-20260810`

Tested commit: `611cb0c9a2190b0e53e4bcfc658e3b2cf053670d`

This is a locally built remediation candidate based on the locked 0.1.27
baseline plus individually readmitted changes. It was not installed, tagged,
published, or promoted to Latest. The workspace version remains 0.1.27 until a
separate release decision.

## Remediations

- Bare Claude catalog selections resolve to the authenticated `claude-plan`
  provider. Explicit incompatible provider/model pairs fail locally and preserve
  the requested provider; no request is sent.
- Headless resume restores the saved model and provider together unless the user
  supplies an explicit CLI model/provider override. Kimi first-turn and resume
  now both remain on `kimi-code` + `k3`.
- Telegram pairs its scoped `default_model` with that model's canonical provider
  rather than inheriting an unrelated top-level provider.
- Task Node session tests use an in-memory store for state-machine assertions,
  leaving vault cryptography to the vault package. The normal-profile timeout is
  eliminated.
- The updater revalidates a cached target before prompting and supports recall
  downgrade prompts when the installed GitHub release has been deleted.
- Telegram reconciliation keys on a structured app-server error code instead of
  English error text.
- Task Node helper instructions resolve stable/debug binaries against the
  effective overridden state homes and fail closed on an unknown home.

## Candidate artifacts

| Artifact | Reported version | SHA-256 |
|---|---:|---|
| `pfterminal` | 0.1.27 | `2b237f839c0f15a6e5fe80c7ffc31cd9a5b1f3c12a500e6ba8a219a371377434` |
| `pfterminal-debug` | 0.1.27 | `6b99ae356b761b7d7bceb232daccb2f096edd63f8762dd8ba6a1ebb9dc55b37e` |
| `pfterminal-acp` | 0.1.27 | `3bd05461033f207a861171fc63fb0923f023c456cb9286369a65cb284998db82` |

Build command:

```text
cargo build --release -p codex-cli --bin pfterminal --bin pfterminal-debug --bin pfterminal-acp
```

Result: PASS, exit 0.

## Focused source gates

| Command | Result |
|---|---|
| `just fmt` | PASS |
| `just test -p codex-model-provider-info` | PASS, 53/53 |
| Three new `codex-core` config tests, individually | PASS, 3/3 |
| `just test -p codex-tasknode-session` | PASS, 10/10 in normal profile |
| `just test -p codex-exec` | PASS, 130/130 |
| `just test -p codex-telegram` | PASS, 125/125 |
| Structured unmaterialized-thread app-server integration test | PASS, 1/1 |
| Three updater recall/revalidation tests in release profile | PASS, 3/3 |
| `just test -p codex-cli --test pfterminal_acp` | PASS, 5/5 |
| `just test -p codex-skills` | PASS, 3/3 |
| `python3 scripts/install/test_pfterminal_release_contract.py` | PASS, 2/2 |
| `python3 scripts/install/test_install_sh.py` | PASS, 17/17 |

PowerShell installer tests were not run because `pwsh` is not installed on this
Linux host.

## Live surface gates

| Surface | Effective pair | Result |
|---|---|---|
| Claude subscription turn | `claude-plan` + `claude-fable-5-plan` | PASS, exit 0, exact `OK-CLAUDE-PLAN-COMMITTED` |
| Explicit incompatible pair | `vercel-anthropic` + `claude-fable-5` | PASS fail-closed behavior, exit 1 in 62 ms, message confirms no request was sent |
| Kimi first turn | `kimi-code` + `k3` | PASS, exit 0, exact marker |
| Kimi resumed session | `kimi-code` + `k3` | PASS, exit 0, same session and exact marker |
| Z.AI turn | `zai` + `glm-5.2` | PASS, exit 0, exact `OK-ZAI-COMMITTED` |
| Telegram health | scoped `k3` resolved to `kimi-code` | PASS, exit 0; bot identity, one chat/user, and workspace verified |
| ACP version/path resolution | candidate launcher | PASS, exit 0; launcher and candidate terminal path reported |

No direct Anthropic API call was made. The user confirmed that Claude production
traffic must use the logged-in Claude Plan subscription, so the rejected review
plan's direct-Anthropic spend row is replaced by the Claude Plan live turn plus
the local incompatible-pair fail-closed check.

## External state not represented as a code pass

- Direct `openai` + `gpt-5.6-terra` returned HTTP 401 because this debug home has
  no OpenAI bearer/basic credential.
- A fresh-home Ambient vanilla turn stopped locally because no
  `AMBIENT_API_KEY` was available to that new home.
- Task Node reports a pending GitHub link. Completing the account authorization
  is a user action; fresh/unlinked and state-machine behavior are covered by the
  focused tests.
- Telegram Bot API health passed, but an actual authorized user-message staging
  interaction still requires a Telegram user-session credential.
- OpenRouter was not retried: the prior audit recorded its key-limit failure and
  this remediation does not touch OpenRouter.

These external credential/account states are reported verbatim; none was
silently converted into a passing result.
