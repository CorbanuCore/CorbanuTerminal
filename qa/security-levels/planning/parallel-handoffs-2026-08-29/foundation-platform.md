# Foundation/platform lane handoff

Owner: primary agent, under integration owner Jim Ricketts.

This is the primary agent's leg. It continues the already-active PF-13-S05 and
does not create a second foundation sprint until PF-13-S05 is completed and
archived or explicitly returned to draft with a recorded handoff.

## Coordinates and start gate

- Current PF-13-S05 coordinates remain authoritative until an explicit transfer.
  Its sprint record still names its original branch/worktree; do not silently
  relabel prior evidence or move the sprint.
- Proposed next-lane worktree:
  `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-foundation-platform`
- Proposed next-lane branch: `feat/p0-security-foundation-platform`
- Build/cache root:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/`
- PF-27-S03 base: record the exact 40-character post-handoff `main` commit at
  dispatch. The analysis baseline was
  `a753283f9cd1a59ff2ae3b03319c3c4a3264326f`, but that is not execution authority.

Before PF-27-S03 becomes ready, update the plan and sprint with these exact
coordinates, a named owner, parallel lane `foundation-platform`, literal scope,
and integration gate; then run both governance checkers.

## Immediate work: close PF-13-S05

PF-13-S05 retains its existing literal `write_scope`. Remaining work is limited
to the recorded clean complete Core rerun and the final Windows canary on the
published repaired candidate, including source/artifact identity and the
directory-junction posture case. Do not broaden runtime scope without returning
to sprint classification.

The Windows and Linux routes are in local gitignored credential files referenced
from `AgentCredentials.md`. Do not disclose them. Use only synthetic canaries.

## Next work: PF-27-S03

Product citation: **Reconciled security scope — TO BUILD** — “Unknown or
unsupported protected paths fail visibly rather than falling back to raw secrets
or unscreened execution.”

PF-27-S03 freezes the containment contract and synthetic probes. It does not
activate a broker, protected mode, or automatic host-wide configuration.

Proposed literal unique scope:

```text
codex-rs/secret-broker/src/platform_contract.rs
scripts/security-platform-probe
qa/security-levels/platform/
qa/security-levels/sprints/PF-27-S03/
docs/sprints/current/p0-security-levels/pf-27-s03-platform-containment-contract.md
```

Exclude shared Cargo/Bazel manifests and locks, root module registration, shared
Core/TUI/Vault paths, plan/navigation, and all PF-13-S05 paths. The integration
owner alone registers the crate at G1.

Deliver a versioned Linux/macOS/Windows capability schema and probes from the
untrusted worker context for process, filesystem/config, inherited handle, IPC,
network, process-memory/debug, signing/entitlement, and elevation boundaries.
Distinguish supported, unsupported, and untested. Same-user separation,
notarization, installation, or a config flag is not containment proof. Define
authenticated human-controller IPC and protected-store ownership against
delete/rename/symlink/rollback/restart attacks. Unsupported paths fail visibly.

## Sequence after PF-27-S03

The foundation slot normally advances through dependency-ready work, one sprint
at a time:

1. PF-19-S02 and PF-20-S02 after PF-27-S03 as their dependencies permit.
2. PF-41-S03 after the policy/state contracts, then PF-21-S02.
3. PF-22-S02 after PF-19/PF-20/PF-21/PF-41 convergence.
4. PF-27-S04, then PF-27-S02, with all-OS probes against the actual launch path.
5. PF-13-S06 only under a separately audited scope; serialize it against any
   retained PF-13-S05 or broker overlap.

Re-evaluate the global dependency graph at each handback. Do not reserve later
sprint files early.

## Verification and review

For PF-27-S03 run schema/fixture tests plus the synthetic probe on all three OSes.
Record OS and engine versions, CPU/architecture, expected denial, actual result,
unsupported configurations, and test counts. Prove that fixture preparation
creates no runtime route. Run fix/format, affected suites, both governance
checkers, and `git diff --check`.

Follow the packet's Claude Opus 5.0 Max Computer Use protocol. Review questions
must cover same-user attacks, inherited handles, debugger/process-memory access,
IPC peer identity and replay, store rollback/symlinks, unsupported-platform
fallbacks, elevation/password persistence, and whether any probe result can
incorrectly enable protected mode.

At handback, include the frozen schema/fixture hashes, three-platform matrix,
candidate/base commits, scope audit, review evidence, unresolved mechanisms, and
recommended serialized workspace changes. PF-27-S04/S02 must rerun the probes
against the real implementation; PF-27-S03 alone qualifies no protected path.

