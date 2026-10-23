Follow-up from PF-33-S02: requests that went through the isolated credential broker skipped connection pinning, and the broker looked the name up again. This change closes that gap. It is recorded in PF-28-S02, the open sprint record in the broker lane.

Applies only when `url_destination_policy` is on. With the flag off, Core attaches no pins and the broker resolves as before.

- **Signed pins.** `ProviderRequestOperation` (secret-broker) now carries the checked DNS answers (`pinned_addrs`, up to 16) inside the HMAC'd frame, so they cannot be changed on the way to the broker.
- **Core (client.rs).** Signs the operation with the request's `PinnedPeers`. It refuses the request (`Unpinned` → 502) if the pin is for another host or port, or if there is no pin while pinning is required.
- **Broker (server.rs).**
  - The Hello carries `pin_connections`. When it is on, the broker refuses an unpinned operation (`unpinned`).
  - Otherwise it puts the signed answers on the upstream request, so its connector dials only those and never resolves the name.
  - Under the guard the broker also gets no local-binding grant, so its private-address check stays strict.
- **connect_policy.rs.** A `PinnedPeers` now binds wherever it is present, including with the guard off. A pin can only narrow a dial, and the broker has no guard of its own. The flag-off test was rewritten to cover this.

Tests (fake DNS: unresolvable `.invalid` names, so reaching the server proves the pinned address was used):
- `pf_33_s02_brokered_request_dials_the_pinned_answer_without_dns`: the pinned request reaches the server with the broker's credential. No pin, or a pin for the wrong host or port, is refused by Core; an unpinned signed frame is refused by the broker.
- `pf_33_s02_unpinned_broker_resolves_and_pins_keep_the_private_peer_check`: control case where the broker resolves the name itself and the request fails. A pinned loopback address is refused when the broker has no local-binding grant.
- `pf_33_s02_pins_bind_even_with_the_flag_off`, `pf_33_s02_provider_frame_carries_authenticated_pins`: tampering with a pinned address fails the MAC; unpinned frames keep their old wire format.

Local: `just test -p codex-network-proxy -p codex-secret-broker`, 384 passed. RTX (Linux): clippy `-D warnings` clean on both crates; tests pass.

Gate:
- Opus 5.5 High review: APPROVE with nits, all dispositioned ([review 4](qa/security-levels/sprints/PF-28-S02/review-opus-4-pinning.md)). Pins are capped at 16, the flag-off test also covers a pinned request through an upstream proxy, and the no-pooling assumption is now a comment.
- GLM 5.2 tmux demo ([mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-brokered-pinned-ea006ee308e4-2026-10-06.mp4)). With the guard, isolated broker and output gate on, a brokered request to httpbin.org returns 200 and the echoed header comes back as `Bearer [REDACTED:broker:credential]`. Core's log shows `brokered request pinned (host=httpbin.org, port=443, answers=8)`. This covers the real MITM-to-broker path.
