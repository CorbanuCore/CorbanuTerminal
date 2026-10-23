# PF-33-S02 evidence: connection pinning and alternate-egress denial

Behind the existing default-off flag `url_destination_policy` (the PF-33-S01 guard). Flag off is unchanged.
Base `39c1f06213`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s02-20261006`.

## What the guard now does

| Control | Where | Behaviour |
| --- | --- | --- |
| Pinning | `network-proxy/src/connect_policy.rs`, `destination.rs`, `mitm.rs` | Each authorized inner HTTPS request carries the guard's checked DNS answers (`PinnedPeers`). The connector dials only those addresses, for exactly that host and port, with no second DNS lookup. A dial without a pin, or with a pin for another authority, is refused. Each address still passes the non-public peer check. TLS is still verified against the request host. |
| No pooled reuse | `upstream.rs` (unchanged) | Every upstream request dials fresh; tests count one accept per request. |
| Upstream proxies | `mitm.rs`, `connect_policy.rs`, `upstream.rs` | An inherited `HTTP(S)_PROXY`/`ALL_PROXY` route is refused with `upstream_proxy` (403) before dialing; the connector also refuses any proxy address. Deliberately fail-closed: a proxy resolves and connects on its own, so the checked answers could not be pinned, and bypassing a corporate proxy would skip its egress controls. |
| Open tunnels | `mitm.rs` | A host denied after its tunnel opened is refused on that tunnel (`host_denied`). |
| Proxy Unix-socket route | `http_proxy.rs` | The `x-unix-socket` escape hatch (proxy dials a granted local daemon) is refused (`unix_socket`). |
| Raw tunnels | `connect_policy.rs` | S01 forces interception of every guarded CONNECT/SOCKS tunnel, so the opaque forward and SOCKS direct dials never run; if they did, the unpinned dial is refused. |
| OS sandbox | `network-proxy/src/proxy.rs`, `core/src/config/network_proxy_spec.rs` | `allow_local_binding`, `allow_unix_sockets` and `dangerously_allow_all_unix_sockets` are dropped, so the sandbox reaches only the proxy's loopback ports: no other loopback service, no raw DNS on port 53, no Unix sockets. |

Agent-visible reasons: "Destination policy denied the request (upstream_proxy|host_denied|unix_socket)."; logs and blocked
records keep host, port and reason only.

## Platform capability matrix

| Platform | Direct sockets / QUIC / raw DNS | Loopback services | Unix sockets | Evidence |
| --- | --- | --- | --- | --- |
| macOS (Seatbelt) | Denied: only proxy ports are allowed | Denied under the guard (was allowed by `allow_local_binding`) | Denied under the guard (was allowed by grants) | `pf_33_s02` seatbelt test; demo videos |
| Linux (bwrap netns + seccomp, proxy-routed) | Denied: isolated network namespace, only the proxy bridge | Isolated namespace loopback | `socket(AF_UNIX)` denied by seccomp | Unchanged, existing tests; not re-run here |
| Windows | Firewall provisioning gets `allow_local_binding = false` under the guard | Same | Provisioning has no AF_UNIX allow | `pf_33_s02` provisioning test only; not run on Windows |

## Tests (final tree)

- `cargo test -p codex-network-proxy pf_33_s02`: 15 passed. Real loopback TCP and TLS fixtures; targets use
  RFC 6761 `.invalid` names that cannot resolve, so a response proves the pinned answer was used.
- `just test -p codex-network-proxy`: 296 passed (the S01 local-peer test now carries a pin to reach the peer check).
- `cargo test -p codex-sandboxing pf_33_s02`: 1 passed. `just test -p codex-sandboxing`: 79 passed, 2 failed
  (`create_seatbelt_args_for_cwd_as_git_repo`, `..._with_read_only_git_and_codex_subpaths`); both fail identically on
  the base commit with this machine's temp directory, unrelated to networking.
- `cargo test -p codex-core --lib -- windows_sandbox network_proxy`: 58 passed (`RUST_MIN_STACK=16777216`; the
  unfiltered debug binary overflows its stack on this machine regardless of this change).
- `python3 -m unittest docs/sprints/tests/test_check.py`: 24 passed; both checkers pass.

## Functional run (GLM 5.2, real TUI in tmux, demo SOP)

Recorded on `cde3d0f499` (final code) with `glm-5.2` / `zai`; host-side fixtures on 127.0.0.1:18733 (loopback canary),
`/tmp/pf33s02.sock` (Unix-socket canary) and 127.0.0.1:18734 (a CONNECT proxy that logs each CONNECT). Videos:
[`qa/demos/index/PF-33-S02.md`](../../../demos/index/PF-33-S02.md).

| Demo | Result |
| --- | --- |
| `pf33s02-pinned-dial` | Both HTTPS requests 200; log shows one pinned dial each (httpbin.org 8 answers, postman-echo.com 2). |
| `pf33s02-sandbox-egress` | Flag on: loopback canary refused, `dig @1.1.1.1` bind denied, Unix socket refused; proxy path 200. |
| `pf33s02-sandbox-egress-flag-off` | Control: loopback canary, DNS answer and Unix-socket canary all reachable directly. |
| `pf33s02-upstream-proxy` | Flag on, `HTTPS_PROXY` set: 403 `upstream_proxy`; the upstream proxy saw 0 httpbin.org CONNECTs. |
| `pf33s02-upstream-proxy-flag-off` | Control: 200; the upstream proxy saw 1 httpbin.org CONNECT. |

## Independent review

Opus 5.5 High via `corbanu exec -m claude-opus-5-5-plan` (read-only) on `0622d18e0a`: [review 1](review-opus-1.md),
**APPROVE WITH NITS**, no P0/P1. Dispositions:

- P2 "SOCKS TCP and non-MITM CONNECT refused": not a regression. Under the guard both are intercepted (S01), so
  inner requests are pinned; only the never-reached opaque dial would hit the unpinned refusal (tested).
- P2 flag read twice per request: a reload between reads either refuses (off to on) or takes the flag-off path
  (on to off), which is what flag off means. Accepted.
- P3 IDN: the guard pins the A-label form and clients send it; test added. P3 dial time: a 30 s total budget now
  bounds all pinned dials of one request. P3 checker: `gate_evidence` must already be repository-relative without
  `..` (tested). P3 tests: the upstream-proxy refusal happens before `authorize_request` (code order; the demo shows
  0 upstream CONNECTs); a live rebinding resolver is not used (see below).

## Not done here

- Brokered credential routes (`CredentialRouting::Brokered`, isolated broker process) dial without the pin; the
  broker re-resolves (its own non-public peer check still applies). Broker lane: carry `PinnedPeers` through the
  broker protocol.
- SearXNG: no adapter exists in the tree; nothing to route. Private services remain exact-grant only (contract).
- Runtime-approved (decider) hosts are not revoked on an open tunnel; that is PF-25-S02 revocation.
- Real DNS-rebinding server fixture: rebinding is covered by pinning tests with unresolvable names, not by a live
  flipping resolver.
