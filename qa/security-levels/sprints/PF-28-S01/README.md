# PF-28-S01 central secret and protected-output gate: gate evidence (2026-10-06)

Branch `feat/pf28-s01-secret-gate-20261006`, macOS arm64 debug build, Rust 1.95.0. Feature flag:
`[features] secret_output_gate = true` (default off; off is today's behaviour). Synthetic canaries only. The demo
canary is passed as `--credential PF28_CANARY_API_KEY=file:<canary file>` (a 28-byte random value kept outside the
repository).

## What ships (flag on)

- **One registry** (`secret-broker/src/output_gate.rs`) of managed values with labels and classes, compiled into a
  longest-first literal matcher. Each value is matched raw, JSON-escaped, percent-encoded, as base64 (standard and
  URL alphabets, all three alignments, partial characters and padding removed with it) and as hex. Overlapping
  matches are merged into one marker, `[REDACTED:<label>]`. Values under 3 bytes are refused; values of 3 to 5
  bytes match only as whole words. Capacity is 4096 representations or 4 MiB. Registration is all-or-nothing and
  never evicts. Retiring a value takes effect only when its last owner and lease are gone. Seed phrases, private
  keys and financial values withhold the whole payload. Payloads over 16 MiB are withheld.
- **Streams** hold back only a tail that could still grow into a value, plus one boundary byte. Ordinary output
  streams through. Tails are released, gated, at the end of the command or item, at a reasoning section break, and
  at the end or abort of the turn.
- **What is registered.** At arm time: Core's secret-looking environment values with a credential shape, URL
  passwords and secret-named URL query values, and secret-named values in the sign-in files and `config.toml`.
  Ordinary settings are skipped (`GIT_AUTHOR_*`, `*_ADDR`, `*_PATH`, `*_URL`, `postgres:postgres`). Sign-in tokens
  are registered on every load and save (login, refresh, keyring); the previous value stays protected. Vault values
  are registered when this process reveals them; a value the gate cannot admit is not released.
- **Gated sinks.** Model requests (turns and compaction); recorded history (tool results and model output);
  rollout transcript; client events, both at `send_event` and again at delivery (`Codex::next_event`, which covers
  MCP startup and elicitation events sent on a cloned sender); tool-result, prompt and error telemetry; the TUI log
  file, the feedback ring, the log database and prompt history. Rollout traces and shell snapshots are off while
  armed. If policy pins snapshots on, the session does not start. Memory summaries that would carry a value are
  refused.
- Stream state is per turn: a sub-session's gated output forwarded into its parent's turn (review mode) is
  gated again with its own state, and neither turn's events release the other's tails.
- An event that cannot be rebuilt with markers is delivered with the affected fields set to `[WITHHELD]` (or
  emptied for byte fields), with ids kept, so approvals and completions are not lost.

## Tests

Final tree, after `just fix` and `just fmt`:

| Command | Result |
| --- | --- |
| `just test -p codex-secret-broker -p codex-vault -p codex-login -p codex-otel -p codex-core -E 'test(pf_28_s01)'` | 35 passed (secret-broker 20, core 13, vault 1, login 1) |
| `just test -p codex-secret-broker -p codex-vault -p codex-login -p codex-otel -p codex-feedback -p codex-state -p codex-message-history -p codex-features` | 792 passed |
| `just test -p codex-core -E 'test(pf_28) \| test(security::) \| test(session::) \| test(client::) \| test(exec::) \| test(config::) \| test(schema) \| test(compact)'` | 1422 of 1426. The 4 failures (`config_schema_matches_fixture`, two `skills_*` developer-message tests, `remote_compact_trim_estimate_uses_session_base_instructions`) fail the same way on `origin/main` `ce25d20b59` in a clean worktree |
| `just test -p codex-tui -E 'test(log) \| test(pf_28)'` | 104 of 105. `tmux_first_run_anthropic_account_selects_claude_login_after_success` times out because the long worktree path truncates the status line it waits for (`Corbanu Terminal · T…`); flag-off path, unrelated |

The named tests cover: exact values and every encoding; overlapping and repeated values; short values;
capacity exhaustion without eviction; more than 512 representations; rotation with leases, also concurrent;
per-owner retirement; chunk splits at every byte; and prefix-only hold-back. They also cover: a stream forwarded
from a sub-session and gated twice; a reasoning section break; an aborted turn; delivery of events sent on a cloned
sender; events that cannot be rebuilt; values in object keys and numbers; false-positive seeding; URL query secrets; sign-in login and two refreshes
across separate stores; seed-phrase windows; bare and re-cased hex keys; and keystore scaffolding.

## GLM 5.2 runs and videos (SOP, `qa/demos/index/PF-28-S01.md`)

TUI runs driven in tmux by `scripts/demo_video.py` with `-m glm-5.2`, provider `zai`, on the final implementation
commit `9261bd416f` (published). Unpublished runs at `05f1053d53`, `eb59a698d6` and `cf1a9ec012` gave the same
results. The final commit changes only how structured values with keys are gated, which these runs do not exercise.

| Demo | Flag | Outcome |
| --- | --- | --- |
| `pf28s01-tool-output-gated` | on | The agent prints the canary raw, base64, URL-encoded and in JSON. Every form shows as `[REDACTED:env:PF28_CANARY_API_KEY]`, in the tool cell and in the model's reply (the model never received it). |
| `pf28s01-split-output` | on | The canary is printed in two halves two seconds apart. Neither half appears; one marker results. |
| `pf28s01-persistence-clean` | on | After the run, `files holding the canary: 0` across the profile and log directories; the transcript holds 4 markers. |
| `pf28s01-baseline-flag-off` | off | Control: `files holding the canary: 2` (TUI log and rollout), markers 0. The recorder redacts the published cast. |

## Review

Independent Opus 5.5 High review in three rounds:
- `review-opus-1.md`: CHANGES REQUIRED (4 P1, 8 P2). Fixed in `cf1a9ec012`, except P1-1, which is disclosed and
  depends on PF-27-S02 (see Known limits).
- `review-opus-2.md`: CHANGES REQUIRED (1 P1, 2 P2, 12 P3). Fixed in `8cef6630bc`: F1 to F3 and P3-1, 2, 4, 5, 6, 7, 9
  and 12. The rest are recorded below.
- `review-opus-3.md`: CHANGES REQUIRED (R1: the stream finishers still ignored the turn, so a forwarded reasoning delta
  drained the child's held prefix). Fixed in `9261bd416f` with R2 to R4. The forwarding test now runs only
  through `gate_event_with`, with real end, reasoning and abort events. R5 to R7 are recorded below.
- `review-opus-4.md`: R1 to R4 verified, with no regressions. It found K1 (P2): a value held only in a JSON object
  key was passed raw. Fixed in the final commit: keys are gated, and a value still found after the walk is withheld.
  A re-check of that commit is `review-opus-5.md`.

## Known limits (not claimed)

- Values revealed outside Core's process (`corbanu vault auth-helper` run by an agent command) are not registered.
  `secretless_agent_launch` (PF-27-S02) keeps the vault store out of agent reach, and Core warns at start-up when
  this gate runs without it. Reflected secrets in responses are PF-28-S02.
- MCP OAuth tokens refreshed after start are not registered (only `.credentials.json` at arm time).
- Sign-in tokens are registered only while the gate is armed. If an app-server arms it only for a later thread,
  keyring-held tokens loaded before then are protected only from their next load or save; `auth.json` is seeded at
  arm time. Registrations are keyed by field, not by store.
- An approval request whose fields cannot hold a placeholder is replaced by an error notice rather than
  auto-denied. No realistic request reaches this: it would need a withheld-class or oversized match in a typed field
  such as `cwd`. The fallback to empty strings applies to the whole event, not per field.
- Secrets in URL paths (webhook paths) and query names other than secret-named ones, `key`, `sig` and `signature`
  (for example `X-Amz-Signature`) are not seeded. The `codex_otel.network_proxy` log target is not gated.
- Base64 or hex wrapped across lines (`base64` at 76 columns, `xxd -p`) is not found. Neither are seed phrases
  written comma-separated or numbered. A decode-and-rescan pass belongs to PF-28-S02.
- A rebuilt item can lose fields that do not round-trip through serde (`FunctionCallOutputPayload.success`).
- Arming is one-way and process-wide. Data written before arming (the feedback ring, earlier rollouts) is not
  rescrubbed. Each event is scanned up to three times; stream state uses one global mutex; registration recompiles
  the matcher under the state lock.
- The base64 partial character before a value can be emitted in an earlier chunk (at most 4 bits).
- The `pf_28_s01` tests in login, vault and core arm the process-wide gate, so they need nextest (`just test`).
- Memory summaries containing a managed value are refused as a batch (the endpoint is unused in production).
