# PF-28-S01 security review 5 (Opus 5.5 High, re-check of 0eb27852aa)

## Verdict: CHANGES REQUIRED

K1 is fixed. However, the new fail-closed rescan always withholds any value whose serialized form is larger than
`MAX_SCAN_BYTES` (16 MiB), even when every field is clean (V1). For example, a resumed long session's
`SessionConfigured` (which carries `initial_messages`) becomes an error notice. Before this commit that event passed.
The fix is small.

Checks:
- `git show 0eb27852aa` and the full `disclosure_gate.rs`;
- `output_gate.rs` (`scrub_bytes`, `find_all`, `representations`);
- the callers `persist_rollout_items`, `record_conversation_items_from`, `gate_prompt` and `next_event`/`send_event_raw`;
- README "Known limits".

Test runs:
- `cargo test -p codex-secret-broker pf_28_s01`: 20/20 pass.
- I did not run the core test, per the brief. I traced it by hand.
- I confirmed V1 and V2 with a scratch binary against the broker crate only
  (`.codex-work/pf28-s01-20261006/review/scratch5`). It uses a synthetic value, not a real one.

## Verified

- **K1:** `scrub_json` now gates keys through `gate_keys` (`disclosure_gate.rs:374`), and so does `strip_value`
  (`:243`).
  - Collisions get a `#n` suffix, so no entry is lost.
  - The rescan (`:288-294`) turns any surviving match (a number, or a match across fields) into `Withheld`.
- **New test** (`disclosure_gate_tests.rs:521`):
  - A canary used as a key comes back `Changed` with a marker and both entries present.
  - A 12-digit managed value held in a JSON number is `Withheld`.
  - Both halves would fail on the pre-fix code, which returned `Unchanged`.
- **N2 and N1:** both are recorded in Known limits ("No realistic request reaches this ... such as `cwd`", plus
  whole-event fallback).
- **Flag off:** unchanged. Every entry point returns early when `active()` is `None`.
- **False withholds in normal use (other than V1):**
  - Numbers: a match needs a managed value of 6 or more digits inside a JSON number. Core admits env values only at
    8 bytes or more.
  - Cross-field matches: the value would have to contain JSON punctuation such as `","`.
  - Escape artefacts: a match on `\n`/`\t` plus the value's tail needs most of the value to be present.
  - Struct field names that equal a vault credential: the event was already withheld when the same word appeared as
    an enum tag.
  - Conclusion: none is realistic. Call/output pairing in history is affected only by V1.

## P2

**V1. Fields that are each clean but total more than 16 MiB are now always withheld.**
Location: `disclosure_gate.rs:288-294`, with `output_gate.rs:468`.

Why it happens:
- `scrub` returns `Some` (withheld marker) for any input larger than `MAX_SCAN_BYTES`. The size alone triggers it.
- Before this commit:
  1. The quick scan hit on size.
  2. The walk scanned every string and key, each under the limit, and found nothing.
  3. The result was `Unchanged`, which was correct.
- Now the rescan of the over-limit document hits again on size, so the result is `Withheld`. Scratch check: two clean
  9 MiB fields produce a hit with `withheld=true`.

What breaks with the gate on:
- `SessionConfigured` on resume holds every prior event (`session.rs:1265-1284`), including user images as data URLs
  and exec output. A long or screenshot-heavy session can exceed 16 MiB.
  - It now goes through `present` → `strip_value`. That rescan also hits on size, so it returns `None`, and the event
    becomes the "withheld an event" error notice.
  - The client never receives the session configuration.
- `persist_rollout_items` silently drops large `RolloutItem`s. `Compacted { replacement_history }` with images is the
  likely case, and losing it corrupts resume.
- A history or request item over the limit is dropped, which breaks call/output pairing. This is rare.

Fix:
- Do not treat the size marker as a finding when the walk has covered every leaf. One option: rescan only when
  `serialized.len() <= MAX_SCAN_BYTES`. Otherwise, scan the leaves the walk cannot rewrite (numbers) one by one.
- Equivalently, have the walk report "unrewritable leaf hit" directly. Do the same in `strip_value`.
- Add a test: a clean composite over 16 MiB must return `Unchanged`.

## P3

**V2 (pre-existing, not from this commit). The quick scan misses a value written in JSON-escaped form inside a
string field.**
Location: `disclosure_gate.rs:282`, and the same blind spot in the new rescan, `strip_value` and `would_disclose`.

How it fails:
- `representations()` (`output_gate.rs:899-915`) adds the raw and singly JSON-escaped forms.
- A string field that already contains the escaped form gets escaped a second time when the whole value is serialized,
  so neither form appears. Example: `cat config.json` output or tool arguments holding a password with `"`, `\` or a
  control character.
- Scratch check: the per-field scan hits, but the quick scan of the serialized value misses. So
  `gate_value_with` returns `Unchanged`, and the item reaches history, the model and the rollout raw.
- This only applies to values containing `"`, `\` or control characters. Generated tokens never do; passwords can.

Fix: also register the doubly escaped form (JSON-escape the JSON representation) when it differs, or record this in
Known limits.

## Nits

- **N3.** The `TurnComplete`/`TurnAborted` exceptions (`disclosure_gate.rs:202-207`) send the original event. Any
  remaining numeric or cross-field match, which the commit message says is now withheld, is still delivered there.
  This is practically unreachable (timestamps and durations), but "never delivered raw" is not quite true. Note it in
  Known limits.
- **N4.** Nothing tests the key branch of `strip_value` (`:243`), which is the `present` fallback when a key match
  sits beside an unrebuildable field.
