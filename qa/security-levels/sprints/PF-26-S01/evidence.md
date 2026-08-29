# PF-26-S01 — early harness evidence

Status: **PF-26-S01 completed and archived**, 2026-08-27.
This is **not** completion of PF-26 or qualification of a product candidate.

## Authority and allocation

- Class: product initiative, PF-26-S01 of the active P0 `/security` plan.
- Product heading: **P0 `/security` levels**; excerpt: “Permissive preserves the
  shipping behavior and does not silently change existing policies.” Also
  **Required adversarial tests**: “Every critical attack-class regression passes
  and no critical finding remains open.”
- Activation: Travis Good, 2026-08-27, ensure PF-27 is fully committed/pushed,
  then complete PF-26. Only S01 has complete dependencies; no downstream native
  consumer or release gate is activated by this harness work.
- Owner: Jim Ricketts; implementation and automated checks: Codex.
- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf26-s01`.
- Branch: `codex/pf-26-security-harnesses`.
- Base and PF-27 contract commit: `cb808c30c0058c101597ab2ada3da16238565c5e`.
- PF-27 push verified twice with `git ls-remote`: remote
  `origin/codex/pf-27-shared-security-contracts` equals that full commit, and
  its implementation worktree is clean.
- Upstream: `openai/codex` commit
  `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, inherited without an upstream update.
- Code candidate: `bed9c5bfeece2414cbf7e3f54af09fcb646959ed`. [Source digests](code-sha256.txt) identify
  the harness, tests and versioned fixture data independently of evidence-only edits.

## Delivered contract

[Handoff instructions](../../fixtures/README.md) specify commands and schemas.
The fixture catalog maps ten source classes, 17 sinks, 18 scenarios, 124
source/level cases, 16 controls and all seven PF-27 adapter definitions. The
prepared standards crosswalk has 65 result slots, all pending. Stronger
assertions apply only to Moderate/Aggressive; Permissive retains its frozen
PF-21 baseline and five immutable probes covering ten surfaces.

Task integrity, policy and confidentiality have separate assertions. Tests cover
task hijacking, weakened tests, reviewer-output injection, benign controls,
forced detector misses, provenance/lineage, reflected errors/environment,
current epochs after revoke/restart/reconnect, duplicate fake actions, browser
containment/fallback and independent health. No real transaction is executed.

The loopback capture fixture accepts one synthetic provider-shaped request,
does not forward traffic, rejects duplicates/unsupported requests, scans known
canary encodings and cleans up its listener/threads. It is HTTP behind a future
test TLS terminator, not HTTPS or platform containment proof.

The checker rejects missing coverage, stale/mixed candidate identity, changed
artifact bytes, missing native expectations, failed assertions and synthetic
evidence submitted as qualification. Trusted native harnesses must collect and
retain raw proof; a model's verdict is not evidence, and hashes do not authenticate
a producer or prove the binary's source. PF-26-S04 owns that build/run audit.

## Upstream seams and pins

No `codex-rs/` code, manifests, dependencies, provider schemas, UI registrations,
or reconnect transport settings changed. PF-13 qualification remains isolated.
All 34 PF-27 source hashes and its exact contract selectors resolve at the pinned
historical commit. Those checks do not rerun or extend PF-27's Rust qualification.

The inherited plan's old `read_file.rs` and `core/src/memories/` paths are absent.
The catalog records a reserved PF-29 ingress owner and the current native memory
read boundary; the handoff flags native shell/unified-exec/exec-server file
ingress and extracted memory writes for PF-29's literal adapter inventory.
No absent adapter is marked supported, and no runtime boundary is guessed here.

| Frozen input | SHA-256 |
| --- | --- |
| PF-21 baseline | `45d1f2bd96733381638bb62961ee59fb1c026bc05a6a78d03b560cb794406b8d` |
| PF-27 adapter definitions | `0060f1f292177a3afac9e67b9c7f4cbdc5ea0765706aa949d6af29c6a541d004` |
| PF-26 pins manifest | `cfb85951b90e68fb92128b14b978178cb9d5b4b1bbf5d154d175ef32f617964f` |
| PF-26 fixture catalog | `4a232b44eb12d94de0304508766da0ed1ed0583c4daaaf553a304abfdd8b3cf3` |

