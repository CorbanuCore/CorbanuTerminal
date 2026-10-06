# PF-28-S02 reflected-secret response scrubbing: gate evidence (2026-10-06)

Candidate: `10357ba50c` on `feat/pf28-s02-reflected-scrub-20261006` (base `24a57e38c4`, main with PF-28-S01),
macOS arm64 debug build, Rust 1.95.0. Feature flag: `[features] secret_output_gate = true` (default off). With the
flag off nothing changes. Synthetic credentials only: the PF-27 fixture token (sha256 prefix `946ae98e9fbe`) and a
40-byte random canary kept outside the repository.

## What ships

| Area | Behaviour with the flag on |
| --- | --- |
| Reflected credentials | When the proxy injects a credential (legacy broker record, scoped OpenAI route or MITM hook header), a per-request gate removes it, in every encoding the output gate knows, from response header names and values, the streamed body and trailers before the agent gets a byte. The isolated broker (PF-27-S04) does the same for its own credentials, in its own process. Labels: `broker:<VAR>`, `broker:credential`, `hook:<header>`. |
| Unreadable responses | The request asks for `Accept-Encoding: identity`. A response with any non-identity `Content-Encoding` occurrence, a transfer coding other than chunked, or `101 Switching Protocols` is refused with 502 and a fixed message; none of its bytes are returned. |
| Credential binding | Legacy and brokered credentials go only over HTTPS on 443, with GET/HEAD/POST/PUT/PATCH/DELETE, to provider paths: OpenAI `/v1/`; GitHub `api.*` hosts any path, web hosts `/api/`, git smart HTTP and LFS. Dot segments, empty segments, `\`, `;`, `%2e`, `%2f`, `%5c` and `%25` are refused (403, credential not sent). Plain-HTTP legacy injection is off. |
| Wrapped and nested encodings | Line breaks (raw or JSON-escaped, plus indentation) inside base64/hex blocks are joined after a run of 32 encoding characters, so `base64` (76 columns), `openssl base64` (64) and `xxd -p` (60) are found. Runs of 24 or more base64/hex characters are decoded two levels deep (any alignment) and rescanned; a hit redacts the whole run. |
| Seed phrases | Any three consecutive words of a registered phrase, in order, with any separators of up to 16 bytes (commas, numbering, JSON arrays, escapes, new lines, case), withhold the payload. |
| MCP OAuth | Access and refresh tokens are registered whenever a store loads or saves them (start, refresh persist, login save). The current and previous value per server and field stay protected; older ones are retired. |

## Latency and buffer bounds

| Stream | Held back between chunks |
| --- | --- |
| Core display streams (`StreamScrubber::new`) | The longest representation plus one byte (S01); a wrapped block's tail up to its last line break; two seed words and a cut third; an encoded run from 8 characters, up to 16 KiB (`MAX_BLOCK_CARRY`). Ordinary words stream through. |
| Proxied responses (`StreamScrubber::for_responses`) | The same, and every trailing encoded run up to 16 KiB, so an origin cannot split a nested encoding at a chunk boundary. A body that pauses mid-word waits for its next chunk or its end. |
| Whole payloads | Unchanged: over 16 MiB is withheld. Rescans cost about 5 ms per MiB in a release build (probe: 4 MiB of code, of wrapped base64 and of JSON took 21–26 ms; the direct scan 0.5–0.8 ms). |

## Tests (`just test`)

| Command | Result | Log |
| --- | --- | --- |
| `just test -p codex-secret-broker -p codex-network-proxy -p codex-rmcp-client` | 569 of 570 passed; 25 new `pf_28_s02` tests | `affected-crates.log` |
| `just test -p codex-core -p codex-login -p codex-vault -p codex-otel -E '…network_proxy, credential, pf_27, pf_28, pf_33, schema, disclosure, redact'` | 145 passed, including the 18 `pf_28_s01` tests in core, login and vault | `core-related.log` |

The one failure, `streamable_http_oauth_store_pinning::auto_store_remains_pinned_across_session_recovery`
("Native keyring is unavailable in an isolated test fixture"), fails the same way on clean `origin/main`.
`just fmt` and `just fix -p` ran on secret-broker, network-proxy and rmcp-client. `codex-core` clippy stops on an
existing `expect()` in `core/src/security/tainted_action.rs:394` (PF-30-S03, not this change). Linux clippy on the
RTX box: see the PR.

## GLM 5.2 tmux runs and videos

Recorded with `scripts/demo_video.py` on `10357ba50c` (GLM 5.2, real keys, disposable profiles) and published to the
`demos` prerelease; links in [`qa/demos/index/PF-28-S02.md`](../../../demos/index/PF-28-S02.md). The reflection runs
use the real isolated broker and httpbin.org as the permitted host (`GH_HOST=httpbin.org`, synthetic
`GH_ENTERPRISE_TOKEN` via `--credential ...=file:`), which echoes the request headers.

| Demo | Observed |
| --- | --- |
| `pf28s02-reflected-credential` | Agent env token hashes to a dummy; echoed header `Bearer [REDACTED:broker:credential]`, hash `e7eee689e782`, not the real `946ae98e9fbe`. No file in the run holds the token. |
| `pf28s02-reflected-baseline-flag-off` | Flag off: the echoed token hashes to `946ae98e9fbe`; the scan found the raw token in the agent's `echo.json` (the expected control finding). |
| `pf28s02-credential-path-bound` | `/anything/o/r.git/info/refs` 200; `/anything/settings/tokens` and `/anything/o/%2e%2e/x/info/refs` 403. |
| `pf28s02-wrapped-encodings` | `base64 -b 76`, `xxd -p` and base64 of hex of a credentials file: each block shows `[REDACTED:env:PF28_CANARY_API_KEY]` on screen and to the model. The scan found the canary only in `creds.txt`, which the agent wrote from its own environment (secretless launch off in this demo). |

The `python3: couldn't create cache file … xcrun_db` line is the macOS python shim inside the sandbox, not product
output.

