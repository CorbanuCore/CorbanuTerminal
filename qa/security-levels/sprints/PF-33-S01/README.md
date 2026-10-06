# PF-33-S01 URL, DNS and redirect policy: gate evidence (2026-10-06)

Branch `feat/pf-33-s01-url-dns-redirect`, macOS arm64 debug build, Rust 1.95.0. Flag:
`[features] url_destination_policy = true` (default off). It applies when the managed network proxy runs
(`[features.network_proxy] enabled = true`). Synthetic credentials and synthetic DNS fixtures only.

## What ships (flag on)

The guard runs after the existing host allow/deny policy and can only narrow it. It reuses the frozen PF-33-S03
contract (`pf33-destination-policy/v1`, `network-proxy/src/destination_contract.rs`); the runtime is
`network-proxy/src/destination.rs`.

| Control | Behaviour |
| --- | --- |
| Scheme, port, method | Public retrieval is HTTPS on 443 with GET/HEAD/OPTIONS/POST/PUT/PATCH/DELETE (the network mode still clamps methods). Plain HTTP and other ports are refused. |
| URL form | Contract canonicalization: IDNA, case, trailing dot, numeric IPv4 forms, mapped IPv6. Userinfo, fragments, backslashes, controls and empty ports are refused. |
| DNS | Every A/AAAA answer must be public: loopback, private, link-local, metadata, CGNAT, multicast, reserved, NAT64/6to4/Teredo, mapped IPv6, mixed answers, no answer, failure and more than 16 answers are refused. Reserved names (`localhost`, single labels, `.local`, `.internal`, ...) never reach a resolver. |
| Private networks | Only exact private-service grants (host, port, methods, paths, pinned addresses). `allow_local_binding` and literal local allowlist entries are not grants under the guard; the connector refuses non-public peers. No config surface yet: the protected policy has no grants. |
| Tunnels | Each CONNECT and SOCKS5 TCP tunnel is checked (authority plus DNS) and always intercepted, so a non-TLS stream fails instead of passing through. Missing MITM state is refused (`mitm_required`). Every SOCKS5 UDP relay is refused (`udp_relay`). |
| Redirects | Every 3xx with `Location` is re-authorized before it is relayed: one `Location` only, raw value screened, target on the host allowlist before any DNS lookup, then downgrade, method/body replay (contract) and destination checks. `Location` is rewritten to the exact absolute URL that was checked. The follow-up request is authorized again. |
| Chains | Ledger per execution and peer address: 10 hops, 120 s (late follow-ups and retries are refused until 240 s), per-client cap 32, global 256. `Authorization` is removed on any hop outside the chain's origin, before the broker or hooks add headers. |
| Reporting | Tunnel and plain-HTTP refusals fail the agent command with `Network access to "<host>" was blocked by the URL destination policy (<reason>).` Inner-request and redirect refusals answer 403 (`x-proxy-error: blocked-by-destination-policy`). Records and logs carry host, port and a reason code only, never a path or query. |

Flag off: no guard code runs beyond reading the flag; behaviour is unchanged (demo `pf33s01-baseline-flag-off`).

## Known limits (recorded, not fixed here)

- Credential adapters are bound to host only for legacy per-host brokered credentials (the scoped OpenAI route
  already binds method and path). That is `credential_broker.rs`, broker lane. Sprint Remaining.
- A redirect to a host approved at runtime but not on the allowlist is refused (`redirect_host_not_allowed`): the
  check runs before DNS so unlisted names never reach a resolver.
- A pending hop evicted by the per-client cap (32) leaves no marker; its follow-up starts a new chain.
- No in-process end-to-end MITM test: the guard refuses local upstreams by design. The GLM tmux runs cover the
  wire path; a test-only resolver/connector seam is suggested for PF-33-S02.
- POST answered with 301/302 is refused: the frozen contract forbids it, although clients would follow as GET.
- Hosts with more than 16 A+AAAA answers are refused (contract limit). Timed-out lookups are abandoned, not
  cancelled (`tokio::net::lookup_host`).
