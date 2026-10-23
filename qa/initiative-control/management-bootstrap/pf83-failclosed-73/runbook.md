# PF-83 round 73 — guarded future handoff

This is routine QA preparation under **Permission selection confirmation — TO
BUILD**, “A submitted selection is not a confirmed change,” supporting
`p0-security-levels.md` / `PF-83-S01`. Product code and its authorization
boundary are unchanged. Guest contact, package staging and functional execution
are forbidden in this allocation and were not performed. Functional handoff,
independent evidence review, integrator acceptance of internal-stage N/A, human
acceptance and release qualification remain open.

The owner supplied the public SSH key in [known_hosts](known_hosts), copied
byte-for-byte from `/private/tmp/pf83-known-hosts.txt`, SHA-256
`7366c535a80863731b09bb21a628cece6bd5750b8af9ef5646c339d32217418b`,
and guest UUID `F9AABE1C-BA2C-55D7-9859-A6ED560D3218`, macOS `26.2`.
Expected account is `agent`, uid `503`, architecture `Darwin arm64`,
address `192.168.64.3`. These are expected inputs, not a claim of current live
verification. Private SSH key bytes are never read, copied or printed by helpers.

## Mandatory future sequence

Only a later allocation with explicit staging and live-harness write permission
may perform this sequence. Run from the checkout root in one shell; each command
also has an explicit abort. Supply the allocated harness and a new receiving path.

```bash
set -euo pipefail
R="$(pwd -P)"
P="$R/qa/initiative-control/management-bootstrap/pf83-handoff-70"
Q="$R/qa/initiative-control/management-bootstrap/pf83-failclosed-73"
: "${PF83_HARNESS:?owner must supply allocated harness root}"
: "${PF83_RECEIVING:?owner must supply new host receiving directory}"
H="$PF83_HARNESS"
D="$PF83_RECEIVING"
A=9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27
mkdir -m 700 "$D" || exit
cp -cpR "$P/artifacts/handoff-root" "$D/bundle" || exit
python3 -B "$P/verify_bundle.py" "$D/bundle" "$A" > "$D/host-check.json" || exit
python3 -B "$P/rebind_packets.py" --harness "$H" "$D/coordinator-packets" "$D/bundle" "$A" || exit
cmp "$P/packet-bindings.tsv" "$D/coordinator-packets/packet-bindings.tsv" || exit
cmp "$P/packet-rebinding.json" "$D/coordinator-packets/packet-rebinding.json" || exit
```

Next, the authorized harness owner must replace the single
`fixtures.py:CANDIDATE` assignment with the exact `binding` object in
`D/coordinator-packets/packet-rebinding.json`, as a literal Python dictionary.
Preserve the prior file hash/bytes and record the revised file hash. This update
is mandatory; regeneration alone does not fix the stale pin. Do not regenerate
original packets or change their non-candidate fields. No live pin was changed
in round 73, because that path and permission are outside this assignment.

```bash
: "${PF83_SSH_IDENTITY:?owner must supply authorized SSH identity path}"
: "${PF83_OWNER_CHECK:?owner must supply approved read-only prerequisite verifier}"
: "${PF83_OWNER_CHECK_SHA256:?allocation must pin prerequisite verifier bytes}"
python3 -B "$Q/preflight.py" --harness "$H" --packets "$D/coordinator-packets" --bundle "$D/bundle" --receiving "$D" --key "$PF83_SSH_IDENTITY" --owner-check "$PF83_OWNER_CHECK" --owner-check-sha256 "$PF83_OWNER_CHECK_SHA256" --contact-guest > "$D/staging-preflight.json" || exit
python3 -B "$Q/prepare_payload.py" "$D/bundle" "$D/actor-payload" > "$D/payload-preparation.json" || exit
python3 -B "$Q/stage.py" "$D/actor-payload/actor-assets.tar" "$D/actor-payload/receipt.json" --key "$PF83_SSH_IDENTITY" --known-hosts "$Q/known_hosts" --evidence "$D/staging-evidence" || exit
```

Run `preflight.py` after the host move, regeneration and authorized pin update,
before the first payload preparation or guest staging step. It reports every
precondition and exits **1** if any fails, including missing owner inputs or
suppressed live checks. It checks receiving paths/fresh outputs, attestation at
the moved location, payload allowlist, all candidate/packet bindings, the pinned
host key, identity-file metadata, owner prerequisites and live guest identity.
It performs no copy, upload, pin write, product launch or case dispatch.
Without arguments it provides a local diagnostic run that must fail while owner
inputs are absent; a local diagnostic is never a successful staging preflight.
Round 82 makes the receiving-location check a prerequisite for all three artifact
checks: missing `--receiving`, wrong receiving paths or occupied outputs explicitly
refuse package/allowlist/packet checks instead of verifying the in-repo defaults.
The live identity report retains exact stdout/stderr bytes as base64, exit status
and decoded stdout when available, including mismatch/failure/timeout observations.
`stage.check_identity` compares those observed bytes to the frozen expectations;
a differing platform, architecture, UID, account, version or UUID fails the check.
The returned identity detail is the observation, not a copy of expected constants.

The remaining owner inputs require an owner-supplied, allocation-hash-pinned,
read-only verifier executable (`PF83_OWNER_CHECK`). This interface is not yet
supplied and is itself a failing prerequisite. For each `--check` below it must
exit nonzero when unmet; on success emit JSON with the matching `check`,
`status: "passed"` and nonempty `evidence` references. The script passes the
exact harness/bundle/packets/receiving paths, attestation and guest identity.
Its approved implementation must verify, rather than merely assert:

