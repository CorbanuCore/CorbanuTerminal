# PF-28-S02 security review 2 (Opus 5.5 High, independent re-check, `corbanu exec -m claude-opus-5-5-plan`, commit efa0e2bca5)

**VERDICT: APPROVE WITH NITS**

I reviewed `git diff 5b107095a6 HEAD -- codex-rs` and read through the whole change. I did not build anything or run the tests.

**The claimed fixes check out**
- **(1) Encodings and 101:** `response_scrub.rs:76-110` checks every `Content-Encoding` header with `get_all`. A `Transfer-Encoding` value other than chunked or identity is refused, and so is 101. Every refusal message is fixed text, so none of the origin's bytes reach the client.
- **(2) Encoded runs split across chunks:** `output_gate_rescan.rs:126-135` holds back the trailing encoded run while it is no longer than `MAX_BLOCK_CARRY`. The new test covers every split point for both double encodings (base64 of hex, and base64 of base64).
- **(3) Hook header values:** a non-text value now fails closed. Values are split on `=`, `:`, `,`, `;` and whitespace, and parts of 12 bytes or more are registered alone.
- **(5) Bound paths:** `;` and `%25` are refused (`providers.rs:99-108`), with tests for both.
- **(6) Content-Length:** kept for 204/304; HEAD losing it remains a recorded limit.

**Medium**
1. **Ordinary text can now stall mid-stream** (`output_gate_rescan.rs:126-135`). The held run is anything made of base64 characters, which includes plain letters and digits. A chunk ending mid-word, or a long minified identifier or hash, is held until a non-base64 byte or end of stream arrives. SSE is fine because events end with `\n\n`. Streamed text with no delimiters (NDJSON without a trailing newline, or a progress bar) can stall for up to 16 KiB. This does not leak anything, but it is a latency regression.
   - **Fix:** apply the long hold only once the run is at least the shortest encoded length of a registered value (`snapshot.min_encoded_len`). Below that, fall back to the S01 bound. Add a test that `push(b"hello wor")` emits `hello ` right away.
2. **The S01 test was weakened rather than kept** (`output_gate_tests.rs:262`). The S01 contract "holds back at most the longest value" no longer holds for runs of letters and digits; the test was changed to `"z "` input so it still passes. That quietly changes S01's guarantee for tool output.
   - **Fix:** keep the original `[b'z'; 4096]` case and assert the new bound (`<= MAX_BLOCK_CARRY`). Update the S01 sprint record or docs to say the hold limit changed.

**Low / nits**
3. **Runs longer than 16 KiB** (recorded limit). If the origin can be made to pad a reflected value (for example, an echo endpoint that base64-wraps a body the agent controls, next to a reflected header), a split nested encoding can get past the scrubber. Please record in the sprint's limits that this assumes the origin controls the wrapping, and add a test where a run larger than 16 KiB still triggers a direct-encoding match.
4. **Misleading error for non-text hook values** (`response_scrub.rs:57`). The failure reuses `RegisterError::TooShort`, which is wrong for a non-text value and could be mistaken for the "ignore short values" branch further down (lines 64-66). Add a distinct `RegisterError::NotText`, or map it at the call site, so logs show the real cause.
5. **Transfer-Encoding is checked after hyper has decoded the body.** With hyper, a non-chunked coding never reaches this point, so the check only does anything on other client paths. That's fine as defence in depth, but a code comment should say so.
6. **HTTP/2 upstream on the MITM Direct path** was not addressed. It still needs a tracked follow-up with an owner. The body abstraction is the same, so I expect no new bypass, but the change has no test that proves it.

**Nothing missed by review 1 that blocks this**
- I found no false-refusal regressions.
- The false-positive test set (git SHAs, a JSON image blob, English text with seed-like words) is a good addition.
- Keeping Content-Length on 304 is correct.
