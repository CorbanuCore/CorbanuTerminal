# Live permission transition — first durable checkpoint

Date: 2026-09-13. Status: design/allocation proposal only. **Runtime implementation is not authorized under any recorded allocation for this worktree. No product source has changed.**

| Coordinate | Verified value |
| --- | --- |
| Worktree | `/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913` |
| Branch | `fix/live-permission-transition-20260913` |
| Base / source inspected | `005cc644f59b1e762e5497b329e106c67925d4ed` |
| Initial status | Clean |
| Reported affected package | `0.1.42`, `c0d5b38fd`; supplied by parent, not inspected or executed here |
| New candidate | None; no runtime patch, compiled candidate, installation or release |

The source supports the parent diagnosis: a settings update acknowledges session configuration while an existing turn retains its approval/sandbox snapshots. Shared network and MCP configuration can refresh separately. The permission picker also updates its local state and prints success before the Core acknowledgement. See [source findings](source-findings.md).

Classification: **product initiative** for the complete correction. Root `AGENTS.md`, “Change classes”: “Any change to authorization, vault access, financial action, or protected-data disclosure is a product initiative even when reported as a bug.” A restore-only UI change could be a bounded fix under **Shipping MVP — LIVE**, excerpt “approvals, existing general sandboxing,” but it would not repair runtime authority. No such partial patch is presented as the fix.

Product citation: `docs/corbanu-product-spec.md`, exact heading **Live MVP versus the P0 security controls**: “The shipping MVP already has a wallet, vault, scoped signing, approvals, and general workspace sandboxing. These are live product capabilities.” The human's present request supplies the specific live-permission repair mandate. `/permissions` is distinct from **Product surface** under **P0 `/security` levels**; its immediate security-level transition promise is not rewritten or claimed implemented here.

The development skill routes this hold to root policy. `/Users/Neo/.codex/skills/corbanu-terminal-development/SKILL.md`, step 5: “Before product-initiative implementation, read `docs/sprints/index.md` and select or create one sprint”; step 7: “Implement only the selected sprint's remaining checklist in its recorded worktree.” The user additionally explicitly requires parent approval of the allocation before runtime changes.

Recommended next step is the [concrete allocation proposal](allocation-proposal.md): a PF-22 follow-up under the existing P0 security plan, with a safe explicit idle application boundary, backend enforcement, and acknowledgement-driven UI. Parent must accept the boundary, record the permitted plan amendment, and transfer the security sprint reservation explicitly. **Do not resume PF-27 or add a fourth reservation.**

Independent design is frozen externally at [frozen-functional-cases.md](/Volumes/CorbanuDrive/Corbanu/.codex-work/permission-transition.eocbhB/frozen-functional-cases.md). SHA-256 independently checked: `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`. Original F01–F11 and ambiguities are unchanged. These are acceptance input, not execution results. The implementer has not authored acceptance or dispositioned any case.

Actual checks: both governance checkers pass; their test suites pass 5 and 22 tests. Pinned Rust/Cargo 1.95.0 and locked offline metadata preflight pass. No runtime regression, live inference, true-TUI or acceptance test was run. See [verification and gates](verification.md).

Owned changes at this checkpoint are exactly these four records:

- `qa/reliability/live-permission-transition-20260913/README.md`
- `qa/reliability/live-permission-transition-20260913/allocation-proposal.md`
- `qa/reliability/live-permission-transition-20260913/source-findings.md`
- `qa/reliability/live-permission-transition-20260913/verification.md`

No canonical integration checkout, running app, profile, auth/session log, shortcut, release artifact, external branch or shared build cache was changed. No push, merge, process control, credential access or acceptance execution occurred.
# Current integration checkpoint

The original investigation below is historical. Current implementation, review
dispositions and remaining qualification gates are in
[integration-handoff.md](integration-handoff.md). V3 passed353focused tests;
independent functional acceptance and installation are not complete.
