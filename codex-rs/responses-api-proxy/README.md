# codex-responses-api-proxy

## Internal synthetic transport — Cargo-tested, acceptance pending

PF-80-S01 internal infrastructure, **Internal delivery control — TO BUILD**,
“Use sequential sprints per initiative”. The default proxy below is unchanged.
This opt-in is not a general Responses proxy, model service or isolated executor.
It admits only the four literal packets in `src/synthetic_policy.rs`, preserving
inline-image/local-tool wire data, with no inference or tool execution.

After compilation and qualification, the bounded invocation is:

```text
codex-responses-api-proxy --synthetic-loopback-upstream 127.0.0.1:UPSTREAM_PORT --port DOWNSTREAM_PORT
```

Both ports must be explicit, nonzero and distinct; the fixture upstream must be
owned by the test operator. No hostname, remote IP, TLS, redirect, auth, cookie,
environment-proxy or stdin credential route is used. Dump/server-info/shutdown
options and a nondefault legacy upstream URL are incompatible. No file or console
I/O occurs during the timed run. Exit success means joined runtime termination,
not that every request succeeded or a peer consumed its response.

Fixed ceilings: two admitted tasks, eight attempts without refunds, sixty-second
run. Head16KiB/64fields/2s from accept; upload1MiB DATA plus16KiB raw head allowance
and5s from accept. Require Content-Length; no compressed/chunked/Expect ingress.
Upstream connect/final head each2s, clipped by10s whole exchange/run deadline.
Response4MiB DATA/4MiB+32KiB raw wire, no trailers. Client backpressure is included
in the same deadline. SSE DATA streams incrementally; HTTP chunk framing is not
preserved. Budgets never renew with trickle traffic. Backlog2 is an OS request,
not an exact global kernel connection bound; this is not a real-time guarantee.

Exact-byte admission rejects even equivalent JSON, extra/duplicate keys, unknown
tools/models, hosted actions and remote references. The four fixtures follow
`codex-api/src/common.rs`, `protocol/src/models.rs`, and `tools/src/responses_api.rs`;
image data demonstrates transport shape, not image comprehension. Arbitrary CLI
multi-turn schemas, encrypted continuations and an approved live model/service
route are outside this increment. Failed/partial responses are not completed SSE.

### Historical worker receipt — source-authoring phase, September13

- Clean launch HEAD `e2400ed9524301d70c7b2888da5c64dc52df0308`; allocated source
  base `6dde60fface29b0678b792926cbcff7bc8cc542e`, imported manager allocation.
- Exact worktree: `tasknode-owned-ingress-20260913`; branch
  `workstream/tasknode-owned-ingress-20260913`. Ten crate-local paths only;
  Cargo.toml remains untouched until the explicit manager dependency/build lease.
- Actual command `python3 docs/sprints/check.py`: exit0,115current/126archived.
  `python3 docs/plans/check.py`: exit0,3/3 active; `git diff --check`: exit0. Rust tests,
  compilation, Clippy, formatting, Cargo/Bazel parity: **NOT RUN, lease held**.
- Source checkpoint:927 authored changed lines,424 non-test,17 new test methods,
  zero executed. No Cargo manifest edits. These are unformatted counts, not final
  size acceptance: estimated formatted1,600–1,800total requires manager disposition
  before expansion beyond1,500. This is a paused uncompiled candidate, not review-ready.
- Authored tests use real sockets, actual binary/held-open stdin, explicit
  monotonic deadlines and join/socket metrics. Controller timeout/kill or cleanup
  fallback is a failure, never enforcement success. Large upstream-write fixture
  is supporting driver proof; production admission remains the four small packets.
- Manager owns root aliases, derived Cargo/Bazel locks, serialized Mac build lease,
  one material review and necessary correction. No worker reviews/children/commits.
- Root1.7 internal-only N/A is integrator-accepted for this increment, not PF80
  completion. Fresh confined executor/children/PTY, approved mediated service,
  negative access probes and independent execution/evidence review remain open.
  Live Slack authentication/phone/native ACK/overnight reception are not qualified.
- Prior accepted646-line/12-test engineering and all original failed reviews/raw
  attempts remain frozen; none were rerun, modified or upgraded by this work.

