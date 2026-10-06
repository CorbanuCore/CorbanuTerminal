---
sprint_id: "PF-28-S02"
title: "Reflected-secret response scrubbing"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-28"
execution_order: 31
owner: "broker lane worker (codex, 2026-10-06)"
parallel_lane: "broker"
write_scope: "codex-rs/secret-broker/src/output_gate.rs, codex-rs/secret-broker/src/output_gate_tests.rs, codex-rs/secret-broker/src/output_gate_rescan.rs, codex-rs/secret-broker/src/output_gate_rescan_tests.rs, codex-rs/secret-broker/src/response_gate.rs, codex-rs/secret-broker/src/response_gate_tests.rs, codex-rs/secret-broker/src/lib.rs, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/network-proxy/src/credential_broker/, codex-rs/network-proxy/src/mitm.rs, codex-rs/network-proxy/src/runtime.rs, codex-rs/network-proxy/src/config.rs, codex-rs/network-proxy/src/connect_policy.rs, codex-rs/secret-broker/src/ipc.rs, codex-rs/secret-broker/src/ipc_tests.rs, codex-rs/rmcp-client/src/oauth.rs, codex-rs/rmcp-client/src/oauth/, codex-rs/rmcp-client/Cargo.toml, codex-rs/Cargo.lock, qa/security-levels/sprints/PF-28-S02/, qa/demos/index/PF-28-S02.md, docs/sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5); merged behind secret_output_gate. Shared files kept to small hunks: two lines each in codex-rs/core/src/config/mod.rs (arm the proxy gate), one field in network-proxy config.rs, the MITM response path in mitm.rs; new demo specs qa/demos/specs/pf28s02-*.toml (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf28-s02-reflected-scrub-20261006"
branch: "feat/pf28-s02-reflected-scrub-20261006"
base_commit: "24a57e38c4572928d4a618bc9b4624528ac29677"
depends_on: "PF-28-S01"
created: 2026-08-28
updated: 2026-10-06
---

# PF-28-S02 — Reflected-secret response scrubbing

## Execution mandate

