PF-28-S02 is merged into main in PR #216 (merge commit `b9f215ec50`). It sits behind the existing `secret_output_gate` flag, which is off by default, so nothing changes with the flag off. A small docs PR, #221 (`828ca96597`), records the merge. All CI checks were green on both PRs.

**What shipped (flag on)**
- **Echoed credentials are removed.** When the proxy adds a credential to a request (old-style broker record, scoped OpenAI route, or a MITM hook header) and the server echoes it back, it is taken out of the response headers, body and trailers before the agent sees anything. Every encoding the output gate knows is covered. The isolated broker does the same for its own credentials, inside its own process.
- **Responses it can't check are refused.** If a response is compressed, uses another transfer coding, or switches protocols, the agent gets a 502 with a fixed message and none of the server's bytes.
- **Credentials are tied to method and path** (the item moved over from PF-33-S01). They are only sent over HTTPS on port 443, with ordinary methods, to paths that provider uses: `/v1/` for OpenAI; for GitHub, any path on API hosts, plus `/api/`, git-over-HTTP and LFS. Tricky paths (`..`, `;`, encoded slashes or dots) are refused. Injecting credentials into plain-HTTP requests is turned off.
- **Gaps handed over from PF-28-S01 are closed:**
  - encodings wrapped across lines (`base64` at 76 columns, `openssl base64`, `xxd -p`);
  - decoding and rescanning nested encodings, two levels deep;
  - seed phrases written with commas, numbers or any other separator;
  - MCP OAuth tokens, which are now registered every time they are loaded, refreshed or saved.

**Gate evidence** (full write-up in `qa/security-levels/sprints/PF-28-S02/README.md`)
- **Focused tests:** 585 of 586 pass across secret-broker, network-proxy and rmcp-client, including 26 new `pf_28_s02` tests. The one failure, `auto_store_remains_pinned_across_session_recovery`, fails the same way on a clean main (keyring not available inside the test sandbox).
- **Wider tests:** the core, login, vault and otel subsets pass 146 of 146 after merging main.
- **Linux clippy:** clean with `-D warnings` on the RTX box for the three changed crates.
- **Review:** Opus 5.5 High, three rounds. Round 1 asked for changes; the two that mattered were a repeated `Content-Encoding` header slipping past the check, and a doubly encoded echo split across two chunks. Both are fixed. Rounds 2 and 3 approved with small fixes, all applied. The reports are `review-opus-1.md` to `review-opus-3.md` in the evidence folder.
- **GLM 5.2 tmux runs, recorded as four videos** using the real isolated broker and httpbin.org, which echoes request headers. They are published and listed in `qa/demos/index/PF-28-S02.md`:
  - **flag on:** the agent receives `Bearer [REDACTED:broker:credential]`, and what it receives does not hash to the real token;
  - **flag off (control):** the echo hashes to the real token (`946ae98e9fbe`), so the agent got the raw value;
  - **path binding:** a git path returns 200; other paths and `%2e%2e` paths are refused with 403;
  - **wrapped and nested encodings:** each one is redacted.

**Still open**
- **The sprint record stays `draft`.** PF-28-S01 isn't archived on main yet (PR #214 does that), and PF-27-S02 still occupies the broker lane, so the sprint checker won't accept `in_progress`.
- **Not done:**
  - Two items that PR #214 adds to this sprint weren't in my brief: scrubbing known text fields by type instead of re-serialising, and per-session stream state.
  - The published latency table needs target times approved by product.
  - These stay unchecked in Remaining.
- **PR #214 will conflict** with this sprint file and its index row, so whoever merges it needs to resolve that.
- **Known limits, recorded rather than fixed:**
  - An encoded run longer than 16 KiB that is split across chunks is only caught in its direct encodings, not decoded.
  - Display streams can show up to 7 leading characters of an encoded run before holding back.
  - HEAD responses lose `Content-Length`.
  - HTTP/2 servers on the proxy's direct path have no dedicated test.
- **Not mine, noticed in passing:**
  - Clippy on `codex-core` stops at an existing `expect()` in `core/src/security/tainted_action.rs:394`, from the PF-30-S03 work.
  - PF-33-S01 sets `url_destination_policy` in one config path in `config/mod.rs` but not the second one used when the permission profile changes. I set this sprint's flag in both places.

A short result file is at `.codex-work/workers-20261002/sec-broker4.result.md`.