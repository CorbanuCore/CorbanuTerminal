---
title: "1. PF-13 security and protected credentials"
status: active
change_class: product-initiative
priority: P0
owner: "Jim Ricketts"
parallel_sprint_limit: 3
parallel_lanes: "broker, untrusted-content, tui"
integration_owner: "Codex /root security round-five coordinator"
activation_authority: "Product authority defined in the product specification"
activation_basis: "P0 sequencing plus Travis Good’s 2026-08-28 decision to reconcile the complete security program into this active plan; scope cut and lanes per Travis's 2026-10-06 decisions."
target_release: "TBD — candidate qualified by 2026-10-09"
deadline: 2026-10-09
created: 2026-08-23
updated: 2026-10-06
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged."
implementation_worktrees:
  - path: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
    branch: "feat/p0-security-levels"
    base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911"
    branch: "feat/security-broker-resume-20260911"
    base_commit: "d870c92dab2bf3fbb602dc3b8447fe9f3534aecb"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf27-s04-broker-20261006"
    branch: "feat/pf27-s04-broker-20261006"
    base_commit: "cb78550a31b1d6e5cab33ad27ef28a5ae4fa21b1"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf27-s02-secretless-20261006"
    branch: "feat/pf27-s02-secretless-20261006"
    base_commit: "13cf4a2d0c07046312dc6d32c757dce90e0b14fc"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s02-20261006"
    branch: "feat/pf-30-s02-persistent-taint"
    base_commit: "b96b23344ba68e8a484b5e68834b6a62e392e007"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s03b-20261006"
    branch: "feat/pf-30-s03-finish"
    base_commit: "38516a5b22e96336ae712e1a39889cc3818db466"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf23-s01-20261006"
    branch: "feat/pf-23-s01-slice2-3"
    base_commit: "8cf46179f569050bf066cc3367f593057023e178"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf13-s07-20261007"
    branch: "feat/pf-13-s07-qualification-20261007"
    base_commit: "64137b71894fb15fb9d6bf754dc69c41d4cb0406"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-provenance"
    branch: "feat/security-round5-provenance"
    base_commit: "07791288b6feeccfaee5a57c12452359cc666957"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-24-s03-security-picker"
    branch: "codex/pf-24-s03-security-picker"
    base_commit: "24242c1b0ee5388cbfbfe1b0d8f63459945b588d"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s01-20261006"
    branch: "feat/pf-33-s01-url-dns-redirect"
    base_commit: "a662c2ce357ee542fdac08ecaf083d27fd58391b"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s02-20261006"
    branch: "pf-33-s02-20261006"
    base_commit: "39c1f06213da3f15ec41cdd673a09a98cbc7d027"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-29-s01-20261006"
    branch: "pf-29-s01-20261006"
    base_commit: "4b42012daa8e532d7ae2f9a62b6829f55b044b0b"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-29-s02-20261006"
    branch: "pf-29-s02-20261006"
    base_commit: "72d9a5dfbf01370d481630742d038c317c2dd624"
---

# P0 `/security` levels

Policy: repository-root `AGENTS.md`. Plan lifecycle: `docs/plans/index.md`.
Everything written before 2026-10-06 (allocations, review ledgers, round notes,
expanded contracts) is in the [history file](../history/p0-security-levels-2026-10-06.md).
Deferred features are in the [P1 security hardening plan](../proposed/p1-security-hardening.md).

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active**, slot 1 of 3 |
| Authoritative decision | “Accountable sequencing,” item 1: `/security` is P0 and begins immediately |
| Binding scope decisions | Travis, 2026-10-06: PF-83 closes; flagged picker ships early; 20-sprint core; 3 lanes; tiered gate |
| Delivery owner / integration owner | Jim Ricketts / Codex /root security coordinator |
| Deadline | 2026-10-09 (Travis, 2026-10-06) |

## User pain

Corbanu has approvals, sandboxing, a vault, wallet scopes and tool permissions,
but they are spread across the product. A user cannot answer “How locked down
is my agent right now?”, and turning protection on must not silently change an
existing installation.

## Product intent and ideal flow

The user types `/security` and picks one of three levels. The current level is
obvious, a change shows its differences before confirmation, `Esc` changes
nothing, and only a human can change or downgrade the level.