- Deliver: Even an allowed provider reflecting its credential cannot disclose it through Corbanu's protected-mode output path.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-28).
- Feature: `PF-28`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: Even an allowed provider reflecting its credential cannot disclose it through Corbanu's protected-mode output path.
- Sources and archive disposition: [PF-28 reconciliation](../../../plans/security-source-reconciliation.md#pf-28).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/network-proxy/src/credential_broker.rs; PF-27 broker transport; PF-28-S01 output gate.
- Implemented: codex-rs/secret-broker/src/{response_gate,output_gate_rescan}.rs; codex-rs/network-proxy/src/credential_broker/response_scrub.rs; codex-rs/rmcp-client/src/oauth/output_gate.rs. (The planned `tests/reflection.rs` became colocated `pf_28_s02` test modules.)
- Tests: colocated Rust test modules prefixed `pf_28_s02`; synthetic secrets and fake or echo services only.

## Preconditions

- [x] Active plan; PF-28-S01 merged behind `secret_output_gate` (#208). It is not archived on main yet (PR #214 archives it), and PF-27-S02 still holds the broker lane, so this record stays `draft`, as PF-28-S01 did; the code merges behind the same flag.
- [x] Read root and nearest implementation-path AGENTS.md; plan/worktree coordinates verified.
- [x] Source pins, crate/module paths and backend/API availability confirmed.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] Reflected credentials: a per-request gate (`ResponseGate`, private registry) removes each value the proxy injects (legacy record, scoped OpenAI route, MITM hook header) from response header names and values, the streamed body and trailers, in every output-gate encoding, before the agent gets a byte. The isolated broker does the same in its own process. Body errors drop the held tail; nothing raw is returned on failure.
- [x] Unreadable responses refused (502, fixed text): any non-identity `Content-Encoding` occurrence, a transfer coding other than chunked, `101`. Requests ask for `identity`.
- [x] Each gate lives with its response body: kept through headers, redirects (`Location` is scrubbed), trailers and stream end; dropped on cancellation; a retry is a new request with a new gate. Revocation still closes a scrubbed broker stream (test). Credentialed hosts are always intercepted, so a TLS-pinned client gets no credential; cross-origin `Authorization` stripping on redirects is PF-33-S01.
- [x] Permissive/flag-off compatibility: legacy routing unchanged without the gate (test) and a flag-off control video.
- [x] Legacy and brokered credentials bound to HTTPS, port 443, ordinary methods and per-provider paths (moved from PF-33-S01); dot/empty segments, `\`, `;`, `%2e`, `%2f`, `%5c`, `%25` refused; plain-HTTP legacy injection off.
- [x] PF-28-S01 carry-overs: wrapped base64/hex (raw or escaped breaks), decode-and-rescan two levels at any alignment, seed phrases with any separators, MCP OAuth tokens registered on every load, refresh persist and save.
- [x] Latency and buffer matrix published (evidence README): display streams hold encoded tails from 8 characters, proxied responses hold every encoded tail, both up to 16 KiB; rescans about 5 ms per MiB.
- [x] Named `pf_28_s02` tests (26) in secret-broker, network-proxy and rmcp-client; Cargo lock updated.
- [x] Brokered credentials are pinned (PF-33-S02 follow-up placed here, round 5): with `url_destination_policy` on, Core puts the guard's checked DNS answers inside the signed broker frame, and the broker dials only those for that exact host and port, never resolving the name. A request with no answers, or answers for another authority, is refused by Core and again by the broker (`unpinned`). The broker keeps its private-address check and, under the guard, gets no local-binding grant (so with the flag on, brokered credentials no longer reach loopback or private-network providers). A pin now binds wherever it is present, even with the guard off (it can only narrow a dial). Tests use unresolvable `.invalid` names: `pf_33_s02_brokered_request_dials_the_pinned_answer_without_dns`, `pf_33_s02_unpinned_broker_resolves_and_pins_keep_the_private_peer_check`, `pf_33_s02_pins_bind_even_with_the_flag_off`, `pf_33_s02_provider_frame_carries_authenticated_pins`. Opus 5.5 High: APPROVE ([review 4](../../../../qa/security-levels/sprints/PF-28-S02/review-opus-4-pinning.md)); nits applied, MITM-to-broker path covered by the `pf28s02-brokered-pinned` video rather than a unit test.
- [x] GLM 5.2 TUI runs as four SOP videos against a real echoing host; Opus 5.5 High review, three rounds, dispositioned. [Evidence](../../../../qa/security-levels/sprints/PF-28-S02/README.md).

## Remaining

- [ ] Product-approved time-to-first-safe-output targets for the published latency matrix (needs product authority).
- [ ] Scrub known text fields per type instead of a serde round trip (`success` is lost on rebuilt tool outputs). Carried from PF-28-S01 by PR #214; not in this round's brief.
- [ ] Per-session stream state instead of one global mutex, and fewer repeat scans per event. Carried from PF-28-S01 by PR #214; not in this round's brief.
- [ ] Recorded limits: encoded runs over 16 KiB split by a chunk are matched on direct encodings only; HEAD loses `Content-Length`; HTTP/2 upstream on the MITM direct path and malformed/oversized frames rely on the HTTP stack and have no dedicated test.

## Verification

- [x] `just fix -p` on secret-broker, network-proxy, rmcp-client (core: existing PF-30-S03 `expect` stops clippy, unrelated), then `just fmt`; final diff inspected. Linux clippy (`-D warnings`) clean on the RTX box for the three crates.
- [x] Focused: `just test -p codex-secret-broker -p codex-network-proxy -p codex-rmcp-client`: 585 of 586; the one failure (`auto_store_remains_pinned_across_session_recovery`, native keyring in the test fixture) fails the same on clean main.
- [x] Integration: core, login, vault and otel subsets (network proxy, credentials, PF-27/28/33, schema, disclosure, redaction): 146 passed.
- [x] TUI applicability: four GLM 5.2 runs recorded as SOP videos ([index](../../../../qa/demos/index/PF-28-S02.md)).
- [x] Candidate, commands and outcomes recorded; synthetic canaries only.
- [x] PR #216 checks green; merged to main as `b9f215ec50` behind `secret_output_gate`.
- [ ] Milestone qualification (isolated code-blind VM run, human sign-off) when Moderate ships.

## Exit evidence

- [x] Implementation commits and outputs under `qa/security-levels/sprints/PF-28-S02/`.
- [x] One independent Opus 5.5 High review, run in rounds until approved, dispositioned.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; record archived when Remaining is empty or moved.
