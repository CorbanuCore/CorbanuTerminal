# PF27 pair allocation: lifecycle first, admission second

Manager disposition following the owner's758-line scope stop, September12.
Read the current pair.rs, complete pair_tests.rs, peer.rs/peer_tests.rs and all
five tracked registration/signature diffs at owner HEAD685110d6f. No service
qualification or review is claimed. The single passing actual pidfs identity
test is supporting evidence only. No missing test is waived.

Accept the owner's staged option, not the1000–1100-line combined alternative.
Both are sequential internal units of PF-27-S04, not new sprints or initiatives.
Original owner worktree/branch/plan based870c92dab2bf3fbb602dc3b8447fe9f3534aecb
remain unchanged. Incremental accepted source remains62956038e77d41dd4ee1a3130b8b8ab77558a83f;
685110d6f contains the later allocation docs, not qualified pair implementation.
Record the actual launch checkpoint before narrowing the code.

## First preserve the incomplete combined candidate

Preserve exact tracked AND untracked bytes for all nine current source/test
files, manifest hashes and original758-line count in a recoverable private WIP
snapshot outside the landing tree. Preserve matching RTX copy and identify it.
No discarded tests, broad cleanup, reset/checkout-overwrite or push of the
unqualified candidate. Use normal reviewed file edits to extract Stage A;
retain admission code/tests as explicitly deferred Stage B evidence, not Done.

## Stage A — executable after allocation and both checkers

Deliver stable descriptor identity plus private two-child lifecycle ownership.
The minimum production-facing private surface is PairImages and
Reservation::launch_pair(images, deadline) -> Result<LaunchHandle,
(io::Error, PairImages)>, reusing the existing status/cancel/Drop behavior.
No generation/UID arguments needed solely for future admission. If actual
source requires a materially different signature, freeze that literal delta
before coding; do not expose placeholder admission functionality.

Remove admission entry points/types, request queues, socket polling/teardown,
generation/UID spec and fake Process::matches seam from the Stage A landing
tree. PairChild can depend on the existing Child lifecycle seam. The separately
tested safe OwnedChild::is_same_process predicate remains within the approved
adapter construction contract: identity only, not current liveness/readiness;
cached exited ownership rejects. No unsafe/FFI semantic or numeric PID fallback.

Exact Stage A code/test/fixture writes, repository-relative:

- codex-rs/linux-pidfd-spawn/src/peer.rs
- codex-rs/linux-pidfd-spawn/src/peer_tests.rs
- codex-rs/linux-pidfd-spawn/src/spawn.rs
- codex-rs/linux-pidfd-spawn/src/lib.rs
- codex-rs/secret-broker-service/src/launch/manifest/spawn.rs
- codex-rs/secret-broker-service/src/launch/manifest/spawn_tests.rs
- codex-rs/secret-broker-service/src/launch/manifest/pair.rs
- codex-rs/secret-broker-service/src/launch/manifest/pair_tests.rs
- codex-rs/secret-broker-service/src/launch/manifest/mod.rs
- qa/security-levels/sprints/PF-27-S04/descriptor-pair-20260912/qualify-rtx.sh

Reuse qualified static hold/inspect/probe images; no new socket connector in A.
Hard800 changed code/build/fixture lines remains, target at most300 runtime and
the balance for clear tests/runner. Additions AND deletions count against the
accepted62956038e source; do not reset the count against the758-line WIP.
Plan/sprint/allocation/evidence/review ledger writes remain in the owner's
already-reserved QA/docs paths and preserve all historical boundaries.

Required A proof: one permit before both images; worker failure returns both;
cancel/deadline before first spawn; first failure avoids second; second failure
retains/reaps first; cancellation between spawns prevents the second when it has
not begun; blocked late second spawn across cancel/caller Drop/deadline retains
capacity then cleans both; either child death fences pair; asymmetric signal/
wait failures attempt cleanup of BOTH without early release; panic quarantine;
only positive reaping permits reuse. Real non-root two-hold-child cancellation/
Drop and reap evidence must accompany deterministic failure/race tests.
Keep all existing single-child and adapter regressions and true-key TMUX;
strict scoped lint, actual host identity negatives and supported library checks.
No socket-admission or whole-product readiness claim in this stage.

Existing additional reviews28 AstraHigh and29 Fable5.1High via Corbanu TMUX now
cover this narrowed A candidate, not the incomplete combined WIP. Preserve
all prior budget usage and failures. No review is consumed merely to choose
the split. Record allocation and run plan/sprint checkers before implementation;
return qualified A for manager receiving integration/combined proof.

## Stage B — prepared scope, NOT executable yet

After A is accepted and locally integrated, freeze an incremental B allocation
against its actual accepted source. B adds the bounded two-request admission
queue, generation/UID/socket pidfd checks, role receipts, duplicate/unrelated/
old-generation refusal, delivery/caller/peer death fencing and socket shutdown.
Reuse preserved WIP where correct, never claim it previously tested/accepted.
The connector and actual kernel socket-admission tests belong here. Preserve
every original missing case, including queue-full, UID mismatch, late delivery,
peer death and blocked-read wakeup, mapped to B until actually executed.

Expected seams: pair.rs/pair_tests.rs, a private sibling admission module/tests
if needed, manifest registration, named connector fixture and qualify runner.
Manager will freeze exact files/API and its own hard800 incremental budget
after A defines the tested lifecycle seam. No speculative B code alongside A,
new process owner/permit, PF20 bridge or premature review. Stage B review
allowance will be recorded as an explicit extension, not borrowed or reset.

Both stages exclude children.rs/root.rs/protected-state/Core/Vault, credential
data plane, new dependencies/lock edits, installer, main/push/release and
protected activation. Owner uses its dedicated non-root RTX target/build lease;
canonical Mac accounting verification target remains untouched. Report an exact
scope break to the manager again if a complete A cannot fit800; no human blocker.

Actual narrowing launch HEAD db971e9b8 (UNQUALIFIED checkpoint, not a new size
baseline). Original WIP was already backed up to the owned remote branch before
this staged disposition arrived; no further unqualified push is planned. Nine
file bytes are additionally preserved outside the landing tree in local
/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-pair-wip-db971e9b8/source.tar
and RTX /home/travis/security-round5/evidence/pf27-pair-wip-db971e9b8/source.tar,
each with a per-file SHA256SUMS manifest. Four originally untracked files are
included. Original count743add+15delete=758. Existing owner branch and remote
pair worktree/target remain unchanged; compare final StageA against62956038e.
