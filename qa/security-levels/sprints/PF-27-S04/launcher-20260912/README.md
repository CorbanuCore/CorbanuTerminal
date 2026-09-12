# PF27 launch recipe and identity preparation — construction qualified

Product initiative PF27-S04; exact owner/allocation and frozen cases are in
[launcher-next](../launcher-next-20260912.md). Requirement heading:
**Non-negotiable controls**, “Permit agents to reference credentials only by
label; resolve them solely inside the trusted execution boundary.”

## Frozen scope / review packet

Baseline `08a32a80e534a8ff7656e11e2e27649a90f9b6b8`. Review staged/uncommitted
source while it is local, or that branch base after committing; never an empty
local diff. Local and RTX Rust tree **`49d901362418ee11232d70e468cebe84ec37abfb`**.
Nine source/test files,584 changed lines (572 additions/12 deletions), all inside
`codex-rs/secret-broker-service/`. Retain the complete failure/identity/recipe
regressions despite exceeding the500-line target; still below800 hard ceiling,
with production modules below500. No dependency, lock, Core/PF20/Vault change.
Qualified source commit **`f0b1209e003ac56f17fa0713a12ca1721bfd88e1`**.

This is a launch **recipe**, not executable pin validation or root authority.
Synthetic feature gates the recipe API and existing probe binary. Recipe checks
an absolute host-local path and a complete distinct nonzero/non-sentinel role
identity table; config remains untrusted until the future root manifest/pin gate.
The recipe constructs only the exact synthetic preparation argv, cleared env,
fixed `/` cwd and null stdio. It never spawns itself or invokes a shell.

Preparation uses only the dedicated probe's single-threaded process. It requires
actual root UID/GID triplets and a closed FD allowlist before any setter; sets
no-new-privileges and disables keepcaps, exact groups, all three GIDs then UIDs;
re-reads target identity/groups, clears/rechecks capabilities and nondumpability
through the accepted hardening code, then rechecks IDs/groups/FDs. Errors produce
only a fixed diagnostic and exit78; no partial-success output or fallback.
Successful synthetic observation remains native_eligible:false and exits.
No fixed socket, root stores, enrollment, native client or service activation.

Review actual parser/role collisions/sentinel treatment, root/saved-ID/thread
guards, order/error boundary, exact group matching, environment/stdio construction
and evidence limits. No new protocol, supervisor or unsafe pre_exec is in scope.
Private identity-ops fakes prove ordering/failure stopping, not actual root drop.
No root-positive test has been authorized or run. No reviewer should run one.

## Cases and current proof

- Invalid/colliding/zero/sentinel IDs, canonical args, all roles and group rules:
  `src/launch/recipe_tests.rs`.
- Every private operation failure stops later calls; exact saved-ID and group
  mismatch checks: `src/probe/prepare_tests.rs`, no in-process kernel mutations.
- Real nonroot process rejects preparation, malformed and extra arguments;
  existing actual hardening/FD isolation tests remain: `tests/probe_lifecycle.rs`.
- Actual nonroot test-owned shell executable records exact numeric recipe argv,
  null stdio, cwd and absence of HOME without recording environment values:
  `tests/launch_recipe_lifecycle.rs`. Shell is fixture-only, not recipe execution.
- Deferred P3 interpreter path fixed in touched tests: resolve python3 via PATH
  and fail with a clear prerequisite message, never silently skip missing Python.

RTX `/home/travis/worktrees/security-broker-launch-20260912` is a fresh detached
worktree at the baseline, staged with the exact tree above. Qualify script and
TMUX script are committed beside this packet. Runtime evidence goes under
`/home/travis/security-round5/evidence/pf27-launch-20260912/verified`.
Fix/format, scoped strict Clippy default/synthetic, parity, default3/fixture29,
affected356 (2 pre-existing PF20 subprocess-helper skips), build and actual-key
TMUX all pass. No compiler/test failure occurred in this stage. Published output
under `rtx/` strips trailing spaces/blank lines only; raw output remains on RTX.
Ignored `.log` files are explicitly tracked before the final Fable evidence pass.
Probe SHA256 `f1d68f0d8d7ada9dd10ebfbf4ff8c48496710fa9ee8d902c26d8ef9051019a3f`;
other candidate hashes are recorded in `rtx/binaries.sha256`.
Inherited full transitive telemetry lint debt remains separate, owned by manager;
`--no-deps` is not a full transitive lint pass. Locks unchanged after parity.

## Gates and disposition

Manager granted +3 necessary passes for this stage, without resetting history.
Astra14 and Fable15 both finished with **exit0, findings[]**, unchanged source:
[Astra result](astra-fourteen.json), [Fable result](fable-fifteen.json).
Two passes consumed; the third remains unused. No extra review is required.
Both used the global structured helper in local mode on the staged baseline
diff, explicit High effort and this scope packet. Fable5.1 ran through Corbanu
under private TMUX, not a substituted engine. Exact scripts/results remain at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-launch-review-20260912/`.
Fable's initial TMUX new-window attempt found Astra's already-finished server
gone; its engine had not started. A guarded new-session launched Fable once.
The original exit/JSON/text outputs are published; no findings were suppressed.
No code-blind user UX/TUI or live-repository benchmark is applicable to this
uncalled synthetic bootstrap; TMUX supports construction, not human readiness.
No claim of root-positive, all-platform, native containment or PF27 completion.
Main integration requires a fresh serialized window and exact tested source.
