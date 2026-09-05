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
- `d85a3515d` plus RTX formatter (synced back): scoped Core/memories-write fix and
  complete formatter passed. Core policy tests **10/10**, nextest
  `3b6a30c5-2f3f-494d-9943-34a1c6e0e3f0`; full memories-write **43/43**,
  `3194d479-9c70-4ae5-a72f-ad0798866e55`; full memories-read **3/3**,
  `a9480ea8-1f04-4fa1-8cc2-c957fddfc4ad`; `tests-4.log`.
  Tests include real pending-HTTP owner cancellation, post-connect WS policy
  change, live HTTP execute/stream barrier, inherited level, kill switch, provider
  replacement/new-job refresh, protected worker finite backoff and EOF rejection.
  Public Core integration and typed TMUX modules are authored but not yet
  registered by the coordinator; they are not counted among these passes.

## Required final matrix

Final registered qualification: Core **11/11** (including public host factory),
`9b8fdd5a-d123-427e-a4cc-723ddc827356`; memories-write **43/43**, final mixed
legacy/tool/forged-source run `2e111402-cd92-4239-b92b-044f43fe2125` after scoped
fix/full fmt; memories-read **3/3**, `7833ad45-658c-452e-9f86-713f26c6fe01`.

Actual-key TMUX **1/1 test containing four scenarios and four same-home restarts**:
`be38c029-38b9-41c2-917d-b572f1f4329a`, `tmux-qualified.log`, after TUI fix/full
fmt with a clean remote diff. Initial TUI failures were fixture eligibility
(missing preview, deliberately excluded by startup SQL); production was not
changed to admit empty-preview histories. Successful captures/key events/outcomes
are committed in `tmux/`; full synthetic traces remain under RTX
`candidate-72c210c5f/tmux-qualified/` and the preceding equivalent `tmux-final/`.

| Scenario | Raw-canary requests | Persisted stage-one outputs |
| --- | ---: | ---: |
| Permissive | 1 | 1 |
| Moderate | 0 | 0 |
| Aggressive | 0 | 0 |
| Exit during pending Permissive extraction | 1 | 0 |

The locked CLI was built from `72c210c5f` plus formatter. Later changes are tests,
captures and documentation only; the Core/worker runtime diff is empty. Immutable
RTX launch path: `/home/travis/security-round5/evidence/memory/candidate-72c210c5f/codex`.
SHA-256 before and after actual-key tests:
`75ea4bd96d56ad919b83add8275cf0d391ae8942c9fac3e33542e9c0e9365668`.
This is a Linux candidate, not a rebuilt Mac app shortcut.

- [x] Configured Moderate/Aggressive, inherited stronger floor and live increase.
- [x] Missing policy, wrong owner, terminated/dropped owner, live kill switch.
- [x] Same owner exact provider replacement; new-job current routing.
- [x] HTTP execute/stream attempt barrier and WebSocket connect-time barrier.
- [x] Real worker synthetic output persistence in Permissive; zero protected
      canary requests and outputs; finite retry backoff; EOF cannot save JSON.
- [x] Actual-key TMUX memory-enabled startup, foreground turn and exit/restart
      with isolated synthetic fixtures and recorded DB/provider outcomes.
- [x] Scoped fix, full formatter, Core/memories-write/memories-read tests on the
      authored runtime checkpoint; public integration/TMUX registration and
      final combined-tree rerun remain required.
- [ ] Astra High then Fable5.1High through Corbanu/private TMUX, parent allocated.
- [ ] Exact immutable RTX candidate, hash, run IDs and capture paths.
- [ ] Combined integration/governance and human handoff; human acceptance pending.

## Review ledger

0 of maximum 5 invocations used. No independent review has been requested or run.
Five is a ceiling, not a target. Parent allocates each numbered invocation.
