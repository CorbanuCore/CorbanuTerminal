# Browser/retrieval lane handoff

Owner: a separately named browser/retrieval agent; integration owner Jim
Ricketts controls shared surfaces and merges.

## Coordinates and authority gate

- Proposed worktree:
  `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-browser-retrieval`
- Proposed branch: `feat/p0-security-browser-retrieval`
- Parallel lane: `browser-retrieval`
- Build/cache root:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-browser-retrieval/`
- Dispatch base: the exact 40-character post-handoff `main` commit. The analysis
  baseline was `a753283f9cd1a59ff2ae3b03319c3c4a3264326f`; do not use it if main advances.

This document is not sprint authorization. Before implementation, update the
active plan and PF-31-S04 with a named owner, exact coordinates, literal scope,
and integration gate; set `ready` only after both governance checkers pass.

Product citation: **Reconciled security scope — TO BUILD** — “Unknown or
unsupported protected paths fail visibly rather than falling back to raw secrets
or unscreened execution.”

## Sequence and stop points

1. PF-31-S04 — retriever artifact and engine preparation.
2. Complete, integrate, archive, and freeze PF-31-S04 before starting PF-33-S03.
3. PF-33-S03 — pure destination-policy contract.
4. Complete, integrate, archive, and return the active slot. Wait for downstream
   foundation dependencies; preparation does not activate retrieval.

Downstream dependency chain:

```text
PF-33-S03 + PF-27-S02 -> PF-33-S01 -> PF-33-S02
PF-31-S04 + PF-33-S02 + PF-27-S02 -> PF-31-S01
PF-31-S01 + PF-30-S01 -> PF-31-S02
PF-31-S02 + PF-24-S01 -> PF-31-S03
```

Do not begin a downstream sprint until every dependency is completed and
archived and the integration owner reallocates the lane.

## Literal first-sprint scopes

PF-31-S04:

```text
scripts/security-retriever-artifact-check
qa/security-levels/retriever/artifact-manifest.json
qa/security-levels/retriever/engine-fixtures/
qa/security-levels/sprints/PF-31-S04/
docs/sprints/current/p0-security-levels/pf-31-s04-retriever-artifact-preparation.md
```

PF-33-S03, only after PF-31-S04 is archived:

```text
codex-rs/network-proxy/src/destination_contract.rs
codex-rs/network-proxy/tests/destination_contract.rs
qa/security-levels/sprints/PF-33-S03/
docs/sprints/current/p0-security-levels/pf-33-s03-destination-policy-contract.md
```

Exclude `codex-rs/network-proxy/src/lib.rs`, Cargo/Bazel manifests and locks,
Core/protocol registries, sandbox manager/spawn hooks, TUI roots, plan/index/nav,
and PF-13-S05 broker paths. Shared registration belongs to the integration owner.

## Deliverables

PF-31-S04 pins the current supported retriever image/artifact and runtime/browser
dependencies: exact digests, source/build lock, signatures, licenses, SBOM,
architectures, resources, update/rebuild policy, and three-platform identities.
Define safe existing Podman/Docker selection without replacing user choices or
touching unrelated resources. Implement idempotent fake-engine checks for absent,
stopped, stalled, mismatched, tampered, offline, wrong-architecture, duplicate,
and concurrent-client states. No automatic installation/elevation and no live
protected route.

PF-33-S03 freezes pure normalized scheme/host/port/method/path, DNS answer-set,
redirect, credential/body-replay, and private-destination decisions. Cover
absent versus empty, wildcard/public versus explicitly authorized private,
IDNA/userinfo/suffix/trailing-dot confusion, unusual IPv4, mapped IPv6,
mixed/private answers, downgrade redirects, and malformed policy. It opens no
socket and makes no SSRF-prevention claim until PF-33-S01/S02 qualify real peer
binding and alternate-egress resistance.

## Verification and review

For PF-31-S04 run manifest validation and fake-engine tests, with actual counts,
on Mac/Linux/Windows; record digests, licenses, SBOM, signatures, architecture,
disk, CPU, and RAM. For PF-33-S03 run deterministic table/property tests for all
policy polarities, normalization, redirect, replay, and malformed inputs. For
each sprint run fix/format, affected suites, both governance checkers, and
`git diff --check`; prove no preparation fixture creates a runtime route.

Follow the common Claude Opus 5.0 Max Computer Use protocol for every sprint and
combined-chain reviews after PF-33-S02 and PF-31-S03. Ask specifically about URL
ambiguity, SSRF, rebinding, connected-peer mismatch, redirect replay, stale
connection pools, proxy/`NO_PROXY`, raw sockets/UDP/QUIC, IPC/isolation, fallback,
fail-open behavior, taint loss, digest mutation, and Permissive regression.

## Blockers and handback

Retriever artifact/API pins, licensing, SBOM, signatures, platform artifacts,
and resource limits are unresolved long-lead decisions. Downstream foundation
dependencies are drafts. Stop rather than selecting unreviewed floating
dependencies or broadening runtime scope.

Hand back the candidate/base commits, exact scope audit, artifact/contract
versions and hashes, licenses/SBOM, commands and counts, three-platform matrix,
limitations, immutable review evidence, and recommended integration order. The
integration owner performs shared workspace edits, final-tree reruns, plan/nav
updates, and archive transitions.

