# PF-28-S01 security review 1 (Opus 5.5 High, independent)

Scope: `git diff origin/main...HEAD -- codex-rs` on `feat/pf28-s01-secret-gate-20261006`. There are no uncommitted changes.
Ran `cargo test -p codex-secret-broker pf_28_s01`: 17/17 pass. I did not build or run the codex-core or codex-vault tests.

## Verdict: CHANGES REQUIRED

The flag-off path does not change behaviour. The secret-broker matcher is carefully built: escaping, longest-first union, all-or-nothing registration, no eviction and fail-closed compile are all correct. The blockers are in coverage and seeding:

- the canonical credential flow (`vault auth-helper` in a child process) is never registered;
- Core's own credentials are captured only once;
- several event and trace routes are not gated;
- the seeding heuristics redact ordinary text in model input, which will corrupt normal agent work.

## P1

**P1-1: Values revealed outside Core's process are never registered, so canaries can reach the model.**
Locations: `vault/src/lib.rs:737` (`protect_revealed`), `cli/src/main.rs:2473` (`run_vault_auth_helper`).

- `protect_revealed` registers into the gate of the process that reveals the value.
- The documented operational flow is `CREDENTIAL="$(corbanu vault auth-helper <label>)" cmd`. In that flow the reveal happens in a child CLI process, so Core's registry never learns the value.
- If the consuming command echoes it (for example `curl -v`, a failing client that prints its config, or `env`), the raw value reaches FunctionCallOutput, the model request, rollout and events, all ungated.
- The same applies to values resolved inside the separate `secret-broker` process (`resolver.rs`).

Suggested fix: when armed, register every vault label that permits programmatic use, either at arm time or when Core unlocks the vault. Alternatively, have the helper or broker push the registration to Core over the existing broker IPC before releasing the value. Add a canary test where the reveal happens out of process.

**P1-2: Core's own credentials are captured once at Config load; later logins, refreshes and keyring-stored tokens are unprotected.**
Location: `core/src/security/disclosure_gate.rs:64-92`.

- `SEEDED` is a `OnceLock` that reads `auth.json`, `provider_auth.json` and `.credentials.json` a single time.
- Not covered:
  - a first-run login after startup;
  - ChatGPT and provider OAuth refreshes (new access tokens and rotated refresh tokens);
  - MCP OAuth refreshes;
  - any credentials stored in the OS keyring (`cli_auth_credentials_store` / `mcp_oauth_credentials_store = "keyring"`), which never touch these files.
- `OutputGate::rotate`, `lease` and `retire` have no production callers. The claim that "rotation registers the new value first" is true only of the API, not of any real rotation.

Suggested fix: register in AuthManager and the token stores at every load, save and refresh, using `rotate` and holding a lease for the duration of the request. Test login-after-arm and refresh.

**P1-3: Seeding heuristics redact ordinary text from the model request sink.**
Locations: `core/src/security/disclosure_gate.rs:433-490`, `protocol/src/secretless_launch.rs:26`.

- **URL passwords from any env var are admitted at 3 bytes, with no plausibility check.**
  - `DATABASE_URL=postgres://postgres:postgres@localhost/app` is a common local-dev setting. It makes `postgres` a substring match everywhere: source, tool output and the prompt.
  - Passwords such as `root`, `test` or `dev` become whole-word matches, so `/root` or a word like `test` gets redacted.
- **The name heuristic was built for *dropping* variables, where over-matching is safe. It is reused here for *redacting*, where it is not.** `AUTH` matches `GIT_AUTHOR_NAME` and `GIT_AUTHOR_EMAIL`, so the user's name and email are redacted from every git log, diff and blame. `KEY` matches names like `*KEYBOARD*` or `SSH_KEY_PATH=~/...`. `VAULT` matches `VAULT_ADDR=https://...`.
- Because the ModelRequest and ToolResult sinks are affected, the model sees `[REDACTED:...]` in code it is editing. The likely results are `apply_patch` context mismatches, or the marker being written into files.

Suggested fix:
- give URL passwords the same ≥8-byte and plausibility rule as other values;
- use a credential-specific allow/deny list for seeding (exclude `*AUTHOR*`, `*_ADDR`, `*_PATH`, `*_DIR`, `*_FILE`, `*_URL` except for the URL password);
- skip dictionary words or values with low character diversity;
- add tests for these named false positives.

**P1-4: Some client-event routes bypass `send_event`.**
Locations: `codex-mcp/src/connection_manager.rs:443-570`, `codex-mcp/src/elicitation.rs:325-328`, `core/src/environment_selection.rs:229`.

- MCP startup updates, failures and summaries, and MCP `ElicitationRequest` messages, are sent on a cloned `tx_event` (`session/mcp_runtime.rs:161`). They are delivered without passing the gate.
- MCP startup errors commonly include URLs, headers or server stderr. Elicitation text is content controlled by the server.

