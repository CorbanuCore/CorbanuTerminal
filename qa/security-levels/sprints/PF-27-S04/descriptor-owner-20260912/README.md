# PF27 private single-child owner — exact-stage evidence

## Current correction checkpoint (supersedes initial-source rows below)

Final source4d830cbb31330da21cf84b6f0193e7d56dc30776, Rust tree
7e488200b2cfc23ee9c99bf188ea9e76e9693558, clean matching RTX checkout.
Astra25 exited0/findings[] on initial30aebfb7. Fable26 exited1, overall patch
correct, with one P2 non-test expect_used lint failure. Strict feature-enabled
Clippy reproduced all three calls; [original failure](rtx/fable-lint-repro.log).
The earlier just fix exit0 is not strict-lint proof: Clippy fix mode can report
unfixed diagnostics without failing. Its command did enable synthetic-fixture;
the review's inference that the feature was omitted is not accepted.

Accepted correction is only20 changed lines in spawn.rs: return the owned image
on a missing reservation; use explicit let-else for impossible pending-image
violations, with documented panic-to-quarantine and Builder::spawn contracts.
No new successful cleanup inference, public API, dependency or feature change.
Final cumulative code/build/fixture715 lines, including runtime249/tests396.

Post-correction [fix](rtx/repair-fix.log), [format](rtx/repair-fmt.log) and
[strict feature-enabled Clippy](rtx/repair-clippy.log) exit0. Exact-source
actual-key TMUX rerun: [focused8](rtx/repaired-tmux/focused.log),
[real owner3](rtx/repaired-tmux/real-owner.log),
[profile hold1](rtx/repaired-tmux/profile-hold.log),
[profile inspect1](rtx/repaired-tmux/profile-inspect.log),
[service52 with4ignored](rtx/repaired-tmux/service.log), all exit0.
All four ignored cases are separately exercised as above. [Default3](rtx/repaired-default.log)
also pass. [Final capture](rtx/repaired-tmux-capture.txt) records matching source,
Rust tree and PF27_OWNER_TMUX_COMPLETE. No dependency files changed after the
successful Bazel parity check; its receipt remains applicable.

Fable27 is the necessary correction closeout, reviewing overfe007f32b (the
pre-correction owner checkpoint), not another review of the clean adapter.
Read the original full-stage contract below as context; do not expand this
correction into public service wiring or whole-workspace lint repairs.

Allocated synthetic-only increment, not a completed broker or native deployment.
Original PF-27-S04 owner/branch/base remain unchanged. Product requirement:
**Non-negotiable controls** — “Permit agents to reference credentials only by
label; resolve them solely inside the trusted execution boundary.”
Allocation/API: [frozen contract](../descriptor-owner-next-20260912.md).

## Frozen review scope

Review only this increment over fb7523f4b1fe3c8cfeafd8e11dca5d5f1be5c32f.
Source60068b9678c1db5bf53ec10a9ef2556400e327e6, Rust tree
30aebfb77785fe5108276a651a4005fca3c5bae3, identical clean RTX checkout.
The already-qualified adapter is unchanged; its clean Astra23/Fable24 reviews
must not be repeated. New private service manifest spawn.rs239 lines,
spawn_tests.rs396, sealed fixed-recipe bridge22 and narrow dependency/module
registration total669 code/build changed lines. QA fixture36 gives705, below800.
No public API, children.rs/PF20/Core/Vault/adapter semantic change.

Reservation precedes image preparation and worker construction. The worker
owns the pending image and eventual pidfd; caller cancel/Drop only sets an
atomic flag, with no join or new worker allocation. Absolute deadline is sticky
cancellation, not a hard interrupt or proof of cleanup. Launching/Running is
reported as CleanupPending when cancellation is observed. Shared mutexes are
never held over spawn, wait or signal. A late child is killed/reaped by the
existing worker before permitting another launch. No ready claim from early exit.

Worker creation failure returns image ownership without spawn. Pre-spawn
cancellation and qualified adapter rejection are terminal no-launch outcomes;
otherwise only observed reaping releases the permit. Signal/wait failures keep
the same owned child and retry at5ms intervals; recovery is permitted only when
that worker subsequently observes exit. Permanent failures can retain a worker
indefinitely. Panic is quarantined forever even if Drop happened to reap; no
force-reset or elapsed-time cleanup claim. Process-crash durable ownership and
external reapers are outside the qualified contract. Fixed synthetic recipe
IDs must match the adapter or reject before FFI; no identity substitution.

## Executed proof

RTX travis@100.99.88.49 UID1001, Linux7.0.0-31-generic x86_64, GNU libc2.43.
All builds/fix/format occur there. The service remains forbid(unsafe_code).
GNU2.39 was already proven unsupported by the adapter, not positively qualified;
this owner does not broaden its platform support. No live credential use.

| Check | Result / receipt |
| --- | --- |
| Scoped just fix, just fmt | Exit0 before final affected tests; local files copied from formatted RTX tree |
| Default service feature gate | 3 pass,0 skipped; [log](rtx/default-service.log), same frozen Rust tree |
| Focused owner | 8 pass,48 excluded; [log](rtx/tmux/focused.log) |
| Real pidfd owner | 3 pass,53 excluded: hold cancel/drop/early-exit; late real-child publication after caller drop; recipe mismatch rejection; [log](rtx/tmux/real-owner.log) |
| Static-profile artifacts | 1 pass each for hold/inspect,55 excluded each; [hold](rtx/tmux/profile-hold.log), [inspect](rtx/tmux/profile-inspect.log) |
| Full synthetic service | 52 pass,4 ignored; [log](rtx/tmux/service.log). Ignored owner3 run separately, existing profile case run twice above. |
| Actual-key TMUX | Text and Enter sent separately; private socket pf27owner20260912/session lifecycle; PF27_OWNER_TMUX_COMPLETE after all five commands exit0; [capture](rtx/tmux-capture.txt) |
| Cargo/Bazel parity | just bazel-lock-update exit0; [log](rtx/bazel-lock.log). MODULE blob unchanged5ecff077dbbbf7887a61c03a8e9aa354f002e1ed; Cargo.lock adds only the service dependency edge. Existing platforms/rules_cc resolution warnings are unchanged. |

Reproducer: [lifecycle-tmux.sh](lifecycle-tmux.sh). Qualified hold SHA
59ba145dc8178ae9680914943e8ba12c23c37d17ac4cc4a05a423539cbda1a6f,
probe f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0;
coordinator checked both before execution. Existing inspect hash/profile receipts
remain in the previous adapter evidence. These are static synthetic fixtures,
not new privileged service executables. Initial focused/real/service receipts
are retained alongside the final TMUX receipts; no source repair was needed.
An initial SCP brace path failed and was retried with literal paths; it was a
copy-tool error, not a test failure. Final source matches the frozen Rust tree.

Internal-only seam: no new TUI surface; code-blind functional UX design is
reasonably not applicable. TMUX provides lifecycle execution evidence, not
human acceptance or the two live-product repository release flows.

## Remaining scope and review

Astra25 and Fable26 results and accepted correction are recorded above. Fable27
correction closeout is the remaining scheduled allowance, reserved at dispatch.
PF-27-S04 remains in_progress: two-child admission composition, native service
containment, trusted credential migration/data-plane wiring, cross-platform
qualification, protected activation and final user-facing/live-repository
acceptance remain unimplemented or unqualified. No privileged install, main
merge, release, or dashboard update is performed by this stage.
