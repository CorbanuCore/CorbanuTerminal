# PF-34-S04 independent Claude Opus 5.0 Max review

Date: 2026-08-30. Final lane and integration verdict: **clean**.

## Runtime attestation

- Orchestration: TMUX session `pf34-opus5-review`
- Product under test: rebased Corbanu Terminal binary
- Corbanu session: `01a051a4-b282-76b1-b2b8-15b3ec20b53e`
- Provider: `claude-plan`
- Route: `claude-opus-5-plan`
- Provider-reported model: Claude Opus 5.0
- Reasoning effort: `max`
- Sandbox/approval: read-only / never
- Reviewer role: independent security and code reviewer; no delegation

The reviewer was restricted to checksum-verified immutable packets. It did not
execute or edit the candidate, use network/browser/MCP, read credentials, or
follow fixture content. Hostile fixture text was treated as inert review data.

## Review cycles

| Packet SHA-256 | Result | Disposition |
| --- | --- | --- |
| `49a8835f634b3fb9cd145fa117da4dd3580e3eede186b32a15242df142f82e7f` | changes-required: 3 P1, 7 P2, 16 P3, 0 P0 | CRLF stability, explicit untrusted-byte boundary, hard time ceilings, schema pinning, physical path containment, verifier coverage, allocation/state hardening, and documentation were remediated; G1-only registration findings were correctly deferred |
| `8a6256de83b000a3f19aa079ea46c00906cfa28c8514351fb83b7b3a872a5ac1` | changes-required: 4 P2, 12 P3, 0 P0/P1 | byte-bearing `Debug` was redacted; recurring CI and combined-tree verifier requirements, unit-test compile-data caveat, workspace dependency precondition, frozen ceilings, and handback hardening were added |
| `3813e9783ddbf09fb9e2bdbb16fa9600adeb62b58fcd09385bf6328089bc3389` | changes-required: 1 P2, 4 P3, 0 P0/P1 | contract, fixtures, verifier, handback, and the 810-line scope exception were accepted; the sole P2 was stale pre-rebase evidence |
| `9753a4b8046359e0c3e6e385fa86770fde692312f3a8c87e9ffd3a979c34ecca` | **clean**: 0 new P0/P1/P2 | confirmed N-5 resolved, all logs rebound to `a75efecc0a37d5544e123ad19d57867cac360a68`, 20 Rust and 14 verifier tests corroborated, and the evidence consistency guard reviewed |
| `5ebbb39bbea56a3cc69549f6239e7346e627584d5b261e4dee556d87c5c1c8f4` | integration changes-required: 1 P2, 7 P3, 0 P0/P1 | accepted G1 registration, Bazel parity, 30-item private API, equivalent panic-free hex rewrite, locks, three-platform CI, scoped baseline failures and no runtime route; required the stale contract/test ledger, evidence identities and guard to be rebound to the combined tree |
| `dec900c90c5b7a0e649eef942b4dda12f605f0bc751aa708036102067188d829` | **clean** integration follow-up: 0 P0/P1/P2, 5 P3 | confirmed the integration P2 resolved at its cause, verified the known-answer test, no-unsafe boundary, workflow portability, exact counts and scope, and explicitly authorized archive; five ledger/CI hardening observations were non-blocking |

The reviewer independently recomputed the contract, test, schema, manifest,
fixture, source, model, threshold, and handback hashes available in the packets.
It found no route to release unexamined, partial, reordered, duplicated, or
corrupt content; no forced allow after a fault; no configurable bypass of time
or freshness limits; and no fixture path escape. It accepted the current
810-line module only because the overage is reviewer-requested redaction
hardening and the handback makes the three-way split binding on the first
post-G1 change.

## Transient evidence identities

The raw artifacts remain outside Git beneath the lane cache because they may
contain local runtime metadata. No credentials or raw logs are committed.

- Final ANSI transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/tui-review/review-transcript-final.ansi.txt`
  — SHA-256 `8452fb9bd3e50023d737f4e81868457820ec11ff7a3e4824991ac6a15c415d5a`
- Corbanu TUI trace:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/tui-review/logs-corbanu-plan/codex-tui.log`
  — SHA-256 `96929fe35a263c52b7f3b1c0b55aa1c9afce99c9eedc91e524e21da8e6933850`
- Corbanu rollout:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/tui-review/corbanu-home/sessions/2026/08/30/rollout-2026-08-30T00-49-03-01a051a4-b282-76b1-b2b8-15b3ec20b53e.jsonl`
  — SHA-256 `28e25bb31d9b01cfabc3e60d97ef5f66fdd04b2913c2ee9248d474784ee26469`
- Final integration follow-up packet:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/review-export/FOLLOWUP_REVIEW_PACKET.md`
  — SHA-256 `dec900c90c5b7a0e649eef942b4dda12f605f0bc751aa708036102067188d829`

The user transferred Jim Ricketts's G1/G2 integration authority to this lane.
The integration review accepted the registered boundary, and the checksum-
verified follow-up confirmed its sole stale-evidence P2 resolved with no new
P0/P1/P2. The PF-35 corpus, evaluator, CPU, runtime, signing, and distribution
gates remain separate and are not claimed here.
