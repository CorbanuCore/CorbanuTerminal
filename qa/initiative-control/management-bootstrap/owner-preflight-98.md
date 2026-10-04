# RETURN — owner-preflight-98

## Basis and scope

Brief SHA-256 verified: `50484d8ff2d833d7a7cdb49fd108b42f8d0a097a7686e80defbd2e87ba8ae584`.
Base: `992bc253700c787eb9e5e37b06f5d8db8ceb4ad3`. Runtime: gpt-6-astra / high.
Allocation digest: `ee74c5495b65c0eb0b140b24cf6c0c54ad86abbfba02a66dddb29b14bcd5282f`.
Claim: `5379242a-906a-4fa5-a149-649186981240`.

Routine internal process clarification and regression evidence; no runtime or
security/authority rule changes. Product-spec heading **Internal delivery control
— TO BUILD**: “durable event dispatch, acknowledgments and watchdog”; “preserve
the last good publication on failure.” Existing initiative-delivery-control /
PF-80-S01 context; this does not advance its acceptance. Proposed code-blind/TUI
N/A: this change documents and tests existing administrative gates, without
changing a user workflow. Integrator acceptance of that N/A remains theirs;
the later isolated real-worker functional gate remains required. No live owner
root/coordinator, installation or credential contents were accessed. No promotion,
release, real inference, human approval, benchmark or live-repository qualification
is claimed. No commit or push.

## 1. Item 3 did not establish a live recovery

The supplied current result says **publication_pending PASS — checked**. Under
`publication_preflight`, that means the receipt's publication directory passed
private-directory checks and contained no direct entry ending in `.pending` at
the observation time. It does not establish publisher quiescence, publication
freshness, successful recovery or continuing absence of a racing writer.