| Check | Evidence it must validate |
| --- | --- |
| `executor_mediator_interface` | Independent executor allocation; actual approved case interface and mediated inference; frozen launcher/runtime/driver hashes, session and budget. |
| `owner_admission` | Fresh schema-2 package/runtime/policy/session-bound admission; actual actor-and-child filesystem/tool/process/IPC/network denials and positive package/PTY/private-state/profile/inference controls. |
| `live_pin_authorization_and_hashes` | Explicit allocated-harness write permission and preserved before/after fixture bytes/hashes; after hash matches the live pin. |
| `independent_reviewer_and_staging_authorization` | Named reviewer independent of implementer/executor; explicit guest/account/snapshot/path staging and read-only actor-access permission, including assessment of embedded binary metadata below. |

Pin the verifier digest from the trusted allocation, not from an untrusted receipt.
This helper contract does not supply the missing verifier, authorize a checker,
prove independence, or replace admission/evidence review. Supplying a program
that only returns success would not meet it. Live identity contact runs only with
`--contact-guest` and all preceding checks passed. Preserve the report (including
failed checks) in a fresh attempt; rerun after any input changes.

`stage.py` repeats the staging identity precondition immediately before upload.
It first validates the public host-key file against the frozen digest; SSH then
requires that exact ed25519 host key, disables global known-host fallbacks and
interactive authentication, and aborts on any SSH error. With stdin closed, it
queries OS/account/UUID; explicit exceptions reject any mismatch, including under
`python -O`. Only successful identity verification reaches archive opening and
upload dispatch. Missing identity, host-key mismatch, wrong UUID or SSH exit 255
cannot fall through into transfer, regardless of the caller's shell options.
The collector checks guest identity again and verifies every transferred file
hash/mode and the exact directory set after extraction.

The new payload includes exactly four package binaries, original F01–F11 cases,
neutral navigation and a relative-name inventory. Sidecar attestation, provenance,
manifest, normalized packets, runtime source and gate-log files remain on the
trusted host; the collector is streamed and never stored under the actor root.
The packaged binaries still carry host build paths: these reveal the local account
name `Neo`, volume/worktree and QA-round names, internal Rust source/module filenames,
Rust 1.95.0 / aarch64-apple-darwin toolchain details, dependency names/versions and
build-object names. Excluding sidecars does not remove that implementation metadata
or establish code-blind isolation. The owner must assess this exposure before staging.
The original relocatable bundle and attestation digest are unchanged and are
verified on the host. Round-70 tar and its receipt remain historical host-only
evidence and are prohibited as guest payloads. No new product tar was generated
or staged in this round. Binary-internal build strings are not stripped or
claimed absent; changing those would change the frozen package identity.

After separate admission, every actual dispatch must use
`dispatch_guard.py --harness "$H" --packets "$D/coordinator-packets" --bundle
"$D/bundle" -- <owner-supplied admitted command and arguments>`.
The angle-bracket text is a required missing interface, not an executable command.
The wrapper verifies the original attestation/package, frozen packet inventories,
all 288 packet hashes and exact candidate dictionaries, and the live harness pin
before invoking the supplied command; disagreement raises an error and invokes
nothing. Record stdout/stderr and exit status per attempt. Keep the harness and
packets fixed for the whole dispatch; changes require a fresh check. This wrapper
does not grant admission or repair the absent case seam.

Before the first functional case, the future execution owner must also satisfy
[the round-83 per-case capture and retention contract](../pf83-f09gap-83/runbook.md).
The unrestricted stdout/stderr publisher here is not admitted for real capture;
source isolation and the specified publication filter must be verified first.
Admission must demonstrate the observer, event ordering and raw-source capture
needed by that contract. A successful staging preflight alone cannot establish those
capabilities or validate a functional result.

## Remaining owner asks

- Admitted executor/mediator seam: the owner must authorize and supply an independently allocated executor and an actual approved case interface with mediated inference, or authorize the separate work needed to provide it.
- `owner_admission` record: the owner must supply a fresh admission record bound to that executor, exact package/runtime/policy/session, with actual actor-and-child denial probes and positive package/PTY/profile/inference controls.
- Live harness pin permission: the owner must explicitly authorize updating the allocated harness's `fixtures.py:CANDIDATE` to the frozen rebound binding and record its before/after hashes.
- Independent evidence reviewer and staging permission: the owner must name a reviewer independent of implementer and executor and explicitly authorize the collector's guest/account/snapshot, staging paths and read-only actor package access for that review/execution allocation.

Host key and UUID are no longer missing owner inputs. Live validation of them is
still pending. Neither these inputs nor a successful guard supplies admission.
The current preflight-only runtime remains undispatchable until the admitted
interface is supplied. Preserve the round-70 queue order, per-case captures,
stop conditions, raw failures, sealing and functional gates; none is waived.

## Local evidence and limits

[Guard controls](guard-controls.json) and [raw output](guard-controls.log) record
a deliberately changed host key (zero SSH calls; zero transferred bytes/files),
a wrong UUID returned by a local transport double (one identity call with closed
stdin; zero upload calls or transferred bytes/files), SSH host-key rejection
(exit 255), valid identity, and a mock-only positive upload control.
No guest was contacted: the UUID and SSH-error controls prove local control flow,
not live remote SSH rejection or guest isolation. The wrong public-key file is
rejected by the actual pinned-file check before SSH can start.

The candidate-negative control uses a stale synthetic fixtures pin against the
real frozen packet inventory/package and proves zero dispatch calls; a matching
temporary pin checks all 288 packets. These are internal guard tests, not
functional cases or independent acceptance. Raw prerequisite and both guarded
Rust gate logs are included with this round, with exact results in RETURN.md.
