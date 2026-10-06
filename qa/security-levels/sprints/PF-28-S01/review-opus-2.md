# PF-28-S01 security review 2 (Opus 5.5 High, follow-up)

Scope: `git diff origin/main...HEAD -- codex-rs` with fix commit `cf1a9ec012` as the delta. No uncommitted changes.
Ran `cargo test -p codex-secret-broker pf_28_s01`: 19/19 pass. Also ran a scratch copy of `output_gate.rs` in /tmp (outside the worktree) to confirm F1 and F5 below. Did not build or run the codex-core, login or vault tests.

## Verdict: CHANGES REQUIRED

Most of review 1 is fixed correctly:
- prefix-only stream hold-back;
- the delivery choke point;
- turn-end and section-break releases;
- withheld tombstones;
- seeding false positives;
- compaction gating;
- traces off and telemetry gated;
- shell snapshot fails closed;
- per-owner retire;
- seed windows;
- hex key forms;
- keystore/PEM scaffolding.

With the flag off, behaviour is still unchanged (see P3-6 for one small memory-only difference).

One blocker remains. The global stream table is keyed without a session identity. The review sub-agent's events pass through it twice, so a value split across chunks is shown to the client in pieces. This is a confirmed leak that a small change fixes.

## P1

**F1. A value split across chunks leaks when a sub-session's stream is re-gated by its parent (review mode).**
Locations: `core/src/security/disclosure_gate.rs:379,394,402,410,425` (stream keys) and `core/src/tasks/review.rs:179-183` (forwarding).

- The review sub-agent's `exec.rs`/`send_event` pushes deltas into the global `STREAMS` entry `exec\0<call_id>\0Stdout`.
- `process_review_events` then forwards each emitted delta to the parent's `send_event`. That pushes into **the same key, so the same `StreamScrubber`**.
- The parent's push drains the carry the child was holding (a possible value prefix) and emits it immediately, out of order. The child's next chunk then arrives with an empty carry, so the remainder of the value no longer matches.
- Confirmed with the real scrubber, shared as the keys share it. Chunks `hello <first 10 bytes>` and `<rest> done` were delivered as ` <prefix>hello<rest> done`. Both the prefix and the suffix reached the client, and the order was garbled.
- Forwarded reasoning deltas use the same keys and are affected the same way.
- `gate_delivered` cannot catch this, because it scrubs each chunk on its own.
- Normal display is also garbled whenever a forwarded chunk ends in a byte that can start a representation. With a few dozen managed values, that is most alphanumeric bytes.

Fix: either
- include the session or thread identity in every stream key (for example `Session` conversation id, passed alongside `scope`); or
- forward already-gated sub-session events through a non-streaming path (`gate_delivered`-style per-chunk scrub, or `send_event_raw`) instead of `send_event`.

Add a test that pushes the same delta through `gate_event_with` twice (child, then parent) with a value split across two chunks.

## P2

**F2. The P1-1 deferral (out-of-process `vault auth-helper`) is acceptable only if the dependency is enforced or disclosed.**
Location: `core/src/config/mod.rs:3661-3675`.

- The deferral relies on `secretless_agent_launch`, but `secret_output_gate` can be enabled on its own.
- In that configuration, the documented `CREDENTIAL="$(corbanu vault auth-helper <label>)" cmd` flow puts unregistered values into tool output, the model request and the rollout. Nothing tells the operator.

Fix: either
- emit a startup warning (or refuse to arm) when `secret_output_gate` is on and `secretless_agent_launch` is off; or
- state in the sprint record and docs that vault values revealed out of process are protected only together with PF-27-S02, with reflection handled by PF-28-S02.

With that in place I accept the deferral behind the default-off flag.

**F3. Dropped request-type events still hang the turn.**
Location: `disclosure_gate.rs:186-199` (`present`) and `session/mod.rs:2347-2354`.

- The fix keeps `TurnComplete` and `TurnAborted` when they are withheld. Every other `Withheld` event is still dropped silently, at three layers (`gate_event`, `send_event_raw_with_persistence`, `gate_delivered`).
- That includes `ExecApprovalRequest`, `ApplyPatchApprovalRequest`, `RequestUserInput`, `ElicitationRequest`, `ExecCommandEnd` and `ItemCompleted`. If one of these fails the serde round trip after scrubbing (or carries a withheld-class value in a typed field such as a path), the turn waits forever for an approval that never reaches the user, or a cell never completes.

Fix: for request-type events, either
- deliver a stripped placeholder (text fields replaced, ids kept); or
- auto-deny the request and abort the turn.

Never drop them. Add a test that forces `Withheld` on an approval request.

## P3

1. **Sign-in tokens are effectively never retired, and the registry fills up in long-lived processes.** Location: `login/src/auth/storage_gate.rs:40-75`, `login/src/auth/manager.rs` (`create_auth_storage` per load/save).
   - Every `create_auth_storage` call builds a fresh `GatedAuthStorage`, and each `load()` adds an owner that is never released (no `Drop`). A long-lived instance's `rotate` therefore never brings the owner count to zero.
   - This is the safe direction: an old access token is still valid until it expires, so it should stay protected.
   - However, each ChatGPT refresh adds roughly 50 KB of representations (two JWTs with base64 and hex forms). After about 70 refreshes plus vault reveals, the 4 MiB cap is reached. From then on, `save` returns PermissionDenied and sign-in refresh fails until restart.
   - The "old one retired" comment and the unit test describe behaviour that production never exhibits.
   - Fix: keep one gate owner per field per process (a process-wide map rather than per instance). Retire an old token only after its expiry, or deliberately never retire it and document a bound.