The historical stale file was in the **disposable round-93 rehearsal**. The
[assertion audit](owner-shape-93-assertion-audit.md#additional-disposable-failure-observed)
explicitly says it is “not proof that the live installation has a pending file.”
Round 94 retained that failure and added the preflight guard; it did not clear
live state. No recorded live recovery is supplied in this round or established
by those artifacts. Thus there is no demonstrated live refusal-to-pass transition
requiring an explanation of who cleared it. The brief's premise that the old
review predicted a live stale publication is unsupported by the recorded finding.

I confirm the meaning of the supplied PASS and its consistency with the historical
evidence, **not a fresh live observation**: the assignment forbids touching the
live owner/coordinator. If the manager has an earlier *live* UNMET, retain both
runs' receipt/publish-state identities and timestamps and have the manager audit
the intervening publisher/recovery records. Do not invent a recovery receipt.

## 2. Exact symlink diagnosis and correction

Metadata-only inspection of the three noncredential paths supplied in the brief:

| Input | Observed symbolic-link components | Resolved path |
| --- | --- | --- |
| `binary` | None | `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-final9-20260910/bin/corbanu` |
| `tmux` | `/opt/homebrew/bin/tmux` | `/opt/homebrew/Cellar/tmux/3.7c/bin/tmux` |
| `runs_dir` | None | `/private/tmp/owner-runs` |

All three existed at inspection. No candidate executable was run and no auth
path was inspected. `owner_tmux.validate` checks runs_dir first, then binary,
then tmux with `fable_launcher.no_links`, which rejects a link in the path or
any ancestor. `auth_link` only receives absolute-path and basename (`auth.json`)
checks here; it cannot produce this `symlink_path` refusal. Its contents are not
opened by this validator.

Change **only the transport's tmux value** to the observed resolved executable:

```json
"tmux": "/opt/homebrew/Cellar/tmux/3.7c/bin/tmux"
```

Keep `binary_sha256` bound to the actual candidate; this is a binary digest,
not a tmux digest. Recheck the resolved tmux path at use time after any Homebrew
upgrade; do not automatically follow a newly retargeted Homebrew link. Keep runs_dir
private (0700, owned by the executing account), absolute and free of symlink
ancestors. `/private/tmp` avoids macOS's `/tmp` symlink.

**The check is right and this input is wrong.** A legitimate Homebrew convenience
link can be resolved explicitly without relaxing execution-path identity. The
transport/receiver later observes tmux identity and hashes its resolved executable.
No check was weakened or new auto-resolution behavior introduced.

Fixing this refusal permits item 4's qualification checks to run; it does **not**
mean those checks will pass. `/Users/Neo/.codex/auth.json` names the operator
profile, not evidence of an isolated test credential. For the required isolated
qualification lane, the manager must supply the authorized isolated test account's
`auth.json` path (or provision that lane's approved mediated setup), never link or
copy the operator's file. This is a separate prerequisite from the tmux input
correction. The validator's shallow auth-path check proves no isolation.

## 3. Preparation record and the correct integration point

Item 1 reads this top-level `preparation` object in the private preflight manifest:

```json
{
  "preparation": {
    "bridge": "owner_tmux.freeze_worker_inputs",
    "provider": "openai",
    "policy": "--yolo",
    "authority": {
      "path": "/ABSOLUTE/PRIVATE/approved-provider-policy.md",
      "sha256": "SHA256_OF_ACTUAL_REDACTED_AUTHORITY_FILE"
    },
    "adoption": {
      "path": "/ABSOLUTE/PRIVATE/actual-preparation.md",
      "sha256": "SHA256_OF_ACTUAL_REDACTED_PREPARATION_FILE"
    }
  }
}
```

`openai` is a worked example for an explicitly authorized OpenAI allocation,
**not inferred authorization from the model name or this worker's runtime**.
Use the provider actually authorized for every selected action; this manifest
has one provider/policy, so mixed-provider selections need separate promotions
or an explicitly designed future schema. `policy` must be exactly `--yolo`.

Each reference is an absolute, nonsymlink, single-link, executing-UID-owned 0600
redacted `.md` file, with nonsymlink ancestors. Hash the exact final bytes with
`shasum -a 256 /ABSOLUTE/PRIVATE/file.md`; never reference or hash credentials.
`authority` records who authorized the provider, model/effort and `--yolo` policy,
the bounded scope, decision identity/date and applicable restrictions. `adoption`
records the real preparation callsite/version, original frozen task fields,
bridge invocation with explicit provider/policy, resulting worker block and
allocation digest/registration evidence. Use actual observations, not the future
tense or the template itself as proof. Hashes establish file identity, not the
truth or authorship of its claims; the manager must review the contents.

The correct place is the manager's **allocation-preparation callsite before
`put_allocation` and before `run_cycle`**, not inside the accepting cycle.
`manager_cycle.run_cycle` briefs from already registered allocations, validates
the manager response, and calls `accept_decision`; that method rejects inputs
that differ from the registered allocation. It has no provider/policy authority
parameter. Patching the cycle to append `worker` would change frozen inputs after
registration and/or invalidate the accepted manager response. No cycle change
was made. The external preparation callsite is not supplied in this assignment
and is not in its writable scope. Manager adoption is still required.

This worked adapter returns exactly the inputs to register and the record to
place in the manifest. The referenced adoption file must describe actual bridge
use, and be finalized before using this record for preflight. It is neither a
live-state writer nor an authority generator. The regression test executes this
exact block with disposable synthetic evidence:

```python
# owner-preflight-98: worked preparation adapter
from owner_tmux import freeze_worker_inputs
import owner_promotion_94_preflight as gate


def prepare_for_registration(inputs, *, provider, policy, authority, adoption):
    frozen = freeze_worker_inputs(inputs, provider=provider, policy=policy)
    preparation = {
        "bridge": "owner_tmux.freeze_worker_inputs",
        "provider": provider,
        "policy": policy,
        "authority": authority,
        "adoption": adoption,
    }
    gate.bridge({"preparation": preparation})
    return frozen, preparation
```

Use the administrative environment from the promotion recipe, adding
`qa/initiative-control/management-bootstrap` to `PYTHONPATH` for `gate`. For example,
start with the existing allocation's exact frozen inputs:

```python
inputs = {
    "allocation": "YOUR_ALLOCATION_ID",
    "base_commit": "YOUR_EXACT_BASE_COMMIT",
    "brief_file": "/ABSOLUTE/PRIVATE/brief.json",
    "brief_sha256": "YOUR_ACTUAL_BRIEF_SHA256",
    "model": "gpt-6-astra",
    "reasoning_effort": "high",
    "task": "YOUR_EXISTING_FROZEN_TASK",
    "worktree": "/ABSOLUTE/AUTHORIZED/WORKTREE",
}
frozen, preparation = prepare_for_registration(
    inputs, provider="openai", policy="--yolo",
    authority=authority_reference, adoption=adoption_reference,
)
# Copy the current allocation envelope; keep its sprint/kinds/resources/scope/
# timeout unchanged. The envelope's inputs exclude the allocation identifier:
replacement = {**current_allocation,
               "inputs": {k: v for k, v in frozen.items() if k != "allocation"}}
# After quiescence and settlement, the manager uses the existing owner API:
# coordinator.put_allocation(inputs["allocation"], replacement, True,
#                            observed_revision, actual_replacement_evidence)
# Then run a fresh verified manager cycle; do not manufacture a launcher receipt.
# manifest["preparation"] = preparation
```

The added block is exactly:

```json
"worker": {
  "model": "gpt-6-astra",
  "provider": "openai",
  "effort": "high",
  "worktree": "/ABSOLUTE/AUTHORIZED/WORKTREE",
  "policy": "--yolo"
}
```

All original fields remain byte-for-byte values; flat `reasoning_effort` maps to
nested `effort`. The allocation digest is `coordinator.digest(replacement)` over
the **whole allocation envelope**, not the brief hash or merely the worker block.
Call `put_allocation(..., replace=False, ...)` only for a genuinely new allocation;
item 2 in this migration preflight specifically requires an old/new replacement.

## 4. Replacement evidence is a real state transition plus a sidecar

The manifest must contain one entry for **each selected NEW action ID**, no
missing or extra entries:

```json
{
  "replacements": {
    "YOUR_NEW_ACTION_ID": {
      "old_action": "YOUR_CANCELLED_OLD_ACTION_ID",
      "evidence": {
        "path": "/ABSOLUTE/PRIVATE/replacement-and-decision.md",
        "sha256": "SHA256_OF_ACTUAL_REDACTED_REPLACEMENT_RECORD"
      }
    }
  }
}
```

That Markdown file should cite the old/new action IDs, shared allocation ID,
old/new allocation digests, actual replacement revision and `owner_cancellation`
evidence reference, distinct old/new `manager_run` IDs, and the fresh verified
manager acceptance/receipt. Record reservation settlement, exact preserved scope,
worktree/runtime and the new prepared status. It uses the same file/hash rules
as item 1. The gate checks the hash and the corresponding coordinator state;
it does not parse prose to create that state or authenticate the source receipt.

For an old hand-prepared action lacking `worker`, **a sidecar is insufficient**.
Settle active reservations through their proper completion/reconciliation paths.
Use `put_allocation` with `replace=True`, an exact observed revision and actual
replacement evidence to register the bridged inputs. This operation atomically
cancels *all prepared actions on that allocation*, retains their old frozen
inputs, records `owner_cancellation`, and stores the allocation version history.
It refuses active reservations and duplicate allocations. Settle any owned
manager cycle first: replacement changes the revision and would stale its decision;
item 2 separately requires no owned manager cycle. Then obtain a fresh accepted
manager decision with new action IDs.
Do not edit the SQLite JSON, alter an existing action, or fake ACK/RETURN/acceptance.

Item 2 verifies: new prepared worker; exact bridge result; configured worktree;
current allocation digest and exact inputs; distinct cancelled old action with
`owner_cancellation`; same allocation ID but changed digest and different manager
run; nonempty new manager run; no active allocation reservation; hashed evidence;
executable sprint and available resources. An action prepared by hand is not
rejected merely because of that label: if it **already** satisfies these recorded
replacement/binding conditions, only the missing manifest sidecar may be needed.
A first-time bridged action with no cancelled predecessor still fails this
migration-specific gate. Do not invent a predecessor or replace an identical
allocation just to satisfy it; the recipe is scoped to replacing old flat actions.

## 5. Shortest honest path from today's results

1. **Yours (manager):** change tmux to the resolved Cellar path. Item 3 needs no
   presumed recovery. Preserve today's supplied result and rerun it later while
   quiescent; investigate only if an actual live pending entry/refusal is found.
2. **Yours:** adopt the bridge in allocation preparation with already-authorized
   provider/policy, and record/hash actual authority and adoption evidence. Settle
   affected reservations, replace the old flat allocations through the owner API,
   obtain fresh verified manager decisions, and populate the replacement map.
   These satisfy items 1/2 only when real state matches, not when templates exist.
3. **Yours:** provision the authorized isolated inference lane and execute the
   exact candidate's frozen functional cases with a separate code-blind executor,
   enforced filesystem/tool/process/IPC/network/native-store boundaries, negative
   access probes, positive package/PTY controls, and real correlated ACK/START/RETURN.
   Obtain independent evidence review and hash its records for item 4. The tmux
   correction is only the first item-4 prerequisite. **Travis's** step is required
   if new provider/policy/credential/spend authority is missing or if requesting a
   named limited-testing exception; record his actual scope/remaining-proof
   acceptance. No exception is supplied here and LIMITED is not full qualification.
4. **Yours:** stop raw-key dispatch and quiesce dispatchers *and publishers*;
   record the actual revision and fresh observations (at most 300 seconds old),
   hash the audit, and run all five gates. Item 5's `item_2_required` is a dependency
   skip, not a completed audit or evidence that other technical checks pass.
5. **Yours:** under existing explicit promotion authority, execute the corrected
   [promotion recipe](owner-handoff-80-promotion.md), including its repeated gate
   before disarm, exact revision/generation fences, staged inputs, and actual
   first-interval/postcondition observations. Preserve any refusal and reconcile
   without blind retries. **Travis's** step is needed if that promotion/go-no-go
   authority is absent or the requested action exceeds it.

**Mine (this worker):** this diagnosis, exact adapter/schema, regression evidence
and full requested test campaign. No further worker code change is necessary to
relax gates. I did not operate the live system or adopt the manager's external
callsite on its behalf. The actual isolated qualification and live audit remain
open; the brief's first item-4 error does not establish that either already exists.

## Verification and changed lines

Final counts, logs and diff accounting are recorded in
[the verification record](owner-preflight-98-verification.md).
