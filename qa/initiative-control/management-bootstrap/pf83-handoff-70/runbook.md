# PF-83 execution handoff — round 70

The round-67 four-binary package can now move without changing its attestation.
This round prepares transport and coordinator packet bindings. **Functional dispatch
is still blocked:** the current boundary only implements preflight, no admitted
independent executor/mediator receipt is supplied, and live identity verification remains pending (round 73 now has the owner-supplied
host key and UUID). This runbook does not authorize repairing or bypassing
those boundaries. No guest contact, staging, packaged launch or case occurred.

Routine QA preparation for product heading **Permission selection confirmation —
TO BUILD**, excerpt “A submitted selection is not a confirmed change.”
It supports active plan `p0-security-levels.md`, sprint `PF-83-S01`
(`in_progress`); it changes neither product behavior nor its authorization policy.
Independent functional execution and evidence review remain the later gate;
this internal preparation is not human-test acceptance or release qualification.
Integrator acceptance of this internal-stage N/A is not supplied.

## Immutable identity and relocation

In the commands below, run from the assigned checkout root:

```bash
set -euo pipefail
R=/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916
P="$R/qa/initiative-control/management-bootstrap/pf83-handoff-70"
: "${PF83_HARNESS:?supply the allocated harness root}"
H="$PF83_HARNESS"
A=9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27
M=32529d0a894503c021ff756ebc27a75555ab05ed2ec63dc47dc0d9d7bb878024
T=3b1afa1c84666f5b3dbc1a514d89bfd138fb15bae9a3813c7b6f5f7d66298ff0
I=c2dede899f790055990ef62fa5199794a628f4d93f408b78d6175f5897201f30
python3 -B "$P/verify_bundle.py" "$P/artifacts/handoff-root" "$A"
```

The declared root is the directory containing `build-attestation.json`.
`package.path=package`, `package.manifest=package-manifest.json` and
`binary.path=package/corbanu` are relative to it. The exact round-67 build
record is retained as `build-provenance.json`, bound by
`349ecadeae3d1d57125acc012103aa6bb92acf1eec42678669a817aaec162d53`.
Its absolute build/source/cache/tool paths are historical observations and are
never relocation locators. Source commit is
`ae5981d2d7d762dfd6d54e0ac0967729bf88f082`, tree
`834f57388066d33c27fdee6e3b234dec7fa0b0e6`; no claim that the binary was
built from this worker's later QA-only base commit.

[Move proof](move-proof.json) records verification, a real directory rename from
`artifacts/before-move` to `artifacts/handoff-root`, absence of the old root,
and identical verification after the move. [Stage proof](stage-receipt.json)
also records archive creation and fresh local extraction verification.
Round-67 evidence was preserved.

After every move or extraction, an inspector must independently supply A,
rehash the exact attestation bytes, rehash its referenced provenance and manifest,
check successful clean source provenance and exact commit/tree, enumerate all
four package members, reject links/special files/extras, and recompute each
binary SHA-256 and executable mode. `verify_bundle.py` performs these checks.
Do not edit or repin A because a location changed. Rehash a newly created
transport archive separately; T is a transport digest, not A or M.
The prepared archive's complete asset inventory is separately pinned by I.
Modes do not establish an isolation boundary; actual denied writes and
read-only enforcement still require admission evidence.

**Cache limit:** the build-time seed has only a recorded path, not a content
digest. This attestation cannot exclude reuse of an artifact compiled from
another source pin, establish a clean rebuild, or prove reproducibility.
Hashing the cache today cannot reconstruct its build-time contents. Successful
build/source-state evidence and final executable digests remain valid observations;
the stronger cache-origin exclusion is explicitly unproven.

## Packet rebinding and exact case order

All **288 selected** rows now have additive repointed copies at
`P/artifacts/rebound/packets`. [Packet bindings](packet-bindings.tsv) records
each original hash, new hash, path, projected case and all shared attributions.
Only `candidate` changed; [receipt](packet-rebinding.json) records checks for
unchanged non-candidate fields and unchanged originals. New candidates bind
commit/tree, M and A; version is unexecuted/null and the transport digest is
bound separately by the stage receipt, avoiding an archive self-hash cycle.
The whole historical normalized directory has **380** packets; 288 is the
F05–F09 selected subset. The other 92 are outside this handoff.

Mandatory first step of the future execution round, after host handoff below:
regenerate into a new destination and compare the resulting inventory. This
regenerates packets but is insufficient alone: the round-73 sequence also requires
an authorized live pin update and a successful dispatch guard:

```bash
python3 -B "$P/rebind_packets.py" --harness "$H" "$D/coordinator-packets" "$D/bundle" "$A"
cmp "$P/packet-bindings.tsv" "$D/coordinator-packets/packet-bindings.tsv"
cmp "$P/packet-rebinding.json" "$D/coordinator-packets/packet-rebinding.json"
```

Stop on any mismatch or occupied output; do not overwrite an old attempt.
The live `fixtures.py:CANDIDATE` is still stale and must be updated only after
owner permission; round 73 refuses dispatch until it exactly matches all rebound
packets. The historical normalized design, packets and `macos_adapter.py` remain unchanged. Do not run that adapter: it resolves
absolute attestation paths, uses its stale source pin and stages all 380
source-informed packets/amendments. The exact transport below consumes the
verified bundle directly; it neither changes nor disables that old verifier.