### Worker receipt — actual Cargo proof, September13

Same PF-80-S01 allocation and dirty-source checkout, HEAD
`613663f238d7085c19af85ecd35e543bb708d510`. Original927/424, formatted1370/614,
old1250target/1500total850non-test and later1800/1000target/1950/1100STOP are retained.
Parent supplied three root aliases and six observed crate dependency additions;
metadata exit0 verified unchanged package versions/sources/checksums. Parent
owns root Cargo.toml, Cargo.lock and MODULE.bazel.lock; none were edited by worker.

Raw evidence directory (private, outside published source):
`/Volumes/CorbanuDrive/Corbanu/.codex-work/owned-ingress-cargo.0NvIR9/`.
Every run below has START/END UTC and actual exit in its named log. All Cargo
commands used Rust1.95.0 with `RUSTUP_AUTO_INSTALL=0`, `CARGO_NET_OFFLINE=true`,
`RUSTUP_TOOLCHAIN=1.95.0`, the pinned toolchain bin first on PATH, and exclusively
`CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/codex-rs/target`.
No dependency repair, download, shared-source edit or runtime network beyond
owned synthetic loopback fixtures occurred. No source changed during test runs.

| Log | Actual command after environment prefix | UTC start–end | Exit / result |
| --- | --- | --- | --- |
| check-01.log | `cargo check -p codex-responses-api-proxy --offline --locked` |02:39:45–02:40:07|0 |
| fix-01.log | `just fix -p codex-responses-api-proxy --offline --locked --no-deps` |02:40:18–02:40:23|0; one automatic fix, three new unwrap/expect warnings corrected |
| fmt-01.log | pinned rustfmt1.95 toolchain, eight allocated Rust paths, `--edition 2024 --config-path rustfmt.toml --config skip_children=true` |02:41:23–02:41:23|0; known stable imports_granularity warning |
| test-01.log | `just test -p codex-responses-api-proxy --test-threads 1 --locked --offline` |02:41:23–02:41:39|101; CLI-helper compile error, zero executed |
| test-02.log | same full test command, after scoped CLI rustfmt |02:41:56–02:42:27|100;26passed/1failed/0skipped,26.059s; both failed attempts retained |
| fix-02.log | same scoped fix command |02:43:49–02:43:51|0, no warnings |
| fmt-02.log | same exact eight-file pinned rustfmt |02:44:16–02:44:16|0; known configuration warning only |
| test-03.log | same full test command |02:44:17–02:44:46|0;**28passed/0skipped,26.647s**, all ten pre-existing methods plus18new methods |
| clippy-final.log | `cargo clippy -p codex-responses-api-proxy --tests --no-deps --locked --offline` |02:45:17–02:45:19|0, no warnings |
| check-final.log | `cargo check -p codex-responses-api-proxy --locked --offline` |02:45:34–02:45:35|0 |

The first compile error was an incorrectly placed Ok in my test-helper Result
conversion; it was corrected without changing the startup/credential assertions.
The failed socket assertion required two Cancelled outcomes, although each
connection timer is clipped to exactly the root run deadline. Literal output
was Expired(Head),Cancelled. The correction permits only those two named terminal
paths while retaining two admissions, peak2, two joined outcomes, zero owned
sockets and actual client closure. It does not allow Complete/Failed/panic or
drop a proof. An additional real saturated owned accept-queue fixture proves
the connect deadline itself; no fake connection result or unowned endpoint.

Final nextest run `5b49a450-9833-4be3-9f17-14dcb9c218c2`; failed run
`9de4fcb0-daa5-47f5-bb22-a5355fbdd83d`. Both JUnit reports are preserved as
`test-02.junit.xml` and `test-03.junit.xml`. Final JUnit SHA-256:
`1247539963c96fb791b780daaa3cd0c5b786c8ce0db4e4c84aa34a79baa2a249`.
Final test log SHA-256: `6ea9aef50fd0fabfa7bf1362fd7203cc032f383027ee896c207e4af769d2ffb4`.
Tested debug binary SHA-256: `f6074f551eebc61484caae2916618b72a1d1a73aecc41b87c180e76bbe4b6370`.
`test-03.sources.sha256` verifies all eight Rust files and supplied crate manifest
unchanged through final check/Clippy. The final ten-file manifest includes this
post-test receipt; only this documentation was updated after the full suite.