## Independent review

Opus 5.5 High via `corbanu exec -m claude-opus-5-5-plan` (read-only), three rounds:
[review 1](review-opus-1.md) on `5b107095a6`: CHANGES REQUESTED. Fixed in `efa0e2bca5`: every `Content-Encoding`
occurrence and transfer coding checked, 101 refused (H1); encoded tails held so a nested encoding split by a chunk
is decoded whole (M2); hook values that are not text fail closed and are split into parts (M3); `;` and `%25`
refused (L5); `Content-Length` kept for 204/304 (L6); tests for trailer names, repeated encodings, a false-positive
corpus (L8). [Review 2](review-opus-2.md): APPROVE WITH NITS. Fixed in `59e841402e`: display streams hold encoded
tails only from 8 characters, responses hold all (latency); S01 bound test keeps its case with the new bound;
alignment-tolerant decode; `RegisterError::NotText`. [Review 3](review-opus-3.md): APPROVE WITH NITS; nits fixed in
`10357ba50c` (odd-length hex runs decoded, display hold documented, emitted-lead test).

## Known limits (not claimed)

- An unbroken encoded run longer than 16 KiB that a chunk boundary splits is matched on its direct encodings only,
  not decoded (an origin would have to pad its own reflection that far). Display streams can emit up to 7 leading
  characters of a run before the hold starts.
- `HEAD` responses lose `Content-Length` when a credential was injected. HTTP/2 upstreams on the MITM direct path use
  the same body wrapper but have no dedicated test.
- A credential reflected in a header *name* is matched only byte for byte (names arrive lowercased).
- Values the agent transforms itself (compression, encryption, splitting across fields) are not found.
- Carried over, not done here: scrub known text fields per type instead of a serde round trip, and per-session
  stream state with fewer repeat scans (PF-28-S01 items added to this sprint by PR #214).