The **execution queue is the row order in packet-bindings.tsv**, starting at
its first data row: F05 (8), F06 (8), F07 (16), F08 (40), F09 (216).
This preserves the frozen variant order, including each fresh/existing profile,
both TensorCash/Isometric repositories, and existing-named variants where listed.
Finish the current attempt and seal evidence before advancing. A shared
attribution is retained, not a second execution or an independent pass.

The original headings in `packet/original-F01-F11.md`, digest
`c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`,
are the functional expectations. Rebound packets are **coordinator-only**
source-informed traceability, not a blind actor prompt. The actor gets only
the assigned original heading, neutral navigation, live screens and approved
synthetic fixture observations. The archive contains only that original file
and `navigation.md` as packet assets; no normalized packet, amendment, design,
prior findings, checkout, cache, profile or credential is staged.

Apply the original F05–F09 contract and case-specific limitations in the
[previous runbook](../pf83-packaged-63-runbook.md#exact-case-files-and-environment).
F05's withheld-approval premise is not removed. F06/F09 need actual turn/effect
observations; F07 uses actual offered selection handling; F08's exposed benign
failure branch cannot be replaced by synthetic injected variants. Unsupported
seams/navigation mean blocked/not exercised, never assumed passes.
F04's recorded refusal is not a proven F05–F09 refusal, nor permission to reroute
a refused case. Existing F01–F11 residuals remain open.

## Host handoff and guest staging — superseded by round 73

The round-70 full-bundle tar includes host build provenance and must remain
host-only. Do not upload it to the actor root. The old staging builder now
refuses execution. The [round-73 runbook](../pf83-failclosed-73/runbook.md) replaces
this section with an actor-assets-only payload, explicit identity preconditions,
the owner-supplied host key and UUID, and a mandatory candidate-disagreement
dispatch guard. It also lists the remaining owner actions. No guest operation
was performed in either preparation round.

## Remaining admission and execution prerequisites

The packaged runtime retains `native_dispatch=false` and
`op == "preflight"` guards. There is **no valid case-dispatch command** for it.
No invented `run-case` argument, direct SSH/tmux actor, new executor provider,
credential arrangement or weakened sandbox may substitute. The feature's product
heading also records the isolated broker as paused. A separate authorized
launcher/mediator work item, fresh admission evidence and independent executor
allocation must supply the actual approved case interface before this queue can
run. That is a real brief blocker, not a completed preparation step.

Require actual-executor and child filesystem/tool/process/IPC/network negative
probes, positive package/PTY/private-state/profile and mediated-inference controls,
and schema-2 identity/policy/nonce receipts. Use known synthetic witnesses only;
do not probe by reading real auth or provoking the operator's Keychain.
Freeze the admitted launcher/runtime/driver hashes and session/cost/budget.
Record TensorCash and Isometric canonical origin, resolved path and base commit;
use disposable worktrees and synthetic fresh/existing profiles. The four binaries
are an unsigned local qualification package, not a notarized release.

## Per-case captures and stop conditions

Each TSV row uses a new write-once host evidence directory
`D/evidence/<projected-case>/<profile>/<repository>/<case-id>/<attempt-id>/`.
Preserve `intent.json`, `launch.json`, `package-verification-before.json`,
`isolation.json` (schema 2), `controls.jsonl`, `actions.jsonl`, `pty.raw`,
`screens.jsonl`, numbered frames, fixture-before/after observations,
`observer.raw`, `evidence-index.json`, `package-verification-after.json`,
`cleanup.json`, `receipt.json`, `results.json`, and applicable
`failure.json`/verbatim refusal. Bind them to A/M/T, binary/helper digests,
original/neutral/variant packet hashes, profile/repo base, actor/child IDs,
effective policy hash, clock times, driver bytes and execution cost.

Record text and Enter separately, visible requested/applied/failed/uncertain
states, actual approvals/cancellation/recovery, stopped/completed turns, and
independent absent/started/completed/repeated effects. A screen or UI “Ran”
label cannot prove execution/non-execution. Index frames against raw-stream
prefix offsets and SHA-256; retain duplicate frames and timing limitations.
Reverify package hashes before and after every case and runtime asset hashes
against stage-inventory; mutable isolated state is separately inventoried.

Stop keys and all successor/retry dispatch immediately on a native credential
prompt, live profile/credential access, reachable prohibited source host, executor
refusal, identity/hash/inventory drift, failed/missing isolation or positive
control, unavailable case seam, loss of observer/PTY, exhausted deadline/calls/
evidence budget, or unexpected behavior that prevents evaluating the contract.
Preserve last checkpoint and exact error, mark blocked/contaminated appropriately,
and reap only owned processes through the admitted launcher. No credential
dialog response, sign-in, provider substitution, sandbox weakening or automatic
retry. A fresh repaired environment requires fresh admission and a new attempt.

Seal evidence with the existing `fixtures.py:seal/verify_tree` exact-byte
algorithm: sorted relative file hashes excluding only seal.json/seal.sha256,
hash the exact seal bytes, read-only modes 0400/0500, verify on collection.
Keep raw failures/timeouts and all corrections as separate attempts.
Map every original/variant to evidence or an explicit accepted disposition;
unexecuted branches remain visible. Independent evidence review and the
code-blind handoff checker remain required; neither unit tests nor this document
establish functional acceptance, human sign-off or release readiness.