Suggested fix: gate at a single choke point. Wrap the `Sender<Event>` handed out by `get_tx_event()` (or gate in the consumer that receives from it) so every producer is covered. Add a test that sends an event through `get_tx_event()`.

## P2

**P2-1: Model requests outside `stream_with_same_turn_attempt` are not gated.**
Locations: `core/src/client.rs:1275` (`compact_conversation_history`, used by `compact_remote_request.rs:81`) and `client.rs:1508` (`summarize_memories`).

The legacy remote compaction sends the full `prompt.input` without gating. Items recorded before a value was registered (or resumed from old rollouts) therefore still reach the model. `gate_prompt` already exists to cover exactly this case. Apply it on these paths too.

**P2-2: Trace and telemetry sinks are ungated.**
Locations: `rollout-trace/src/inference.rs:370` (raw model output items), `rollout-trace/src/tool_dispatch.rs:169` (raw tool responses), `rollout-trace/src/code_cell.rs:105,120`, `otel/src/events/session_telemetry.rs:1123-1134` (`codex.tool_result` logs `output` and `arguments`).

- Only the rollout trace's *event* recording receives gated input.
- The rollout trace is enabled by `CODEX_ROLLOUT_TRACE_ROOT`. When it is set, or when an OTEL exporter is configured, raw tool output and model output are exported.
- The module doc at `disclosure_gate.rs:5-6` claims that "rollout traces" are gated.

Suggested fix: gate these writers with `OutputSink::Trace`, or disable them while armed.

**P2-3: Common wrapped encodings are missed.**
Location: `secret-broker/src/output_gate.rs:812-857`.

- GNU `base64` wraps at 76 columns and `openssl base64` at 64. `xxd -p` wraps at 60 hex characters.
- For values longer than about 45–57 bytes (base64) or 30 bytes (hex), no full representation appears on a single line, so `echo $TOKEN | base64` leaks.
- For 6-byte values, only alignment 0 of base64 is generated: the offset-1 and offset-2 cores are 7 characters, below `MIN_ENCODED_BYTES = 8`.

Suggested fix: add a bounded decode-and-rescan pass. Find base64 and hex runs, strip whitespace and newlines, decode, and match against the raw representations. Alternatively, document wrapped output as an explicit non-goal, with a test that pins that behaviour.

**P2-4: Withheld-class representations are both too narrow and too broad.**
Location: `vault/src/lib.rs:749-776`.

- **Seed phrases.** Only the full phrase, joined by spaces or newlines, is registered. Comma-separated or numbered forms, or any subset of the words (for example the first 6 of 12), are not withheld. That contradicts "never shown in part". Suggested fix: register each window of k consecutive words (for example k=3) with flexible separators, or normalise whitespace and punctuation before matching.
- **Private keys.** A hex key stored as `0x…` is not matched when printed without the prefix or in a different case.
- **Every ≥8-byte line of a multi-line value becomes a PrivateKey-class trigger.**
  - For `KeystoreJson`, this registers ordinary JSON lines (`"version": 3,`, `"kdf": "scrypt",`, `"cipher": "aes-128-ctr",`).
  - For a PEM key, it registers the `-----BEGIN … PRIVATE KEY-----` and `-----END …-----` lines.
  - Any later payload containing those lines, such as docs, fixtures or `cat` of an unrelated keystore, is withheld whole. Streams stay withheld for the rest of the stream.
  - Suggested fix: only register lines with high entropy or base64/hex bodies; never headers or JSON scaffolding.

**P2-5: Stream hold-back is sized by the largest representation, which effectively turns streaming off for ChatGPT sign-in.**
Location: `output_gate.rs:588`.

- `hold = max_len + 4`, where `max_len` includes the 2× hex form and the 3× percent form.
- With ChatGPT sign-in, the access and id JWTs (1.5–2.5 KB) give a hold of roughly 3–5 KB.
- Most agent messages and command outputs would then appear only at `ItemCompleted` or `ExecCommandEnd`.
- Interactive unified-exec prompts (for example a `Password:` prompt) never render until much more output arrives.

Suggested fix: hold back only the longest buffer suffix that is a proper prefix of some representation. For base64 cores, also hold the one preceding partial character. That is O(max_len) to compute but usually holds back about 0 bytes.

**P2-6: The tail of an earlier reasoning summary is released after the next section has started.**
Location: `disclosure_gate.rs:292-298, 378-431`.

Each `summary_index` has its own stream, finished only at `ItemCompleted`. The sequence is: the tail of summary 0 is held, then `AgentReasoningSectionBreak` and summary 1 deltas are sent, and only then is summary 0's tail released. The TUI appends it to the wrong section.

Suggested fix: finish earlier streams of the same item on `AgentReasoningSectionBreak` or when a new `summary_index` first appears.

**P2-7: Turns that end without `ItemCompleted` or `ExecCommandEnd` (abort, error, unified-exec session killed) never release their tails.**
Location: `disclosure_gate.rs:308-315`.

