# Promotion prerequisites you can run

The previous recipe could remove the job before discovering a stale publication
write. Round 94 checks publication state and the other known prerequisites before
disarm, uninstall or replacement. The same gate runs again immediately before
disarm. Uninstall also independently checks publication state before bootout.

From the received checkout, use the disposable administrative venv described in
[the recipe](owner-handoff-80-promotion.md):

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control \
  "$OWNER80_ADMIN/venv/bin/python" -B \
  qa/initiative-control/management-bootstrap/owner_promotion_94_preflight.py \
  --manifest "$OWNER94_PREFLIGHT" --schedule "$OWNER80_SCHEDULE" \
  --transport "$OWNER80_TRANSPORT" --actions $OWNER80_ACTIONS
```

The program does not disarm, uninstall, write owner/coordinator state, launch
workers, open authentication files or resolve a pending write. Exit **2** means
one or more prerequisites are unmet; JSON reports every item by number/name and
its reason. Exit **0** means the supplied records and observed conditions satisfy
this advisory check. It does not create authority, authenticate a human signature,
verify prose evidence conclusions, or reserve state against concurrent changes.
The integrator must review the actual evidence and authority references.

| Item | Reported name | What is checked |
| --- | --- | --- |
| 1 | manager_bridge_authority | Named bridge adopted; explicit provider and --yolo policy; hashed preparation and authority evidence. |
| 2 | fresh_allocations_and_decisions | Each selected action prepared, exact bridged inputs and allocation digest, configured worktree, executable sprint/resources; old action cancelled by replacement, changed digest and fresh manager run; no active reservation on that allocation. |
| 3 | publication_pending | **Every direct *.pending entry** in the receipt's publish state, including broken symlinks, blocks. Nothing is deleted. |
| 4 | isolated_real_worker_qualification | Exact candidate package/binary binding; isolated transport/profile, enforced probe/mediation evidence, at least one real ACK/START/RETURN and independent review, OR a separately recorded named Travis limitation with allowed scope and remaining proof. |
| 5 | quiescent_live_audit | Fresh (at most 300 seconds) dispatcher/raw-delivery/publisher quiescence record tied to current coordinator revision; receipt, Python/runtime/config pins, service ownership/domains, plist/template/copy targets, paths, pending writes, armed fixture state, holds, unapplied operations, owner claims, authority schema and exhaustive planned handoff. |

The recipe additionally checks the installed runtime's status and the computed
BEFORE watchdog coverage, stages replacement/handoff/activation JSON and validates
the replacement config before disarm. It then rechecks the full gate. The exact
preflight revision/selection is retained for handoff; it does not silently choose
different inputs after uninstall. Future-state postconditions and command fences
remain at their actual steps.

The audit's future observations (successful bootout/install, OFF tick, arm,
recovery, final coverage and real worker effects) cannot be facts before those
effects occur. Likewise disk failures, service failures, a racing writer or a
process crash can occur after any preflight. Keeping dispatchers and publishers
quiescent is essential; the script checks the supplied attestation, it cannot
prove an external/raw sender has stopped. Do not describe this as an atomic
transaction or a guarantee that no later refusal is possible.

## Resolving a stale publication refusal

Keep the refused run and pending artifact. The integration owner first identifies
and quiesces the actual publisher, including any currently running scheduled tick;
a dispatch pause alone does not stop publication. Record the pending file's
metadata/digest and the last-good publication's digest without changing either.

After diagnosing the interrupted write and recording the recovery decision,
move the stale pending entry intact into a newly created private evidence
directory on the same filesystem, using a non-overwriting rename. Retain its
original name and provenance. Do not delete it, overwrite the last-good output,
or install its potentially incomplete bytes as a successful publication. If
authorship/completeness is uncertain, remain refused and escalate to the
integration owner. Only the operator performing that reviewed recovery may move
the live artifact; this preflight never does.

With the original writer still quiescent, rerun publication through its supported
path if required by that diagnosis, inspect its result, then rerun the preflight.
A new pending entry or another publication error is a new refusal, not permission
to loop on renames. This worker performed no live recovery.

## Manifest

Use a private 0600 JSON file. All evidence references are **redacted Markdown
files**, absolute paths, 0600, no symlinks, and SHA-256 pinned. The reader hashes
only those explicitly supplied .md files and never prints their contents. Do not
point references at credentials. Do not copy these placeholders as authority.

```json
{
  "preparation": {
    "bridge": "owner_tmux.freeze_worker_inputs",
    "provider": "ACTUALLY_AUTHORIZED_PROVIDER",
    "policy": "--yolo",
    "authority": {"path": "/absolute/redacted-authority.md", "sha256": "SHA256"},
    "adoption": {"path": "/absolute/manager-preparation-evidence.md", "sha256": "SHA256"}
  },
  "replacements": {
    "NEW_ACTION_ID": {
      "old_action": "CANCELLED_OLD_ACTION_ID",
      "evidence": {"path": "/absolute/replacement-and-decision.md", "sha256": "SHA256"}
    }
  },
  "qualification": {
    "mode": "qualified",
    "binary_sha256": "EXACT_TRANSPORT_BINARY_SHA256",
    "package_digest": "RECEIVED_OWNER_PACKAGE_DIGEST",
    "isolated_transport": true,
    "isolated_profile": true,
    "negative_access_probes": true,
    "mediated_inference": true,
    "real_ack": 1,
    "real_start": 1,
    "real_return": 1,
    "independent_reviewer": "ACTUAL_REVIEWER",
    "evidence": {"path": "/absolute/real-worker-evidence.md", "sha256": "SHA256"},
    "review": {"path": "/absolute/independent-review.md", "sha256": "SHA256"}
  },
  "audit": {
    "dispatchers_quiescent": true,
    "raw_key_delivery_stopped": true,
    "publishers_quiescent": true,
    "observed_at": 0,
    "coordinator_revision": 0,
    "evidence": {"path": "/absolute/quiescent-audit.md", "sha256": "SHA256"}
  },
  "activation": {
    "decision_id": "ACTUAL_DECISION",
    "revision": 1,
    "authority": "ACTUAL_NAMED_AUTHORITY_AND_REFERENCE"
  },
  "activation_authority": {"path": "/absolute/activation-authority.md", "sha256": "SHA256"}
}
```

Replace example counts/booleans only with actual evidence. Each selected action
needs a replacement record; the preparation provider/policy applies to this
selection. Refresh audit time/revision only after actually repeating the
quiescent inspection. The three activation fields must match the recipe's
OWNER80_DECISION_ID, OWNER80_DECISION_REVISION and OWNER80_AUTHORITY.
Use `owner_daemon.package_digest()` from the received source for the package
binding; do not reuse a digest from an older candidate.

For an explicitly accepted limited path, replace the qualification object with:

```json
{
  "mode": "limited",
  "binary_sha256": "EXACT_TRANSPORT_BINARY_SHA256",
  "package_digest": "RECEIVED_OWNER_PACKAGE_DIGEST",
  "accepted_by": "Travis Good",
  "limitation": "EXACT_MISSING_PROOF",
  "allowed_scope": "EXACT_ALLOWED_FIRST_ACTION_OR_TEST_SCOPE",
  "remaining_proof": "PROOF_STILL_REQUIRED_AND_STOP_CONDITION",
  "evidence": {"path": "/absolute/limitation-evidence.md", "sha256": "SHA256"},
  "acceptance": {"path": "/absolute/travis-acceptance.md", "sha256": "SHA256"}
}
```

The gate prints **LIMITED**, not qualified, for this route. The record must be a
real acceptance covering the proposed promotion/selected action, verified by the
manager. The script cannot establish that a person really authored an arbitrary
file. Writing "Travis Good" yourself grants nothing. A changed scope, candidate,
runtime or isolation condition needs an applicable acceptance.

## Which parts are yours, and which need Travis?

"Yours" here means the supervising manager/integrator within the already granted
bootstrap allocation, not this worker's permission to operate the live system.

| Ordered prerequisite | Manager/integrator can do | Travis is needed when |
| --- | --- | --- |
| 1. Adopt the bridge with explicit provider/policy | Modify actual manager preparation, retain exact frozen fields, cite existing provider/--yolo authority. | Existing allocation does not authorize the provider or execution policy. No model-name or profile-based inference of authority. |
| 2. Settle reservations, replace, accept fresh IDs | Reconcile through owner/coordinator APIs and accept a fresh decision using existing scope. Preserve old claims/history. | Reconciliation needs new scope or a product/go-no-go decision; do not manufacture completion to free resources. |
| 3. Clear the publication prerequisite | Observe the state while quiescent; preserve pending bytes/last-good publication; diagnose the interrupted writer and perform reviewed bounded recovery under integrator authority. No automatic delete/retry is provided. | Recovery would discard protected evidence or change product/data boundaries beyond existing authority. Ordinary diagnosis does not require a fresh Travis decision. |
| 4. Isolated transport/profile and real ACK/START/RETURN | Provision and run a previously authorized isolated test lane; use the exact package, authorized model/provider/effort/policy and independent evidence review. | New inference account/spend/credential use lacks authorization, OR any named limitation is proposed instead of the required proof. Only Travis, final product authority, can accept that limited scope. |
| 5. Evaluate unknown audit entries while quiescent | Coordinate cooperating dispatchers/publishers, read the real receipt/state, run this gate, retain evidence and resolve bounded failures. | An audit result needs a product decision or scope/authority change. This worker is explicitly forbidden to inspect live state and has not done so. |
| 6. Run corrected recipe | Execute under existing explicit promotion/activation authority once 1–5 are satisfied; monitor first interval and preserve all results. | Applicable promotion authority is absent or the intended action exceeds it. This repair itself grants no promotion or release authority. |

An **isolated inference profile** is a fresh per-run Corbanu profile in a
disposable VM or dedicated test account with enforced filesystem, process/IPC,
network and credential-store boundaries. All HOME/profile aliases point to that
run. It uses a test-only, explicitly authorized inference account or a mediated
inference endpoint; any auth link points only to that approved isolated test
credential. Never copy/link the operator's real auth file or use their Keychain.
If the exact provider uses native account state, provision it inside the
isolated account with the proper authorization; this worker cannot supply that
credential. No auth contents belong in evidence.

The lane must deny source/history/prior findings and real credential access to
the code-blind executor and its children, retain negative access probes and
positive package/PTY controls, and allow only the needed mediated inference.
Use the exact read-only candidate/assets and frozen cases with a separate
code-blind executor and independent evidence reviewer. Correlate actual
rollout/action/claim/digest/runtime evidence for ACK, START and RETURN, including
the important cancellation/failure/recovery paths of the applicable case set.
An empty CODEX_HOME, env -i, a synthetic worker, or an agent-tool credential
available to this conversation is not that lane.

A **named limitation is legitimate permission for explicitly limited testing**,
as root policy allows; it is **not substitute proof** and does not waive the
functional gate or turn synthetic evidence into real success. If used to label
promotion fully qualified, or without actual Travis acceptance, it is skipping
the proof. Carry the missing proof and stop condition forward. No acceptance or
real qualification was supplied or obtained in round 94.
