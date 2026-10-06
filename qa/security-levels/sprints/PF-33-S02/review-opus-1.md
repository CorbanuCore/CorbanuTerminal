# PF-33-S02 security review 1 (Opus 5.5 High, independent, `corbanu exec -m claude-opus-5-5-plan`, read-only, commit 0622d18e0a)

VERDICT: APPROVE WITH NITS

I found no route around the checked peer with the guard on, and no change with the flag off. Each new branch is gated on `url_destination_policy` or `config.url_destination_policy`. This review is from reading the code only: I didn't run cargo or any tests.

**P0 / P1**
- None.

**P2**
1. **SOCKS5 TCP and non-MITM CONNECT tunnels are now always refused with the guard on.** It fails closed, but the sprint record doesn't say so.
   - Where: `socks5.rs:364` and `http_proxy.rs:282` check the authority with `authorize_tunnel`. The dial then goes through `TargetCheckedTcpConnector` (`socks5.rs:155/647`, `http_proxy.rs:492`) with no `PinnedPeers`.
   - Result: `connect_policy.rs:~140` refuses it with "no checked answers". So guarded SOCKS TCP, and plain CONNECT when MITM is off, can't connect at all.
   - Fix: either make `authorize_tunnel` return a pin and attach it to the tunnel's dial request, or record "SOCKS/raw tunnels refused under the guard" as intended. Either way, add a test that proves the current behaviour.
2. **The guard flag is read twice for one request.**
   - Where: `mitm.rs:317` reads it to decide whether to attach a pin; `connect_policy.rs:119` reads it again before dialing.
   - Result: if config reloads from off to on in between, the request is refused (safe). From on to off, the dial goes through today's unpinned path, which matches what flag-off means. No bypass, but it's a race.
   - Fix (optional): carry one "guarded" decision with the request instead of reading the flag again.

**P3**
1. `pin_host_key` (`connect_policy.rs:~60`) handles case, brackets and a trailing dot. It doesn't convert IDN or punycode names. If `destination.host()` normalizes them, a mismatch only causes a refusal, so it fails closed. Add a test with a Unicode or punycode name.
2. `dial_pinned` gives up after trying every pinned address, so a pin with many addresses could hold a request for up to 10 s each. Consider a cap on total time.
3. Test gaps:
   - No end-to-end MITM test where DNS returns a different answer between the guard's check and the dial (only connector-level tests).
   - No test showing that the `forward_request` upstream-proxy check rejects the request before `authorize_request` resolves the name.
   - Windows provisioning doesn't drop the unix-socket and all-unix-socket grants, only local binding. Fine if Windows provisioning has no AF_UNIX allow; worth stating in the evidence.
4. Checker (`docs/sprints/check.py`): the `merged_behind_flag` exception only checks that `gate_evidence` exists. It should also check that the file is inside the repo (`qa/…`), so an existing file elsewhere can't satisfy it.

**Out-of-scope items:** I agree with all four. Brokered credential routes should get a tracking item in the broker lane so their missing pin isn't forgotten.
