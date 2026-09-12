# PF27 next source step — existing-root composition

Prepared September 12, 2026 from main `bba52cecc996a2f04ec558b7b86514426e67c845`.
Product initiative, PF-27-S04, same `/root` branch/worktree/allocation base.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”

## Decision and exact boundary

The admitted-child library is qualified; the default executable is not a
launcher. Do not repeat its finished review or claim its fixture exercises PF20.
The smallest next source change is an **existing-root composition adapter**:
open both real PF20 namespaces and connect the already-owned child lifecycle to
the correct `ControllerRoot::serve_child`. It is deliberately not an installer,
process launcher, enrollment command, Core policy factory or Vault integration.

Allocate `codex-rs/secret-broker-service/src/root.rs`, a separate
`root_tests.rs`, exports in `src/lib.rs`, its Linux dependency on the existing
`codex-protected-state` workspace package, and required Cargo/Bazel lock parity.
No edits to `codex-protected-state`, Core, Vault or the normal `src/main.rs`.
Keep production modules below 500 lines and this source stage below 800 changed
lines including tests; split rather than expand if that boundary is insufficient.

The adapter owns a `TrustedChildRun`, not caller PID strings. Existing-only
construction opens `ControllerRoot::open_journal_system()` and
`open_policy_system()`; either error drops/fences the owned run. No fallback,
alternate path/root UID constructor, enrollment or directory repair. Admission
selects the root only from `ChildRole` supplied by `TrustedChildRun`, then calls
unchanged `serve_child(stream, child)` outside the supervisor lock. Do not expose
a caller-selected namespace or generic handler through this wrapper. Polling and
shutdown delegate to the owned run; preserve incomplete-reaping status. A root
handler's error, return or panic already fences both children. Preserve PF20
errors, including ambiguous CAS; never retry a request or silently reconnect.

PF20's root storage is mutex-protected, `serve_child` takes `&self` plus the
actual retained `&mut Child`, and fixed-path factories require kernel root.
These are verified source interfaces, not permission to call them as root.
Keep roots alive for their handler lifetime without leaving an unowned child or
opening a replacement generation while previous cleanup is unfinished.

## Frozen launch/argv/descriptor target for the subsequent source stage

This is a target contract, **not implemented CLI syntax or an executable install
manifest**. It narrows the historical root-launcher-v2 proposal for a synthetic
anchor-only probe; broker dispatch, worker launch and real data remain later work.

| Process | Exact proposed argv after argv[0] | Inherited descriptors |
| --- | --- | --- |
| Root supervisor probe | `--open-existing` | 0 `/dev/null`; 1/2 bounded, sanitized supervisor output; no socket activation |
| Journal probe child | `--journal-child` | 0/1/2 `/dev/null`; no other inherited descriptors |
| Policy probe child | `--policy-child` | 0/1/2 `/dev/null`; no other inherited descriptors |

Proposed single executable name: `codex-protected-root-probe`, opt-in synthetic
build only. Unknown/combined flags, positional arguments and no arguments deny.
All three processes use the same frozen, root-owned executable; **actual absolute
deployment path and digest remain unassigned until build/review**. The root
chooses child role/argv and distinct non-root numerical UID/GID from an approved
root-owned manifest, not from child requests. Empty environment and fixed `/`
working directory; no PATH lookup, shell, credential environment or inherited
terminal. No controller socket, root file, channel key or bootstrap secret is
passed as an inherited FD. Children connect post-exec to the unchanged fixed
`/run/corbanu-protected-state.sock` and authenticate its root peer. The root
listener and accepted FDs remain parent-owned/close-on-exec. Channel descriptors
are newly opened by their child; MAC material crosses only that verified channel.

Before implementing that launcher, resolve and test safe supplementary-group
dropping, capability clearing, post-exec nondumpable state, closed FD allowlist,
and pinned executable identity without weakening `forbid(unsafe_code)`. A spawn
success or an arbitrary readiness byte is not containment attestation. Do not
use `pre_exec` unsafe code or a PATH-based external helper as an unreviewed shortcut.

Proposed bounded probe lifecycle: admit exactly two distinct role children
within three seconds; drive health at least every 100 ms; fail the run if either
role is absent at the deadline; no replacement in the same generation. PF20's
unchanged ten-second idle/frame deadline remains authoritative. No keepalive is
introduced to defeat the expiry test. Explicit supervisor cancellation or any
channel failure fences both roles; report unfinished cleanup rather than success.
New boot requires new children/generation and existing-root recovery, never
reenrollment. Probe results contain only synthetic outcome/role/error labels.

No enrollment argv is proposed in this stage. Separate explicit administrative
enrollment and a complete unit/UID/path/digest/stop manifest must be approved
before any native run. Do not create the fixed socket merely to test a parser.

## Frozen construction cases and evidence boundary

1. Non-root existing-only construction fails and relinquishes/fences both owned
   children; no directories, listener or enrollment are created. Run the case
   under verified non-root RTX UID, not root with guessed absent paths.
2. Journal and policy role mapping cannot be reversed by connection order or
   request bytes. Test private selection/dispatch plumbing with synthetic roots
   only where needed; explicitly label it unit construction, not native PF20 CAS.
3. Either root-open failure cleans up the captured run; second-open failure must
   also release the first root. Test private failure injection, not public fake
   authority constructors.
4. Channel return/error/panic and explicit shutdown propagate the existing
   whole-run fence; preserve pending cleanup and no transparent replacement.
5. Default binary still exits 78; non-Linux builds do not expose Linux roots.
   Confirm dependency/lock parity and no new default runnable service.

Run fix/format before final service default/fixture and affected PF20/broker
suites, strict Clippy and parity on RTX under the shared build lock. Exercise
construction/denial checks with actual keys in private TMUX. Record exact
source tree, counts and failures. Use the existing review budget for necessary
Astra High and Fable 5.1 High reviews; do not reuse prior clean conclusions.

True product-TUI/blind UX design and live TensorCash/Isometric workflows are N/A
for this internal uncalled adapter; they remain mandatory for later user-facing
integration. **Positive native journal/policy CAS and installed containment are
unverified**, and cannot be substituted with these unprivileged tests. No release,
benchmark, human-test-ready package or full PF27 completion follows this stage.

## Coordination and next action

Shared manifest/lock ownership request sent to manager task
`01a08522-76a1-7ad1-afe6-ad690d55c0d7` on September 12 before implementation.
Manager confirmed no conflicting manifest/lock/main writes and preserved root's
exclusive PF27 allocation. Private manager staging is separate, not a main
landing. Recheck before later integration. No main push in this planning checkpoint.
Next implement only the adapter above after governance checks. The later probe
launcher needs its own bounded source-scope amendment, then an actual frozen
native installation manifest and separate user approval. Existing review slots
are not installation authority.
