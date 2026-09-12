# PF27 Stage B: private descriptor admission proof

Internal synthetic increment only; not a protected service or human-test candidate.
Product initiative PF-27 / sprint PF-27-S04 remains **in_progress**.
The initial passing suite below is superseded for readiness by Astra30's
confirmed buffered-EOF finding. Manager251074ef7 authorizes the bounded repair,
actual failing regression and corrective Fable32; no final acceptance yet.
Product authority: **Non-negotiable controls** — “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” No credential is used by this increment.

## Frozen scope and candidate

- Sole owner `/root`; local worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, branch `feat/security-broker-resume-20260911`.
- Original allocation base `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb` is unchanged.
- Incremental size/review baseline: accepted A `6d770938c21f002dc572b084a1b428ad4a4379b0`.
- Frozen source: **`b59999635298069831165f0fab08cd6be862f419`**; Rust tree **`194ff9575e4016d2c3b45ddd639945fbbbc836ab`**.
- [Allocation](../descriptor-admission-allocation-20260912.md) manager `67273f7e1`, size amendment `3e76ee4ec`; owner allocation checkpoints `30923e30f` and `7ba9f56e8` passed both governance checkers before their corresponding edits.
- Seven literal code/test/fixture paths, **863 additions plus deletions**: pair38, pair_tests23, spawn3, admission254, admission_tests465, connector51, runner29. Runtime admission remains254. Manager authorized the narrowly reasoned hard900 test/runner exception before exceeding800; no artificial baseline reset or omitted cases.
- No dependency, Cargo/Bazel lock, adapter/unsafe-FFI, children.rs, root.rs, Core, Vault, PF20, installer or native activation changes. No source push, main merge or release authorized here.

The existing reservation and precreated supervisor remain the sole child owner.
The new private capacity-two request queue returns a full/disconnected request's
stream without waiting. Admission uses actual kernel UID and peer pidfd, compares
the retained owned identities, derives role from the matched owner and rejects
duplicates/stale/dead/unrelated peers. The private generation receipt is not PF20
authority or a claim of perpetual liveness. Sticky cancellation and retained
shutdown clones fence both channels; A's both-child cleanup/permit rules remain.

## Exact RTX proof

Clean remote checkout `/home/travis/worktrees/security-broker-admission-20260912`
matched the frozen source/Rust hashes before and after the run. Non-root UID1001,
Linux7.0.0-31 x86_64, GNU2.43, Rust1.95.0; full identity is in
[provenance](rtx/final-provenance.txt). Builds used the exclusive existing
`/home/travis/security-round5/targets/pf27-pair-20260912` target, jobs4 and
`/home/travis/security-round5/locks/build.lock`. No other workload was stopped.

After scoped `just fix` and `just fmt`, the committed [runner](qualify-rtx.sh)
was invoked by separately sent command text and Enter in private RTX TMUX
`pf27admission20260912:proof`. [Capture](rtx/final-tmux-capture.txt) records
**PF27_ADMISSION_B_TMUX_COMPLETE**. All **17 per-command exits and the enclosing
suite exit are zero**. This is actual-key synthetic PTY proof, not a claim that
the Corbanu user-facing TUI or native/protected flow was exercised.

| Final check | Result | Literal log |
| --- | --- | --- |
| Adapter normal / qualified OS cases | 4 / 8 pass | `rtx/final-tmux/adapter{,-os}.log` |
| Default service | 3 pass | `rtx/final-tmux/default.log` |
| Preserved A pair / real pair | 5 / 1 pass | `rtx/final-tmux/{pair,real-pair}.log` |
| Preserved owner / real owner | 8 / 3 pass | `rtx/final-tmux/{owner,real-owner}.log` |
| Hold / inspect / connector ELF profiles | 1 each pass | `rtx/final-tmux/{profile-hold,profile-inspect,connector-profile}.log` |
| Complete B admission cases | 8 pass | `rtx/final-tmux/admission.log` |
| Full synthetic service | 59 pass, 11 ignored separately covered | `rtx/final-tmux/service.log` |
| Strict two-crate Clippy `--no-deps --tests -D warnings` | pass | `rtx/final-tmux/strict-clippy.log` |
| Read-only Bazel parity / unchanged locks and source | pass | `rtx/final-tmux/{bazel-parity,unchanged-locks,unchanged-source}.log` |

The 11 service exclusions are six new OS admission cases, three real owner
cases, one real pair case and one ELF-profile case, all explicitly run above.
Filtered test counts are not additional passes. The adapter's unsupported
GNU2.39 negative is excluded on GNU2.43; its historical fail-closed result is
not a new support claim. Existing Bazel platforms/rules_cc version warnings and
its automatic idle-server restart for the changed checkout are preserved.