| Level | User promise | How it ships |
| --- | --- | --- |
| **Permissive** | Exactly today's behaviour. No policy is added, removed or rewritten. | Default; the flag-off path |
| **Aggressive** | Sensitive access denied by default; narrow, expiring human grants. | First built only from existing controls (PF-24-S03), then deepened by PF-23-S02 and PF-25 |
| **Moderate** | Strong protection around untrusted content, secrets and protected actions while normal work continues. | Added to the picker only as its protections land (broker and untrusted-content lanes, PF-23-S01) |

## Product linkage

| Field | Value |
| --- | --- |
| Product-spec heading | **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.” |
| Credential heading | **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.” |
| Permission confirmation | **Permission selection confirmation — TO BUILD** — “A submitted selection is not a confirmed change.” (PF-83, closed) |

## Scope

In: the `/security` picker behind the `security_levels` feature flag; Aggressive
from existing controls; the broker, output gate, migration and destination
controls (PF-27, PF-28, PF-29, PF-33); durable provenance (PF-30); Moderate and
Aggressive enforcement (PF-23); confirm/downgrade, grants and kill switch TUI
(PF-24, PF-25); the inspector (PF-41-S01); final credential qualification (PF-13-S07).

Out: everything moved to the [P1 hardening plan](../proposed/p1-security-hardening.md)
(PF-26 final program qualification, PF-27-S06 Windows broker and launch, PF-31, PF-32, PF-34, PF-35, PF-36, PF-37,
PF-38, PF-39, PF-40, PF-41-S02); replacing `/permissions`; changing Permissive;
letting a model choose or downgrade a level; conformance claims to external standards.

## Invariants

- Permissive is current behaviour; flag off is byte-for-byte today's product.
- Only a human changes the level; no agent tool, prompt, config or project file can.
- Moderate and Aggressive are deterministic; model judgment can warn, never grant.
- Unknown or corrupt stored state fails visibly; it never becomes Permissive.
- Downgrades are explicit and invalidate incompatible pending authority.
- Child agents inherit the same or a stricter level.
- Protected levels keep managed secrets out of model, env, argv, logs and artifacts.
- The UI never shows a protection as active when it is not; unbuilt controls read “not available”.

## Ownership and implementation worktrees

Each lane worker creates its own worktree under `/Volumes/CorbanuDrive/Corbanu/worktrees/`
off `origin/main` and records it in front matter before its sprint becomes
`ready`. The coordinates above are the existing draft-record and paused PF-27-S04
/ PF-30-S01 worktrees. The integration owner serializes shared Cargo/Bazel/lock,
protocol and schema edits.

## Useful code references

| Path | Why |
| --- | --- |
| `codex-rs/tui/src/security/`, `tui/src/bottom_pane/security_view.rs` | Existing `/security` view (PF-24-S01) |
| `codex-rs/tui/src/chatwidget/permission_popups.rs`, `tui/src/app/permission_confirmation.rs` | Existing permission picker and PF-83 confirmation path |
| `codex-rs/features/src/lib.rs` | Feature flags; add `security_levels` |
| `codex-rs/core/src/security/` | Effective policy, transition, taint, protected surface |
| `codex-rs/secret-broker-service/`, `codex-rs/network-proxy/` | Broker (PF-27) and egress/destination policy (PF-33) |
| `codex-rs/execpolicy/` | Forbidden-command rules used by Aggressive vault denial |

## Sprint execution map

Twenty current sprints: the 19 named in decision 3 plus the picker. A lane is a
worker slot; a sprint still waits for its dependencies in any lane.