- PF-33-S02: pinning the connection to the checked answers, upstream and environment proxies (a local upstream
  proxy is refused under the guard), `NO_PROXY`, any safe re-allowing of UDP (QUIC; the guard refuses all UDP
  relays now), the `x-unix-socket` route and other alternate egress.
- Byte limits: the redirect target is bounded at 4096 bytes. Response-body budgets belong to the retrieval consumer
  (the browser broker already caps 2 MiB per response and 16 MiB per session).

## Tests

```text
cd codex-rs
cargo test -p codex-network-proxy pf_33_s01      # 23 passed
cargo test -p codex-network-proxy                 # 265 passed, plus 16 contract tests
cargo test -p codex-core --lib -- pf_33_s01 config_schema_matches_fixture denied_network_policy_message   # 4 passed
just fix -p codex-network-proxy / codex-features / codex-core; just fmt
```

The `pf_33_s01` tests use a fake resolver; nothing contacts a private endpoint. They cover answer sets, literals and
reserved names, scheme/port/method, canonicalization, IDNA and suffix confusion, per-hop redirect checks, method and
body replay, hop and time limits, credential stripping, private-service grants, the DNS-free host check,
`Location` screening and rewriting, ledger scoping and expiry, and the CONNECT, plain-HTTP, SOCKS5 TCP/UDP and
connector paths.

## GLM 5.2 tmux runs and videos

Real TUI, real keys, GLM 5.2 (`zai`), the agent running `curl` in the workspace sandbox through the managed proxy
against public services (httpbin.org, postman-echo.com, localtest.me, nip.io). No redirect to a private address is
followed with the flag off, so no private endpoint is contacted. Videos: [index](../../../demos/index/PF-33-S01.md).

| Demo | Shows |
| --- | --- |
| `pf33s01-baseline-flag-off` | Flag off: redirects to `http://`, the metadata IP and a loopback name are relayed (302) |
| `pf33s01-redirect-reauthorized` | Flag on: the same redirects come back 403 (`redirect_downgrade`, `private_destination`); a safe HTTPS redirect still passes |
| `pf33s01-dns-answers` | Allowlisted names resolving to 127.0.0.1, 169.254.169.254 and 10.0.0.1 are refused although `allow_local_binding = true`; httpbin.org returns 200 |
| `pf33s01-https-only` | Plain HTTP and port 8443 are refused; HTTPS on 443 returns 200 |
| `pf33s01-credentials-flag-off` / `-same-origin` | With `curl --location-trusted`, the synthetic `Authorization` reaches postman-echo.com with the flag off and is stripped with it on |
| `pf33s01-hop-limit` | A 3-hop chain completes; a 12-hop chain stops after 10 redirects with 403 |

## Independent review

Opus 5.5 High via `corbanu exec -m claude-opus-5-5-plan` (read-only): [review 1](review-opus-1.md) on `b039af3030`
returned CHANGES REQUIRED (2 P1, 3 P2, 8 P3). All were fixed in `42ca056628` except the recorded limits above:
DNS-free host check before resolving redirect targets (P1-1); forced interception and fail-closed without MITM
(P1-2, P3-6); raw `Location` screening, single header and rewrite (P2-3); scoped, capped ledger with expiry markers
(P2-4); added path tests (P2-5; an end-to-end MITM test needs a public upstream, so the tmux runs cover it); cheap
flag accessor (P3-9); plain HTTP refused without a lookup (P3-11); private contract module (P3-12).
[Review 2](review-opus-2.md) confirmed those and found SOCKS5 UDP unguarded (P1) plus four P3s. Fixed in the next
commit: UDP relays refused under the guard (test `pf_33_s01_socks_udp_is_refused_under_the_guard`), expired markers
kept so retries are refused, host patterns compiled only for redirect responses; the rest are listed as known limits.
[Review 3](review-opus-3.md), the final re-check, returned APPROVE WITH NITS (documentation nits, applied).
