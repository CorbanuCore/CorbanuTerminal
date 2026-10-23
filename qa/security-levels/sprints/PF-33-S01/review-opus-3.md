# PF-33-S01 security review 3 (Opus 5.5 High final re-check of b40018175d)

VERDICT: APPROVE WITH NITS

I only read the code. I didn't build or run tests because the sandbox is read-only.

**Fixes confirmed**
- **P1, SOCKS5 UDP:** fixed. `socks5.rs:731-756` checks `guard_enabled` after the proxy-enabled check and before the network-mode and host-policy checks. When the guard is on, every datagram is refused with `destination_policy:udp_relay` and `fail_command: true`. If the flag can't be read, the relay fails closed. That means `allow_local_binding` and literal local allowlist entries can't reach private addresses over UDP. The test at `socks5.rs:980-1017` covers guard on and off against an allowlisted public IP on port 443, and checks both the error and the blocked record.
- **P3-1, expired chains:** fixed. `destination.rs:234-237` returns `Expired` without removing the marker, and markers are only purged after twice the chain age (240 s). The test now checks that a retry is refused too. Expired markers still count toward the 32-per-client cap, which is harmless. The README records that an evicted hop leaves no marker.
- **P3-2, per-response cost:** fixed. `mitm.rs:430-432` only calls `HostPatterns::current` when `is_redirect_response` is true. `check_response` uses the same predicate, so behaviour is unchanged.
- **P3-3, P3-4 and the tunnel wording:** recorded as known limits, and the sprint record now says "SOCKS TCP … (UDP relays refused)".

I found no regressions.

**Nits**
1. `qa/security-levels/sprints/PF-33-S01/README.md:39` still lists "SOCKS5 UDP" among the work left for PF-33-S02. Line 19 says UDP is now refused, so line 39 should say that only re-allowing UDP safely (for example QUIC) is left for PF-33-S02.
2. `README.md:85` links to `review-opus-3.md`, which doesn't exist yet. Save this output there before merging.
3. `pf-33-s01-url-dns-and-redirect-policy.md:85` already ticks "review and re-checks". That will be accurate once this approval is saved. Keeping `status: draft` while PF-27-S02 isn't archived still needs the integrator's explicit acceptance.