- On those turns, the end of the visible message or output is silently lost.
- The 512-stream eviction at `:351-360` also drops a scrubber whose `withheld` flag is set. Later deltas with the same key then start a fresh stream and pass, which breaks "rest of stream withheld".

Suggested fix: also finish streams on `TurnAborted`, `TurnComplete` and `Error` for that turn. Keep a withheld-key tombstone rather than evicting it.

**P2-8: Shell snapshots fail open.**
Location: `core/src/config/mod.rs:3666-3673`.

If `features.disable(ShellSnapshot)` fails because requirements pin it, the gate arms anyway and only a warning is shown. Snapshots, which persist the environment, keep being written. Suggested fix: refuse to arm, or gate the snapshot writer with `OutputSink::Snapshot`.

## P3

- **Serde round trip can lose data.** `disclosure_gate.rs:117-142` round-trips through serde whenever anything matches.
  - `FunctionCallOutputPayload` deserialisation resets `success` to `None` (`protocol/src/models.rs:2044`).
  - Reasoning `content` without `ReasoningText` is skipped on serialisation.
  - Any variant that fails to deserialise is silently dropped (`Withheld`). That can break call/output pairing, or swallow a terminal event such as `TurnComplete` and leave the client waiting.
  - Editing signed or opaque blobs (`encrypted_content`, `anthropic_content_block`) invalidates the provider signature on every later request.
  - Every string leaf is scrubbed, including `type` tags and ids.
  - Suggested fix: scrub only known text fields per variant. Never drop terminal events. Replace a signed or encrypted item rather than editing it.
- **Whole-word shadowing.** `output_gate.rs:710-715`: when the longest alternative at a position fails the word-boundary check, shorter whole-word alternatives at the same start are not tried (for example values `abc` and `abc.` in the text `abc.x`). Suggested fix: on a boundary failure, retry the remaining whole-word representations that are prefixes of the match.
- **Shared handles across owners.** `admit` deduplicates by raw value across labels and returns one shared handle (`:642-654`), so one owner's `retire` unprotects another owner's copy. Values are also never retired in production, so a long-running process that reveals more than about 300 distinct values permanently denies further reveals. Suggested fix: reference-count owners per entry.
- **Startup fails on whitespace-padded URL passwords.** `url_password` checks length before `admit` trims. A password like `"a  "` trims below 3 bytes, causes `TooShort`, fails `arm`, and the session cannot start (`disclosure_gate.rs:489`). Vault values under 3 bytes become unrevealable while armed. Document this.
- **Arming is one-way and process-wide.** A profile or thread with the flag off in the same process (app-server, subagents) is still gated. Feedback ring and log lines written before arming, and old rollouts replayed to clients on resume, are not scrubbed.
- **Doc comment moved.** `vault/src/lib.rs:732`: `format_timestamp`'s doc comment is now attached to `protect_revealed`.
- **Global lock on every delta.** `STREAMS` is a global `std::sync::Mutex` held across a regex scan on async tasks for every delta in every thread. Expected cost is small, but it serialises sessions.
- **Expensive recompiles.** Each new registration recompiles up to a 4 MiB alternation (size limit 256 MiB) while holding the state lock. Large vault values make reveals slow and memory-heavy.
- **Process / record mismatch.** The sprint record is `status: draft`, and its worktree and branch (`/Users/travisgood/...`, `feat/p0-security-levels`) do not match this worktree. AGENTS.md requires `ready` or `in_progress` with matching coordinates before implementation. Also, the `Financial` class is never registered by anything, and the sprint's `codex-secrets` / `sanitizer.rs` foundation is not integrated.

## Test gaps that matter

- Out-of-process reveal: an auth-helper canary echoed by a command must not reach the next model request (P1-1).
- Login or token refresh after arm (P1-2).
- An event sent through `get_tx_event()` (MCP startup failure, elicitation) is gated (P1-4).
- `send_event` integration: a delta followed by `ItemCompleted` releases the tail in order through the real `Session`. The current tests call `finish_streams` directly.
- A reasoning section break with tails held, and an aborted turn.
- Compaction requests (both v1 and v2) and `summarize_memories` with a canary in history recorded before registration.
- The rollout file on disk, the history file, the TUI log file and the feedback snapshot are all canary-free (end-to-end, not unit).
- An `EventMsg` / `ResponseItem` round-trip property test across all variants, asserting `Changed`, never `Withheld`, and that `success` and ids are preserved.
- Named false-positive tests: `GIT_AUTHOR_NAME`, a `postgres:postgres` URL, short URL passwords.
- Withheld-class tests: a partial seed phrase, and keystore or PEM scaffolding lines that must not trigger withholding.
- Config with the feature on: the gate is armed, shell snapshots are disabled, and the fail-closed path works. With the feature off: `output_gate::global().is_armed() == false` after Config load.
- Wrapped base64 and hex encodings (P2-3).
