# PF-28-S02 review 4: brokered pinning (Opus 5.5 High, independent, `corbanu exec -m claude-opus-5-5-plan`, commit a81ea2d193)

VERDICT: APPROVE

I found no blocking issues. Every way I checked for a brokered request to reach an address other than the checked answers either fails closed or needs the client and broker to disagree on their settings, which the Hello message prevents.

**What holds up**

- **Tampering and replay:** `pinned_addrs` sits inside the MAC'd payload, so changing it breaks the signature (ipc_tests covers a one-byte tamper). Replays are rejected by the existing sequence/binding check, and the pins can't be swapped without re-signing. `ProviderRequestOperation` uses `deny_unknown_fields`, so a broker that doesn't know the new field rejects a pinned frame instead of silently ignoring it.
- **Re-resolution:** in server.rs (~L690) the broker inserts `PinnedPeers` built from the signed host/port into the upstream request. connect_policy.rs then treats any request with a pin as guarded: it refuses an upstream proxy (`ProxyAddress`), requires `covers()` to match the authority, and dials only the pinned addresses, each still passing the peer check. A broker started with `allow_upstream_proxy` therefore fails closed on pinned requests rather than tunnelling.
- **Fail-open paths:** with the flag on, the client returns `Unpinned` when the pin is missing or for another authority, and the server repeats the refusal. If the broker is down, the existing error path still runs and there is no fallback to direct.
- **Flag off:** the only place a `PinnedPeers` is created is the destination guard (destination.rs:406), and the broker only re-creates one from signed pins. So with the flag off nothing inserts a pin, and the connect_policy change has no effect there.
- **IPv6 / IP-literal hosts:** `pin_host_key` strips brackets, canonicalises IP literals and normalises DNS names, so `covers_authority` agrees on both sides.

**Low-severity findings / follow-ups**

1. **No end-to-end test from mitm.rs to the broker (test gap).** The new tests call `route.forward` with a hand-inserted pin. Nothing proves the extension survives the real path from mitm.rs into `forward`, or that `Unpinned` shows up as a 502. If a future refactor rebuilt the request without its extensions, the client would refuse every brokered request with the flag on. That fails closed, but it is an untested availability regression. Fix: add a pf_33_s02 test that sends a brokered request through the MITM handler with the guard on and asserts a 200 for a pinned name and a 502 when the pin is stripped.

2. **Client and broker can disagree about requiring pins (server.rs:~666, client.rs:~430).** With the flag off on the client but `pin_connections` on in the broker, the broker would still be safe. The reverse case is covered by the client's own check. Both values come from the same Hello, so this is fine today. Fix: add a short comment, or have the broker assert that the client's Hello matches its configuration, so a future split config can't quietly weaken this.

3. **Connection reuse could carry a stale pin (connect_policy / UpstreamClient).** If the broker's `UpstreamClient` ever adds connection pooling keyed by authority, a later request could reuse a socket dialled under an earlier pin, i.e. answers checked earlier. I found no pooling in upstream.rs today. Fix: add a regression test or a comment saying pooled connections must be keyed by pin, or pooling must stay disabled for pinned requests.

4. **Error code could be more precise (client.rs `with_pinned_addrs`).** When the guard returns more than 16 answers, the request is refused as `Unpinned`, which looks like a missing pin rather than an oversize one. That is acceptable fail-closed behaviour. Fix: either truncate to `MAX_PINNED_ADDRS` (dialling a subset of checked answers is still safe) or add a distinct error code, and add a test for more than 16 answers.

5. **Local-binding change needs documenting (runtime.rs:1122).** With the guard on, `allow_local_binding` is now forced false for the broker. That's the intended security tightening, but it changes behaviour for users who broker credentials to loopback or private providers. It should be noted in the release record or plan.

6. **Flag-off test wording (connect_policy.rs:~610).** The rewritten test correctly asserts that a mismatched pin is refused when the flag is off. Consider also asserting that a flag-off request with a pin and a `ProxyAddress` is refused, to lock in the new "pin present means guarded" rule.

## Disposition

- 1: covered end to end by the `pf28s02-brokered-pinned` GLM 5.2 video (real MITM path, guard on); no unit test added.
- 2: both values come from one config; the broker refuses unpinned frames itself. No change.
- 3: comment added in server.rs; `UpstreamClient` opens a fresh connection per request.
- 4: Core now pins at most 16 answers (a subset of checked answers).
- 5: recorded in the sprint record.
- 6: flag-off test now also asserts a pinned request with an upstream proxy is refused.
