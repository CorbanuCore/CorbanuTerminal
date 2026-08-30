# Independent architectural review — Corbanu P0 `/security` program

| Field | Value |
| --- | --- |
| Reviewer model | Claude Opus 5, Extra effort (user-directed substitution for the Fable 5 / High requirement in `REVIEW_SCOPE.md`; recorded, not silently swapped) |
| Review date | 2026-08-28 |
| Corbanu base | `f173a0bc97c7495d134a67079aadfbe3657d11a7` (uncommitted planning tree included) |
| OpenClaw pin | `13adff02ca3897768d80d2bca18f5acf08c55d91` (MIT) |
| Input integrity | `shasum -a 256 -c FILES.sha256` run directly in this export, no pipeline: **127/127 OK** |
| Mode | Review only. No implementation, no input modification, no repository execution, no delegation, no access outside this export. |
| Artifact | This file is the only file written. |

A correction to my own earlier note in session: I initially said "129 checksums." `FILES.sha256`
contains **127** entries, matching `SNAPSHOT.json` `copiedFiles: 127`. All 127 verified OK.

---

## 1. Overall verdict

**The security content of this program is strong. The delivery structure is not viable as written,
and two architectural claims are asserted without the evidence that would make them true.**

Three things are genuinely good and unusual, and should be protected from deadline pressure
(detailed in §7): deterministic authority strictly separated from model judgment; fail-visible
degradation instead of silent downgrade; and an evidence culture that refuses to relabel reference
probes as product proof. The `qa/security-levels/planning/openclaw-2026-08-28/README.md` line
"10 passed observations — Includes confirmed limitations, not 10 Corbanu security passes" is the
kind of discipline most security programs lack.

Against that, four conclusions:

