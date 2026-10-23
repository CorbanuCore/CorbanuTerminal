# Recovery delivery control → main PF-80 cutover

Status: concrete proposal for manager review; no migration or deployment run.
Product citation: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. PF-80-S01 remains in progress.

Manager-supplied live state: historical batch pending, posting OFF, enrollment
unverified, no delivery. This worker has not inspected live state and therefore
does not claim a batch count, live hashes, destinations or current receipts.
The 25-event fixture in the tests is synthetic and is not the server inventory.

## Identity rules

| Namespace and raw identity | Meaning | Required treatment |
| --- | --- | --- |
| Recovery source / PF-76-S01 | Historical delivery control | Preserve report bytes, event IDs, payloads, sequence, timestamps, attempts, status and receipts. Record provenance outside the raw objects. |
| Main source / PF-76-S01 | Unrelated provider persistence | Never assign its events to delivery control or a PF-80 task mapping. |
| Main source / PF-80-S01 | Receiving delivery-control initiative | Only newly observed work may produce a new report/event under an explicit mapping. |

The old report schema has no source namespace. A bare sprint ID cannot safely
distinguish the first two rows. The receiving collector therefore holds every
PF-76-S01 report outside initiative/run joins and shows a reconciliation warning.
Raw report files remain in place. The adapter refuses PF-76 enqueue/retry and
skips old PF-76 queue records without changing their delivery metadata, even if
a mistaken mapping points that ID at a delivery task. This conservative hold
also excludes genuine provider PF-76 reports from this dashboard until a later
source-aware contract is reviewed. It does not modify the provider initiative.

## Manager-owned inventory before cutover

1. Keep posting OFF. Pause source-sync and timer execution at the service
   boundary with operator authority; wait for any existing publisher to finish.
   Preserve the displayed last-good site. Do not run a flush or enqueue command.
2. Under the existing `.sync.lock` and `.outbox.lock` coordination, inventory
   the exact deployed export and immutable report/outbox files. Preserve a
   private owner-controlled backup outside automated three-generation pruning.
   Do not copy credentials, sessions or raw traces into Git or worker trees.
3. Record a private per-file manifest: source namespace, deployed source commit
   and tree digest, relative filename, SHA-256 of raw bytes, event ID, run ID,
   raw sprint/turn/goal IDs, workspace, task IDs, observation time, queue status,
   attempt/backoff metadata and any existing delivery receipt. Inventory the
   `outbox-index.json` dedup keys too; do not rebuild it from renamed reports.
4. Reconcile **every** report and queued record against recovery provenance,
   current main identities and the intended destination. A manager sidecar may
   record `recovery-delivery/PF-76-S01 → logical receiving PF-80-S01` for human
   navigation only, with original file hash, rationale and reviewer. It must
   not be loaded as a transport alias or rewrite `turnId`, `goal.id`, content,
   task IDs, event ID or report bytes. Unknown provenance remains held.
5. Assign a reviewed disposition per record: retain pending historical hold,
   retain delivered receipt, or retain blocked/conflict state for separate
   operator review. This plan authorizes no deletion, replay, retry, remap or
   requeue. Any future historical delivery decision needs explicit original
   account/destination/payload approval and separate implementation authority.

## Source transition with posting OFF

1. Manager reviews this port manifest and diff, registers the focused suite in
   the appropriate source-sync/CI ownership surface, and reruns it plus both
   governance suites on the combined tree. Do not infer remote CI success from
   local runs. The reported GitHub billing lock is an external gate.
2. Produce a reviewed export from the integrated main commit and compare the
   file manifest against the approved source. Use a temporary staging state
   containing only redacted synthetic reports first. Exercise successful
   publication, a corrupted source, recovery to last-good and the PF-76 hold.
   No live enrollment or credentials are needed for this rehearsal.
3. Review the incoming manager-owned config before any `activate.py` use.
   Require literal `tasknode.enabled: false`. Map only reviewed receiving sprint
   IDs; do not create a PF-76 delivery mapping. The installer starts a publisher
   immediately, so its first tick must remain OFF. Reconcile which historical
   reports remain collectible to avoid creating unintended new local events.
4. With publisher/source-sync serialized, activate only the verified source
   using the existing installer and then restore the scheduled source-sync to
   that reviewed main location. Native auth and runtime stores remain untouched.
   Existing installer preserves server-local enrollment and outbox; source sync
   must never overwrite them. No enrollment receipt may be inferred from config.
5. Compare old/new raw report and queue manifests, dedup index, delivery counts
   and enrollment bytes. Require exact preservation of historical IDs/history,
   zero delivery, posting OFF, visible unknown enrollment, correct PF-80 plan,
   and no historical PF-76 report assigned to provider persistence. Explain any
   new *local* event produced by a newly approved report; otherwise stop.

## First-event qualification is a separate transition

Keep the bulk timer sender OFF. Resolve the native transport, account scope,
target lifecycle, entitlement/enrollment and first-payload questions in
`native-contract-and-decisions.md`. The operator chooses one newly observed
PF-80 immutable ID and reviews `tasknode.py prepare --state STATE --event-id ID`.
No credential flag is valid for prepare/preview. Recheck the payload and scope
at the separately approved send boundary; preparation is not a send receipt.

A later sender must attempt only the approved ID, retain the original payload
on timeout and leave all other queue bytes untouched. Stop on identity conflict,
expired/wrong-owner authorization, entitlement failure, mapping changes or
tombstone responses. Do not use Python `flush`, native `Sync now`, enrollment
followed by bulk enablement, or native `capture` to qualify one event.

## Recovery and rollback

Before any source rollback, keep posting OFF and pause/serialize publishers.
Use the preserved private backup to compare state rather than replacing it.
Activating an older source is not an outbox rollback: it cannot undo remote
writes and must never replace current event/index/enrollment state. The old
source lacks the PF-76 hold, so do not resume its collection/sync against mixed
main/recovery data until the manager restores a provenance-correct source and
config. Recover a failed HTML publication through the verified last-good
generation; do not refresh the source age to disguise a failed transfer.

If any hash, namespace, entitlement or queue discrepancy remains unresolved,
leave publication at last-good, retain all history, keep posting OFF and record
the smallest operator decision. Do not mark PF-80-S01 complete or activate PF-79.
