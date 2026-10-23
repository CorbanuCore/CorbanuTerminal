# PF-28-S02 security review 3 (Opus 5.5 High, final re-check, `corbanu exec -m claude-opus-5-5-plan`, commit 59e841402e)

VERDICT: APPROVE WITH NITS

I found nothing that blocks merge. I reviewed the diff from efa0e2bca5 to HEAD in `codex-rs` and searched for every place that creates a `StreamScrubber`. I didn't build anything or run any tests.

**The four fixes check out:**
- **Two hold modes.** `new()` holds a trailing encoded run only from 8 characters (`output_gate.rs:583`). `for_responses()` holds every run up to MAX_BLOCK_CARRY (`output_gate.rs:591`). The check at `output_gate_rescan.rs:138` applies the minimum, and `.max(1)` stops a zero minimum from holding empty runs. `ResponseGate::body()` now uses `for_responses()` (`response_gate.rs:69`).
- **Default.** Default is now written by hand and calls `new()` (`output_gate.rs:724`). That matches the old derived behaviour except for the intended new display bound.
- **Base64 alignments.** Decoding now tries offsets 0 to 3 (`output_gate_rescan.rs:314`), which covers every 4-character alignment.
- **NotText and comment.** `RegisterError::NotText` exists and is used for non-text hook values (`response_scrub.rs:57`), and the test now checks for that exact error. The comment on the transfer-coding check is accurate.

**No callers broken:**
- Outside the tests, the only `StreamScrubber` users are Core's `disclosure_gate.rs:655` and `response_gate.rs:69`. Core calls `new()` directly; nothing calls `Default` or derives it through a containing struct (`OpenStream` has no derive). No Core code depends on the old "hold every run" behaviour. Core display streams now let words shorter than 8 characters through immediately, which was the point of the fix.
- The S01 bound test still covers ordinary text and the 4096-'z' case. The new tests cover immediate display output and redaction of a value inside a run longer than the carry bound.

**Nits (not blocking):**
1. `output_gate_rescan.rs:296`: hex decoding is only tried on the full run. In display mode, if an odd number of characters (1 to 7) was already sent, the rest of the run has odd length and is never hex-decoded. Hex forms of registered values are still caught by the stored encoded patterns, so this only matters for nested hex. Trying the run with one character dropped would close it.
2. Display mode can still send up to 7 leading characters of an encoded run before holding starts. That is under one base64 group (about 5 bytes), so the risk is small, but the doc comment at `output_gate.rs:581` should say so.
3. The tests don't cover a display-stream run that starts with fewer than 8 characters and then grows to 8 or more in the next chunk, which exercises the new alignment path. Consider adding one in `output_gate_rescan_tests.rs`.