1. **The schedule is arithmetically impossible and the plan already knows it but has not acted.**
   The dependency graph in `docs/sprints/current/p0-security-levels/index.md` has a **34-stage
   critical path** across 63 sprints. Repository policy allows **one `in_progress` sprint per plan**
   (`AGENTS.md` "Sprint execution"; `docs/sprints/index.md` "Non-negotiable sprint shape"), which
   forces all 63 to run **strictly sequentially**. The deadline is 2026-10-08 — **41 calendar days**
   from the base date. That is 63 sequential sprints, several of which are multi-week research or
   packaging efforts (train a CPU classifier to ≤0.1% FPR on ≥100,000 held-out benign segments;
   build and three-OS-qualify a pinned container retriever), in 41 days. The plan states this
   honestly — "this larger program has not been effort-estimated or scheduled" — but records no
   decision. The product spec already defines the mitigation
   (`Risks, dependencies, and decision gates` → "`/security` controls miss 2026-10-08" → "issue a
   revised plan", owner: final product authority). **That mitigation needs invoking now, not in
   October.** (OR-01)

2. **The trusted computing base is asserted but never specified per operating system, and the
   product consequence of an unsupported OS has no recorded decision.** PF-27-S01's acceptance is
   "a compromised agent process cannot call an unrestricted resolver." Against same-user agent code
   this is not achievable by default on Linux, macOS, or Windows without named containment
   primitives that no sprint names, and on Windows it may not be achievable without a separate user
   account. Because Moderate and Aggressive are *mandatory-broker*, an unsupported platform means
   **those levels simply do not exist on that platform** — a product outcome, not an engineering
   detail. (OR-02)

3. **The user's stated goal that browser isolation be independently workable in parallel is not
   supported by the current graph, and the coupling is sequencing, not necessity.** PF-31-S01 sits
   at stage 14 behind the entire credential chain. The blocking edges are removable. (OR-06)

4. **Codex upstream — the fork parent, and the actual continuous integration risk — has no owner,
   no seam definition, and no re-qualification sprint.** The program has excellent discipline for
   the *OpenClaw reference* and almost none for *Codex upstream*, and it conflates the two.
   Meanwhile the 63 sprints modify precisely the highest-churn upstream files. (OR-04, OR-05)

I did **not** find fabricated vulnerabilities in current Corbanu code, and I do not assert any: no
Corbanu runtime is in this export (`corbanu-tracked-code-paths.txt` is a path list only). Findings
below are about the *plan and its adoption reference*, which is what is reviewable here.

On challenging the source review: I inspected 18 pinned OpenClaw files directly. **Every OC-1
through OC-11 claim I could check is accurate**, including the load-bearing OC-2 claim that the
revocation test only opens a *new* CONNECT (`proxy-server.test.ts:552-574` — confirmed verbatim).
I found four adoption hazards the review does **not** name, all verifiable at exact lines in this
export (OR-17 through OR-20).

---

## 2. Must fix before implementation

These change the plan or a product decision. Each is blocking in the sense that starting
implementation without resolving it wastes work or bakes in a wrong boundary.

---

### OR-01 — One-in-progress rule × 34-stage critical path × 41 days is not satisfiable

- **Severity:** Critical · **Confidence:** High · **Type:** Covered but unresolved
- **References:** `AGENTS.md` "Sprint execution" ("A plan has at most one `in_progress` sprint");
  `docs/sprints/index.md` "Non-negotiable sprint shape" (same, plus "executable dependencies must
  already be completed and archived"); `docs/sprints/current/p0-security-levels/index.md`
  (63 rows, full `Depends on` column); `docs/plans/active/p0-security-levels.md` front matter
  `deadline: 2026-10-08` and "Reconciled planning decision — 2026-08-28" ("has not been
  effort-estimated or scheduled"); `docs/corbanu-product-spec.md` "Accountable sequencing" row 1.
- **Affected sprints:** all 63.
- **Failure scenario:** Work starts on PF-15-S01. Because every other record is `draft` until its
  dependencies are "completed and archived," and only one sprint may be `in_progress`, the program
  executes as a strict chain. Around late September the team discovers that PF-35-S02 (train and
  package a reproducible CPU detector) cannot complete in days. The deadline is missed with
  Moderate/Aggressive partially built. Under gate pressure the tempting fix is to relax a control
  — exactly what the invariant "A partially implemented draft is not eligible to advertise a
  working protected mode" exists to prevent.
- **Evidence — critical path (my computation from the index `Depends on` column):** longest chain
  is 34 nodes:
  `PF-15-S01 → 16-S01 → 17-S01 → 19-S01 → 22-S01 → 13-S01 → 13-S02 → 13-S03 → 13-S04 → 27-S01 →
  27-S02 → 33-S01 → 33-S02 → 31-S01 → 31-S02 → 34-S01 → 35-S01 → 35-S02 → 35-S03 → 34-S02 →
  34-S03 → 32-S01 → 32-S02 → 32-S03/04/05 → 32-S06 → 39-S02 → 40-S01 → 40-S02 → 40-S03 → 41-S01 →
  41-S02 → 26-S01 → 26-S02 → 26-S03`.
  So even with **unlimited** parallelism the program is 34 stages deep. With the one-in-progress
  rule it is 63.
- **Recommended plan change (propose, do not enact):** Take three decisions together, through the
  product decision process, before PF-15-S01 starts:
  1. Amend the WIP rule to permit N concurrent `in_progress` sprints within one plan, scoped by
     named lane owner (§9 gives the lanes). This is a `AGENTS.md` + `docs/sprints/index.md`
     amendment, not a scope change.
  2. Apply the edge changes in OR-06/OR-07/OR-08 to cut the critical path from 34 to the low 20s.
  3. Adopt a scope split for the 2026-10-08 gate (§8, "smallest safe vertical slice") or invoke the
     product spec's own "issue a revised plan" mitigation. Do not carry an unassessed date.
- **Tests / expected outcomes:** Not a code test. The verifiable exit is a recorded product decision
  in the plan's "Dependencies, decisions, and blockers" table replacing the row
  "Expanded program capacity … effort estimate and October 8 feasibility pending" with a dated
  decision, a per-lane owner list, and either a revised gate or an explicit reduced v1 scope.

---

### OR-02 — The protected-process TCB is asserted but never specified per OS; the "no supported platform" product outcome is undecided

- **Severity:** Critical · **Confidence:** High · **Type:** Missing
- **References:** `pf-27-s01-isolated-credential-broker.md` "Execution mandate" ("a compromised
  agent process cannot call an unrestricted resolver"); `pf-27-s02-secretless-agent-launch.md`
  Remaining ("Enforce OS-level denial of vault/auth paths, broker memory/process handles,
  unrestricted IPC, host sockets, and secret-bearing mounts; network env hints alone are not
  containment" / "Publish actual backend capability probes for Linux/macOS/Windows");
  `docs/plans/active/p0-security-levels.md` "Dependencies, decisions, and blockers" →
  "Platform isolation capability matrix … Linux/macOS/Windows probes required" (state: pending);
  "Every level above Permissive requires the broker."
- **Affected sprints:** PF-27-S01, PF-27-S02, PF-13-S05, PF-31-S01, PF-23-S01, PF-24-S01, PF-41-S01.
- **Why this is the hardest question in the program:** the threat model is *same-user agent code*.
  A separate process at the same uid is not a boundary by default:
  - **Linux:** same-uid processes can `ptrace` and read `/proc/<pid>/mem` unless
    `kernel.yama.ptrace_scope` is raised, and a parent can trace its child regardless. Real
    containment needs a distinct uid, a user namespace, seccomp, or an LSM profile — all of which
    imply installation-time elevation.
  - **macOS:** `task_for_pid` against another process requires root, *or* the target must be
    debuggable. A signed, hardened-runtime binary without `get-task-allow` resists debugger attach,
    and Keychain ACLs bind to code signature — so containment is achievable **but only for signed,
    hardened, notarized builds**. A local dev build loses the property silently.
  - **Windows:** `OpenProcess(PROCESS_VM_READ)` against a same-user process succeeds by default.
    Protected Process Light is not available to general applications. Realistic containment needs a
    service under a separate account, i.e. installation-time elevation.
  None of these primitives is named in any sprint. PF-27-S02 says "publish probes" — correct shape,
  but a probe cannot be written without first deciding what "supported" means.
- **Failure scenario:** PF-27-S02 ships probes that report "contained" because a container engine is
  present and the environment was stripped, while on Windows any same-user process the agent spawns
  reads broker memory directly. PF-13-S05's canary sweep passes (it scans *sinks*, not process
  memory), and Moderate ships with a secretless guarantee it does not have. The plan's own
  "Bound the claim" invariant is violated in the direction that matters.
- **Recommended plan change:** Add to PF-27-S01 a required, product-reviewed **platform containment
  matrix**, decided *before* PF-27-S01 becomes `ready`, with one row per OS naming: the containment
  primitive, whether installation-time elevation is required, whether code signing is required, and
  the exact negative capability being claimed. Then add a product decision for the outcome
  "platform X supports no qualifying primitive" — the honest result is that `/security` shows
  Moderate and Aggressive as **unavailable on that platform**, which the `/security` tab, the
  product spec's level table, and PF-24-S01's readiness display must all be able to express today.
  Also: PF-27-S02's OS-denial list names "vault/auth paths" but **not the security-level state
  file** — see OR-03.
- **Tests / expected outcomes:**
  - `pf_27_s02_ptrace_containment` (Linux): agent-context process attempts `ptrace(PTRACE_ATTACH)`
    and `/proc/<broker-pid>/mem` read. **Expected:** both fail with EPERM; probe reports contained.
    On a host where they succeed, **expected:** protected-mode activation is refused with a named
    reason, and `/security` shows Moderate blocked — not degraded-but-green.
  - `pf_27_s02_task_for_pid_containment` (macOS): unsigned and hardened-signed broker variants.
    **Expected:** unsigned variant reports *not* contained and blocks activation.
  - `pf_27_s02_openprocess_containment` (Windows): agent-context `OpenProcess` with
    `PROCESS_VM_READ`. **Expected:** denied under the supported configuration; under a same-account
    configuration the probe reports not contained and activation is refused.
  - `pf_27_s02_canary_in_process_memory`: mint a canary into the broker, then have agent-context
    code scan every readable same-user process. **Expected:** canary absent, or activation refused.

---

### OR-03 — The security level is persisted where an agent with shell access can write it

- **Severity:** High · **Confidence:** Medium-High · **Type:** Missing
- **References:** `pf-20-s01-versioned-security-persistence.md` "Code boundaries"
  (`codex-rs/config/src/config_toml.rs`; `codex-rs/core/src/config/{mod,edit}.rs`);
  `pf-24-s02-security-confirm-cancel-and-downgrade.md` (`tui/src/app/config_persistence.rs`);
  `docs/plans/active/p0-security-levels.md` "Invariants" → "**Only a human changes the level.**
  There is no agent tool, prompt command, config instruction, or project-file mechanism that can do
  so"; `pf-27-s02` Remaining (OS-denial list covers "vault/auth paths", not the level state);
  `pf-23-s01-moderate-ingress-and-disclosure-enforcement.md` Remaining ("Deny … policy mutation").
- **Affected sprints:** PF-20-S01, PF-22-S01, PF-23-S01, PF-23-S03, PF-24-S02, PF-27-S02, PF-41-S02.
- **Failure scenario:** Corbanu is a coding harness; the agent has `exec`/`bash`. PF-23-S01 denies
  "policy mutation" at the **Core tool-dispatch boundary** by typed resource/action. A generic
  shell command — `sed -i 's/level = "aggressive"/level = "permissive"/' ~/.codex/config.toml`, or
  a Python one-liner, or a redirect — is not a typed policy-mutation call. It is an ordinary file
  write. Unless the protected-surface classifier knows that exact path *and* the exec path consults
  it, the level is downgraded on next restart, and PF-23-S03's "no transient Permissive fallback"
  logic faithfully restores the attacker's value because it is what is persisted. Indirect prompt
  injection reaches this: hostile page → agent writes config → restart → Permissive.
- **Missing vs covered:** The *invariant* is stated; the *mechanism* is absent. No sprint places the
  level outside agent-writable storage, and PF-27-S02's OS-denial enumeration omits it.
- **Recommended plan change:** Add to PF-20-S01 (and reflect in PF-22-S01/PF-23-S03) that the
  authoritative effective level lives in **broker-owned or OS-protected state**, with the TOML
  value treated as advisory and reconciled by compare-and-activate. The OpenClaw pattern is a good
  model here and is worth adopting deliberately:
  `openclaw/src/secrets/runtime-state.ts:1009-1019` `activateSecretsRuntimeSnapshotStateIfCurrent`
  refuses to activate a snapshot whose credential revision changed under it. Add the level state
  path to PF-27-S02's OS-denial list explicitly. Add tamper detection to PF-41-S02's chain.
- **Tests / expected outcomes:**
  - `pf_23_s01_level_file_write_via_exec`: in Moderate, agent runs a shell command that rewrites the
    persisted level to Permissive. **Expected:** write is denied *or* is inert — effective level is
    unchanged in-session **and after restart**; one secret-free audit event records an attempted
    policy mutation; `/security` shows a tamper indication.
  - `pf_23_s03_restart_after_offline_tamper`: modify the level file while Corbanu is stopped; start.
    **Expected:** mismatch between protected state and file is detected; the *stricter* value wins;
    startup does not silently adopt the weaker file value.
  - `pf_27_s02_level_state_path_denied`: agent-context open-for-write on the level state path.
    **Expected:** denied by OS policy on every supported platform.

---

### OR-04 — "Permissive is current behavior" is defined against a frozen commit while `main` keeps moving

- **Severity:** High · **Confidence:** High · **Type:** Missing
- **References:** `docs/plans/active/p0-security-levels.md` front matter
  `implementation_worktrees.base_commit: 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`;
  "Invariants" → "**Permissive is current behavior.** Its policy snapshot and representative
  workflows must match the pre-feature baseline"; `pf-21-s01-permissive-compatibility-baseline.md`
  Preconditions ("Baseline commit remains the plan's recorded pre-feature commit") and Verification
  (`--baseline 3c1b2f6c…`); `docs/corbanu-product-spec.md` "Accountable sequencing" row 11
  ("Upstream Codex parity | PRINCIPLE | CONTINUOUS").
- **Affected sprints:** PF-21-S01, PF-26-S01, PF-26-S02, PF-24-S01.
- **Failure scenario:** The program runs for months on `feat/p0-security-levels` while `main` takes
  Codex parity merges (an explicit continuous product principle). At PF-26 the harness compares the
  candidate against a commit that is now far behind. Every legitimate upstream behavior change shows
  up as a Permissive "regression." The team then either (a) burns the gate investigating false
  positives, or (b) — the dangerous outcome — starts updating expected values *from the candidate*,
  which PF-21-S01 explicitly forbids ("without rewriting expected behavior from the candidate") but
  which becomes irresistible when hundreds of diffs are upstream noise. Real Permissive regressions
  hide in that noise.
- **Recommended plan change:** Redefine Permissive compatibility as a **differential property at the
  current merge base**, not equality against an old commit: build the candidate tree twice at the
  same commit — once with the security feature compiled in and set to Permissive, once with it
  disabled/absent — and require the two to match. That property is stable under upstream churn,
  which is exactly what "Permissive preserves current behavior" actually means. Keep the frozen
  manifest as a secondary, informational drift signal. Record the chosen build seam (feature flag or
  merge-base rebuild) in PF-21-S01's code boundaries.
- **Tests / expected outcomes:**
  - `security-level-compat --mode differential`: same-commit feature-on-Permissive vs feature-off.
    **Expected:** empty policy/approval/tool/network/spawn diff. A non-empty diff is a hard failure
    and names the specific surface.
  - `pf_21_s01_upstream_merge_drift`: after a synthetic upstream merge that intentionally changes a
    Permissive-visible behavior, the frozen-manifest comparison flags it while the differential
    comparison stays clean. **Expected:** both signals present and distinguishable; the release gate
    keys on the differential result.

---

### OR-05 — Codex upstream has no integration seam definition, no re-qualification obligation, and no owner; the plan's upstream discipline is aimed at the wrong upstream

- **Severity:** High · **Confidence:** High · **Type:** Missing
- **References:** `docs/plans/openclaw-source-review-2026-08-28.md` "Adoption and upstream-upgrade
  discipline" (detailed rules — all about the *OpenClaw* pin); `docs/plans/active/p0-security-levels.md`
  "Reconciled planning decision" ("Keep the source checkout/reference outside the Corbanu runtime
  dependency graph… small Codex integration hook"); `docs/corbanu-product-spec.md` row 11 and
  "Risks" → "**Upstream Codex changes** … Continuous parity review with regression tests for
  Corbanu-specific layers | Lead developer".
- **Affected sprints:** effectively all; concentrated in PF-22-S01, PF-23-S01, PF-30-S01, PF-30-S02,
  PF-27-S02, PF-32-S02, PF-39-S02, PF-26-S01.
- **The conflation:** Corbanu is a **Codex fork**. OpenClaw is a **separate MIT reference** read for
  design. The program has rigorous rules for OpenClaw (pin, license, record differences, semantic
  re-inspection on change) and essentially none for Codex — which is the upstream that will actually
  merge into this branch during the program and after it.
- **The seam problem, concretely.** The sprints' declared code boundaries are diffuse in-place edits
  to the highest-churn upstream files, not thin call sites into Corbanu-owned modules:
  `core/src/tools/{router,registry}.rs`, `core/src/mcp_tool_call.rs`, `core/src/exec.rs`,
  `core/src/exec_env.rs`, `core/src/context_manager/{history,normalize,updates}.rs`,
  `core/src/agent/{control,registry}.rs`, `core/src/config/{mod,edit}.rs`, `protocol/src/models.rs`,
  `tui/src/chatwidget/slash_dispatch.rs`, `tui/src/bottom_pane/approval_overlay.rs`,
  `tui/src/app/config_persistence.rs`, `model-provider/src/auth.rs`,
  `codex-mcp/src/connection_manager/startup.rs`, `sandboxing/src/{manager,spawn}.rs`,
  `ext/web-search/src/*`, `external-agent-migration/src/*`, `memories/{read,write}/src`,
  `state/src/runtime/memories.rs`, `cli/src/main.rs`, `vault/src/lib.rs`, `network-proxy/src/*`.
  PF-30-S02 in particular threads a provenance envelope through context serialization, memory,
  import/export and child spawn. Provenance is inherently cross-cutting — that is not the problem.
  The problem is that carrying it as fields on upstream types in `protocol/src/models.rs` guarantees
  a merge conflict on every upstream change to those types, in the one subsystem where a silently
  dropped field is a security failure rather than a compile error.
- **Failure scenario:** A Codex merge lands during PF-30/PF-32. A rebase resolves a conflict in
  `context_manager/normalize.rs` by taking upstream's version of a serialization function. Taint
  lineage is silently dropped for one path. Nothing fails to compile. PF-30-S02's tests were written
  against the pre-merge shape and still pass. Poisoned memory now launders through that path, and
  the gap surfaces only if PF-26-S01's fixtures happen to cover it.
- **Recommended plan change:**
  1. Define the seam explicitly in the plan: (a) Corbanu-owned crates upstream never touches
     (`codex-security-policy`, `codex-secret-broker`, `codex-content-security`,
     `codex-web-retriever`), plus (b) an **enumerated, capped list** of upstream call sites, each a
     single function call into a Corbanu module, recorded in a table with the upstream symbol it
     attaches to.
  2. Add a **seam contract test per call site** that fails loudly when the upstream signature or
     call graph changes — a compile-time or test-time tripwire, not a comment.
  3. Add a standing release-gate obligation (new PF-26 row or a new sprint): after every upstream
     Codex merge, re-run seam contract tests + the differential Permissive check (OR-04) + the
     provenance propagation suite. Assign it to the named lead developer.
  4. Prefer a side-table keyed by message identity over adding fields to upstream protocol types,
     where the provenance design allows it — and where it does not, say so and accept the conflict
     cost deliberately.
- **Tests / expected outcomes:**
  - `seam_contract_<n>` per call site: asserts the upstream symbol exists with the expected shape and
    that the Corbanu hook is reachable from it. **Expected:** fails at build/test time on upstream
    drift, naming the seam.
  - `pf_30_s02_taint_survives_every_serialization_path`: enumerate every context/memory/export path
    by reflection or an explicit registry; assert each preserves lineage. **Expected:** a newly
    added upstream path with no lineage handling **fails the test rather than defaulting to
    untainted** — closed-world, not open-world.

---

### OR-06 — Browser/retrieval isolation cannot proceed in parallel; the blocking edges are sequencing, not necessity

- **Severity:** High · **Confidence:** High · **Type:** Covered but wrongly sequenced
- **References:** `docs/sprints/current/p0-security-levels/index.md` rows 17, 18, 32
  (PF-33-S01 `Depends on: PF-27-S02`; PF-33-S02 `Depends on: PF-33-S01`; PF-31-S01
  `Depends on: PF-33-S02, PF-27-S02`); `pf-33-s01-url-dns-and-redirect-policy.md` "Code boundaries"
  (`network-proxy/src/{destination,destination_tests}.rs` — pure policy);
  `pf-31-s01-pinned-retriever-isolation.md` (image pinning, SBOM, mounts, three-OS probes);
  `REVIEW_SCOPE.md` "The user wants browser isolation to be independently workable in parallel."
- **Affected sprints:** PF-31-S01, PF-31-S02, PF-31-S03, PF-33-S01, PF-33-S02, PF-37-S01.
- **Analysis:** PF-31-S01 sits at depth 14 behind the full credential chain
  (`PF-13-S01…S04 → PF-27-S01 → PF-27-S02`). Examining the two blocking edges:
  - **PF-33-S01 → PF-27-S02 is not intrinsic.** PF-33-S01's own boundary is URL canonicalization,
    IDNA, port/scheme, IP classification, DNS answer validation and per-hop redirect
    re-authorization — pure functions testable against synthetic DNS fixtures with no broker, no
    secrets, and no running agent. Its Remaining list contains nothing that requires the broker to
    exist; the broker-*binding* requirement ("Bind credential adapters to exact normalized host,
    port, method and supported path") is a consumer of the policy, not a producer.
  - **PF-31-S01 → PF-27-S02 is partly intrinsic but severable.** The retriever must not inherit
    secrets — true, and that needs the launch contract. But pinning an image, producing an SBOM,
    setting mounts/resource budgets, and **probing actual isolation on three OSes** are independent
    of the credential broker and are the longest-lead external dependency in the program
    (`Platform isolation capability matrix … pending`).
- **Failure scenario (schedule, not security):** the single longest-lead external item — three-OS
  container capability — is discovered at stage 14, in late September, after the credential chain
  consumed the calendar. If Podman/Docker turns out unusable on a supported platform, there is no
  time left to respond, and the answer arrives after the gate.
- **Recommended plan change:**
  1. Split PF-33-S01 into **PF-33-S01a "destination policy library"** (dependency-free; pure policy
     + synthetic DNS fixtures) and **PF-33-S01b "bind policy to broker connections"**
     (depends on PF-27-S02). Start S01a at stage 1.
  2. Split PF-31-S01 into **PF-31-S01a "pinned retriever artifact and platform capability probes"**
     (dependency-free; image digest, SBOM, license inventory, mount/resource policy, Linux/macOS/
     Windows probes) and **PF-31-S01b "wire retriever to launch contract and PF-33 policy"**
     (depends on PF-27-S02 and PF-33-S01a).
  3. Record freeze point **FP-6 (destination policy request shape)** at the end of PF-33-S01a so the
     retriever, broker and search adapters all code against it.
  This makes browser isolation a genuine day-one lane (§9 Lane D) and de-risks the longest lead.
- **Tests / expected outcomes:**
  - `pf_31_s01a_platform_capability_probe`: on each of Linux/macOS/Windows, report observed image
    digest, user, mounts, network mode and resource caps — **observed, not configured**.
    **Expected:** mismatch between configured and observed **fails**; missing engine yields
    `unsupported` with a reason, never a host fallback.
  - `pf_33_s01a_destination_policy_table`: table-driven over IDNA, trailing dot, mapped IPv6,
    alternate IPv4 encodings, CNAME chains, mixed public/private DNS answer sets, per-hop redirects.
    **Expected:** deterministic allow/deny with no network access required to run the suite.

---

### OR-07 — The classifier is the true long pole and is buried at stage 17; all screened search sits behind it

- **Severity:** High · **Confidence:** High · **Type:** Covered but wrongly sequenced
- **References:** `docs/sprints/current/p0-security-levels/index.md` rows 35-46
  (PF-34-S01 → PF-35-S01 → S02 → S03 → PF-34-S02 → PF-34-S03 → PF-32-S01 → …);
  `docs/plans/active/p0-security-levels.md` "Local classifier qualification targets"
  (≤0.1% benign FPR on ≥100,000 held-out segments with CI; ≥80% known-family; ≥65% unseen-source;
  p95 ≤50 ms per 2,048-token segment; RSS ≤512 MiB; model ≤300 MiB);
  `pf-35-s01-classifier-corpus-and-evaluation.md` (licensed corpus, evaluator-owned blind holdout
  frozen before training); "Dependencies" → "Local detector hardware/corpus/license pins … pending".
- **Affected sprints:** PF-34-S01/S02/S03, PF-35-S01/S02/S03, PF-32-S01…S06, PF-36-S01/S02.
- **Analysis:** PF-35-S01 depends on PF-34-S01 only because the sanitizer defines the **input
  contract**. That is a *contract* dependency, not an implementation dependency. Corpus licensing,
  evaluator-ownership negotiation, hard-negative curation (benign security research, legitimate
  human trading instructions, trigger-token negatives) and the weakest-supported-CPU pin are
  procurement and research activities with multi-week lead times and no code dependency at all.
  Meanwhile PF-32-S01 (search facade) depends on PF-34-S03, so **every screened search adapter is
  serialized behind a trained, calibrated, blind-qualified detector**, and PF-34-S02 (quarantine
  state machine) is serialized behind it too — though a state machine can be built against a
  stubbed verdict enum.
- **Failure scenario:** classifier work begins in mid-to-late September. The corpus is unlicensed,
  the blind holdout is not yet evaluator-owned, and the FPR target is missed on first training.
  PF-35-S03 cannot complete; PF-34-S02/S03 and all of PF-32 are blocked behind it; the profile
  contract says "Local classifier unavailable → **Pause external ingestion**" in both protected
  modes, so Moderate is unusable rather than merely incomplete.
- **Recommended plan change:**
  1. Freeze **FP-5 (sanitized-segment contract + versioned verdict enum
     `allow | suspicious | hostile | unavailable` with model/version/threshold IDs)** as a small,
     early, dependency-light sprint, drawing the shape from PF-34-S01 and PF-35-S01's first
     Remaining item.
  2. Re-point PF-35-S01 at FP-5 instead of PF-34-S01 completion, and **start it in week 1** —
     licensing, evaluator ownership and the CPU pin are calendar-bound, not code-bound.
  3. Re-point PF-34-S02 at FP-5 (stubbed verdict) instead of PF-35-S03, keeping PF-35-S03 as the
     gate on *enabling* screening, not on *building* quarantine.
  4. Re-point PF-32-S01 at PF-34-S03 **or FP-5**, whichever the facade actually needs — the facade
     needs the verdict type, not the trained model.
- **Tests / expected outcomes:**
  - `pf_34_s02_quarantine_against_stub_verdict`: drive allow/suspicious/hostile/unavailable through
    the state machine with a stub. **Expected:** all transitions, restart recovery and
    capacity-exhaustion behavior pass with no model present; `unavailable` pauses ingestion rather
    than allowing it.
  - `pf_35_s01_holdout_ownership`: assert the blind holdout manifest is frozen and evaluator-owned
    before any training artifact hash exists. **Expected:** training that references holdout data
    fails the check.
  - `pf_35_s03_forced_miss_containment`: force the detector to return `allow` on every hostile
    fixture. **Expected:** zero unauthorized disclosures or actions — deterministic policy holds
    independently. This is the single most important test in the program (see §8 rank 1).

---

### OR-08 — A qualification gate is used as an implementation dependency, serializing three downstream features

- **Severity:** High · **Confidence:** High · **Type:** Covered but wrongly sequenced
- **References:** `docs/sprints/current/p0-security-levels/index.md` rows 22, 25, 26, 41
  (PF-13-S05 depends on five sprints; **PF-30-S03**, **PF-23-S01** and **PF-32-S01** each depend on
  PF-13-S05); `pf-13-s05-credential-boundary-adversarial-qualification.md` "Execution mandate"
  ("**final-tree canary and adversarial evidence**"; "Excludes: new credential behavior") and
  Preconditions ("A named independent security reviewer is recorded before acceptance").
- **Affected sprints:** PF-13-S05, PF-30-S03, PF-23-S01, PF-23-S02, PF-23-S03, PF-32-S01, and
  transitively PF-24-S02, PF-25-S01/S02, PF-37, PF-38, PF-39, PF-40, PF-41.
- **Analysis:** PF-13-S05 produces **no code** — it is a canary sweep plus a named external
  reviewer's sign-off. Making implementation sprints depend on it converts a review milestone into a
  hard serialization point that also depends on **scheduling a human who is not yet named**
  ("Independent security reviewer … Must be named before either review completes"). PF-30-S03's
  actual technical need is the **broker client interface** (`PF-27 broker client`, per its own Code
  boundaries), not the broker's adversarial sign-off. Same for PF-23-S01 and PF-32-S01.
- **Failure scenario:** the independent reviewer is unavailable for two weeks in September.
  PF-13-S05 cannot close. Provenance (PF-30-S03), *all* Moderate/Aggressive enforcement (PF-23-\*),
  and the entire search facade stall behind a calendar conflict, not a technical one — with 26
  downstream sprints idle under the one-in-progress rule.
- **Recommended plan change:** Distinguish "depends on interface" from "depends on qualification."
  Re-point PF-30-S03, PF-23-S01 and PF-32-S01 at **PF-27-S01/PF-27-S02** (interface + containment)
  and keep PF-13-S05 as a **gate on activating protected modes and on release**, recorded in the
  plan's completion checklist and in PF-26-S01's preconditions — where it already appears. The
  invariant that matters ("no protected mode is advertised on an unqualified boundary") is preserved
  by gating *activation*, not by blocking *implementation*.
- **Tests / expected outcomes:** governance, not code — `python3 docs/sprints/check.py` must still
  pass with the amended `depends_on` values, and the plan's Completion checklist must gain an
  explicit "PF-13-S05 passed before any protected mode is selectable in a shipped build" item.

---

### OR-09 — The credential capability bounds the secret's confidentiality but not its authority or spend

- **Severity:** High · **Confidence:** Medium-High · **Type:** Missing
- **References:** `pf-13-s03-openai-exact-host-proxy-substitution.md` Remaining ("Permit only HTTPS,
  default port 443, exact host `api.openai.com`, method `POST`, path prefix `/v1/`, and the OpenAI
  bearer header"); `pf-13-s01-vault-backed-exact-host-credential-substitution.md` Remaining
  (metadata list: purpose, operation, HTTP method, normalized destination — no quantitative limits);
  contrast `docs/plans/active/p0-security-levels.md` "Standards-derived control profile" → RFC 9396
  row ("every temporary grant explicit action, resource, destination, **limits**, actor chain, and
  expiry") and `pf-38-s01` Remaining ("rate/daily notional/loss/leverage caps and cooldowns … with
  atomic budget reservation").
- **Affected sprints:** PF-13-S01, PF-13-S03, PF-13-S04, PF-23-S02, PF-32-S03/S04/S05, PF-39-S02.
- **Failure scenario:** An injected instruction causes the agent to issue authorized-shaped requests:
  `POST https://api.openai.com/v1/...` — correct scheme, host, method, path prefix, correct bearer
  header injected by the broker. The **body is entirely agent-controlled**. Nothing in PF-13 bounds
  request count, token spend, or which `/v1/*` operation is invoked. The user's key is not disclosed
  — the confidentiality guarantee holds perfectly — while their money is spent and their data is
  sent to a third party at agent discretion. The plan's own bounded-grant semantics ("changing any
  bound field or using it for an adjacent operation fails") are not applied to the credential
  capability itself. Note this is a *contract scoping* observation on a draft, not a claim about
  current code.
- **Recommended plan change:** Extend the PF-13-S01 capability metadata with **quantitative bounds**
  — request count, byte/token ceiling, wall-clock window, and an operation classification within the
  path prefix — and enforce them in PF-13-S03 at the same point that validates host/method/path.
  Either that, or record an explicit accepted-residual-risk decision in the plan stating that
  Moderate/Aggressive bound credential *disclosure* but not credential *use*, so the `/security` tab
  and finished docs (PF-26-S03) do not imply otherwise. For a product whose promise is
  "permit action without exposing strategy, credentials, or financial information," the first option
  is the coherent one.
- **Tests / expected outcomes:**
  - `pf_13_s03_capability_spend_bound`: capability issued with a 3-request / N-token bound; agent
    issues 10 authorized-shape requests. **Expected:** requests 4-10 denied with a secret-free
    reason and an audit event; the credential is never disclosed in any case.
  - `pf_13_s03_adjacent_operation_within_prefix`: capability minted for one `/v1/*` operation class;
    agent requests a different one. **Expected:** denied, matching the bounded-grant contract.

---

### OR-10 — Conservative taint union has no granularity or scoping design; Moderate becomes unusable and will be relaxed under pressure

- **Severity:** High · **Confidence:** Medium-High · **Type:** Missing
- **References:** `pf-30-s02-persistent-taint-and-memory.md` Remaining ("Propagate the **conservative
  union** of source authority and taint through compaction, summaries, memory write/read, retrieval,
  cache, export/import and transcript replay"; "Keep source taint **sticky across exact-action
  approvals**"); `pf-30-s03-post-taint-authority-checks.md` Remaining ("Require **fresh exact human
  authority** for sensitive tainted follow-on actions"); `docs/plans/active/p0-security-levels.md`
  "Invariants" → "**Taint is durable.** … Exact human action approval does not erase taint";
  contrast the Moderate promise in `docs/corbanu-product-spec.md` "P0 `/security` levels"
  ("Add strong protection **without making normal agent work painful**") and the acceptance flow
  "Moderate hostile input" → "Normal analysis **may continue**".
- **Affected sprints:** PF-30-S02, PF-30-S03, PF-23-S01, PF-38-S02, PF-39-S01, PF-25-S01.
- **Failure scenario:** This is a trading terminal; research *is* the workflow. Turn 1 fetches a
  market page — the context is now tainted. Compaction folds it into the summary; the summary is
  conservatively tainted; memory inherits it; every child agent inherits it; restart preserves it.
  From turn 2 onward, permanently, **every** protected action requires fresh exact human approval.
  In Aggressive that may be the intent. In **Moderate** it directly contradicts the product promise
  and the plan's own acceptance flow. The realistic outcome is not a security incident — it is that
  Moderate is found unusable during PF-26-S02 and someone relaxes the rule at the worst possible
  moment, with no design to guide *which* relaxation is safe.
- **Missing vs covered:** durability is well specified; **granularity is entirely unspecified.**
  There is no per-value/per-claim lineage, no notion of which action classes a standing bounded
  grant already satisfies, and no clean-derivation path. PF-30-S02's "explicitly clean context"
  is a session-level reset, not an in-session mechanism.
- **Recommended plan change:** Specify taint *granularity* in PF-30-S01/S02 before implementation:
  lineage attaches to **values and claims**, not to the whole context, so that an action whose
  inputs are all human- or account-origin is not tainted merely because the session once read a web
  page. Then define in PF-30-S03 the matrix of (action class × taint state) → (allowed | standing
  grant sufficient | fresh exact approval), and give Moderate and Aggressive different rows.
  Add a usability acceptance target so the tradeoff is measured rather than discovered.
- **Tests / expected outcomes:**
  - `pf_30_s03_realistic_moderate_session`: scripted 20-turn research session — 5 web fetches, 2
    memory writes, 1 compaction, 1 child agent, ending in one order proposal. **Expected:** a
    **stated, asserted number** of human approvals (proposal states the target; my recommendation
    is 1 — the order itself). A test that simply records "many" is not an acceptance criterion.
  - `pf_30_s03_untainted_action_after_tainted_read`: action whose inputs are exclusively
    human/account origin, in a session that previously read hostile content. **Expected:** allowed
    without fresh approval, while the hostile lineage remains attached to the hostile values and
    still blocks any action that consumes them.
  - `pf_30_s03_approval_does_not_launder`: exact human approval of action A; attempt adjacent action
    B derived from the same tainted ancestry. **Expected:** B denied; A's approval authorized only A.

---

## 3. OpenClaw adoption hazards found by direct inspection

The source review is accurate on everything I checked. These four are **not** in it, are verifiable
at exact lines in this export, and each maps to a Corbanu sprint whose Remaining list gets sharper
with the mechanism named. None is a vulnerability claim against Corbanu code, and all sit inside
upstream's documented trusted-operator model (`openclaw/SECURITY.md`).

---

### OR-17 — Proxy TLS-server cache is keyed by run identity, so revoke-then-re-register reuses stale bindings on *new* connections

- **Severity:** High (as an adoption hazard) · **Confidence:** High
- **Evidence:** `openclaw/src/secrets/egress-proxy/proxy-server.ts`
  - `:458-478` `tlsServerFor` caches by `` `${registered.key}\0${target.hostname}:${target.port}` ``
    and the created `createHttpsServer` handler **closes over the `registered` object supplied at
    creation time**.
  - `:118-120` `runKey` = `` `${run.runId}\0${run.instanceId}` `` — identical across a
    revoke/re-register cycle for the same run identity.
  - `:645-647` `revokeRun` deletes only the `tokens` entry. **`tlsServers` is never pruned.**
  - `:609-628` `registerRun` on a deleted key creates a *new* `RegisteredRun` with a new token and
    fresh `sentinelBindings`.
- **Consequence:** after `revokeRun` + `registerRun` for the same `{runId, instanceId}`, a client
  presenting the **new** token authorizes correctly at `:531`, then `tlsServerFor` returns the
  **cached** server whose handler still references the **old** `RegisteredRun`. Substitution at
  `:173-189` therefore consults the *old* `sentinelBindings` and *old* `allowedHosts`. A binding
  narrowed or removed on re-registration is still honored. This affects **new** connections, so the
  existing new-CONNECT-refusal test cannot detect it.
- **Why the source review missed it:** OC-2 correctly identifies that "an accepted CONNECT/TLS
  handler retains its `RegisteredRun` object" — the *established-channel* case. The cache makes it a
  *fresh-channel* case as well, which is a materially different (and more surprising) property.
- **Sprint impact:** `pf-27-s01-isolated-credential-broker.md` already requires "same-run-ID
  replacement" and "broker restart with old handles" — this supplies the exact mechanism and the
  expected result, and adds a resource-lifecycle requirement (revocation must evict cached
  transport state, not just tokens).
- **Test / expected outcome:** `pf_27_s01_revoke_then_reregister_same_run_id`: register run R with
  binding {S→hostA}; open and close a tunnel; `revokeRun(R)`; `registerRun(R)` with binding
  {S→hostB} only; open a **new** tunnel and send S targeting hostA. **Expected:** refused
  (`destination-not-allowed`); Corbanu must additionally assert that all cached per-run transport
  state was destroyed at revocation.

---

### OR-18 — An empty hostname allowlist means "allow every host", and a wildcard-only allowlist normalizes to empty

- **Severity:** High (as an adoption hazard) · **Confidence:** High
- **Evidence:** `openclaw/src/infra/net/ssrf.ts`
  - `:282-287` `matchesHostnameAllowlist` — `if (allowlist.length === 0) { return true; }`.
  - `:223-225` `normalizeHostnameAllowlist` filters out `"*"` and `"*."`. A list containing only a
    wildcard therefore becomes empty → allow-all.
  - `:385-392` `resolveHostnamePolicyChecks` uses that result as the gate.
- **Consequence:** "no allowlist configured" and "allowlist configured as `*`" both mean **no host
  restriction**. That is a defensible default for a permissive tool; it is fail-open for a
  protected mode. A Corbanu port that copies the helper inherits the polarity.
- **Sprint impact:** `pf-33-s01-url-dns-and-redirect-policy.md`, `pf-33-s02-connection-pinning-and-bypass.md`.
- **Test / expected outcome:** `pf_33_s01_absent_allowlist_denies`: protected mode with allowlist
  unset, then set to `["*"]`. **Expected:** both **deny** all destinations in Moderate/Aggressive
  and surface a configuration reason. Permissive keeps existing behavior.

---

### OR-19 — Exact-origin trust escalates into "any RFC1918 address" for IPv4 DNS answers

- **Severity:** High (as an adoption hazard) · **Confidence:** High
- **Evidence:** `openclaw/src/infra/net/ssrf.ts`
  - `:238-257` `resolveSsrFPolicyForUrl` promotes a matching `allowedOrigins` entry into
    `allowedHostnames` for that hop.
  - `:231-236` `shouldSkipPrivateNetworkChecks` returns true when the hostname is in
    `allowedHostnames` — so the standard private-network check is skipped.
  - `:622-629` the skip branch falls through to
    `assertAllowedTrustedHostnameResolvedAddressesOrThrow`.
  - `:467-484` that guard blocks unspecified, loopback (unless the hostname is explicitly loopback),
    link-local, cloud-metadata, and **IPv6** special-use — but `isBlockedTrustedResolvedIpv6Address`
    at `:444-456` returns `false` for IPv4. **IPv4 RFC1918 (10/8, 172.16/12, 192.168/16) is not
    blocked on this path.**
- **Consequence:** upstream documents this as deliberate (`:625-627` "Exact-host trust may allow
  RFC1918/tailnet/private-DNS provider targets"), and for an operator-configured endpoint it is
  reasonable. But it means an *exact-origin exception* is, in effect, a private-network grant
  mediated by DNS: whoever controls the answer for that hostname chooses the internal target.
- **Sprint impact:** this is exactly the boundary `pf-32-s05-searxng-search-adapter.md` ("private
  service access uses the exact PF-33 adapter exception, never arbitrary private URL fetch") and
  `pf-33-s02` ("a caller cannot turn the loopback exemption into general SSRF") intend to hold. The
  sprints state the goal; this names the mechanism that defeats it. **The self-hosted adapter must
  pin the operator-supplied address, not re-resolve the hostname into a trusted-host allow.**
- **Test / expected outcome:** `pf_33_s02_self_hosted_exception_is_address_pinned`: configure a
  self-hosted SearXNG endpoint; then have the synthetic resolver return a different RFC1918 address
  for that hostname. **Expected:** denied — the exception is bound to the exact operator-supplied
  address, and a changed DNS answer does not silently retarget it. Also assert the exception cannot
  be reached from the public-fetch lane.

---

### OR-20 — Memory-provenance capacity exhaustion and `dreaming/` filenames both promote trust

- **Severity:** High (as an adoption hazard) · **Confidence:** High
- **Evidence:**
  - `openclaw/src/memory/memory-artifact-provenance.ts:91-98` — store opened with
    `maxEntries: 50_000`, `overflowPolicy: "reject-new"`. Beyond capacity, **no provenance record is
    written**.
  - `openclaw/extensions/memory-core/src/memory/memory-path-provenance.ts:57-63` — when no record
    exists, workspace memory defaults to `originClass: "agent"` (the comment is explicit: "the index
    default must not fail closed the entire workspace").
  - Composed: **capacity exhaustion silently upgrades untrusted memory to agent-origin.**
  - Separately, `memory-artifact-provenance.ts:69-71` — `normalizeMemoryArtifactRelativePath`
    returns `undefined` for `memory/dreaming/**` and `memory/.dreams/**`, so writes there are never
    recorded; while `memory-path-provenance.ts:42-47` classifies those same paths as
    `originClass: "system"` — the **highest** trust. Trust is therefore assigned by filename.
  - Also confirming the review's prescription: `readMemoryArtifactProvenance` (`:198-208`) validates
    the stored hash's *format* only and never compares it to current file content, so an
    out-of-band edit by any non-wrapped writer retains the previous `originClass`.
- **Why this matters for Corbanu:** OC-11 says "missing records, filenames or model-generated
  summaries" must not promote authority — correct, and these are the two exact mechanisms plus the
  read-time binding gap. The composed capacity→default path is the one an attacker can actually
  drive.
- **Sprint impact:** `pf-30-s02-persistent-taint-and-memory.md` (its Remaining already names
  "capacity rejection", "dreaming/memory filenames", "content identity on read" — this makes each a
  concrete, named regression), `pf-34-s02-quarantine-state-and-store.md`.
- **Tests / expected outcomes:**
  - `pf_30_s02_provenance_capacity_exhaustion`: fill the provenance store to capacity, then write
    untrusted-origin memory. **Expected:** the write is **rejected or quarantined**; it must never
    become readable as agent/trusted. Capacity exhaustion is a fail-closed condition.
  - `pf_30_s02_special_path_no_trust_promotion`: write hostile content to a `dreaming/`-equivalent
    path. **Expected:** classified untrusted; no filename grants `system`.
  - `pf_30_s02_content_identity_on_read`: record provenance, modify the file out-of-band, read.
    **Expected:** hash mismatch → quarantine, not the stale classification.

---

### OR-15 — The debug/capture transport seam is a full-fidelity secret sink and is not in any sink inventory

- **Severity:** Medium · **Confidence:** High · **Type:** Missing
- **Evidence:** `openclaw/src/infra/net/fetch-guard.ts:338-374` `captureGuardedFetchExchange`
  records `requestHeaders` and `requestBody` into a capture runtime, gated only by the environment
  variable `OPENCLAW_DEBUG_PROXY_ENABLED` (`:143`, `:349`). `isManagedProxyActive()` at `:221-223`
  likewise branches network policy on `OPENCLAW_PROXY_ACTIVE`.
- **Consequence for Corbanu:** an environment variable that turns on full request/response capture
  is both (a) a disclosure sink that PF-28-S01's registry must cover, and (b) a variable that
  PF-27-S02's allowlisted launch environment must ensure an agent cannot set. Neither sprint names
  debug/capture paths. `pf-28-s01` enumerates "transcript, traces, errors, audit, snapshots, exports
  and diagnostic artifacts" — a capture proxy is none of those by name.
- **Sprint impact:** PF-28-S01, PF-27-S02, PF-39-S02, PF-41-S02.
- **Test / expected outcome:** `pf_28_s01_debug_capture_sink`: enable every diagnostic/capture path
  and drive an authorized credentialed request. **Expected:** canary absent from all capture
  artifacts; and `pf_27_s02_env_cannot_enable_capture`: agent-set capture/proxy environment
  variables are stripped by the launch contract and cannot re-enable capture.

---

## 4. Later hardening

### OR-11 — Level transitions can invalidate a financial action that may already have been broadcast

- **Severity:** Medium-High · **Confidence:** Medium · **Type:** Covered but underspecified
- **References:** `pf-23-s03-downgrade-restart-and-inheritance-enforcement.md` Remaining
  ("confirmed changes invalidate broker/browser sessions, **pending financial actions**, child
  authority and queued disclosures"); `pf-38-s03-sign-broadcast-and-receipts.md` Remaining
  ("Model submitted/confirmed/failed/**unknown** execution states"; "do not claim revocation can undo
  an already-signed or broadcast transaction"); `pf-40-s03` ("show irreversible actions already
  submitted").
- **Failure scenario:** a transfer is in `unknown` state (submitted, no receipt). The user changes
  level. PF-23-S03 "invalidates pending financial actions"; PF-38-S03 correctly refuses to claim the
  effect can be undone. The two are individually right and jointly ambiguous: the UI may report the
  action invalidated while the chain settles it. The user believes it did not happen.
- **Recommended plan change:** define in PF-23-S03 that a level transition is **blocked** while any
  action is in `unknown` state, or requires an explicit human acknowledgement that names the
  outstanding action and states that invalidation removes *authority*, not *effect*. Add the state
  to PF-41-S01's inspector.
- **Test / expected outcome:** `pf_23_s03_transition_with_unknown_financial_state`: force an
  uncertain submission, then request a downgrade. **Expected:** transition blocked or explicitly
  acknowledged; no UI text asserts the action was cancelled; the outstanding action remains visible
  through restart until resolved.

### OR-12 — Deep child agents have no path to request authority

- **Severity:** Medium-High · **Confidence:** Medium · **Type:** Missing
- **References:** `docs/corbanu-product-spec.md` Glossary ("Sauron → Nazgul → Troll → Orc") and
  Shipping MVP ("Agent orchestration … model-aware delegation, durable mailboxes");
  `pf-22-s01-runtime-policy-and-agent-inheritance.md` ("Propagate level, actor chain, task/session
  identity, revocation generation, and kill state to child creation");
  `pf-25-s01-temporary-grant-tui.md` (grant creation is human-initiated only; no child-originated
  request path).
- **Failure scenario:** in Aggressive, an Orc four levels down needs one sensitive read. Everything
  is deny-by-default; only the human can grant; there is no mechanism to surface the need. The task
  fails opaquely, and the human cannot tell what to grant. The predictable workaround is granting
  broad authority at the top "so the subagents work" — defeating narrow grants.
- **Recommended plan change:** add to PF-25-S01 (or a new sibling) an **authority-request** flow: a
  descendant emits a typed, secret-free request that propagates up the actor chain to the trusted
  human surface, showing the requesting agent, the exact scope and the task. The request is data,
  never authority. Alternatively, record the decision that deep children simply fail and the human
  re-runs at a higher level — but record it.
- **Test / expected outcome:** `pf_25_s01_descendant_authority_request`: depth-4 child requests one
  scoped action. **Expected:** the human sees the exact actor chain and scope; approving grants that
  scope **only** to that actor; sibling and parent agents gain nothing; the request itself cannot
  mint authority if unanswered.

### OR-13 — The ingress registry must be closed-world, or future surfaces bypass provenance by omission

- **Severity:** Medium · **Confidence:** Medium-High · **Type:** Covered but underspecified
- **References:** `pf-30-s01-typed-source-envelope.md` Remaining ("Assign envelopes only at trusted
  ingress for web/search, files, transcripts, **social/trollbox/email**, MCP/tool/plugin/hook output
  and child messages"); `docs/corbanu-product-spec.md` "Social: Corbanu trollbox" (TO BUILD, P1, no
  plan) and "Accountable sequencing" row 7; `pf-23-s01` Remaining ("Register required protected-mode
  subsystems and deny unsupported/unready routes").
- **Failure scenario:** PF-30-S01 enumerates today's ingress points. The trollbox is built later
  under a different plan by a different owner. Nothing structurally forces it through the ingress;
  it arrives as ordinary text with no envelope. If "no envelope" is treated as anything other than
  hostile, the highest-volume untrusted surface in the product bypasses provenance entirely.
- **Recommended plan change:** state in PF-30-S01 that the ingress registry is **closed-world**:
  content reaching model context without a registered ingress envelope is rejected or quarantined in
  protected modes — never defaulted to trusted. This is the same polarity lesson as OR-20.
- **Test / expected outcome:** `pf_30_s01_unregistered_ingress_rejected`: inject content through a
  synthetic new surface with no registration. **Expected:** rejected/quarantined with a named
  reason in Moderate and Aggressive; Permissive unchanged.

### OR-21 — Response gating's latency cost is real but bounded; say so before it is discovered

- **Severity:** Medium · **Confidence:** Medium · **Type:** Covered but underspecified
- **References:** `pf-28-s02-reflected-secret-response-scrubbing.md` Remaining ("Bound
  decompression, buffering and streaming carry"; "Reject unsupported content encodings … do not
  return raw response bytes on failure"); `pf-35-s03` ("prevent streaming prefixes from reaching the
  model before a decision"); `docs/plans/active/p0-security-levels.md` "Local classifier
  qualification targets" ("Measure end-to-end latency as well as per-segment speed").
- **Analysis:** incremental scrubbing of *known* values across chunk boundaries is proven feasible —
  `openclaw/src/secrets/egress-proxy/stream-substitution.ts:36-82` keeps a bounded carry and emits
  incrementally, and the reference probe "request sentinel substituted across one-byte chunks"
  passes. So response scrubbing need not buffer whole responses. The genuine buffering cost is
  confined to unsupported encodings and to ingress screening.
- **Recommended plan change:** state explicitly in PF-28-S02 that reflected-secret scrubbing uses
  bounded-carry incremental scanning sized to the maximum protected value, so token streaming
  survives; and that "deny on unknown encoding" applies at the transport-decode layer. Add a latency
  acceptance target to the plan's acceptance flows so the tradeoff is measured.
- **Test / expected outcome:** `pf_28_s02_incremental_scrub_latency`: stream a long response with a
  canary split across chunk boundaries. **Expected:** canary never emitted; time-to-first-token
  within the stated target; no whole-response buffering for supported encodings.

### OR-16 — `Done` ledgers assert commit *content* that the same record's `Remaining` says is unverified

- **Severity:** Medium · **Confidence:** High · **Type:** Evidence hygiene
- **References:** `pf-15-s01-security-level-domain-foundation.md` Done ("Commit `a4f178fe15` **added**
  the level domain, bounded values, crate manifest, Cargo lock entry, and Bazel target") vs its
  Remaining ("**Review the existing diff** against PF-15 and remove scope outside the domain
  foundation"); same pattern in PF-16-S01, PF-17-S01, PF-18-S01, PF-19-S01, PF-20-S01, PF-21-S01;
  against `docs/sprints/index.md` ("`Done` contains checked items only") and
  `docs/plans/active/p0-security-levels.md` ("Code presence is not completion").
- **Failure scenario:** a reader scanning ledgers concludes seven features are built. The plan's
  narrative says otherwise, but the machine-checkable artifact says "added."
- **Recommended plan change:** reword to "Commit `<sha>` is **present in the worktree** and is
  pending reconciliation," keeping every content claim in `Remaining` until reviewed.
- **Test / expected outcome:** `python3 docs/sprints/check.py` continues to pass; the change is
  textual. Optionally extend the checker to reject Done items asserting unreviewed commit content.

### OR-22 — The final crosswalk is gated on an explicitly optional vendor lane

- **Severity:** Low · **Confidence:** High · **Type:** Covered but wrongly sequenced
- **References:** `docs/sprints/current/p0-security-levels/index.md` row 61 (PF-26-S01 depends on
  PF-36-S02); `pf-36-s02-hosted-bakeoff-and-local-fallback.md` ("no qualifying service leaves the
  lane explicitly disabled"); plan Dependencies ("Optional hosted vendor and data terms … No vendor
  selected").
- **Analysis:** not on the critical path (PF-41-S02 dominates at depth 31 vs PF-36-S02 at 21), so
  the cost is low — but it makes a release gate depend on a commercial decision that may never come.
- **Recommended plan change:** depend on "**PF-36 disposition recorded**" (enabled-qualified or
  disabled-no-qualified-vendor) rather than on PF-36-S02 completion.

---

## 5. Optional ideas

- **OI-1 — Re-point PF-39-S02 at PF-32-S01 instead of PF-32-S06.** Outbound disclosure control needs
  the *search sink interface*, not four live adapters plus routing. Today the entire Sweep →
  inspector → audit tail (PF-40-S01 → PF-41-S02, depths 27-31) is serialized behind Exa, Brave and
  SearXNG. This edge change alone removes ~2 stages and a conceptual coupling that will confuse
  future readers.
- **OI-2 — Make `unavailable` a first-class verdict everywhere, not just in the classifier.** PF-35
  models `unavailable` well. Broker, retriever, quarantine store, audit and Sweep each have their own
  ad-hoc failure vocabulary. One shared typed `unavailable(reason)` across subsystems would make
  PF-41-S01's "configured vs resolved vs observed" display fall out almost for free.
- **OI-3 — Reuse the compare-and-activate pattern deliberately and name it.**
  `openclaw/src/secrets/runtime-state.ts:1009-1019` is a clean revision-guarded activation. PF-20-S01
  already asks for "compare-and-activate revision checks." Naming it as a shared Corbanu primitive
  (used by level transition, migration, grant issuance and snapshot activation alike) avoids four
  divergent implementations of the same race.
- **OI-4 — Record a "no new ingress without a registration" lint.** Cheap CI enforcement of OR-13.

---

## 6. Human decisions required

These are not engineering choices. Each needs the product decision process defined in
`docs/corbanu-product-spec.md` "Ownership and decision rights."

| # | Decision | Owner | Blocking |
| --- | --- | --- | --- |
| HD-1 | **Gate scope vs date.** Either invoke the spec's own "issue a revised plan" mitigation for 2026-10-08, or approve a reduced v1 Moderate (§8). Do not start with an unassessed date. | Travis Good (final product authority) | OR-01 — everything |
| HD-2 | **Amend the one-in-progress WIP rule** to allow N lane-scoped concurrent sprints in one plan, or split browser isolation into the free second plan slot. Both are process/product decisions, not engineering. | Policy owner (lead developer) + product authority | OR-01, OR-06 |
| HD-3 | **Per-OS containment support matrix**, including whether installation-time elevation and code signing are required, and what `/security` shows when a platform qualifies for neither. | Jim Ricketts + product authority | OR-02 |
| HD-4 | **Credential authority vs confidentiality.** Either bound credential *use* (count/spend/operation) or record the accepted residual risk so docs do not overclaim. | Product authority | OR-09 |
| HD-5 | **Moderate taint usability target.** State the acceptable number of human approvals in a realistic research session; this defines PF-30-S03's matrix. | Product authority | OR-10 |
| HD-6 | **Name three humans now:** independent security reviewer (PF-13-S05, PF-26-S03), human acceptance tester (PF-26-S03), and the "human owner" for PF-37-S01's login origin and non-production account. All three are currently unnamed and all three gate release. | Release owner | OR-08, PF-37, PF-26 |
| HD-7 | **Codex upstream merge policy during the program:** freeze the branch, or merge on a cadence with the OR-05 re-qualification obligation. | Lead developer + product authority | OR-04, OR-05 |

---

## 7. Good architecture worth preserving

Under schedule pressure these are the things most likely to be quietly traded away. They should not
be.

1. **"A model deciding that an action looks safe is never authorization"** (product spec, Product
   principle 9) is not a slogan here — it is structurally enforced: PF-30-S03 re-evaluates authority
   at execution; PF-35-S03 forces detector misses and still requires denial; PF-40-S02 makes the
   advisory reviewer incapable of granting; PF-34-S03 forbids review from conferring authority.
   Most agent-security designs collapse the classifier and the policy. This one does not.
2. **Fail-visible over fail-quiet, stated repeatedly and specifically** — "no silent
   native-search/host-browser/raw-auth fallback"; "unsupported isolation blocks protected-mode
   activation with a reason"; "an integrity gap cannot be hidden behind a healthy badge." This is
   the correct polarity and it is applied consistently.
3. **Separating requested / resolved / observed state** (PF-41-S01) and forbidding a green badge
   over a degraded component. Rare, and exactly right for a product whose value proposition is that
   the user can answer "how locked down am I *actually*?"
4. **Taint survives approval** — "approval authorizes only the specific action and never erases
   source taint" (PF-30-S02). Almost every system gets this wrong. (OR-10 asks for *granularity*,
   not for weakening this.)
5. **Sign/broadcast separation with honest irreversibility** — PF-38-S03's refusal to claim
   revocation can undo a broadcast, plus explicit `unknown` state handling and idempotency keys.
   Correct treatment of the hardest part of financial automation.
6. **Sealed download quarantine with exact-digest human promotion**, extended to overflow spill
   files (PF-31-S03) — closing the route that most implementations leave open.
7. **Anti-overclaim discipline throughout**: "Bound the claim" in the plan invariants; "without
   pretending a local hash chain defeats a fully compromised host" (PF-41-S02); and the reference
   evidence README's insistence that 10 passing probes are "confirmed limitations, not 10 Corbanu
   security passes." Preserve this culture specifically — it is what makes the rest of the evidence
   trustworthy.
8. **Permissive as a machine-checked frozen baseline** rather than an intention. (OR-04 fixes *how*
   it is measured, not *that* it is measured.)

---

## 8. Ranked test matrix

Ranked by risk reduction per unit of effort. "Expected" is the pass condition; a test whose expected
outcome is not asserted numerically or structurally is not an acceptance criterion.

| # | Test | Sprint | Finding | Expected outcome |
| ---: | --- | --- | --- | --- |
| 1 | `pf_35_s03_forced_miss_containment` — force detector `allow` on every hostile fixture | PF-35-S03 | OR-07 | Zero unauthorized disclosures or actions. Deterministic policy holds with the detector fully defeated. **The single most important test in the program.** |
| 2 | `pf_27_s02_canary_in_process_memory` + per-OS `ptrace`/`task_for_pid`/`OpenProcess` probes | PF-27-S02 | OR-02 | Canary unreadable from agent-context processes, **or** protected-mode activation refused with a named reason. No green badge over an uncontained host. |
| 3 | `pf_23_s01_level_file_write_via_exec` + `pf_23_s03_restart_after_offline_tamper` | PF-23-S01, PF-23-S03 | OR-03 | Level unchanged in-session and after restart; stricter value wins; tamper audited. |
| 4 | `security-level-compat --mode differential` (feature-on-Permissive vs feature-off, same commit) | PF-21-S01 | OR-04 | Empty diff across policy, approval, tool, network and spawn surfaces. Stable under upstream churn. |
| 5 | `pf_30_s03_realistic_moderate_session` — 20 turns, 5 fetches, compaction, child, 1 order | PF-30-S03 | OR-10 | An **asserted number** of human approvals matching the HD-5 target. |
| 6 | `pf_13_s05_canary_sweep` across request capture, model context, tool payloads, child env, logs, audit, errors, receipts, crash output, artifacts, **and capture/debug paths** | PF-13-S05 | OR-15 | Canary absent from every sink; sink list is closed-world. |
| 7 | `pf_27_s01_revoke_then_reregister_same_run_id` | PF-27-S01 | OR-17 | Stale bindings refused on **new** connections; cached transport state destroyed at revocation. |
| 8 | `pf_30_s02_provenance_capacity_exhaustion` + `pf_30_s02_special_path_no_trust_promotion` + `pf_30_s02_content_identity_on_read` | PF-30-S02 | OR-20 | Capacity exhaustion fails **closed**; no filename grants trust; hash mismatch quarantines. |
| 9 | `pf_33_s01_absent_allowlist_denies` + `pf_33_s02_self_hosted_exception_is_address_pinned` | PF-33-S01/S02 | OR-18, OR-19 | Empty/wildcard allowlist denies in protected modes; self-hosted exception bound to the exact address, not re-resolved. |
| 10 | `seam_contract_<n>` per enumerated upstream call site | PF-26-S01 | OR-05 | Fails at build/test time on upstream drift, naming the seam. |
| 11 | `pf_30_s02_taint_survives_every_serialization_path` (closed-world enumeration) | PF-30-S02 | OR-05 | A new unhandled path **fails** rather than defaulting to untainted. |
| 12 | `pf_13_s03_capability_spend_bound` + `pf_13_s03_adjacent_operation_within_prefix` | PF-13-S03 | OR-09 | Requests beyond the bound denied; adjacent operation class denied. |
| 13 | `pf_28_s02_incremental_scrub_latency` — canary split across chunk boundaries | PF-28-S02 | OR-21 | Canary never emitted; time-to-first-token within the stated target. |
| 14 | `pf_31_s01a_platform_capability_probe` — observed vs configured on three OSes | PF-31-S01a | OR-06, OR-02 | Configured/observed mismatch fails; missing engine yields `unsupported`, never host fallback. |
| 15 | `pf_23_s03_transition_with_unknown_financial_state` | PF-23-S03 | OR-11 | Transition blocked or explicitly acknowledged; no UI claims an unknown action was cancelled. |
| 16 | `pf_25_s01_descendant_authority_request` | PF-25-S01 | OR-12 | Exact actor chain and scope shown; approval binds to that actor only. |
| 17 | `pf_30_s01_unregistered_ingress_rejected` | PF-30-S01 | OR-13 | Unregistered surface rejected/quarantined in protected modes. |
| 18 | `pf_34_s02_quarantine_against_stub_verdict` | PF-34-S02 | OR-07 | Full state machine, restart recovery and capacity behavior pass with **no model present**. |

---

## 9. Proposed parallelization, interface freeze points, and smallest safe slices

**Proposed only.** Structural independence is not authorization; every item below still requires
HD-1/HD-2 and the plan/sprint lifecycle in `docs/plans/index.md` and `docs/sprints/index.md`.

### 9.1 Interface freeze points

Freezing these converts most "sprint completion" dependencies into "contract available"
dependencies, which is what actually unlocks parallelism.

| ID | Contract | Frozen by | Consumers |
| --- | --- | --- | --- |
| FP-1 | `SecurityLevel` + versioned effective-policy bundle | PF-15-S01, PF-20-S01 | everything |
| FP-2 | `ActorChain`, `AuthorizationRequest`/`Decision`, `BoundedGrant`, `HumanMandate`, `ActionReceipt`, `RevocationState` | PF-16→19 | every enforcement point |
| FP-3 | `CapabilityId` + broker IPC frame (versioned, authenticated) | PF-13-S01, PF-27-S01 | broker clients, financial executor, login |
| FP-4 | Source envelope + taint lineage record | PF-30-S01 | every ingress, memory, child, export |
| FP-5 | Sanitized-segment contract + verdict enum `allow\|suspicious\|hostile\|unavailable` (+ model/version/threshold IDs) | new early sprint, drawn from PF-34-S01 / PF-35-S01 | classifier, quarantine, facade, hosted lane |
| FP-6 | Destination policy request (scheme/host/port/method/path + resolved addresses) | PF-33-S01a | broker, retriever, all search adapters |
| FP-7 | Normalized search result + provider capability descriptor | PF-32-S01 | all four adapters, routing, disclosure |
| FP-8 | Typed financial request + complete-effect preview | PF-38-S01/S02 | preview TUI, sign/broadcast, Sweep |
| FP-9 | Sanitized Sweep event + audit chain record | PF-40-S01, PF-41-S02 | inspector, export, alerts |

### 9.2 Lanes

**Stage 0 — Authority core (blocking prerequisite, ~5 stages, one owner).**
`PF-15-S01 → PF-16-S01 → {PF-17-S01 ∥ PF-18-S01} → PF-19-S01`, with `PF-20-S01` and `PF-21-S01`
alongside, then `PF-22-S01`. Publishes FP-1 and FP-2. Nothing else can meaningfully start first, and
nothing else should have to wait for more than this.

After FP-1/FP-2, these run concurrently:

| Lane | Sprints | Starts after | Notes |
| --- | --- | --- | --- |
| **A — Credential boundary** | PF-13-S01…S04 → PF-27-S01 → PF-27-S02 → PF-28-S01 → PF-28-S02 → PF-29-S01 → PF-29-S02 → PF-13-S05 | FP-2 | The genuine long pole for the broker. Publishes FP-3. |
| **B — Destination policy** | **PF-33-S01a** (dependency-free) → PF-33-S01b → PF-33-S02 | stage 1 | Pure policy + synthetic DNS. Publishes FP-6. (OR-06) |
| **C — Isolation & retrieval** | **PF-31-S01a** (dependency-free) → PF-31-S01b → PF-31-S02 → PF-31-S03 | stage 1 for S01a | Longest **external** lead: three-OS container capability. (OR-06) |
| **D — Screening research** | **PF-35-S01 → S02 → S03** | FP-5 | Corpus licensing, evaluator ownership, CPU pin are calendar-bound. **Start week 1.** (OR-07) |
| **E — Sanitize & quarantine** | PF-34-S01; PF-34-S02, PF-34-S03 against a stub verdict | FP-4, FP-5 | Decoupled from the trained model. (OR-07) |
| **F — Provenance** | PF-30-S01 → PF-30-S02 → PF-30-S03 | FP-1; FP-3 *interface* | Re-pointed off PF-13-S05. (OR-08) |
| **G — Enforcement composition** | PF-23-S01 → S02 → S03 | Lane F, FP-3 interface | Re-pointed off PF-13-S05. (OR-08) |
| **H — TUI** | PF-24-S01 early; then PF-24-S02, PF-25-S01, PF-25-S02 | FP-1; Lane G for S02+ | PF-24-S01 is needed by PF-29-S02, PF-31-S03, PF-34-S03. |
| **I — Search facade** | PF-32-S01 → S02 → {S03 ∥ S04 ∥ S05} → S06 | Lanes C, E; FP-5 | Three adapters are genuinely parallel. Publishes FP-7. |
| **J — Financial** | PF-38-S01 → S02 → S03; PF-39-S01 → S02 | FP-2, FP-3, Lane F | PF-39-S02 → FP-7, not PF-32-S06 (OI-1). |
| **K — Browser login** | PF-37-S01 → S02 | Lanes A, C, F | Gated on HD-6 (named human, real origin, test account). |
| **L — Observability** | PF-40-S01 → S02 → S03; PF-41-S01 → S02 | Lanes F, J | Publishes FP-9. |
| **M — Qualification** | PF-26-S01 → S02 → S03 | everything | Strictly last; PF-13-S05 is a precondition here (OR-08). |

With Stage 0 plus these lanes and the OR-06/OR-07/OR-08/OI-1 edge changes, the critical path drops
from **34 stages to roughly the low 20s**, and — more importantly — the two longest *external* leads
(three-OS containment, classifier corpus/licensing) move from stages 14 and 17 to week 1, where a
bad answer is still recoverable.

### 9.3 Smallest safe vertical slices

**Slice 1 — "Honest Permissive + visible posture" (smallest shippable, real user value).**
PF-15, PF-16, PF-20, PF-21, PF-22, PF-24-S01, plus a reduced PF-41-S01. Ships `/security` showing
the current level and an **honest readiness display** where Moderate/Aggressive are visibly
unavailable-not-yet-qualified. Satisfies the product-spec promise that Permissive preserves current
behavior, delivers the actual user pain point ("how locked down is my agent right now?"), and
advertises nothing unbuilt. This alone is a defensible 2026-10-08 deliverable.

**Slice 2 — "Moderate v1: deterministic authority + secretless credentials + durable provenance."**
Adds Lane A through PF-28-S02, Lane F, PF-23-S01, and PF-32-S02's native-search bypass closure.
Delivers: no raw managed secrets in agent environment/context; reflected-secret protection; durable
taint that survives summaries, memory and child agents; deterministic denial of vault enumeration,
protected-data disclosure, policy mutation and approval bypass. **Requires HD-1 to define Moderate
v1's external-retrieval posture**, because the current profile contract says a missing classifier
*pauses external ingestion* — so a classifier-less Moderate is unusable unless the product decision
says Moderate v1 leaves retrieval at Permissive behavior and says so plainly in the tab.

**Slice 3 — screened research** (Lanes B, C, D, E, I). **Slice 4 — protected workflows**
(Lanes J, K). **Slice 5 — observability and full qualification** (Lanes L, M).

### 9.4 Required process amendments

1. `AGENTS.md` "Sprint execution" and `docs/sprints/index.md` "Non-negotiable sprint shape":
   **"A plan has at most one `in_progress` sprint"** → N lane-scoped concurrent sprints with named
   per-lane owners. Without this, §9.2 is unimplementable. (HD-2)
2. `docs/sprints/index.md`: **"executable dependencies must already be completed and archived"** →
   admit a second dependency kind, "depends on frozen interface FP-n," with the freeze recorded in
   the plan. Without this, interface-based parallelism is not permitted even if lanes are.
3. Add a standing **upstream-Codex re-qualification** obligation to the release gate in `AGENTS.md`
   and to the plan's evidence table. (OR-05)
4. Redefine the Permissive compatibility evidence row as the **differential** check. (OR-04)
5. Note: splitting browser isolation into its own plan is an *alternative* to amendment 1, but it
   consumes the second of two active-plan slots (`docs/plans/index.md` "Work-in-progress limit") and
   is therefore a product decision, not a workaround.

---

## 10. Coverage ledger

### Inspected in full

**Scope and integrity (4):** `REVIEW_SCOPE.md`, `SNAPSHOT.json`, `FILES.sha256` (127/127 verified
via direct `shasum -a 256 -c`, no pipeline), `prepare-packet.mjs` (listing only — see limitations).

**Corbanu governance and product (6):** `corbanu/AGENTS.md`, `docs/corbanu-product-spec.md`,
`docs/plans/index.md`, `docs/sprints/index.md`, `docs/plans/active/p0-security-levels.md`,
`docs/sprints/current/p0-security-levels/index.md`.

**Corbanu planning evidence (3):** `docs/plans/security-source-reconciliation.md`,
`docs/plans/openclaw-source-review-2026-08-28.md`, `docs/plans/proposed/arbitrary-model-autoreview.md`.

**All 63 current security sprint records** — read in full, individually:
PF-13-S01, S02, S03, S04, S05 (5); PF-15-S01; PF-16-S01; PF-17-S01; PF-18-S01; PF-19-S01;
PF-20-S01; PF-21-S01; PF-22-S01 (8); PF-23-S01, S02, S03 (3); PF-24-S01, S02 (2);
PF-25-S01, S02 (2); PF-26-S01, S02, S03 (3); PF-27-S01, S02 (2); PF-28-S01, S02 (2);
PF-29-S01, S02 (2); PF-30-S01, S02, S03 (3); PF-31-S01, S02, S03 (3);
PF-32-S01, S02, S03, S04, S05, S06 (6); PF-33-S01, S02 (2); PF-34-S01, S02, S03 (3);
PF-35-S01, S02, S03 (3); PF-36-S01, S02 (2); PF-37-S01, S02 (2); PF-38-S01, S02, S03 (3);
PF-39-S01, S02 (2); PF-40-S01, S02, S03 (3); PF-41-S01, S02 (2). **= 63.**

**Reference evidence (2 of 4):** `qa/.../openclaw-2026-08-28/README.md`, `probes.json`.

**Supplementary Autoreview:** plan + `sprints/current/arbitrary-model-autoreview/index.md`.

**OpenClaw source inspected directly (18 files):** `src/secrets/sentinel.ts` (full);
`src/secrets/egress-proxy/proxy-server.ts` (full, 663 lines); `.../stream-substitution.ts` (full);
`.../registry.ts` (full); `.../runtime.ts` (full); `.../proxy-server.test.ts` (lines 500-576, the
cited revocation test); `src/logging/secret-redaction-registry.ts` (full);
`src/agents/embedded-agent-runner/run/turn-taint-state.ts` (full);
`src/agents/provider-secret-egress.ts` (full); `src/agents/provider-stream.ts` (full);
`src/security/external-content.ts` (full, 471 lines); `src/infra/net/ssrf.ts` (full, 791 lines);
`src/infra/net/fetch-guard.ts` (full, 797 lines); `src/agents/tools/web-guarded-fetch.ts` (full);
`extensions/browser/src/browser/ssrf-policy-helpers.ts` (full);
`src/agents/sandbox/context.ts` (full); `src/agents/sandbox/config.ts` (full);
`src/memory/memory-artifact-provenance.ts` (full); `src/agents/memory-write-provenance.ts` (full);
`extensions/memory-core/src/memory/memory-path-provenance.ts` (full).
**Targeted ranges:** `src/agents/bash-tools.exec-run.ts:400-479` (proxy/exec gating);
`src/secrets/apply.ts:726-812` (preflight completeness); `src/secrets/runtime-state.ts:940-1019`
(compare-and-activate); `src/agents/tools/web-fetch.ts:340-459` (wrapper overhead, spill).

### Listed in the export but not opened

`corbanu/qa/.../source-manifest.json`, `.../upstream-tests.json` (machine outputs; the README
summarizes them and I did not need per-test detail); `corbanu-tracked-code-paths.txt` (339 KB path
list, explicitly navigation-feasibility only); the 7 Autoreview sprint records (index + plan read;
`REVIEW_SCOPE.md` designates these supplementary and non-active); `openclaw/LICENSE`,
`openclaw/SECURITY.md`, `openclaw/docs/gateway/secrets.md`,
`openclaw/docs/gateway/sandbox-vs-tool-policy-vs-elevated.md` (upstream docs; I relied on the source
review's characterization of the trust model, which the code I did read is consistent with);
`openclaw/src/security/external-content.test.ts`, `external-content-source.ts`;
`src/agents/agent-tools.ts`, `src/agents/embedded-agent-runner/run-loop.ts`,
`.../cli-backend-dispatch-transcript.ts`, `src/auto-reply/reply/agent-runner-memory.ts`,
`src/commands/sandbox-explain.ts`, `src/cron/isolated-agent/run-prepare.ts`,
`src/gateway/server-core-runtime.ts`, `src/logging/redact.ts`, `src/agents/sandbox/docker.ts`,
`src/agents/sandbox/runtime-status.ts`, `extensions/browser/src/browser/config.ts`,
`src/secrets/audit.ts` (and the unread majority of `apply.ts`, `runtime-state.ts`, `web-fetch.ts`).

### Limitations — what this review does and does not establish

1. **No Corbanu runtime is in this export.** Every finding concerns plan, sprint and reference
   artifacts. I assert **no** current-code vulnerability in Corbanu, and none should be inferred.
   Source areas absent from this export are **not assessed** — not verified.
2. **Nothing was executed.** No repository code, no tests, no probes, no network access. The
   OpenClaw findings (OR-17 through OR-20, OR-15) are **source-level readings at exact cited lines**,
   not reproduced exploits. They sit inside upstream's documented trusted-operator model
   (`openclaw/SECURITY.md`), and at least OR-19 is explicitly intentional upstream. They are
   presented as **adoption hazards for Corbanu**, not as vulnerability claims against OpenClaw.
3. **The 87 upstream helper tests and 10 observation probes are reference evidence only** and are
   treated as such throughout, consistent with the export's own framing.
4. **The critical-path figure (34) is my own computation** from the `Depends on` column of
   `docs/sprints/current/p0-security-levels/index.md`. It counts dependency stages, not duration,
   and assumes the listed edges are complete and correct.
5. **PF numbers here are the reconciled current IDs** in this export. I have not cross-checked them
   against earlier branch-local IDs, per `REVIEW_SCOPE.md`.
6. **Model substitution is recorded, not concealed.** `REVIEW_SCOPE.md` specifies Fable 5 at High
   effort; this review was completed with **Opus 5 at Extra effort** at the user's explicit
   mid-session direction, which also redirected the artifact filename to `OPUS_REVIEW.md`. All other
   scope boundaries were honored unchanged.
7. **Severity and confidence are my judgment**, calibrated to a draft planning tree. "Missing" vs
   "covered but underspecified" is stated per finding so the owner can distinguish new work from
   sharpening existing work.
