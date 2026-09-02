---
title: "Corbanu Plan large-image requests"
status: active
change_class: product-initiative
priority: "P1"
owner: "Jim Ricketts"
parallel_sprint_limit: 1
integration_owner: "Jim Ricketts"
activation_authority: "User-directed product decision recorded in the 2026-08-30 task"
activation_basis: "Increase the authenticated Corbanu Plan inference-body limit to 30 MiB so large image requests no longer fail at the gateway."
target_release: "TBD"
deadline: "2026-08-31"
created: 2026-08-30
updated: 2026-08-30
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Corbanu Plan — LIVE"
  requirement_excerpt: "Customers authenticate to Corbanu Plan, not directly to xAPI."
implementation_worktrees:
  - path: "/Volumes/CorbanuDrive/Corbanu/CorbanuPlan"
    branch: "fix/30mb-inference-body-limit"
    base_commit: "e333167b7a98c26053ef76fe62070da2edc83285"
  - path: "/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal"
    branch: "main"
    base_commit: "9ccecfc4d2c3c47b7aeea90bdb4889cae5bfe4d6"
---

# Corbanu Plan large-image requests

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | active |
| Active-plan slot | 2 of 2 |
| Product authority | User-directed product decision recorded in the 2026-08-30 task |
| Authoritative decision | Accept the authenticated-service resource tradeoff and raise the inference-body cap from 2 MiB to 30 MiB. |
| Merge decision | On 2026-08-30 the task owner approved merging the trivial gateway fix without the remaining Isometric Game, quiet-host full-suite, or named-human qualifications. This waives those checks for merge only, not for deployment/release readiness. |
| Target release | TBD |
| Deadline | 2026-08-31 |

## User pain

An image whose base64-encoded Anthropic request exceeds 2 MiB is rejected by
Corbanu Plan before it reaches Fable. The terminal retries against a 15 MB
budget, which does not reduce common 2–15 MB requests, so the turn ends with an
HTTP 413 instead of analyzing the image.

## Product intent and ideal flow

An authenticated Plan customer attaches a normal large image and submits it to
Fable. Requests up to 30 MiB pass through the existing authorization, model,
billing and upstream-routing controls unchanged. Requests above 30 MiB fail
before reservation with a precise 413, and a smaller follow-up succeeds with
the same key. No payment, credential, privacy-label or model-routing behavior
changes.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | Corbanu Plan — LIVE |
| Requirement excerpt | “Customers authenticate to Corbanu Plan, not directly to xAPI.” |
| Product outcome advanced | Shipping Plan inference accepts image-bearing Anthropic requests within the terminal's existing 30 MB budget. |
| North-star criterion advanced | Reliable authenticated inference for trader research inputs. |

## Scope

### In

- Raise the JSON inference-body cap for authenticated chat and Messages routes to 30 MiB.
- Preserve a hard cap and return an accurate 413 above it.
- Add regression coverage for a Messages image body above the previous 2 MiB limit and for the new upper bound.
- Update the private service's documented security property.
- Add a typed tmux-harness paste input and exercise image attachment through the real TUI against a deterministic local Fable boundary.

### Out

- Client-side image transcoding, remote image uploads or durable image storage.
- Changes to authentication, billing, model routing, privacy labels or upstream credentials.
- Production deployment or release publication without separate operator action.

## Invariants

- Only JSON requests with valid customer keys reach the enlarged body parser.
- Oversized requests are rejected before usage reservation and upstream dispatch.
- The upstream operator credential remains server-side and customer headers are not forwarded.
- The cap remains finite and matches its documented/error value.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | `/Volumes/CorbanuDrive/Corbanu/CorbanuPlan` | `fix/30mb-inference-body-limit` | `e333167b7a98c26053ef76fe62070da2edc83285` | Gateway cap, regression tests and private-service documentation |
| Codex — Tmux qualification lane | `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal` | `main` | `9ccecfc4d2c3c47b7aeea90bdb4889cae5bfe4d6` | Typed paste input, image-attachment TUI scenario and harness documentation |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `CorbanuPlan/src/app.ts::MAX_PROXY_BODY_BYTES` | Owns the deployed Express raw-body limit and 413 response. |
| `CorbanuPlan/tests/app.test.ts` | Exercises the authenticated chat-wire cap. |
| `CorbanuPlan/tests/xapi-routing.test.ts` | Exercises the Fable `/v1/messages` path that reported the bug. |
| `codex-rs/model-provider-info/src/lib.rs::ProviderRuntimePolicy` | Terminal currently budgets 30,000,000 bytes for initial Anthropic requests. |
| `codex-rs/tui/tests/support/tmux.rs::TmuxPane` | Owns typed key, literal-text and new bracketed-paste injection. |
| `codex-rs/tui/tests/suite/large_image.rs` | Exercises a real composer image attachment and Fable response inside tmux. |

## Sprint execution map

| Feature ID | Current sprint records | Completion evidence |
| --- | --- | --- |
| PF-42 | PF-42-S01 | private commit `35698c4d5d5ef8c596041450fb415a699a39a5e9`; final review clean |
| PF-43 | [PF-43-S01](../../sprints/archive/corbanu-plan-large-image-requests/pf-43-s01-tmux-image-attachment.md) | uncommitted public final tree; final Opus 5 Max review clean; zero-retry tmux qualification passed |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Primary success | Valid Plan key and configured Fable route | Submit a valid image-bearing Messages request larger than 2 MiB and no larger than 30 MiB | Fable request streams normally without a gateway 413 | Request reaches the SkyAPI upstream and returns success. |
| Failure/cancel | Valid Plan key | Submit JSON above 30 MiB | Precise HTTP 413 with the 30 MiB limit | No upstream dispatch or usage reservation occurs. |
| Recovery/resume | Prior oversized request was rejected | Submit a smaller request with the same key | Normal inference resumes | Follow-up succeeds without key or account repair. |

