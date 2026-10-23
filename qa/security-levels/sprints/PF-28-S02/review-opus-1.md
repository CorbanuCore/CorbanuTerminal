# PF-28-S02 security review 1 (Opus 5.5 High, independent, `corbanu exec -m claude-opus-5-5-plan`, commit 5b107095a6)

VERDICT: CHANGES REQUESTED

I only read the code for this review. I did not build it or run any tests.

**High**

1. **A compressed body can get past the encoding check.** `network-proxy/src/credential_broker/response_scrub.rs:70-74` uses `HeaderMap::get(CONTENT_ENCODING)`, which returns only the first header. An origin can send `Content-Encoding: identity` and then `Content-Encoding: gzip`. The check passes, the gzip body goes through the byte scrubber without being decoded, and the client decompresses the credential. `checks_content_encoding` handles comma lists, but not repeated headers.
   - **Fix:** loop over `headers.get_all(CONTENT_ENCODING)` and require every value to pass. Do the same for `Transfer-Encoding`: refuse any coding other than `chunked` or `identity`, because `Transfer-Encoding: gzip` is valid HTTP/1.1.
   - **Test:** add a test for split and duplicated `Content-Encoding` headers.

**Medium**

2. **The decoded rescan stops at chunk boundaries.** `secret-broker/src/output_gate_rescan.rs:103-128` (`stream_hold`) says so in its own comment: "a block split across chunks is matched on its direct encodings, not decoded." The registered encodings only match base64 of the bare value, starting at a 3-byte boundary. Suppose a provider echoes `base64(json{"auth":"Bearer sk-…"})`, or a double-encoded value. Within one chunk, `decode_runs` catches it. If the run straddles a chunk split, which the origin controls with chunked encoding, each half is emitted unmatched and the client can join them. That is the reflection case this sprint is meant to close.
   - **Fix:** in `stream_hold`, hold back a trailing unbroken base64/hex run of at least `MIN_DECODE_RUN` bytes, up to a bound of about `ceil(4/3 · (2·max_value_len + overhead))` for each decode level. Emit it only once a non-base64 byte arrives, or once it goes past the bound; past the bound, rescan the held window plus the overlap.
   - **Test:** split a 2-level-encoded reflection at every byte offset and check that it is always scrubbed.

3. **Hook header values that aren't text are skipped without scrubbing.** `response_scrub.rs:52-54` does `let Ok(value) = header.value.to_str() else { continue; }`. The value is still injected (via `apply_mitm_hook_actions`) but never registered, so it can be reflected in full. A hook token separated by something other than a space (`Token=abc`, or a tab) also registers only the whole string. That leaves the raw token unprotected wherever the origin echoes it without the prefix.
   - **Fix:** if `to_str` fails, deny the request with `RegisterError`, matching the "full gate fails closed" behaviour. Also split on whitespace and on `=` / `:`.

4. **Responses that carry on past the scrubber aren't checked.** `mitm.rs` never looks at status 101 or `Upgrade` / `CONNECT` responses. If an allowed provider route ever returns `101 Switching Protocols`, which websocket endpoints on `api.github.com` or OpenAI realtime could, the scrubbed `Body` wrapper is skipped by the upgraded IO and the frames arrive raw.
   - **Fix:** when `reflection.is_some()`, refuse 101 responses and remove `Upgrade`/`Connection: upgrade` from the request. Add a test.

**Low**

5. **Path binding accepts `;`.** In `providers.rs` `plain_path`, a segment like `..;` or `.;x` is allowed, and some origin stacks treat `;` as a path-parameter separator that normalizes away. `%25` double-encoding (`%252e`) is also allowed.
   - **Fix:** reject `;` in segments, and reject `%25` followed by a hex pair.

6. **Some responses are reshaped.** `response_scrub.rs:87` removes `Content-Length` from every response, including HEAD, 204 and 304. That's harmless for the body, but HEAD loses its length metadata, which tools such as git-lfs size probes read.
   - **Fix:** keep `Content-Length` when the response has no body (HEAD, 204, 304) and the headers didn't change.

7. **A body error discards the held-back bytes silently.** At `response_scrub.rs:174-178`, emitting the scrubbed `finish()` output before the error would be more truthful. That is correctness, not a leak; the current choice is safe.

8. **Test gaps:**
   - trailers that contain the credential in their name;
   - a `Location` redirect containing the URL-encoded credential;
   - an HTTP/2 upstream (`parts.version` is forced to 1.1 only in the isolated broker, not on the MITM `Direct` path);
   - a false-positive corpus for the rescan (git pack hex SHAs, long base64 fields in JSON, ordinary English containing three seed words with punctuation) to show ordinary output survives unchanged.

**Things I checked that look fine:**
- With the flag off, the paths are unchanged: `response_gate` defaults to false and `Direct(None)` matches the old behaviour.
- The isolated broker refuses to admit a value the gate can't hold.
- A gate failure denies the request.
- Header names are scrubbed and unrepresentable values are dropped.
- The amount held back is bounded by the longest value.
- Revocation is unaffected because each request gets its own gate.

Findings 1 and 2 are both ways for the credential to reach the agent through the output path, so this needs changes before it can be approved.