## Required-case mapping

All names below use the `pf_27_s01_admission_` prefix in the final admission log.

| Required behavior | Case suffix and observation |
| --- | --- |
| Actual roles, reverse arrival, duplicate, parent, old generation, dead peer | `real_roles_reverse_duplicate_parent_old_and_dead`: actual children connect twice; role markers only observe; retained dead peer's actual pidfd is rejected before replacement admission. |
| Actual kernel UID mismatch and unrelated child | `real_uid_mismatch_and_unrelated_process`: verifier compares a deliberately different expected UID with real SO_PEERCRED; auxiliary connector is separately pidfd-owned, never an unowned subprocess. |
| Queue full, cancellation/deadline while queued, ticket/caller loss | `bounded_queue_cancel_deadline_and_lost_ticket`: worker held behind a deterministic gate, third exact fd returned, sticky cancellation, explicit gate release/completion, reservation unavailable until completion. |
| Success already formed, then cancellation/receiver loss | `late_success_and_lost_receiver_close_channels`: queued success cannot survive late cancellation or ticket drop; retained shutdown clone closes socket. |
| Worker cannot deliver a formed success | `worker_lost_success_receiver`: receiver deliberately absent, real socket verification with private identity doubles forms a receipt, worker send fails, clone exists and socket closes. |
| Credential/clone/identity/poll failures and pre-delivery cancel/deadline | `kernel_faults_and_pre_delivery_cancellation`: actual ENOTSOCK; process-isolated zero fd limit produces actual poll and clone EINVAL; explicit identity error and cancellation/deadline during identity work reject without installing any channel. Limits restored before assertions. |
| Either remote EOF or process death, receipt or caller drop | `real_peer_eof_death_receipt_drop_and_caller_drop`: eight actual journal/policy cases; synthetic `e` closes remote writes while process stays alive, `x` exits; shutdown wakes the other role's blocked reader; both children reaped. |
| Actual pair pending result / deadline cleanup | `real_pending_ticket_and_deadline_reap_both`: no early reservation reuse, both real children reaped, unconsumed result cannot return a stale success. Pending means not consumed; the worker may already have formed its reply. |

A pair and owner suites retain asymmetric signal/wait, late spawn, caller drop,
deadline and panic/quarantine coverage. No numeric PID is used for authentication.

## Fixture provenance and retained attempts

The connector is separately compiled without libc using the exact command in
[fixture-command](rtx/final-tmux/fixture-command.txt) and recorded compiler output.
Source SHA256 `a510a0787db823bed77d55cd9860284fecf2aefe7e60efc329c50fdf426b48a4`;
static-PIE artifact SHA256 `4b1e56b4bab25158d089197e5fe02c1523e9e52ad77075acba9c7ee96bb3fa17`.
The runner verifies both hashes and accepts the ELF profile **as data before
invoking** the connector through the existing sealed-image/pidfd launch path.
Recipe, compiler, ELF headers and hashes are committed; binary is retained only
in the external RTX evidence directory. Endpoint PID is rendezvous naming only.

Original attempts remain under `rtx/`: initial2 normal tests and initial3 actual
tests passed. The first non-PIE fixture was rejected by the unchanged ELF profile
and never invoked. `fault-initial` failed compilation because the test closure
lacked Send; `fault-send-repair` exposed a wrong EMFILE test expectation (actual
F_DUPFD_CLOEXEC returns EINVAL with zero soft limit); `fault-errno-repair` passed7.
An initial fix invocation duplicated the recipe's `--tests` argument; corrected
`final-fix-retry` and formatting passed before the final freeze. Initial remote
QA-file staging found the target directory absent; it was created, both files
copied, staged bytes compared against the frozen commit, then the clean checkout
selected. These are preserved setup/test defects, not concealed product passes.

## Reviews and remaining gates

Allocated **30 Astra High and31 Fable5.1 High through Corbanu/private TMUX** review
only this increment over accepted6d770938c, using this frozen scope and evidence.
They are not repeat A reviews. Results are pending; see [ledger](../review-budget.md).

Policy1.7 independent functional execution is accepted. Integrator-accepted N/A
applies only to this inaccessible synthetic stage. Later protected-user flows
and PF26 require code-blind design, a separate genuinely isolated executor and
independent evidence review. This proof does not qualify native services,
cross-platform behavior, live credentials, protected activation, live-repository
user workflows, benchmarks, human acceptance or whole-product release.
