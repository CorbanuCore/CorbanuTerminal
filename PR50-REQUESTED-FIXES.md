# PR #50 — Requested Fixes Before Merge

PR: <https://github.com/agtico/PfTerminal/pull/50>

Reviewed commit: `03b97511409e89d4fad39f1723f85414e08734ac`

## Decision

**Do not merge PR #50 in its current state.** Compilation and focused unit tests pass, but the provider is not yet safe for ordinary vault-based authentication, plan-dependent context limits, or sustained agentic execution.

All four product findings and the CI failure below must be resolved on the PR branch. Fix the underlying boundaries; do not special-case the exact observed messages or tests.

## 1. P1 — Make `doctor` recognize provider credentials stored in the vault

### Current failure

Kimi works with `provider/kimi_api_key` stored in the encrypted provider vault, but `pfterminal doctor` checks only whether `KIMI_API_KEY` exists in the process environment. A normal vault-only installation therefore reports:

```text
status: fail
summary: active model provider auth env var is missing
provider auth env var: KIMI_API_KEY (missing)
```

Relevant code:

- `codex-rs/cli/src/doctor.rs`: `auth_check` and `provider_specific_auth_check`
- `codex-rs/login/src/auth/manager.rs`: `provider_api_key_from_auth_storage`
- `codex-rs/login/src/auth/provider_key_vault.rs`: encrypted-vault resolution

### Required repair

1. Make provider-specific doctor authentication use the same credential-resolution boundary as runtime authentication:
   - environment variable, when present;
   - encrypted provider vault;
   - supported legacy provider-key storage fallback.
2. Prefer a non-secret presence API. If the existing resolver must load the key, never log, serialize, or return its value from doctor.
3. Keep the lookup provider-specific. A Kimi key must not satisfy an OpenAI, Ambient, or unrelated provider check.
4. Ensure reachability planning uses the active provider's resolved authentication state rather than a global list of unrelated environment variables.

### Required regressions

- Vault-only Kimi credential: doctor auth check passes.
- Environment-only Kimi credential: doctor auth check passes.
- No Kimi credential: doctor fails with actionable `/providers` or `/vault` remediation.
- Kimi credential present while OpenAI is selected: Kimi must not satisfy OpenAI authentication.
- No test or failure output may contain the test credential value.

The test must exercise the real provider-key storage path in a temporary home. Testing only an `env_var_present` closure is insufficient.

## 2. P1 — Handle Kimi plan-dependent context entitlement safely

### Current failure

`models.json` unconditionally advertises a 1,048,576-token K3 context window. Kimi's official contract is plan-dependent:

- Moderato: K3 with up to 256K context.
- Allegretto and above: K3 with up to 1M context.
- Requests beyond the plan entitlement can return HTTP 401.

Official reference: <https://www.kimi.com/code/docs/en/kimi-code/models.html>

The current metadata can therefore let a Moderato session grow past 256K before PfTerminal compacts, after which an otherwise valid credential appears to fail authentication.

### Required repair

1. Do not promise 1M universally.
2. Use provider-returned entitlement/model metadata if Kimi exposes a stable, authenticated source for it.
3. If reliable discovery is unavailable, use a conservative 256K default and provide an explicit, documented way to select 1M for entitled accounts.
4. Ensure auto-compaction is derived from the effective context limit, not always from the catalog maximum.
5. Classify an entitlement-related 401 separately from an invalid credential and explain how the user can correct the context setting or plan.

### Required regressions

- Default/unknown entitlement compacts before 256K.
- Explicit 1M entitlement/configuration exposes the 1M limit and compacts relative to it.
- A context-entitlement 401 is not reported as a dead or invalid API key.
- Resume preserves the effective context limit.

## 3. P1 — Prevent unfinished agentic responses from ending the turn

### Observed failure

During live PR work, K3 returned HTTP 200 and emitted only:

```text
Adding doctor test coverage for the new env var:
```

It emitted no tool call. PfTerminal immediately recorded `task_complete` even though the requested implementation, tests, commit, and PR were unfinished. The active context was approximately 60K tokens in an approximately 996K effective window, so this was not context exhaustion.

Relevant code:

- `codex-rs/codex-api/src/endpoint/chat_completions.rs`: `ChatChoice.finish_reason` is parsed but discarded; completion always emits `end_turn: None`.
- `codex-rs/core/src/session/turn.rs`: a response with no tool-requested follow-up ends the turn.

### Required repair

1. Preserve the upstream Chat Completions `finish_reason` in the normalized completion event.
2. Handle at least these classes explicitly:
   - normal stop;
   - output/token length truncation;
   - content filtering;
   - tool-call completion;
   - unknown provider finish reasons.
3. Truncation and filtering must never be silently reported as successful task completion.
4. Add a bounded, provider-neutral completion guard for action-oriented turns. It must distinguish a genuine final answer from prospective progress narration when the requested work remains unfulfilled.
5. Use structured semantic classification or existing task/turn state. Do not make the exact sentence above, a regex for trailing colons, or Kimi-specific prose the primary fix.
6. Bound automatic continuation attempts and surface a clear diagnostic if the model repeatedly refuses to act. Do not create an infinite self-continuation loop.
7. Persist enough completion metadata to diagnose the decision from rollout and logs.

### Required regressions

- A text-only, clearly final informational answer completes normally.
- Multiple paraphrases of prospective progress narration do not falsely complete an unfinished action request.
- A `length` finish reason continues safely or fails explicitly; it never produces successful `task_complete`.
- A tool call continues the existing execution loop.
- Repeated incomplete responses hit the bounded guard and stop with one actionable error, without notification flooding.
- Resume/replay preserves the original completion decision without duplicating work.

### Required live acceptance

Run K3 through the real TUI on a temporary workspace with a multi-step coding objective that requires inspection, edits, tests, and a final evidence summary. The session must:

- continue across commentary/progress messages;
- execute the required tools;
- produce the requested artifact and passing test;
- emit `task_complete` only after the objective is actually complete;
- remain resumable for a second follow-up turn.

A two-turn `Reply with exactly OK` benchmark is transport coverage, not sufficient agentic acceptance.

## 4. P2 — Support K3 reasoning levels instead of forcing `max`

### Current failure

The PR states that K3 supports only one reasoning level, exposes only `max` in `models.json`, and overrides every Kimi request to `reasoning_effort: "max"` in `codex-rs/core/src/client.rs`.

Kimi's official contract supports:

- `low`
- `high`
- `max` (default)

Official reference: <https://www.kimi.com/code/docs/en/kimi-code/models.html>

### Required repair

1. Advertise `low`, `high`, and `max` in the model picker.
2. Keep `max` as the default, not a forced override.
3. Map PfTerminal effort values to Kimi's documented values:
   - low/minimum/light to low;
   - medium/high to high;
   - xhigh/max/ultra to max.
4. Reject unsupported values before sending the request, with a useful message.
5. Preserve the selected effort across resume.
6. Note that changing effort invalidates Kimi's prompt cache; do not switch it automatically within a session.

### Required regressions

- Request-shape tests for low, high, max, and default.
- Picker snapshot showing all supported levels.
- Unsupported effort fails locally rather than producing a remote HTTP 400.
- User-selected effort is not overwritten by provider defaults.

## 5. CI — Restore the mandatory real-TUI check

The PR's `real-tui` job currently fails before testing because the runner cannot find `just`:

```text
line 1: just: command not found
Process completed with exit code 127
```

Failed job: <https://github.com/agtico/PfTerminal/actions/runs/29587681799/job/87908612262>

Install the pinned/supported `just` version in the workflow or invoke the underlying checked-in command directly. The job must execute and pass; do not mark it optional or merely suppress the failure.

## Final merge gates

Before requesting re-review:

1. All findings above are implemented with generalized regression coverage.
2. The real-TUI CI job executes and passes.
3. Focused provider/model/core/TUI/doctor suites pass.
4. The live K3 agentic acceptance session passes from a fresh temporary home or workspace.
5. `git diff --check` passes.
6. Secret scan confirms that neither the real Kimi credential nor test credentials appear in source, commits, logs, snapshots, or uploaded artifacts.
7. The PR description is updated to state the effective context behavior, supported reasoning levels, live acceptance evidence, and any remaining limitations honestly.
