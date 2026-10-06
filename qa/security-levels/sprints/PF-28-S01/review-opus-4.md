# PF-28-S01 security review 4 (Opus 5.5 High, re-check of 9261bd416f)

## Verdict: CHANGES REQUIRED

R1 to R4 are fixed and the forwarding test now exercises production behaviour. One pre-existing fail-open path
(P2 below) was missed in earlier reviews and is not in Known limits. The fix is small.

What I checked:
- `git show 9261bd416f`, plus the full `disclosure_gate.rs`;
- every caller of `gate_event`, `gate_delivered` and `gate_presented`;
- `tasks/review.rs` forwarding and the unified-exec watcher scopes;
- the README Known limits.

`cargo test -p codex-secret-broker pf_28_s01`: 20/20 pass. I did not build or run the core tests (no core build,
per the brief); the core test was traced by hand.

## Verified

- **R1/R2:** every finisher is scoped to the turn.
  - Section-break selector: `disclosure_gate.rs:474`.
  - `ExecCommandEnd`: `:497`. `ItemCompleted`: `:503`. `AgentReasoningSectionBreak`: `:508`.
  - `TurnComplete` and `TurnAborted` were already scoped.
  - Production matches this: `process_review_events` forwards deltas, End and non-message `ItemCompleted` into the
    parent's scope. It never forwards the child's `TurnComplete` or `TurnAborted`, so the parent's forwarded streams
    are released only by the parent's own finishers.
- **Forwarding test** (`disclosure_gate_tests.rs:358`): it now uses only `gate_event_with`. In the reasoning half,
  the child holds a canary prefix when its "think " delta is forwarded. On the pre-fix selector, the parent's
  `ReasoningContentDelta` would drain that prefix and the assertion would fail. The test therefore covers R1.
  The exec half covers ordering only.
- **R3:** `strip_value` tries `[WITHHELD]` first, then `""` (`:250`). An approval request falling back to an error
  notice is recorded in Known limits (README:88).
- **R4:** `send_event_raw_with_persistence` goes through `gate_presented` (`:186`). `present` never returns `None`,
  so raw sends are no longer silently dropped.
- **R5 to R7:** recorded in Known limits (README:85-87 and :90-91).
- **Flag off:** `gate_presented`, `gate_event` and `gate_delivered` pass events through unchanged. The removed
  `gate_value` had no other callers.
- **Regressions:** none found.

## P2

**K1. A managed value that appears only in a JSON object key is delivered raw. It is not covered by Known limits.**
Location: `disclosure_gate.rs:288` (`gate_value_with`), with `scrub_json` at `:356-372` and `strip` at `:243`.

How it fails:
1. The serialized scan finds the value.
2. `scrub_json` walks only `map.values_mut()`, so the key is never changed and the walk returns `false`.
3. `gate_value_with` returns `Gated::Unchanged` ("found only across field boundaries"), and the original value goes
   out with the secret intact. This applies to `present`, so client events and raw sends, and to `gate_values`, so
   rollout and model-request items.
4. This includes withheld-class values (seed phrases and private keys).

Reachable carriers:
- free-form JSON in `McpToolCall*` arguments (`protocol.rs:2525`) and `structured_content` (`mcp.rs:156`), which
  holds MCP-server output;
- dynamic tool arguments (`protocol.rs:2603`);
- `HashMap<PathBuf, FileChange>` keys in patch events and approvals (`approvals.rs:410`, `protocol.rs:3825-3852`).

Fix:
- Scrub object keys in `scrub_json` and `strip`. Rebuild the map, and treat a key that collides after scrubbing as
  withheld.
- Or, if the walk changed nothing, re-scan with the keys excluded. If the value is still found inside a key, return
  `Withheld` instead of `Unchanged`.
- Add a test: an `McpToolCallEnd` whose `structured_content` key is the canary must not deliver the canary.

## Nits

- **N1.** `strip_value`'s fallback to `""` is all-or-nothing (`:250`). If one byte field rejects `[WITHHELD]`, every
  matched string, including text fields, becomes empty. A per-field fallback is nicer. This is cosmetic.
- **N2.** README:88 says "No current request type hits this". `ExecApprovalRequestEvent.cwd` is an `AbsolutePathBuf`.
  A withheld-class or oversized match there replaces the whole field with a non-absolute marker, `[WITHHELD]` and
  `""`, so the request becomes an error notice. This is practically unreachable, but "no realistic request type"
  is more accurate.