Parent-owned bytes remain root Cargo.toml `d0b7740f235fb63f0959b1cf17ad290623e861fd846d8e5b081a3a47c93d07bc`,
Cargo.lock `c78783010630c80535844ae75d7d1ccf2ab1ac2255c0734c54317a9ac32d6b40`,
MODULE.bazel.lock `c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979`.
Parent diff is3 root-alias lines +6 Cargo.lock lines; MODULE diff0 and parity
**UNQUALIFIED**. The supplied6 crate lines count within the ten-path worker scope
but are parent-authored. Ten-path total1509/696non-test; worker-authored1503/690,
parent-authored15 total (6crate+9shared), combined1518/705. All additions, no deletions.

Plan/sprint checks pass3/115/126; whitespace passes. Runtime module maximum281
lines. Socket tests verify actual pending writes and cancelled/joined lifetimes;
CLI executes the production2s head deadline with stdin held open. Other timeout
tests use shorter private limits, not an elapsed60s/overnight claim. Large-write
driver proof is supplemental to exact four-packet admission. No test-controller
kill or cleanup fallback was needed for the final passing run. Image fixtures
prove inline wire shape only; neither full Responses schema nor image comprehension.

Frozen uncommitted candidate for parent material review; no worker review or child
agent was launched. Parent must complete MODULE/Bazel parity and combined proof
after the released Mac lease. Internal-only policy1.7 N/A and later independent
isolated executor/service/Slack/phone/native-ACK gates remain unchanged. No live
credential or service qualification, independent acceptance, commit or deployment.

#### tl;dr:

```
# Launch the proxy, dump request/response pairs to /tmp/proxy
cd path/to/codex/codex-rs
cargo build
echo $OPENAI_API_KEY | ./target/debug/codex-responses-api-proxy \
    --port 60001 \
    --dump-dir /tmp/proxy


# Add this to ~/.codex/config.toml:

[model_providers.codex-responses-api-proxy]
name = 'codex-responses-api-proxy'
base_url = 'http://127.0.0.1:60001/v1'
wire_api='responses'

[profiles.proxy]
model_provider = "codex-responses-api-proxy"


# Use it
codex -p proxy
```

# Detailed docs

A strict HTTP proxy that only forwards `POST` requests to `/v1/responses` to the OpenAI API (`https://api.openai.com`), injecting the `Authorization: Bearer $OPENAI_API_KEY` header. Everything else is rejected with `403 Forbidden`.

## Expected Usage

**IMPORTANT:** `codex-responses-api-proxy` is designed to be run by a privileged user with access to `OPENAI_API_KEY` so that an unprivileged user cannot inspect or tamper with the process. Though if `--http-shutdown` is specified, an unprivileged user _can_ make a `GET` request to `/shutdown` to shutdown the server, as an unprivileged user could not send `SIGTERM` to kill the process.

A privileged user (i.e., `root` or a user with `sudo`) who has access to `OPENAI_API_KEY` would run the following to start the server, as `codex-responses-api-proxy` reads the auth token from `stdin`:

```shell
printenv OPENAI_API_KEY | env -u OPENAI_API_KEY codex-responses-api-proxy --http-shutdown --server-info /tmp/server-info.json
```

A non-privileged user would then run Codex as follows, specifying the `model_provider` dynamically:

```shell
PROXY_PORT=$(jq .port /tmp/server-info.json)
PROXY_BASE_URL="http://127.0.0.1:${PROXY_PORT}"
codex exec -c "model_providers.openai-proxy={ name = 'OpenAI Proxy', base_url = '${PROXY_BASE_URL}/v1', wire_api='responses' }" \
    -c model_provider="openai-proxy" \
    'Your prompt here'
```

When the unprivileged user was finished, they could shutdown the server using `curl` (since `kill -SIGTERM` is not an option):

```shell
curl --fail --silent --show-error "${PROXY_BASE_URL}/shutdown"
```

## Behavior