2. **Over-redaction of short (3–5 byte) whole-word values at a cut.** Location: `secret-broker/src/output_gate.rs:604-648`.
   - `emit` scrubs `ready` on its own. A whole-word value that ends exactly at the cut counts as a match even though the next byte in the buffer is a word character.
   - Confirmed: value `abc`, stream `abcdzzq` + `x end` gives `[REDACTED:w]dzzqx end`.
   - `gate_delivered` repeats the effect on every delivered chunk.
   - This only affects short vault values, and it errs safe.
   - Fix: when a whole-word candidate ends at `cut`, move `cut` before it, or pass a "more follows" flag to `scrub_bytes`.
3. **A base64 partial character just before a core can be emitted in the previous push.** Location: `output_gate.rs:604-607`. If a chunk ends exactly on that character (and it is not itself a representation prefix), up to 4 bits of the value are shown. This is negligible. Note it in the module doc rather than fixing it.
4. **Tombstone overflow re-opens withheld streams.** `disclosure_gate.rs:505-507`: when the tombstone set reaches 4096 entries, `clear()` drops every tombstone at once. Tombstones are also never removed for keys that are no longer open. Fix: evict the oldest tombstones (FIFO) rather than clearing all.
5. **The gate arms before the fail-closed shell-snapshot check.** `config/mod.rs:3664-3674`: if the pin check then errors, the process is already armed but has no session. In app-server, other threads are gated even though their configs leave the flag off. Fix: check the pin first, then arm.
6. **With the flag off, sign-in tokens are copied into the never-armed global registry.** `storage_gate.rs:40`: each new token's representations are computed and kept for the life of the process. No output changes, but the flag-off path now holds extra copies of credentials in memory. Fix: skip registration unless a process-level "gate requested" bit is set, or accept this and document it.
7. **OTEL is still partly ungated.**
   - `codex.user_prompt` (`session_telemetry.rs:976-982`, when prompt logging is on) is not gated.
   - Neither are the `error.message` fields of `codex.api_request`, `websocket_*` and `sse_event` (`:586,643,693,892-919`), nor the OTEL tracing-log exporter layer.
   - Fix: gate these with `OutputSink::Trace` as well.
8. **Memory summaries are refused permanently.** `client.rs:1523`: any raw memory that contains a managed value blocks the whole batch on every run. It never sends the value, so this is availability only. Prefer gating the per-memory text fields, or dropping just that memory.
9. **The `_URL` and `_URI` suffix exclusions drop secrets embedded in URLs.** `disclosure_gate.rs:56-57`: a webhook path secret or a `?token=`/`?api_key=` value in a `*_URL` variable is no longer seeded. Only userinfo passwords are extracted. Fix: also extract query values whose parameter names look secret.
10. **Each event is serialized and scanned three times while armed** (`gate_event`, `send_event_raw_with_persistence`, `gate_delivered`). Large `ExecCommandEnd` and `TurnDiff` payloads pay this cost three times. Fix: mark already-gated events, or skip the second pass for events that came from `send_event`.
11. **Tests arm the process-global gate.** `login/src/auth/storage_gate.rs:161`, `vault/src/tests.rs:595`, `core/src/security/disclosure_gate_tests.rs:75`. This is safe under nextest (`just test`). Under plain `cargo test`, other tests in the same binary run armed: rollout traces are off, the capacity is shared, and loads can fail. Fix: use a local `OutputGate` where possible, or document that these tests require nextest.
12. **Doc comment separated from its function.** `vault/src/lib.rs:827-829`: a blank line now separates `format_timestamp`'s doc comment from the function. Clippy's `empty_line_after_doc_comments` will flag this. Remove the blank line.

## Items deliberately left open

All are acceptable behind the default-off flag, provided F2 is handled and each is recorded in the sprint's Remaining ledger:
- MCP OAuth refresh registration;
- wrapped base64/hex (add a test that pins this as a non-goal);
- comma-separated or numbered seed phrase forms;
- the serde `success` field loss;
- the global `STREAMS` mutex;
- recompiles under the state lock;
- the one-way, process-wide arm.

## Missing tests that matter

- F1: two-layer forwarding of a value split across chunks (child, then parent `gate_event`).
- F3: a `Withheld` approval request or `ExecCommandEnd` must not be dropped.
- Through a real `Session`: `ExecCommandEnd` and `ItemCompleted` release tails in order (current tests call `finish_streams` or `gate_event_with` directly).
- The reasoning section-break release (the new code path has no test).
- Sign-in refresh across two separate `create_auth_storage` instances (the production shape), asserting the intended retire behaviour.
- Config with the flag on and shell snapshot pinned: the error path, and that the gate is not left armed.
