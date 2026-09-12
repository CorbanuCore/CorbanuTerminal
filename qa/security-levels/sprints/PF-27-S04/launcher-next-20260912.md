# PF27 launcher allocation — launch recipe and child identity preparation

Prepared September12 after accepted main `3e8bf6c9574892dc8630775c6dea6c74ae02a693`.
Product initiative **PF-27-S04**, existing owner `/root`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, branch
`feat/security-broker-resume-20260911`, unchanged allocation base
`d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`. No second security worker or sprint.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”

## Concrete next source boundary

Prepare the deterministic child-launch recipe and the dedicated post-exec
identity-preparation mode. This is the next part of the actual launcher, not a
claim that the root supervisor/listener, installed manifest or provider service
is complete. The accepted probe, admission and root adapters are reused.

Allocate only:

- `codex-rs/secret-broker-service/src/launch/` and its separate sibling tests;
- that crate's `src/probe/` for a narrowly named synthetic preparation mode;
- `src/lib.rs` only if a minimal launch-recipe export is needed;
- `tests/probe_lifecycle.rs` and a sibling launch-construction integration test;
- its Cargo/Bazel registration and existing dependency feature/lock parity only;
- PF27 QA, active plan, this sprint and the security progress record.

No Core, PF20 API/protocol, Vault, normal service startup or other feature edit.
Target under500 changed source/test lines; hard ceiling800 and modules below500.
If the coherent implementation cannot fit, update this boundary before coding;
do not silently bundle the root listener, enrollment and full data plane.

## Identity/group decision within existing PF27 authority

The historical v2 proposal retains three distinct non-root UIDs, three distinct
primary GIDs, and one distinct anchor-connect supplementary GID. Broker/journal
and policy receive exactly that supplementary GID. Worker receives none. Names
and numerical IDs remain unassigned until separately approved installation.
No shared principal, world-connectable socket or added worker access is proposed.

Use a short, pinned child bootstrap that begins as root after exec, checks that
it is single-threaded, then calls safe nix0.30.1 `setgroups`, `setresgid` and
`setresuid` in that order. All real/effective/saved IDs must become the target.
This is trusted launcher code, never arbitrary model/tool code running as root.
It avoids unstable `CommandExt::groups` and unsafe `pre_exec` callbacks.
Rechecked the pinned nix source on RTX: these safe functions return syscall
errors, unlike the noted `CommandExt::uid` EPERM exception for group clearing.

Validate the complete role identity table before creating any command: distinct
UIDs, distinct primary groups, distinct anchor group, no zero IDs and no
`u32::MAX` sentinel IDs (Linux treats minus-one specially in setres* calls).
Worker must not acquire anchor membership. A command recipe is configuration,
**not validated executable authority**, and must not be named or exposed as such.
Root-controlled manifest parsing and executable pin validation are a later step.

## Exact preparation and descriptor contract

- Keep current `--inspect-post-exec` behavior. Add exactly
  `--prepare-synthetic-child <journal|policy|worker> <uid> <primary-gid> <anchor-gid|none>`.
  IDs are canonical decimal; `none` is required only for worker. Arguments come
  from the trusted recipe; reject malformed, unknown, extra and contradictory input.
- Preparation must require actual real/effective/saved root identity and one
  thread before the first setter. An ordinary user invocation returns78 before
  group/UID/GID changes. Do not execute a root-positive test under this allocation.
- Disable keepcaps and set no-new-privileges; set exact groups, GID triplet,
  UID triplet; reuse post-exec capability clearing/nondumpability checks.
  Re-read exact IDs/groups and hardening. Any failed setter/check exits78;
  no partial-success record, retry, escalation or fallback to original identity.
- Recipe uses a host-local absolute executable path, exact non-secret argv,
  cleared environment, fixed `/` cwd and `/dev/null` stdio. No shell, PATH
  executable lookup, inherited secret environment, arbitrary command or script.
- Before preparation, deny unexpected inherited FDs; do not adopt/close raw
  descriptors unsafely. The later root supervisor must create CLOEXEC handles.
  No root-channel key or private descriptor is passed through argv/environment.
- A synthetic-only observation may use a test-owned diagnostic stdout pipe;
  label it explicitly, never call that exception the native child's FD contract.
  Production recipe stdio remains null. Preparation emits only bounded
  non-sensitive success/failure observations and always native_eligible:false.
- This mode terminates after preparation/inspection. It does not connect to
  the fixed socket or open enrollment. Existing native-child/open-existing modes
  remain78 until the actual supervisor/client stage is separately allocated.

Local path-bearing recipe values use `AbsolutePathBuf` or an explicitly checked
host-local `PathBuf`, not a worker-provided URI/string treated as authority.
No new package is expected; existing safe nix/rustix APIs suffice.

## Frozen construction cases and proof

1. Exact recipe argv/environment/cwd/stdio and role table; reject duplicate,
   zero/sentinel IDs, colliding anchor group and worker anchor membership.
2. Inject each private identity-setter/check failure; verify ordering and stop
   before later operations. Fakes are construction proof, not OS containment.
3. Actual non-root preparation invocation denies without changing the parent;
   invalid/mixed native modes deny. Existing actual inspection remains successful.
4. Exact supplementary groups and saved-ID mismatches fail closed. No secret,
   path, arbitrary environment or unexpected-FD contents enter diagnostics.
5. Recipe transport via a non-root, test-only subprocess records intended argv
   without taking real root authority or changing system identities/groups.
6. Retain current probe cases and the first failed FD9 fixture evidence. Address
   the deferred interpreter portability item in touched tests with a declared
   interpreter lookup and clear prerequisite failure, not silent skipping.

Use RTX serialized builds, fix/format then focused and affected tests, scoped
strict lint, Cargo/Bazel parity, and actual-key TMUX denial/inspection checks.
Record tested Rust tree and binary hashes. Existing transitive telemetry lint
debt stays separate. No product-TUI/code-blind UX gate is applicable to this
uncalled internal bootstrap; no user-facing or all-platform qualification claim.

## Review, next integration and installation boundary

Manager authorized up to3 necessary new review passes for this material stage;
record dispatches in review-budget.md, beginning14, without resetting history.
Use Astra High and required Fable5.1High Corbanu/TMUX on the frozen source;
the third is contingency, not a target or automatic extra opinion.
Request a fresh main window before landing; manager currently owns other lanes.

After this stage, separately allocate actual root-owned manifest/executable
validation, spawn/retained-Child cleanup, fixed listener and native client wiring.
Child connect may occur only after identity/hardening checks; preserve PF20's
kernel-peer/actual-Child API and timeout/CAS semantics. No new handshake is
authorized implicitly by this recipe. Full privileged qualification requires
approved frozen files/digests/argv/FD/unit/IDs/enrollment and exact stop actions.
No accounts, services, ACLs, root stores, real Vault data or installed app change
is authorized here. PF27 remains in_progress after this source stage.