- Reads the API key from `stdin`. All callers should pipe the key in (for example, `printenv OPENAI_API_KEY | codex-responses-api-proxy`).
- Formats the header value as `Bearer <key>` and attempts to `mlock(2)` the memory holding that header so it is not swapped to disk.
- Listens on the provided port or an ephemeral port if `--port` is not specified.
- Accepts exactly `POST /v1/responses` (no query string). The request body is forwarded to `https://api.openai.com/v1/responses` with `Authorization: Bearer <key>` set. All original request headers (except any incoming `Authorization`) are forwarded upstream, with `Host` overridden to `api.openai.com`. For other requests, it responds with `403`.
- Optionally writes a single-line JSON file with server info, currently `{ "port": <u16>, "pid": <u32> }`.
- Optionally writes request/response JSON dumps to a directory. Each accepted request gets a pair of files that share a sequence/timestamp prefix, for example `000001-1846179912345-request.json` and `000001-1846179912345-response.json`. Header values are dumped in full except `Authorization` and any header whose name includes `cookie`, which are redacted. Bodies are written as parsed JSON when possible, otherwise as UTF-8 text.
- Optional `--http-shutdown` enables `GET /shutdown` to terminate the process with exit code `0`. This allows one user (e.g., `root`) to start the proxy and another unprivileged user on the host to shut it down.

## CLI

```
codex-responses-api-proxy [--port <PORT>] [--server-info <FILE>] [--http-shutdown] [--upstream-url <URL>] [--dump-dir <DIR>]
```

- `--port <PORT>`: Port to bind on `127.0.0.1`. If omitted, an ephemeral port is chosen.
- `--server-info <FILE>`: If set, the proxy writes a single line of JSON with `{ "port": <PORT>, "pid": <PID> }` once listening.
- `--http-shutdown`: If set, enables `GET /shutdown` to exit the process with code `0`.
- `--upstream-url <URL>`: Absolute URL to forward requests to. Defaults to `https://api.openai.com/v1/responses`.
- `--dump-dir <DIR>`: If set, writes one request JSON file and one response JSON file per accepted proxy call under this directory. Filenames use a shared sequence/timestamp prefix so each pair is easy to correlate.
- Authentication is fixed to `Authorization: Bearer <key>` to match the Codex CLI expectations.

For Azure, for example (ensure your deployment accepts `Authorization: Bearer <key>`):

```shell
printenv AZURE_OPENAI_API_KEY | env -u AZURE_OPENAI_API_KEY codex-responses-api-proxy \
  --http-shutdown \
  --server-info /tmp/server-info.json \
  --upstream-url "https://YOUR_PROJECT_NAME.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT/responses?api-version=2025-04-01-preview"
```

## Notes

- Only `POST /v1/responses` is permitted. No query strings are allowed.
- All request headers are forwarded to the upstream call (aside from overriding `Authorization` and `Host`). Response status and content-type are mirrored from upstream.

## Hardening Details

Care is taken to restrict access/copying to the value of `OPENAI_API_KEY` retained in memory:

- We leverage [`codex_process_hardening`](https://github.com/openai/codex/blob/main/codex-rs/process-hardening/README.md) so `codex-responses-api-proxy` is run with standard process-hardening techniques.
- At startup, we allocate a `1024` byte buffer on the stack and copy `"Bearer "` into the start of the buffer.
- We then read from `stdin`, copying the contents into the buffer after `"Bearer "`.
- After verifying the key matches `/^[a-zA-Z0-9_-]+$/` (and does not exceed the buffer), we create a `String` from that buffer (so the data is now on the heap).
- We zero out the stack-allocated buffer using https://crates.io/crates/zeroize so it is not optimized away by the compiler.
- We invoke `.leak()` on the `String` so we can treat its contents as a `&'static str`, as it will live for the rest of the process.
- On UNIX, we `mlock(2)` the memory backing the `&'static str`.
- When using the `&'static str` when building an HTTP request, we use `HeaderValue::from_static()` to avoid copying the `&str`.
- We also invoke `.set_sensitive(true)` on the `HeaderValue`, which in theory indicates to other parts of the HTTP stack that the header should be treated with "special care" to avoid leakage:

https://github.com/hyperium/http/blob/439d1c50d71e3be3204b6c4a1bf2255ed78e1f93/src/header/value.rs#L346-L376