## Commands and results

Host: macOS 15.6.1 (24G90), Python 3.12.5, Ruff 0.12.8. No additional packages
or Rust build storage were needed. Formatting preceded the affected tests.

| Check | Result |
| --- | --- |
| `ruff format scripts/security_level_*.py scripts/test_security_level_*.py` | complete; no behavior changes after final formatting |
| `ruff check scripts/security_level_*.py scripts/test_security_level_*.py` | passed |
| `python3 -m unittest discover -s scripts -p 'test_security_level_*.py'` | 39 passed; includes CLI preparation, missing adapter contract tests, and case-insensitive hex canaries |
| `python3 -m unittest discover -s scripts -p 'test_security_credential_canary.py'` | 6 existing tests passed |
| `python3 docs/plans/check.py` | passed, one active plan |
| `python3 docs/sprints/check.py` | passed; closeout has 23 current / 86 archived sprints |
| `git diff --check` | passed |
| `security-level-compat --prepare` with frozen baseline | five immutable probes / ten surfaces valid; product result pending |
| `security-level-adversarial --prepare` | 124 generated cases; product result pending |
| `security-level-standards-check --template` then `--check-plan` | 65 pending result slots; no qualification pass |

Retained outputs: [exact final commands/results](final-checks.json),
[compatibility preparation](compatibility-preparation/compatibility-report.json),
[pending crosswalk](crosswalk-pending.json). The temporary attack preparation
bundle was created at `/tmp/corbanu-pf26-harness.1o2LRy/attack-run`; its public
bundle digest was `0f416a1082dc8c8a6b21c9bc1cb5d600f752c7ff11bc0078c528a7c519543798`.
The temporary bundle and private synthetic canary were removed after the check;
neither is committed. Each new run creates
a different canary and run identity; use the catalog digest for fixture identity.

Autoreview: Codex `gpt-5.5`, high reasoning, local diff and `review-scope.md`,
with `--no-web-search`; first review identified a missing exact contract-test
requirement in adapter evidence. Accepted and fixed in scope, with a regression
for all seven adapters. The second review found lowercase-only hex detection;
case-insensitive hex matching now covers both sinks and request capture, with
uppercase/mixed-case regressions. Both findings were classified in scope;
final rerun exited 0 with no accepted/actionable findings. The clean result is
retained in [review-final.txt](review-final.txt) and [review-final.json](review-final.json).
Findings remain preserved in `review-initial.{txt,json}`
and `review-second.{txt,json}`. This is the development closeout review,
not the release's separately required named independent security acceptance.

## Applicability and remaining PF-26 work

- Rust suites: not rerun; no Rust or runtime behavior changed. Historical PF-27
  evidence is preserved, not promoted to a new final-candidate result.
- True-TUI/tmux: not applicable to this internal harness sprint. Loopback HTTP
  tests are not actual-key terminal proof. PF-26-S02 owns that proof, including
  the PF-13 credential boundary after its harness is merged and Ubuntu-green.
- Live repositories: no TensorCash or Isometric Game product workflow is
  exercised by Python fixture construction. Both remain required for release.
- Linux/Windows: no execution claim. The other machine still owns the pending
  Windows qualification; this work does not mark it passed.
- Human sign-off, independent security acceptance and release benchmarks:
  pending/not claimed. No release is authorized or published.
- PF-26-S04 still depends on PF-23-S02/S03, PF-25-S01/S02, PF-28-S01,
  PF-29-S02 and PF-30-S02. PF-26-S02 follows S04; S03 follows S02.
- Once S01 is archived, PF-30-S01 browser isolation and PF-29-S01 ingress can
  be separately allocated from this dependency-complete base under the plan's
  three-lane cap. They are not activated by this sprint's completion.
