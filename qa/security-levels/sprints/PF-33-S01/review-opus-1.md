# PF-33-S01 security review 1 (Opus 5.5 High, independent, `corbanu exec -m claude-opus-5-5-plan`, commit b039af3030)

VERDICT: CHANGES REQUIRED

I couldn't run `cargo test`: this sandbox is read-only, so the build couldn't write to the target directory. Everything below comes from reading the code. With the flag off, I found no change in behavior beyond the per-request config read noted under P3.

**P0**
None.

**P1**

1. **The proxy now looks up blocked domains in DNS, which gives a new way to leak data.** `destination.rs:360-361` (`authorize_redirect`, called from `mitm.rs:420-450`)
   - **Failure:** before relaying a redirect, the proxy resolves whatever host the upstream's Location points to. It never checks the host allow/deny list first. An agent can fetch `https://allowed.example/redirect?to=https://<secret>.denied.example/` through any allowed open redirector. The proxy then resolves `<secret>.denied.example` through the system resolver, so the data reaches that domain's DNS server even though the domain is blocked. With the flag off, the client's follow-up CONNECT is refused by the host policy before any DNS lookup.
   - **Fix:** run the host allow/deny check on the redirect target before resolving it. If the host policy would deny it, skip DNS: either deny the redirect or relay it and let the CONNECT fail. Add a test with a denied target that asserts the resolver is never called.

2. **"HTTPS-only" and the path/method/redirect checks can be skipped with a non-TLS tunnel.** `http_proxy.rs:327`, `socks5.rs:409` (`DetectTls` → `peek_tls_prefix` at `mitm.rs:118`)
   - **Failure:** with the guard on, an approved tunnel to a public host on port 443 is set to `DetectTls`. If the client sends non-TLS bytes, or nothing within the first-byte timeout (protocols where the server talks first, such as SSH on 443), the tunnel is forwarded opaquely. Plain HTTP or any other protocol inside CONNECT/SOCKS to :443 is never checked for path, method or redirects, which contradicts "plain HTTP refused".
   - **Fix:** when the guard is on, treat a non-TLS prefix or timeout as a denial and record a reason code, or use `Enabled`. Add tunnel tests that send plain HTTP and that send nothing.

**P2**

3. **The proxy checks a different redirect URL than the one the client may follow.** `destination.rs:347-352`, `mitm.rs:421`
   - **Location parsing:** the raw Location is run through `Url::join`, and only the serialized result goes to `normalize`. This skips the contract's screen for backslashes, control characters and whitespace. Values like `https:\\x`, `/\host` or `https:host` may resolve differently in the client.
   - **Multiple Location headers:** `headers().get` only checks the first one.
   - **Unchanged header:** the original Location header is relayed as-is.
   - **Consequence:** the proxy records a ledger entry for one URL while the client fetches another. The follow-up misses the ledger, starts a fresh chain, and skips the Authorization strip, the hop count and the body-replay check.
   - **Fix:**
     - Apply the contract's ambiguity screen to the raw Location.
     - Deny responses with more than one Location header.
     - Rewrite Location to the exact absolute URL that was checked.

4. **The redirect ledger can be confused, flooded or aged out.** `destination.rs:181-216, 304`
   - **Failure:**
     - One ledger per `MitmState`, keyed only by URL and shared by all clients. Any request for the same URL takes another chain's entry, so the real follow-up misses it.
     - An upstream can plant entries with high hop counts or a foreign origin for URLs the agent will fetch later. That causes spurious hop-limit denials or stripped Authorization.
     - 256 junk redirects push out (FIFO) the entries of legitimate chains.
     - `take` silently drops expired entries, so waiting more than 120 s starts a fresh chain with hops at 0 and no strip. The time limit is advisory only.
   - **Fix:**
     - Include the client identity in the key, at least the peer IP or session.
     - Keep expired entries as markers and deny (or at least strip) when one matches.
     - Set a per-client cap.

5. **The tests don't exercise the wiring.** The `pf_33_s01` tests are good unit tests of `DestinationGuard`, but nothing tests the real proxy paths:
   - MITM actually removing `Authorization` on a followed cross-origin hop.
   - A 3xx being replaced by a 403.
   - A Location header that isn't valid UTF-8, or multiple Location headers.
   - The SOCKS5 guard path.
   - `DetectTls` with non-TLS traffic.
   - Ledger expiry. `pf_33_s01_redirect_chains_have_a_time_limit` (`destination_tests.rs:325`) edits `chain.started` directly and misses the expiry → fresh-chain path in item 4.

   Add MITM-level tests with a stub upstream.

**P3**

6. **Guard on, no MITM, fails open.** `config.rs:156`: `url_destination_policy` is a `pub` field on a public struct. If it is set directly without `mitm`, `mitm_state` is `None`, the tunnel gets `Disabled`, and inner checks are silently skipped. Fail closed when the guard is on and MITM state is missing (`http_proxy.rs:326`, `socks5.rs:408`), or make the field private.

7. **Every POST answered 301/302 becomes a 403.** `destination.rs:355-358` with contract `:765-771`, even though real clients follow it as a body-less GET. This is the frozen contract's choice. Record it as a known availability cost of the flag, or model it as a GET follow-up in S03 v-next.

8. **DNS lookups aren't cancelled and can pile up.** `destination.rs:399`: the 3 s timeout gives up on `lookup_host`'s `spawn_blocking` but doesn't stop the `getaddrinfo` call. Redirects to slow-resolving names can fill tokio's blocking thread pool. Also, a host returning more than 16 A+AAAA records (common for 8+8 CDNs) is always denied.

9. **Overhead when the flag is off.** `guard_enabled` calls `current_cfg()`, which triggers a reload check and clones the full config on every MITM request, CONNECT, SOCKS request and non-public connect (`destination.rs:455`). Add a cheap boolean accessor.

10. **A local upstream proxy is refused under the guard.** `connect_policy.rs:88-99` refuses all non-public peers, and `HttpProxyConnector::optional(TargetCheckedTcpConnector)` (`http_proxy.rs:489`) uses that check. An upstream proxy on loopback or a private address is therefore refused. This fails closed, but S02 should know.

11. **Plain HTTP is resolved only to pick an error message.** `http_proxy.rs:794-823` resolves DNS for requests it will refuse anyway, just to choose a reason code. Refuse without the lookup.

12. **Wider public API.** `lib.rs`: `pub mod destination_contract` adds public API. `pub(crate)` is enough, because the integration test compiles the file standalone.

13. **Process.** The sprint record still says `status: draft` with every checklist item unchecked, but implementation has landed; AGENTS.md requires `ready`/`in_progress` first. The sprint also lists "byte limits" for redirects, which isn't implemented. Implement it or move it to another sprint.

**Checked and fine:**
- Logs, blocked records and responses carry only host, port and reason code: no paths or queries.
- Authorization is stripped before the credential broker and hooks inject anything.
- Mapped IPv6 and IPv6 literals are handled correctly.
- Every error path I traced fails closed.
