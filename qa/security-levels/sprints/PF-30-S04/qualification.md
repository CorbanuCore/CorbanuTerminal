# PF-30-S04 qualification (in progress)

Allocation: `db141e9cb`; source base: `526926934`.
Branch: `feat/security-memory-dispatch`.
Remote authoring mirror: `/home/travis/worktrees/security-memory-dispatch`.
Build evidence: `/home/travis/security-round5/evidence/memory/` on RTX100.99.88.49.
No local build, protected-memory activation, credential changes or privileged setup.

## Reviewable implementation stages

The whole boundary must qualify together. The smallest coherent intermediate
stage is `3b9058370`: host-owned opaque facade plus dormant transport guard,
without routing the worker into it. `51d4ce975` connects the worker and failure
persistence. Later checkpoints add the canary/host/TMUX matrix and per-job current
routing. These are recovery checkpoints, not separate completed features.
The combined diff exceeds the ordinary 800-line guidance once formatted because
it includes the cross-crate dispatch boundary and its independent test fixtures;
production code and tests are separated for review. No context fragment, prompt,
output schema or cache-key format is added or rewritten.

## Contract

- A real owning thread creates an opaque client; owner/provider arguments are
  assertions, not authority selectors. No public constructor, setter, clone or
  inner-client accessor is exposed.
- The configured/current/live/inherited floor is conservative. Every protected
  raw-rollout request denies, even if other source-screening code is present.
- Each lower HTTP attempt rechecks after async auth/backoff; WebSocket checks
  after connection immediately before the first/next request frame.
- The binding becomes unusable after a denial, owner termination or provider
  replacement. A new job/startup refreshes the host routing and must obtain a new
  binding; a race invalidates the snapshot rather than redirecting its request.
- Completed response is required. EOF, cancelled owner and typed denial cannot
  produce successful output or successful no-output persistence. Policy denial
  uses existing finite job backoff and stops this startup before phase two.
- Cancellation means dropping the extraction future or terminating its owner;
  this slice does not change the existing foreground-turn interrupt contract.
  It cannot retract a request already sent or promise atomic filesystem/DB
  rollback after the final persistence check.
- Historical rollout IDs remain data, not runtime capability selectors. This is
  not persisted historical-policy lineage, protected memory reading, positive
  screening, phase-two qualification or complete memory security.

## Evidence so far

- `3b9058370`: RTX `cargo check -p codex-core` passed in 40.65s;
  `check-stage1.log`.
- `51d4ce975`: RTX worker check passed; initial six Core binding tests passed,
  nextest `b585fa5e-69fb-448d-be63-1aa00fdfac93`, `check-worker.log`.
- Intermediate `just fix` at `b3c92c094` stopped on a fixture-only missing direct
  reqwest dependency. Corrected by using the existing route-aware HTTP factory;
  no dependency added. This is not final qualification evidence.

## Required final matrix

- [ ] Configured Moderate/Aggressive, inherited stronger floor and live increase.
- [ ] Missing policy, wrong owner, terminated/dropped owner, live kill switch.
- [ ] Same owner exact provider replacement; new-job current routing.
- [ ] HTTP execute/stream attempt barrier and WebSocket connect-time barrier.
- [ ] Real worker synthetic output persistence in Permissive; zero protected
      canary requests and outputs; finite retry backoff; EOF cannot save JSON.
- [ ] Actual-key TMUX memory-enabled startup, foreground turn and exit/restart
      with isolated synthetic fixtures and recorded DB/provider outcomes.
- [ ] Final scoped fix, full formatter, Core/memories-write/memories-read tests.
- [ ] Astra High then Fable5.1High through Corbanu/private TMUX, parent allocated.
- [ ] Exact immutable RTX candidate, hash, run IDs and capture paths.
- [ ] Combined integration/governance and human handoff; human acceptance pending.

## Review ledger

0 of maximum 5 invocations used. No independent review has been requested or run.
Five is a ceiling, not a target. Parent allocates each numbered invocation.
