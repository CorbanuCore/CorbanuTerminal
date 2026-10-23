# PF-33-S01 security review 2 (Opus 5.5 High re-check of 42ca056628)

VERDICT: CHANGES REQUIRED

I reviewed the code by reading it only. I didn't build or run anything because the sandbox is read-only.

**Verified as fixed**
- **P1-1:** Redirect targets are now checked against the host allow and deny lists before DNS (`destination.rs:488`). The patterns are compiled the same way as the main host policy, and hosts are normalized the same way. The test `pf_33_s01_unlisted_redirect_targets_never_reach_the_resolver` asserts that only the original host is looked up.
- **P1-2 / P3-6:** With the guard on, CONNECT and SOCKS tunnels are forced to `Enabled` (`http_proxy.rs:330`, `socks5.rs:414`). If MITM state is missing, the tunnel is refused with `mitm_required`, and both paths are tested. The inner request's authority comes from the CONNECT target, not the Host header.
- **P2-3:** `check_response` refuses empty Location values, surrounding whitespace, backslashes and control characters. It also refuses non-UTF-8 values and more than one Location header. It then rewrites Location to the checked absolute URL with the fragment removed. Locations over 4096 bytes are refused by the contract.
- **P3-9, P3-11, P3-12:** All confirmed fixed.

**Remaining findings**

**P1:** SOCKS5 UDP skips the guard completely. I missed this in round 1.
- **Where:** `socks5.rs:665-820` (`inspect_socks5_udp`) runs only the host policy and never calls `guard_enabled`. `enable_socks5_udp` defaults to `true` (`config.rs:170`).
- **Effect:** with the guard on, an agent can still send UDP datagrams to allowed IPs on any port, such as QUIC or DNS. This breaks the HTTPS-on-443-only rule.
- **Private networks:** with `allow_local_binding` or a literal local allowlist entry, UDP reaches private addresses. The guard says neither of those counts as a private-service grant. The TCP connector (`connect_policy.rs:90`) enforces that, but UDP does not.
- **Record is wrong:** the sprint record says "Every CONNECT and SOCKS tunnel is checked" (sprint md:63), which is false for UDP.
- **Fix:** when `guard_enabled` is true, refuse every UDP relay request. Record a `destination_policy:*` reason, and fail closed if the flag can't be read. Add a `pf_33_s01` test for a UDP datagram to an allowlisted public IP.

**P3**
1. **Expired-chain refusal works only once.** `destination.rs:232-240`: `take` removes the expired marker, so a retry of the same URL starts a fresh chain with the hop count at 0 and no Authorization strip. When a client goes over its cap of 32, its oldest entries are also dropped without leaving a marker. Since all local processes share one peer IP, "per client" effectively means per execution.
   - **Fix:** keep the marker until twice the chain age, or record evicted entries as markers. Otherwise, document that a late follow-up is refused once and a retry starts a new chain.
2. **Redirect checks have a per-response cost.** `mitm.rs:431`: `HostPatterns::current` recompiles both glob sets for every guarded response, including 200s. Compile them only when the status is 3xx and a Location header is present, or reuse the runtime's compiled sets.
3. **Some valid redirects are refused.** Redirects to hosts that the user approved at runtime but that aren't on the allowlist get `redirect_host_not_allowed`. This fails closed, so it costs availability, not security. List it under known costs.
4. **P2-5 is only partly fixed.** There is still no test of the real MITM path: header stripping on a followed hop, 3xx turned into 403, and the Location rewrite on the wire. The stated reason (the guard refuses local upstreams) is acceptable for this sprint because the GLM tmux runs cover that path. A test-only resolver or connector seam would let PF-33-S02 add the test.
5. **Sprint record:**
   - The Verification section already checks off "review plus re-check" before this re-check.
   - The tunnel claim needs correcting (see P1).
   - Keeping `status: draft` is an AGENTS.md deviation. It's recorded and follows the PF-28-S01 precedent, so the integrator should accept it explicitly.

P1-1, P1-2, P2-3 and P3-6 introduced no new problems. Once the UDP guard and its test are in place, I'd expect APPROVE WITH NITS.