| Lane | Sprints in order |
| --- | --- |
| broker | PF-27-S04 (completed 2026-10-06, [archived](../../sprints/archive/p0-security-levels/pf-27-s04-isolated-credential-broker.md)) → PF-27-S02 (done, PR #191, [archived](../../sprints/archive/p0-security-levels/pf-27-s02-secretless-agent-launch.md)) → [PF-27-S05](../../sprints/current/p0-security-levels/pf-27-s05-model-client-auth-broker.md) (model-client auth, added 2026-10-06) → PF-28-S01 (done, PR #208, [archived](../../sprints/archive/p0-security-levels/pf-28-s01-central-secret-output-gate.md)) → [PF-28-S02](../../sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md); Windows follow-up [PF-27-S06](../../sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md) moved to the P1 hardening plan |
| untrusted content | [PF-30-S01](../../sprints/archive/p0-security-levels/pf-30-s01-typed-source-envelope.md) (done, PR #178) → [PF-30-S02](../../sprints/archive/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md) (done, PRs #190, #198) → [PF-30-S03](../../sprints/archive/p0-security-levels/pf-30-s03-post-taint-authority-checks.md) (done, PRs #204, #212) → [PF-23-S01](../../sprints/archive/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) (done, PRs #223, #233) → [PF-23-S02](../../sprints/archive/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md) (done) → [PF-23-S03](../../sprints/current/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) |
| tui | [PF-24-S03 flagged picker](../../sprints/archive/p0-security-levels/pf-24-s03-flagged-security-picker.md) (done, PR #186) → [PF-24-S02](../../sprints/current/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md) → [PF-25-S01](../../sprints/current/p0-security-levels/pf-25-s01-temporary-grant-tui.md) → [PF-25-S02](../../sprints/current/p0-security-levels/pf-25-s02-revocation-and-kill-switch-tui.md) |
| first free lane | PF-33-S01 (done, PR #210, [archived](../../sprints/archive/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md)) → [PF-33-S02](../../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md); [PF-29-S01](../../sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md) → [PF-29-S02](../../sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md) (after PF-28-S02) |
| convergence | [PF-41-S01](../../sprints/current/p0-security-levels/pf-41-s01-effective-security-inspector.md) (after PF-23-S03, PF-24-S02, PF-29-S02), then [PF-13-S07](../../sprints/current/p0-security-levels/pf-13-s07-integrated-credential-boundary-qualification.md) |

Cross-lane waits: PF-24-S02 needs PF-23-S03 and PF-29-S02; PF-25 needs PF-23-S02/S03;
PF-23-S01 needs PF-30-S03. The [sprint index](../../sprints/current/p0-security-levels/index.md)
lists exact dependencies. PF-41-S01 no longer waits on PF-32-S06, PF-37-S02 or
PF-40-S03; it shows those controls as “not available”.

### TUI lane bugs from PF-83

Handed to the TUI lane on 2026-10-06 by decision 1. Each is a bounded fix against
**Permission selection confirmation — TO BUILD** and runs under the per-sprint gate.

| Bug | Found in | State |
| --- | --- | --- |
| Typed text in an open approval prompt can approve it with “don't ask again” (`/permissions` hit the `p` shortcut) | Campaign 35, F05 | open; fix before Aggressive ships |
| “Enable full access?” defaults to Cancel and gives no message when cancelled | Campaign 35, F03 | open |
| A declined request renders both “You canceled…” and “Ran … (no output)” | Campaign 35, F05; PF83-DEF-015 | source fix pf83-unran-18 is on main; verify on the next candidate |
| `zai-anthropic` route: `glm-5.2` has no catalogued max-output limit, so every turn fails | Campaign 35, F11 | open (`core/src/client.rs` catalog) |

### Hosted non-security records

[PF-76-S01](../../sprints/current/p0-security-levels/pf-76-s01-provider-profile-persistence.md)
(provider profile persistence) and [PF-77-S01](../../sprints/current/p0-security-levels/pf-77-s01-tasknode-reliability.md)
(Task Node reliability) are drafts hosted here for history, outside the security
core and lanes. Completed hosted records PF-77-S02, PF-78-S01/S02 and PF-82-S01 are archived.

## Acceptance flows

| Flow | Pass criterion |
| --- | --- |
| Flag off | Product behaves and renders exactly as before; `/security` unchanged |
| Open and cancel | `Esc` closes the picker; no config, session, child or audit change |
| Select Aggressive | Differences shown first; after confirmation every listed control is enforced on the next turn, for children too, and survives restart |
| Return to Permissive | Prior settings restored exactly; incompatible pending approvals invalidated |
| Agent attempts a change | No agent-reachable path changes the level |
| Select Moderate (later) | Offered only when its protections are complete; unsupported routes fail visibly |
| Hostile content (Moderate) | No protected value or unauthorized action reaches model output or execution |
| Grants, revocation, kill switch | Narrow expiring grants; revocation and kill switch hold across restart |

## Implementation sequence

1. Picker (PF-24-S03) and the PF-83 TUI bugs → milestone **Aggressive ships**.
2. Broker and untrusted-content lanes in parallel; PF-33 and PF-29 take the first free lane.
3. PF-23-S01..S03, then PF-24-S02 and PF-25 → milestone **Moderate ships**.
4. PF-41-S01 and PF-13-S07 → milestone **flag removal**.

### Delivery gate

**Per sprint** (merge behind the flag when all pass):

- focused tests via `just test` on the final tree, after `just fmt` and `just fix -p <crate>`;
- a tmux functional test with real keys, product on GLM 5.2 (`-m glm-5.2 -c model_provider="zai"`);
- one independent review by an Opus 5.5 High subagent;
- short videos showing every core feature of the sprint end to end, recorded with
  `qa/demos/README.md` once it lands; until then an asciinema recording of the tmux
  session, keeping the `.cast` under `qa/security-levels/sprints/<sprint>/`.

**Milestones** (Aggressive ships, Moderate ships, flag removal): the full isolated
code-blind VM run under the root `AGENTS.md` rules, plus named human sign-off.
Up to three security sprints may be reserved at once, one per lane, with
disjoint `write_scope` (enforced by `docs/sprints/check.py`).

## Automated evidence

| Check | Command | Result |
| --- | --- | --- |
| Governance | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | per PR |
| Per sprint | `cd codex-rs && just test -p <affected-crate>` after `just fmt` / `just fix` | per sprint record |
| Permissive compatibility | PF-21 harness against the frozen baseline `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb` | pending, each milestone |

## True-TUI evidence

Per sprint: tmux run on GLM 5.2 plus `.cast` videos, recorded in the sprint record.
Per milestone: isolated code-blind execution with schema-2 receipts and a separate
evidence check ([workflow](../../../qa/code-blind-functional/README.md)).

## Live-repository applicability

| Repository | Applicable | Use |
| --- | --- | --- |
| TensorCash | yes | Permissive compatibility and Moderate protected-action workflow |
| Isometric Game | yes | Aggressive, inheritance, downgrade and recovery workflow |

## Human acceptance

| Milestone | Tester | Result |
| --- | --- | --- |
| Aggressive ships | named by release owner | pending |
| Moderate ships | named by release owner | pending |
| Flag removal | named by release owner | pending |

## Documentation

`docs/features/security.md`, `docs/slash_commands.md` and the vault/auth pages
describe only behaviour verified at a milestone, citing **P0 `/security` levels**.

## Dependencies, decisions, and blockers

| Item | State |
| --- | --- |
| PF-83 open items (designer packet, review count 7/5, process isolation) | Recorded in the archived [PF-83-S01](../../sprints/archive/p0-security-levels/pf-83-s01-permission-confirmation.md); not blocking the core |
| Aggressive vault denial from existing controls | PF-24-S03 must prove it; if impossible, stop and escalate |
| Aggressive sandbox design (A: `corbanu-aggressive` permission profile; B: literal `SandboxPolicy` table; C: `on-request` approvals) | Decided: A, the `corbanu-aggressive` profile as merged in #186 (Travis, 2026-10-07) |
| Global reservation cap | 3 security lanes + 1 accounting + 1 Task Node = 5 (`docs/sprints/index.md`); confirmed by Travis 2026-10-06 |
| October 8 deadline | Unchanged and not re-estimated against the 20-sprint core |

## Release linkage

Each milestone links its `qa/release/<version>/` record. Remaining blockers: the
20 core sprints, milestone code-blind runs and human sign-off.

## Completion

- [x] PF-83-S01 closed with per-case results; defects handed to the TUI lane (2026-10-06).
- [x] Deferred scope moved to the P1 hardening plan with dependencies intact (2026-10-06).
- [ ] Milestone: Aggressive ships.
- [ ] Milestone: Moderate ships.
- [ ] Milestone: flag removal; Permissive compatibility proven; no critical finding open.

## History

| Record | What it holds |
| --- | --- |
| [Plan history through 2026-10-06](../history/p0-security-levels-2026-10-06.md) | All earlier prose: allocations, PF-27 stage log, expanded contracts PF-27–41, standards profile, profile/failure matrix |
| [Source reconciliation](../security-source-reconciliation.md), [architecture refinements](../security-architecture-refinements-2026-08-28.md), [upstream reconciliation](../security-upstream-reconciliation-2026-08-28.md), [OpenClaw review](../openclaw-source-review-2026-08-28.md) | August 28 design inputs |
| [Completed archive](../../sprints/archive/p0-security-levels/) | PF-13-S01–S06, foundations PF-15, PF-16, PF-17, PF-18, PF-19, PF-20, PF-21, PF-22, PF-24-S01/S03, PF-26-S01, PF-27-S01/S02/S03/S04, PF-28-S01, PF-30-S01–S04, PF-31-S04, PF-33-S01/S03, PF-34-S04, PF-41-S03, PF-83-S01, plus hosted PF-77/78/82 |
