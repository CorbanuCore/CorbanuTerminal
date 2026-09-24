# Corbanu Terminal 0.1.47 — authorized image-limit and loop-guard fixes

## Authority and scope

On 2026-09-24, after 0.1.46 was started, the requesting repository operator was
asked whether to cut 0.1.47 with the result-aware loop guard and answered “yes
except” the Anthropic image-size failure, which “is also broken in prod with
opus”. This is explicit human release authorization under the root AGENTS.md
release gate, with that defect in scope. The requesting operator is the release
owner.

Classification: **bounded fix**. Both changes keep existing routes and the
existing tool-call guard within documented provider limits and intended
behavior. No user goal, credential, financial, data or persistence boundary
changes. Product specification heading: **Shipping MVP — LIVE**, row
**Multi-provider inference**.

Branch: `release/corbanu-0.1.47`, cut from `release/corbanu-0.1.46`
(`46bddfa755`). It contains two fix commits plus this version bump; the same
commits are on PR 126.

## Defects fixed

1. **Sessions with many images failed on every Anthropic turn** (`cd5b16e4e3`).
   Anthropic rejects a request that carries more than 20 images if any image
   exceeds 2000 px on a side (“At least one of the image dimensions exceed max
   allowed size for many-image requests: 2000 pixels”). High-detail image
   preparation capped images at 2048 px, so a long Opus session with wide
   screenshots failed on every turn, including after “continue”.
   - High-detail preparation now caps the longer side at 2000 px. Anthropic
     downsamples past 1568 px, so no detail is lost. Resumed sessions re-prepare
     their history with the new limit.
   - Anthropic Messages requests (Claude Plan, Anthropic, Vercel Anthropic) now
     fit every inline image to the documented limits — 8000 px, or 2000 px once
     a request has more than 20 images — without mutating history. Only image
     headers are decoded to find oversized images. An image that cannot be
     resized becomes an explicit omission note.
2. **Tool-call loops that cycle or oscillate** (`6f119e60f0`). 0.1.46 refuses a
   call repeated more than three times in a row. A model can instead cycle
   through several calls with unchanged results, or toggle an edit back and forth
   and re-run the same test. Each tool result now has an identity without
   per-call metadata; a result the call has never returned before is progress.
   A call that has returned an already-seen result three times since the last
   progress is refused without running; the turn stops at the eighth refusal.

## Evidence

- New tests: header-only dimension reading (PNG, JPEG, WebP); Anthropic
  requests with 21 images capped at 2000 px; 20 images left unchanged; a single
  8100 px image fitted to 8000 px; undecodable images left for the provider;
  wide high-detail screenshots prepared at 2000 px; call cycles and oscillating
  results refused; polling tools exempt; end-to-end loop stop.
- Full `codex-core` and `codex-tools` nextest run on the loop-guard commits:
  3,614 tests. Every failure also fails with the fixes reverted or passes in
  isolation (environment-dependent suites, and tests contaminated by stray
  `.git`, `.codex` and `.agents` directories another test leaves in the test
  temporary directory).
- Replaying 84 captured benchmark traces through the loop guard: no refusals in
  the 77 normal runs; all 7 runaway runs caught.
- Live, GLM 5.3 Flash on Vercel, 16 runs each, before and after the guard:
  runaway runs 1 → 0; most expensive run $1.35 (571 requests) → $0.54; total
  $2.35 → $1.78. The guard fired in 2 runs. Hidden tests 139 → 132 of 152; the
  drop comes from runs where the guard did not fire.
- The Anthropic first-party many-image rejection could not be reproduced live
  here (the available direct key has no credit, and OpenRouter served Opus from
  hosts that accept 2048 px). The fix follows the provider's error text and
  documented limits.

## Disclosed gaps

- A session already open in a running 0.1.46 or older process keeps failing
  until Corbanu is restarted on 0.1.47 and the session is resumed.
- Candidate binaries used for the live loop runs were built locally; release
  binaries are built by the release workflow.
- Full workspace tests, cross-platform true-TUI qualification and the release
  suite in both default live repositories are not claimed.
- The 0.1.44 cancellation-recovery limitation is carried forward unchanged.
- The competitor/model benchmark cycle remains incomplete; see
  [benchmarks](benchmarks/README.md).