## Implementation sequence

1. Freeze the 30 MiB constant and accurate error contract in the private gateway.
2. Add authenticated chat and Fable Messages regression coverage around the old and new limits.
3. Run formatting-equivalent checks, typecheck, tests and build; record remaining deployment/TUI gates.
4. Add a typed bracketed-paste action to the tmux harness and run the image success and recovery workflow through the real TUI.

## Automated evidence

Run checks on the final private-service tree.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Focused | Bundled Node + focused body-cap, account-cap and image-routing selectors | pass, 3/3 | terminal output, 2026-08-30 |
| Integration | Bundled Node + `tsx --test --test-concurrency=1 tests/*.test.ts` | pass, 65/65; four environment-dependent suites were not exercised | terminal output, 2026-08-30 |
| Type/build | Bundled Node + `tsc -p tsconfig.json --noEmit`; `tsc -p tsconfig.build.json` | build pass; typecheck blocked by unchanged baseline errors in `tests/store.test.ts` and existing indexed access in `tests/xapi-routing.test.ts` | terminal output, 2026-08-30 |
| Governance | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | pass; 2/2 active plans, 59 current and 98 archived sprints | terminal output, 2026-08-30 |
| Review | Corbanu Terminal 0.1.35 in tmux; `claude-opus-5-plan` at `max` | corrected pass clean; no in-scope actionable findings | private session and trace log, 2026-08-30 |
| Harness support | `CORBANU_TMUX_REQUIRED=1 cargo nextest run -p codex-tui --test all paste_input --retries 0` | pass, 2/2 | terminal output, 2026-08-30 |
| True TUI | `CORBANU_TMUX_REQUIRED=1 cargo nextest run -p codex-tui --test all tmux_plan_fable_large_image_succeeds_and_recovers --retries 0` | pass, 1/1 in 5.977s | successful scenario emitted no failure bundle |
| Existing tmux smoke | `CORBANU_TMUX_REQUIRED=1 cargo nextest run -p codex-tui --test all tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly --retries 0` | pass, 1/1 in 1.326s | terminal output, 2026-08-30 |
| Public full workspace | `just test` | non-diagnostic under host contention: 3,954 passed, 24 unrelated failures, 2 timeouts, 12,040 interrupted after 306.591s | keychain lookup, loopback listener and app-server startup failures; no image/tmux failure |
| Harness review | Corbanu Terminal 0.1.35 in tmux; `claude-opus-5-plan` at `max` | corrected pass `CLEAN`; no remaining actionable findings | `.codex-work/pf43-review/logs/codex-tui.log`, 2026-08-30 |

## True-TUI evidence

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Primary | `corbanu 0.1.35`, uncommitted PF-43 final tree | Corbanu Terminal worktree + local Wiremock Fable boundary | Bracketed-paste a deterministic 800×800 PNG, type prompt, send Enter separately | `[Image #1]`, then `LARGE_IMAGE_ACCEPTED`; captured `/messages` body and image data both exceed 2 MiB | pass | zero-retry selector; successful scenario emitted no failure bundle |
| Failure/retry | `corbanu 0.1.35`, uncommitted PF-43 final tree | Corbanu Terminal worktree + local Wiremock Fable boundary | Attach smaller image; boundary returns one 413 and then success | `RECOVERED_AFTER_413`; exactly four total message calls and retry body is byte-identical | pass | zero-retry selector; successful scenario emitted no failure bundle |
| Failure/cancel | pending deployed candidate | Isometric Game disposable worktree | Submit over-limit fixture, then cancel/dismiss | Accurate bounded error; UI remains usable | pending | blocked on deployment and repository availability |
| Recovery/resume | pending deployed candidate | Isometric Game disposable worktree | Submit smaller image and Enter | Same session returns successfully | pending | blocked on deployment and repository availability |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | no | pending | pending | Transport limit is visual-input specific; release-wide qualification remains separate. |
| Isometric Game | yes | unavailable | unavailable | Documented `goodalexander/isometricgame` URL returned repository-not-found over authenticated `gh`; no local checkout exists. Live qualification remains a release gate. |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| pending | pending | pending | Large image success and smaller-request recovery | pending | pending |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `CorbanuPlan/README.md` security properties | This plan owns the product-spec citation | verified against private service branch |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| 30 MiB authenticated body cap | Product decision | Task owner | implementation | approved in this task |
| Private service repository access | Dependency | Jim Ricketts | implementation | available at `CorbanuCore/CorbanuPlan` |
| Production deployment credentials | Release dependency | Operator | live verification | not exercised by this implementation task |
| Isometric Game repository | Release dependency | Repository owner | live qualification | documented URL is unavailable to the authenticated account; restore access or update the canonical URL |
| Quiet full-suite host | Integration dependency | Jim Ricketts | final combined-tree verification | rerun `just test` without concurrent loader/keychain/loopback contention |
| Merge-only qualification waiver | Product decision | Task owner | mainline integration | approved 2026-08-30; does not convert missing hard release evidence into a pass |

## Release linkage

- Release record: `qa/release/<version>/`
- Benchmark tracker row: not due for this transport-only change
- Remaining blocker: deploy, Isometric Game live-image qualification, quiet-host full-suite pass and named human acceptance

## Completion

- [x] Product linkage, scope, invariants, and worktrees are current.
- [x] Every implementation unit is represented by a valid single-feature sprint.
- [ ] Required final-tree automated evidence passes; tests/build/governance pass, baseline test-typecheck debt remains.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [x] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
