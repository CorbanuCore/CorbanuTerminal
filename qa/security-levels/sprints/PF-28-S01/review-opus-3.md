# PF-28-S01 security review 3 (Opus 5.5 High, re-check of 8cef6630bc)

## Verdict: CHANGES REQUIRED

What I checked:
- the delta in `8cef6630bc` against review 2;
- every caller of `gate_event`, `gate_delivered` and `send_event_raw`;
- review forwarding (`tasks/review.rs`), `codex_delegate.rs`, the unified-exec watcher, and how turn ids are generated.

`cargo test -p codex-secret-broker pf_28_s01`: 20/20 pass. I built a scratch crate in /tmp (outside the worktree) on the real `StreamScrubber`. It confirms the mechanism behind R1: draining a scrubber in the middle of a value, then pushing the next chunk into a fresh one, emits the whole value. I did not build or run the core, login or OTEL tests.

What is fixed:
- **F2:** the startup warning works.
- **F3:** fixed on the `gate_event` and `gate_delivered` paths.
- **P3 items:** the snapshot pin check now runs before arming; registration happens only while armed; whole-word cuts; FIFO tombstones; URL query secrets; OTEL prompt and error fields.
- **Sign-in registry:** previous-value retention is correct. I traced A→B→C and A→B→A against owner counting, and the owners balance. The registry is bounded.
- **Turn scope:** every in-session producer passes `turn_context.sub_id` (`session/mod.rs:2089`) or `stream.sub_id` (`exec.rs:1143`). These match `TurnComplete.turn_id` and `TurnAborted.turn_id` (`tasks/mod.rs:899-921`, `lifecycle.rs`).
- **Sub-session turn ids:** these are fresh UUIDv7s (`submit_with_trace`), so they are distinct from the parent's.
- **Flag off:** behaviour is unchanged. Login and vault use `active()`, stream code runs only when armed, `trace_gated` passes text through, the warning appears only when the flag is on, and the config order is unchanged with the flag off.

F1 is only half fixed. The stream keys carry the turn, but the stream *finishers* still ignore it.

## P1

**R1. In review mode, a forwarded reasoning delta drains the child's open reasoning stream in the middle of a value. The value leaks.**
Location: `core/src/security/disclosure_gate.rs:457-462`, with forwarding at `tasks/review.rs:179-183`.

How it fails:
1. The section-break selector picks every stream where `stream.item_id == event.item_id && key.starts_with("reasoning\0") && key != section`. It does not check scope.
2. `process_review_events` forwards the child's `ReasoningContentDelta` unchanged, so the item id is the same, into the parent's `send_event`.
3. Gating that event with the parent scope selects the child's key `reasoning\0<child-turn>\0<item>\0<idx>`, because it differs from the parent's key, and calls `finish`.
4. `finish` emits the child's carry, which is the held value prefix. Scrubbed on its own, the prefix does not match, so it is sent to the client raw.
5. The child's next chunk then starts in a fresh scrubber, so the suffix is also emitted raw.

Every forwarded reasoning delta drains whatever the child holds at that moment, so a value split across two reasoning chunks leaks. The leak is the same as the original F1; only the event kind differs.

Fix: add `&& stream.scope == scope` to the section-break selector. Also scope the `ExecCommandEnd`, `ItemCompleted` and `AgentReasoningSectionBreak` selectors (R2). Unified exec sends its deltas and its end event with the same `turn_ref`, so scoping is safe there.

Missing test: the new forwarding test (`disclosure_gate_tests.rs:312`) finishes through hand-written selectors that do match scope. It therefore does not exercise production behaviour. Rewrite it so both layers run through `gate_event_with` only:
- forward the child's output, including real `ExecCommandEnd`, `ItemCompleted` and `ReasoningContentDelta` events with a value split across chunks;
- assert that the parent output holds no fragment of the value and keeps its order.

## P3

**R2. Unscoped End, ItemCompleted and SectionBreak selectors reorder the parent's forwarded text.**
Location: `disclosure_gate.rs:480-492`.
- The child's own `ExecCommandEnd`, reasoning `ItemCompleted` or `AgentReasoningSectionBreak` drains the parent's stream for the same id: the stream for already-forwarded chunks.
- That tail goes out on the child's channel after the child's own tail, then is forwarded again. The 1–2 bytes the parent held appear after later text.
- There is no leak, because the parent only ever holds output the child has already gated. The display is garbled.
- The fix is the same scope check as R1.

**R3. Stripped delivery can make an approval prompt misleading.**
Location: `disclosure_gate.rs:215-240`.
- `strip_value` empties every string that holds a managed value. An `ExecApprovalRequest` reaches this path when the rebuild with markers fails (for example, a path-typed field). The user then sees `""` for an argument that is actually a secret.
- If even the stripped event fails to rebuild, it becomes an `Error`. The request is lost and the turn still hangs. This is rare.
- Fix:
  - put a fixed placeholder (for example `[WITHHELD]`) in string fields;
  - empty a field only where the placeholder does not deserialize (byte fields);
  - for request-type events whose stripped rebuild fails, auto-deny the request rather than replacing it with `Error`.

**R4. `send_event_raw_with_persistence` still drops on `Withheld`.**
Location: `session/mod.rs:2347-2354`.
- Direct `send_event_raw` producers (errors, warnings, rollback and settings events, `handlers.rs`, `network_approval.rs:917`) bypass `present`, so an event that cannot be rebuilt is silently lost.
- No request-type event uses this path today. Route it through `present` for consistency.

**R5. Sign-in values loaded before a later arm are no longer registered.**
Location: `login/src/auth/storage_gate.rs:52`.
- This is a consequence of the "register only while armed" fix (review 2 P3-6).
- In app-server, the gate can arm only when a later thread's config enables the flag. `AuthManager` has already cached its auth by then.
- File mode is covered by the arm-time `auth.json` seeding. Keyring or auto mode tokens stay unprotected until the next load or save.
- Fix: replay protection at arm, for example `disclosure_gate::arm` triggering an auth reload or a login-side hook. Or record this in Known limits.

**R6. The sign-in registry is keyed by field, not by store.**
- The ephemeral, persistent and legacy-home stores share field names. With three distinct values for one field across stores, the oldest is retired even if a store still uses it.
- I could not find a realistic live case: the superseded value is always the older one. Note it in the module doc.

**R7. Small coverage gaps that are not listed in Known limits.**
- Secrets embedded in URL paths (for example webhook paths) and `X-Amz-Signature`-style query names (only `key`, `sig`, `signature` and the secret-key fragments are recognised).
- The OTEL log-export target `codex_otel.network_proxy`, which is not gated.
- Either fix these or add them to the sprint's Known limits.

## Not re-raised

These are accepted as recorded in Known limits:
- the triple scan;
- the global mutex;
- tests that arm the global gate (nextest);
- the base64 partial character;
- memory summaries;
- MCP OAuth refresh;
- wrapped encodings;
- serde field loss.
