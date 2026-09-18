# RETURN — owner-limited-113b (recipe corrected by owner-broker-114)

The operational recipe and scope below incorporate round-114 corrections.
[Current return, route analysis, authority and disclosures](owner-broker-114-return.md)
is the current qualification reference. Round-113b provenance, tests and change
accounting at the end remain historical; they do not describe round-114 edits.

Allocation digest: `1b7fc8d814c30a8f18786f03bb20ff9b26321abea393bd3dbd3ca9c4b0981383`.
Claim: `9826e71e-84b9-47e9-9942-be30f5e44687`. Runtime: gpt-6-astra / high.
Base verified: `36b092a80bb77df0ca6a6f5682913ace9f6fa843`.
Brief SHA-256 verified first:
`eb7983dfb6b3f465982a4d8e13d29674a8e6da81cd00da54f74c54f2e5619988`.

Routine correction of administrative instructions and regression evidence; no
runtime, authorization or gate change. Product context: **Internal delivery
control — TO BUILD**, “durable event dispatch, acknowledgments and watchdog”;
“authorizes the bootstrap and bounded live qualification, not premature product
sprint resumption.” No plan/sprint advance. This document is a future recipe,
not qualification, product acceptance or an approval record. No VM connection,
live journal/schedule/transport/coordinator access, live preflight, qualification,
credential read, commit or push occurred. “Mine” below means the Fable manager,
not this revise worker. Values in capitals are operator-supplied inputs, never
invented pins or authority. The supplied VM facts were not independently verified.

## P2 correction and exact scope

**The minimum buildable boundary is the guest machine.** Run the pinned controller,
TMUX and worker as `agent`, UID 503, inside that disposable guest. Keep production
state, the Corbanu checkout/history, prior review reports, host files/IPC and real
provider credentials outside it. Use an external, bounded inference broker.
Record controller/worker same-UID trust as accepted scope of this transport
lifecycle measurement. The worker can read the staged Python controller runtime,
its auth symlink target, other same-UID files and applicable process/IPC surfaces.
The guest boundary does not deny those accesses. `--yolo` supplies no confinement.

`owner_tmux.processes()` selects `ps -U <uid>`; `BridgeReceiver.observe()` checks
socket ownership and worker UID; `TmuxAdapter.prepare()` records its UID and
creates the auth symlink. `validate()` checks that auth path is absolute and
named `auth.json`, not that its target is safe. Permissions owned by UID 503
cannot protect the controller from that same worker. Even a chmod-read-only
package owned by 503 can be made writable by 503; hash verification detects drift,
it does not prove immutable storage. A host-provided read-only mount can enforce
package immutability, but does not hide readable helper source from this worker.

**Transport changes, not provisioning:** separate actor UID or namespace with
new identity/collection rules; a transport-native broker-only credential contract
that removes its readable auth-file interface; enforced worker-level child/tool/
process denial from the controller. Each needs a new allocation, implementation,
review and re-pin before qualification. The current recipe keeps long-lived
upstream credentials broker-only outside the guest, while retaining a readable
synthetic auth marker for compatibility; it does not claim the stronger interface.

The independent outer executor may receive only frozen cases and a constrained
PTY/control capability, but the in-guest worker remains trusted with the controller.
**Product-authority basis:** Travis's existing written ruling, relayed verbatim
in the round-114 brief, says the sandbox exists only to stop the functional tester
reading the codebase; if it cannot read the codebase and can perform functional
tests, the arrangement is successful; take the simplest possible path and treat
it as qualified when those conditions are met. This is existing authority, not a
new decision or this worker's narrowing. The current return maps each of the four
gate booleans to its exact assertion and exclusions under that ruling. No new
Travis acceptance is required for those covered conditions. Exposed staging
helpers are accepted transport machinery, not a successful source-denial probe;
the tested Corbanu codebase/history/prior findings stay outside the guest.
Stronger worker-versus-controller confinement remains unproven and unclaimed.

Changed location: replaced sections 1–4 and the corrections section of
`owner-isolated-112-return.md` with an explicit supersession/correction; retained
its historical test provenance and raw logs. No contradictory worker-level
confinement requirement remains in that return.

## Item 4 predicates, in code order

Source: `owner_promotion_94_preflight.py`: `evaluate`, `qualification`, `reference`;
`owner_tmux.py`: `validate`; `owner_daemon.py`: `private_file`, `package_digest`.

Before either branch, `evaluate()` loads the transport JSON through `owner.load`:
regular file, executing UID, mode exactly 0600, one hard link, no symlink in file
or ancestors, strict JSON within 262144 bytes. `validate()` then checks:

1. Exactly keys `kind,binary,binary_sha256,tmux,runs_dir,auth_link`; `kind == "tmux"`.
2. `binary`, `tmux`, `runs_dir`, `auth_link` are absolute paths (in that order).
3. `runs_dir` is a directory owned by the executing UID, no group/other access,
   no symlink in it or any ancestor (0700 is the usable provisioning choice).
4. SHA-256 of nonsymlink `binary` equals `binary_sha256`.
5. Nonsymlink `tmux` is a file. Executability is operationally necessary but is
   not a separate predicate here.
6. `auth_link` basename is exactly `auth.json`. Target existence/content/ownership
   are not validated at this step; do not read it to diagnose this validator.

For the **limited branch**, `qualification` checks exactly:

1. `qualification.binary_sha256 == transport["binary_sha256"]`, then (short
   circuit AND) `qualification.package_digest == owner.package_digest()`.
   Failure: `qualification_candidate_mismatch`.
2. `reference(record, "evidence")`: its `path` must be absolute with literal `.md`
   suffix; `owner.private_file` then rejects symlinks in file/ancestors and requires
   a regular executing-UID-owned file, **exactly 0600**, **one link**; SHA-256 of
   those bytes must equal `evidence.sha256`. The named refusals are
   `evidence_redacted_evidence_required`, private-file errors such as `symlink_path`
   or `unsafe_file`, and `evidence_evidence_digest_mismatch` (I/O/type errors may
   instead surface their exception class through `evaluate`).
3. Branch selector `mode == "limited"`.
4. In a short-circuit conjunction: `accepted_by == "Travis Good"` exactly,
   truthy `limitation`, truthy `allowed_scope`, truthy `remaining_proof`.
   Failure: `named_product_authority_limitation_required`. Code checks truthiness,
   not string type; author meaningful nonempty text, not a boolean workaround.
5. **A second reference call**, `reference(record, "acceptance")`, with the same
   absolute `.md`, nonsymlink ancestors, regular file/UID/0600/single-link and hash
   checks; analogous `acceptance_redacted_evidence_required` and
   `acceptance_evidence_digest_mismatch` errors.
6. Return detail `LIMITED: ` plus limitation. `evaluate()` calls that item `PASS`;
   a green overall `ok` therefore does not distinguish qualified from limited.

To reach the intended missing-product-decision refusal, first supply authentic
candidate-matching hashes and a valid private evidence reference, select limited,
then leave the actual missing acceptance absent: step 4 refuses before step 5.
A completely successful limited record also needs step 5. Do not create an
acceptance file claiming a decision that never occurred. These checks do not
cryptographically authenticate Travis's authorship.

**Qualified branch, one sentence in check order:** after transport validation,
require matching binary hash then five-module package digest, validate the private
`evidence` reference, bypass `mode == "limited"`, require `mode == "qualified"`,
require `isolated_transport`, `isolated_profile`, `negative_access_probes`,
`mediated_inference` each **is True** in that order, require `real_ack`, `real_start`,
`real_return` each has **type exactly int** and is **>= 1** in that order (booleans
fail), require truthy `independent_reviewer`, then validate the separate private
`review` reference with identical path/UID/mode/link/hash rules.

Qualified failure codes after the common checks are respectively
`real_worker_qualification_required`,
`isolated_transport_profile_and_probe_evidence_required`,
`real_ack_start_return_required`, `independent_evidence_review_required`, and the
review reference errors. Neither branch bypasses items 1–3 or 5. Qualified has no
`accepted_by`, `acceptance`, or named-Travis predicate. Evidence truth and scope
remain the integrator's responsibility; setting booleans is not execution proof.

## Executable qualified-path recipe (future work only)

1. **Mine — freeze inputs and scope on the host.** Select the already received
   candidate binary from its accepted build artifact; do not rebuild it and assume
   the old hash still applies. Record its version/build commit, architecture,
   actual SHA-256 and the approved transport's expected SHA-256. Select owner
   runtime from this allocation's received base (or a newly received successor),
   all non-test `scripts/initiative_control/*.py`, the plist template and pinned
   requirements. Record same-UID trust and the bounded lifecycle claim above.
   Obtain frozen cases from the independent designer and assign a separate
   executor and reviewer. Preserve original cases and later amendments separately.
   Manager delegation covers scoped review extensions; retain the ledger.

2. **Mine — prepare a clean guest image/run.** Use `192.168.64.3`, `agent`, UID 503;
   verify with `id -u`, record OS/architecture, mounts and service inventory through
   the manager's SSH session. The manager has a separate administrative account
   with sudo as well as the standard `agent` account. Use the administrative
   account only to provision the guest system config in step 6; keep its login,
   sudo capability and management session unavailable to the worker and children.
   `agent` needs no sudo for per-run files, private TMUX or `user/503` launchd.
   Disable shared host directories, clipboard/
   host automation and agent forwarding at the VM/hypervisor boundary. Provision
   an external guest egress allowlist: the broker path only, plus explicitly
   declared fixtures. GitHub-only blocking is insufficient. If the host/network
   owner has not supplied that control, stop at provisioning. The administrative
   account's existence does not itself prove the external egress fence.
   Keep incoming key-based administration restricted to the manager; never stage
   its private key or forward its agent. Use a fresh image if the guest already
   contains source, old findings or credentials; absence at one guessed path is
   insufficient. Inventory through known nonsecret paths, never search auth data.

3. **Mine — stage an allowlisted package from the host, not GitHub.** Make a host
   staging directory containing `bin/corbanu`, `runtime/` as above, frozen case
   packet, neutral fixture seed/task data, wheels downloaded from the pinned
   requirements for the guest Python, and the matching Python/TMUX dependencies
   if not already installed. Transfer with `scp -r STAGE agent@192.168.64.3:RUN`
   over the manager-held key with `ForwardAgent=no`; no repository tarball, `.git`,
   historical QA, existing HOME or auth file. Use physical guest paths under
   `/private/tmp/q113-pkg-UNIQUE` for assets, `/private/tmp/q113-data-UNIQUE`
   for staged fixture/brief data, and `/private/tmp/q113-runs-UNIQUE` for TMUX runs.
   Reserve `/private/tmp/q113-UNIQUE` for step 8 to create; do not pre-create it.
   Keep paths short enough for TMUX sockets, private directories
   0700 and control/evidence JSON/Markdown files 0600. Resolve TMUX to the actual
   Cellar executable before recording it; `/opt/homebrew/bin/tmux` may be a link.
   A fixture Git repository is newly initialized task data, not Corbanu history.
   Mount candidate assets read-only from a separate package image if asserting
   immutable assets; retain writable coordinator/schedule/run/worktree/evidence
   areas. The manager needs no in-guest sudo to use an already provisioned mount.

4. **Mine — verify pins on host and guest before any launch.** Use the exact
   candidate runtime for imports. Example commands (paths are noncredential):

   ```sh
   shasum -a 256 /ABS/PACKAGE/bin/corbanu
   env -i PATH=/usr/bin:/bin:/opt/homebrew/bin PYTHONDONTWRITEBYTECODE=1 \
     PYTHONPATH=/ABS/PACKAGE/runtime /ABS/PYTHON -B -c \
     'import owner_daemon as o; print(o.package_digest())'
   ```

   `package_digest()` is `coordinator.digest` of a map from the five literal
   filenames `owner_daemon.py`, `owner_tmux.py`, `coordinator.py`, `manager_cycle.py`,
   `fable_launcher.py` to their SHA-256s. It is not the directory hash, Git commit,
   binary hash, or concatenation of files. Copy bytes unchanged and assert host
   and guest package digests agree. Set the **disposable** guest transport's
   `binary_sha256` to that verified binary value and the guest configuration's
   `package_digest` to that verified package value. Later evidence for the target
   transport must match its already approved binary and the runtime imported by
   the gate; if mismatched, locate the exact artifact or qualify a newly received
   candidate and re-pin through the normal review path. Never relabel old proof.
   Hash all other helpers/assets/dependencies separately: the five-module hash
   does not cover the whole package. Install the venv offline with
   `python -m pip install --no-index --find-links WHEELS -r requirements.txt` in
   a fresh `env -i` HOME; no inherited profile. Verify hashes again after testing.

5. **Mine — mediate inference outside the guest.** Provision/reuse an approved
   Responses-compatible broker with its real provider credentials held on the
   host/service side, inaccessible to guest files, process inspection and IPC.
   Give it an upstream allowlist, exact authorized provider/model/effort, quota,
   expiry and request/response-ID audit. Deny generic proxy/CONNECT, arbitrary URL,
   file retrieval and credential export. It must support streaming and model
   tool-call round trips, not the tools-disabled isolated text helper. The guest
   endpoint may be unauthenticated *inside this dedicated, network-bound lane*;
   admission is by the restricted tunnel/VM identity and server-side budget.
   A readable dummy marker must not grant access from elsewhere or to upstream.

   One concrete endpoint arrangement uses a manager-owned SSH reverse tunnel:

   ```sh
   ssh -N -o ForwardAgent=no -o ExitOnForwardFailure=yes \
     -R 127.0.0.1:18443:127.0.0.1:28443 agent@192.168.64.3
   ```

   Here the already provisioned broker listens on the manager host's loopback
   port 28443; guest requests reach only that fixed service through loopback
   18443. This command is a tunnel, **not a broker implementation**. Do not start
   qualification if that approved service, upstream entitlement or egress fence
   is missing. Use a dedicated SSH connection with only this reverse forward;
   do not expose a host shell, agent socket or arbitrary forwards to the executor.
   VM management must disable unapproved forwards/egress for guest-originated
   connections. The worker may use this lane like the controller; that is accepted
   same-UID scope, bounded by the broker's limits.

6. **Mine — provision the supported system layer with the administrative account.**
   For an explicitly authorized `openai` allocation, install this nonsecret file
   as root-owned, worker-readable and worker-nonwritable
   `/etc/codex/config.toml` in the disposable guest (on macOS, record the physical
   `/private/etc/codex/config.toml` path as well):

   ```toml
   openai_base_url = "http://127.0.0.1:18443/v1"
   cli_auth_credentials_store = "file"
   ```

   Stage those exact bytes as a nonsecret manager asset. From the administrative
   account, in the clean guest only, install them before any worker is prepared:

   ```sh
   sudo /usr/bin/install -d -o root -g wheel -m 0755 /private/etc/codex
   sudo /usr/bin/install -o root -g wheel -m 0644 /ABS/STAGED/broker-config.toml /private/etc/codex/config.toml
   ```

   Verify parent ownership/no worker write permission, config bytes/hash and
   worker readability; record the guest OS and approved binary build provenance.
   Use a fresh guest if pre-existing config/managed settings are unknown; do not
   overwrite unrelated state. Ensure no cloud/managed/profile endpoint override
   conflicts. This is guest provisioning, not a change to the transport's six-key
   JSON schema, five-module package, generated profile or launch argv. Keep the
   administrative capability outside the executor.

   The loader reads the Unix system file before merging the user layer and strips
   the endpoint only from project layers. `TmuxAdapter.prepare()` authors trust,
   UI and analytics settings but no endpoint. Its `Worker.launch()` pins
   provider/model/effort but no endpoint; its fresh environment does not suppress
   system config. The surviving top-level URL builds the OpenAI Responses
   provider and disables default-endpoint websocket prewarming in favor of SSE.
   Code citations and the evaluated alternatives are in the current return.
   Do not put this endpoint in the fixture's `.codex/config.toml`, use nested
   provider overrides, patch generated user config, inject variables, or edit
   launch argv. No other recipe step relies on any of those denied routes.

   Provide a newly authored synthetic `auth.json` with only `OPENAI_API_KEY`
   containing a fixed nonsecret dummy marker with no upstream validity. Point
   the disposable `auth_link` at it; never copy/read/hash/print real auth files.
   Broker strips dummy authorization and injects its upstream credential outside
   the guest. File auth selection comes from the same system config; the
   per-worker HOME and all profile aliases remain transport-authored.

   Before the frozen acceptance run, use a separate disposable control run
   through the exact pinned adapter/binary: require a real SSE response and
   tool-call round trip with joined broker upstream IDs. In a separate bounded
   negative control, stop/reject the broker lane, require inference failure and
   zero public-endpoint connections under continuous external egress observation;
   restore it for a fresh successful control. Retain all attempts and retries.
   No upstream credential exists in the guest, and the fence denies direct
   provider access, alternate IP/IPv6 routes, DNS retrieval and proxy bypass.
   Merely observing a loopback socket or TUI answer is insufficient.
   These controls are future proof, not tests performed by this worker. A
   mismatching artifact or provider is a blocked prerequisite; do not silently
   reroute or claim runtime validation from source inspection alone.

7. **Mine — prove the boundary, then admit the case.** From the actual fresh
   executor's permitted tool interface and its child shell, record UID, policy,
   tool inventory and these outcomes without reading secrets:

   - Attempt list/read of the *recorded actual host* checkout, `.git` and prior-QA
     paths; attempt a symlink from fixture data to those paths. Combine denied/
     absent guest paths with host mount/export inventory and a known-existing
     host synthetic source/history canary behind the boundary. ENOENT alone
     proves only absence, not an access control. Record returned errno/result and
     the policy that prevented host mount/IPC/network retrieval.
   - Attempt retrieval of a synthetic source canary at a controlled disallowed
     network destination and connection to a known host-only IPC test socket.
     The host observer establishes that each target exists; the actual executor
     and its child must fail because of the external fence. Also try the approved
     broker with a disallowed fetch/CONNECT request and require broker rejection.
     Probe a broker-side synthetic secret canary, never an actual credential.
   - Try access to a second isolated run's socket and package modification. If
     cross-run resources are on this same UID's writable guest, they are accessible;
     use a separate VM/run boundary or record that limitation. Do not report the
     readable runtime or dummy auth target as denied. For read-only mounted assets,
     an attempted write must fail; chmod 0444 alone is not this control.
   - Positive controls: frozen packet read, exact binary/version launch, actual
     PTY key input, and a real model response whose broker log confirms authorized
     upstream identity. Confirm no login/Keychain prompt, reroute or fallback.
     Stop on a native credential prompt. Children must remain inside the guest
     and its external egress boundary, not be denied all same-UID guest resources.

   Preserve parent/child raw outcomes, policy and control evidence by run/group.
   The outer blind executor receives only the frozen packet and mediated PTY;
   the independent reviewer receives the full redacted boundary record. A failed
   required probe blocks the relevant claim; do not simply set the boolean true.

8. **Mine — execute the actual owner lifecycle against disposable resources.**
   Create a new fixture coordinator, schedule, journal, publish destination and
   toy worktree, never a copy of the live coordinator. Use the existing rehearsal
   initialization sequence (`Coordinator.initialize`, `set_enabled`, allocation
   registration, accepted fixture manager decision, `owner.setup`, explicit
   revision-bound `owner.handoff`, then `arm_owner`)
   with explicitly labelled fixture authority for local state. Freeze inputs via
   `freeze_worker_inputs(..., provider="openai", policy="--yolo")` **before**
   allocation registration; use the authorized model/effort, guest worktree and
   frozen case brief. Fixture manager acceptance is not real product authority
   and contributes no real model counts. Do not run the existing synthetic-worker
   promotion rehearsal and relabel its output. Set the real guest transport in
   the disposable owner config. This initialization block runs only in the
   disposable guest, with the pinned runtime on `PYTHONPATH`. Supply an existing
   private JSON `INPUTS` with `root` (not-yet-created run directory), `worktree` (staged
   trusted toy repository), `brief` (frozen case JSON), `transport` (guest transport
   JSON), `model`, `effort`, `base_commit` (toy repository base), `task`,
   `timeout_seconds` (case budget), `authority` (actual manager fixture permission
   reference), `decision_id` and `decision_revision` (that fixture permission's
   actual identifier/revision). No live production action IDs or roots.
   Do not treat the fixture's synthetic manager receipt as a real manager response.

   ```python
   # Future fixture setup; this revise worker has NOT executed this block.
   INPUTS = "/ABS/PRIVATE/qualification-inputs.json"  # replace before execution
   import json
   from pathlib import Path
   import fable_launcher as f
   import owner_daemon as owner
   from coordinator import Coordinator, digest
   from owner_tmux import freeze_worker_inputs, validate

   p = json.loads(Path(INPUTS).read_text())
   root = Path(p["root"])
   assert root.is_absolute() and not root.exists(), "new disposable root required"
   root.mkdir(mode=0o700)
   worktree, brief = Path(p["worktree"]), Path(p["brief"])
   assert worktree.is_dir() and brief.is_file()
   assert p["authority"] and p["decision_id"]
   transport = validate(owner.load(Path(p["transport"])))
   frozen = freeze_worker_inputs(dict(
       allocation="q113", base_commit=p["base_commit"], brief_file=str(brief),
       brief_sha256=f.file_digest(brief), model=p["model"],
       reasoning_effort=p["effort"], task=p["task"], worktree=str(worktree)),
       provider="openai", policy="--yolo")
   allocation = dict(sprint="Q113", kinds=["functional_test"],
       resources=["q113-worktree"], scope=[str(worktree)],
       inputs={k: v for k, v in frozen.items() if k != "allocation"},
       timeout_seconds=p["timeout_seconds"])
   c = Coordinator(root / "state")
   c.initialize({"qualification": dict(sprint="Q113", mode="enabled")},
       {"Q113": dict(workstream="qualification", status="in_progress",
                     dependencies=[], archived=False)}, {"q113": allocation})
   evidence = dict(disposable_fixture=True, authority=p["authority"])
   c.set_enabled(True, evidence)
   c.event(dict(id="q113-case", reason="frozen qualification case"))
   packet = c.begin_manager()
   action = dict(id="q113-worker", kind="functional_test",
       workstream="qualification", sprint="Q113", rationale="frozen case",
       timeout_seconds=p["timeout_seconds"], inputs=frozen,
       expected_revision=packet["state_revision"])
   c.accept_decision(packet["manager_run"],
       dict(state_revision=packet["state_revision"], actions=[action]),
       dict(synthetic_fixture_manager=True, authority=p["authority"]))
   config = dict(coordinator=str(root / "state"), worktrees=[str(worktree)],
       package_digest=owner.package_digest(), manager_enabled=False,
       transport=transport)
   config_path = root / "config.json"
   f.write_json(config_path, config)
   owner.setup(config_path)  # OFF; no worker launch
   state = c.snapshot()
   assignments = {key: dict(
       **{k: a.get(k) for k in ("claim", "allocation_digest", "status")},
       **{"from": c.dispatch_owner(state, a), "to": "owner"})
       for key, a in state["actions"].items()}
   owner.handoff(config_path, dict(expected_revision=state["revision"],
       assignments=assignments, evidence=evidence))
   owner.arm_owner(config_path, dict(decision_id=p["decision_id"],
       revision=p["decision_revision"],
       authority=p["authority"], scope="tmux-workers", generation=1,
       config_digest=digest(config), package_digest=owner.package_digest()))
   print(json.dumps(dict(config=str(config_path), package=owner.package_digest())))
   ```

   This creates its coordinator/journal; schedule, runs and publication areas are
   separate private fixture paths. `root` must agree with the command below;
   create `root/admin` as 0700 before invocation. Drive the real kernel with the
   pinned interpreter:

   ```sh
   env -i HOME=/private/tmp/q113-UNIQUE/admin \
     CODEX_HOME=/private/tmp/q113-UNIQUE/admin \
     CORBANU_HOME=/private/tmp/q113-UNIQUE/admin \
     PFTERMINAL_HOME=/private/tmp/q113-UNIQUE/admin \
     PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin \
     PYTHONDONTWRITEBYTECODE=1 /ABS/PYTHON -E -s -S -B \
     /ABS/PACKAGE/runtime/owner_daemon.py \
     --run --config /private/tmp/q113-UNIQUE/config.json
   ```

   Repeat bounded ticks only until the frozen case deadline/terminal result;
   inspect each outcome and retain holds/timeouts, never replay an uncertain send.
   For recurrence cases install **only** a uniquely named guest test schedule
   through `activate.py --owner install --domain user --interval 30` with the
   recorded Python/hash/runtime/config/root/publish-state arguments. A direct
   `--run --config` lifecycle does not prove recurring launchd behavior. A separate
   blind executor applies frozen success/failure/cancel/recovery/resume cases with
   real keys. Owner sends prompt text and Enter separately and sends START once
   after correlated ACK; the executor must not inject a duplicate START.

9. **Mine — count correlated evidence, not text.** Retain `worker.json` binding,
   claim/action/allocation digest, binary/package hashes, boot/UID/socket/pane/PID
   identity, owner operations/observations and raw rollout/broker receipts.
   `real_ack` counts unique completed first turns with exactly
   `ACK <action_id> <allocation_digest> <model> <effort>`, matching session cwd,
   source `cli`, provider, model, effort, approval `never`, sandbox
   `danger-full-access`, original prompt and nonempty model response ID.
   `real_start` counts unique bound START turns observed submitted in the rollout
   with matching context and `user_message == "START"`, linked to the durable
   `start-intent` and owner `working` operation; a keys receipt alone is zero.
   `real_return` counts unique subsequent completed turns with standalone first
   line RETURN, matching response provenance and owner `return_observed`/`returned`.
   Join by action+claim+allocation+session/thread+turn and deduplicate repeated
   polling observations. Preserve both ACK and START/RETURN turn IDs (the latest
   `inspect()` turn ID is not both). `claim` is in the binding/prompt, not ACK text.
   Reject echoes, synthetic responses, forks, wrong runtimes, errors/reroutes or
   incomplete turns. Pair broker upstream response IDs with rollout completion
   and retain server-side identity; claimed provider strings alone are not proof.
   Count at least one complete lifecycle; the three minima do not waive any
   frozen case. A durable correlated RETURN can remain valid after pane death.

10. **Mine — independent evidence review and manifest.** Reviewer must be named,
    independent of implementer and executor (original designer is allowed), and
    actually inspect raw attempts, parent/child probes, positive controls, exact
    package/binary, model/session/turn joins, same-UID accepted scope and every
    frozen-case disposition. Review records reviewer identity/session, artifact
    hashes, failures/limitations, replay separation and result; a truthy name or
    copied template is not a review. Collect integrator acceptance. Export redacted
    Markdown evidence and review to absolute, nonsymlink, single-link, executing-
    UID-owned 0600 paths on the **gate machine**, hash final bytes, and populate
    qualified fields in the order above using the current return's Travis-backed
    four-boolean scope mapping. Link that existing ruling and the nonsecret
    system-config hash, broker joins and fail-closed controls in the evidence.
    Recheck binary and package matches at
    gate time. Do not copy the guest UID 503 ownership blindly to a host with a
    different executing UID. Gate PASS remains an attestation check, not a new
    proof of authenticity or full general-purpose worker confinement.

11. **Mine — teardown and preserve.** After frozen recovery/resume observations,
    stop only owned test schedules/workers/TMUX servers, retain raw and redacted
    evidence separately, close the SSH tunnel and expire the broker lane. The
    administrative account then removes only its recorded disposable system
    config, or destroys the guest; confirm no active test client remains. Keep
    failed attempts; a corrected replay gets new isolated state and fresh executor.
    Verify cleanup does not touch live service labels or unrelated processes.

**Travis's steps only when needed:** item 4's **limited** alternative requires his
actual named acceptance (`Travis Good`) with limitation/scope/remaining proof and
hashed acceptance record; product-scope/gate exceptions or missing promotion
permission require the actual product authority; new account/spend permissions
come from their real holder (Travis only if he holds that authority). The qualified
branch itself uses Travis's existing source-blindness ruling and does not require
a new signature or limited-test agreement for that covered narrowing. No new
product decision is invented here. VM staging, system config and broker provisioning
within existing authority, evidence collection and scoped review are mine. This
recipe supplies no new permission and does not resume paused security work.

## Item 5: audit-only procedure for tonight

This procedure observes, never promotes. Do not invoke the full promotion recipe.
Leave coordinator `enabled`, owner **armed fixture-only**, receipt **installed**,
service **present**, plist/runtime/timer/pins unchanged. Disarm, bootout or disabling
dispatch would fail the current gate. No live actions in this procedure were run
by this worker. The following steps are **mine**, within existing maintenance
permission; they do not need a new Travis decision merely for an audit.

1. Record the exact installed schedule path, receipt label/domain, coordinator,
   publication directory, configured worktrees, selected prepared replacement
   actions and candidate preflight/runtime paths. Build the administrative venv
   and stage the non-audit manifest/evidence before the quiet window. Check the
   five-module digest is the received candidate. Inventory **every caller** of
   dispatch, raw TMUX keys, manager decision/ingest/reconciliation, manual owner
   ticks, status publication/export and their schedules. Record each caller's
   owner and its actual supported pause/resume control. There is no repository
   command that freezes arbitrary external callers; an unidentified writer blocks
   the attestation. Avoid guessed `pkill` or broad launchctl commands.
2. First stop new manager allocations/decisions/dispatch and raw-key submissions
   at those caller controls. Obtain timestamped pause acknowledgments with last
   submission sequence IDs. Let already-started deliveries/cycles finish or
   reconcile through their proper owner; no abandoned manager cycle or fabricated
   reservation settlement. Do not kill unrelated worker computation.
3. Next drain resulting lifecycle/ingest/reconciliation writes, then pause those
   producers/consumers at an idle boundary and record their last committed revision
   and queue watermarks. Buffer new observations outside the coordinator without
   dropping them. Finally let publishers finish and pause their timers/callers;
   record last publication receipts/sequences and absence of active writers.
   Prevent automatic restart of these **competing** callers through their actual
   supervisory controls. Keep the recurring owner loaded and unchanged for now.
4. Observe the recurring fixture owner until its scheduled CLI process has exited,
   including the publication performed **after** `scheduled_tick` returns. Require
   no active PID for its exact launchd job, settled `tick.json`, no hold, no pending
   publication files and no known manual owner invocation. Record `launchctl print
   DOMAIN/LABEL` including its invocation **runs** counter and no active PID, the
   tick timestamps/count and metadata+hash of `tick.json`, `owner-recurrence.json`,
   owner stdout/stderr logs and ticks-directory listing. Do not treat a held
   owner-daemon/tick lock as a fence: the outer handler still publishes. No SIGSTOP
   inside a write, SQLite write lock, pending-file deletion or falsified receipt.
5. Capture the read-only coordinator snapshot and revision **after** the last
   completed writer. Start wall-clock observation time now, not at the beginning
   of draining. Immediately create a private redacted audit `.md` referencing the
   actual caller pause/drain acknowledgments, process/job-counter observations,
   revision and publication identities; hash final bytes. Write a **new private
   manifest copy outside live state** with `audit` fields shown below. Set each
   boolean only after that fact is observed. Mark the observation provisional
   until the postflight comparison passes. Both revision and timestamp can be
   captured together without opening a writer connection (supply the recorded
   `SCHEDULE`, existing non-audit `manifest`, finalized `ABS_AUDIT_MD` and
   `AUDIT_SHA256`; never use guessed live paths):

   ```python
   # Administrative interpreter; PYTHONPATH includes received runtime and QA gate.
   import time
   import owner_promotion_94_preflight as gate
   ctx = gate.context(SCHEDULE)
   state = ctx[4].snapshot()  # ReadOnlyCoordinator: readonly, busy_timeout=0
   observed_at = time.time()
   revision = state["revision"]
   # After saving the real private audit .md and hashing its final bytes:
   manifest["audit"] = dict(
       dispatchers_quiescent=True, raw_key_delivery_stopped=True,
       publishers_quiescent=True, observed_at=observed_at,
       coordinator_revision=revision,
       evidence=dict(path=ABS_AUDIT_MD, sha256=AUDIT_SHA256))
   # Write only the separate private manifest, never the coordinator or receipt.
   ```

6. Immediately run the **read-only** gate CLI, preserving its exit and JSON even
   when UNMET; use the same candidate runtime imports as the pins:

   ```sh
   env -i HOME="$AUDIT_HOME" CODEX_HOME="$AUDIT_HOME" \
     CORBANU_HOME="$AUDIT_HOME" PFTERMINAL_HOME="$AUDIT_HOME" \
     PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin \
     PYTHONDONTWRITEBYTECODE=1 \
     PYTHONPATH="$CANDIDATE_RUNTIME:$CANDIDATE_QA" \
     "$ADMIN_PYTHON" -B "$CANDIDATE_QA/owner_promotion_94_preflight.py" \
     --manifest "$PRIVATE_MANIFEST" --schedule "$SCHEDULE" \
     --transport "$TRANSPORT" --actions "$ACTION_ONE"
   ```

   Pass further selected IDs as separate arguments. `$TRANSPORT` is the existing
   approved input, not the qualification guest's disposable path. Item 4 may
   legitimately remain UNMET; item 5 still runs if item 2 produced its state.
   An item-2 failure means item 5 cannot establish the audit. Do not change a
   limited/qualified declaration merely to make an audit green. At the actual
   item-5 check, `0 <= time.time() - observed_at <= 300` and the observed revision
   must equal item 2's snapshot. Clock jump, expiry or changed revision requires
   a new observation and new hashed evidence, never editing the old timestamp.
7. Immediately recapture coordinator snapshot, exact launchd job counter/no PID,
   tick/publication/log identities, directory listings and caller watermarks.
   Require unchanged whole coordinator snapshot, unchanged invocation counter,
   no active job, unchanged tick/publish/log observations, no pending files and
   continuing pause acknowledgments throughout the interval. Preserve the raw
   launchctl before/after output (including job identity) and actual pause-control
   logs, plus a timestamped process observation trail during the gate. A stable
   revision alone is insufficient: a tick can publish without changing it. If
   launchctl does not expose a reliable runs counter, or monitoring has a gap,
   do not infer no short-lived invocation from two empty `ps` samples; require
   an independently complete launch/process trace or call the attempt inconclusive.
   These controls prove an observed interval under cooperating trusted callers;
   they are not an OS-wide adversarial writer lock. Save a separate postflight
   verdict binding the provisional audit hash, gate output hash and final checks.
8. **Retry at most three measured gaps total.** On any overlapping owner tick,
   publisher, caller restart, snapshot change, freshness expiry or missing trace,
   retain the attempt as invalid/inconclusive. Wait for the complete next tick and
   publication to exit, repeat steps 4–7 with fresh files/time/revision. Do not
   retry unresolved holds/pending writes; those need their actual owner to reconcile.
   The owner interval is 30 seconds, not 30 seconds of usable time after completion;
   scheduler timing and audit duration can defeat every attempt. Three is an
   operational retry budget, not a gate constant. After the third failed gap,
   report **fixture tick must be paused for a reliable observation**. Current code
   has no proven maintenance fence preserving installed/present/armed predicates
   and blocking both tick and publication. Allocate/review/re-pin that narrow
   mechanism or an authorized audit-procedure change; do not claim `launchctl stop`,
   `disable`, bootout, a lock or early disarm solves this unchanged gate.
9. On audit-only completion or a clean inconclusive abort, save the final verdict,
   then resume ingestion/reconciliation first and process buffered real observations
   in sequence; resume publishers on the resulting coherent state, then manager
   allocations/decisions/dispatch/raw-key admission under the unchanged ownership
   map. Use the recorded caller resume controls, verify acknowledgments and new
   watermarks. The recurring owner was never stopped, so it has no restart action.
   If a real hold/interrupted write was discovered, keep the affected component
   paused and hand its evidence to its owner; do not blindly restart it. Audit
   results are immediately historical after writers resume. No cutover follows
   from this read, and no promotion approval is implied.

## Verification

`docs/development/test-isolation.md` was read before the campaign. A disposable
venv was built under `env -i` from `scripts/initiative_control/requirements.txt`;
all four HOME/profile aliases point to the recorded disposable root, with
`CORBANU_TEST_NO_NATIVE_KEYRING=1`, `PYTHONDONTWRITEBYTECODE=1`, clean PATH/TMPDIR
and `PYTHONPATH=scripts/initiative_control`. No Rust tests or workspace formatter.

The setup exited 0 with markdown-it-py 3.0.0, mdurl 0.1.2 and slack-sdk 3.44.1.
[Setup log](owner-limited-113b-venv.txt) and
[disposable root](owner-limited-113b-test-root.txt) are retained. From repository
root, the venv interpreter ran these commands in that clean environment:

```text
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -B -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_decision_manager test_slack_transport
```

[Full discovery](owner-limited-113b-suite.txt): **880 passed**, 519.567 seconds,
exit **0**, zero failures/errors/skips. Failure names: **none**.
[Focused modules](owner-limited-113b-focused.txt): **483 passed**, 394.452 seconds,
exit **0**, zero failures/errors/skips. Failure names: **none**. Breakdown: owner
133, owner TMUX 52, feed 39, attention 24, control 39, decision manager 113, Slack
transport 83. These overlap discovery: **880 unique tests**, not 1,363 distinct
tests. Both top-level test commands and venv setup exited 0. Logs retain expected
negative-fixture refusals and ResourceWarnings; no native credential prompt or
live-profile access was observed.

Markdown examples passed syntax-only checks: two Python blocks parsed with
`ast.parse`, four shell blocks passed `bash -n`. These checks did not execute the
qualification or audit examples. No new implementation tests were authored.
No qualification or live-state evidence is implied by unit tests. Independent
functional/TUI, live-repository and benchmark execution is not applicable to this
routine text correction; the later real-worker gate remains open, with integrator
acceptance of applicability. No new human acceptance or qualification is claimed.

## Changed files and final checks

All six changed files are under `qa/initiative-control/management-bootstrap/`.
Existing executable/source files changed: **0**. `git diff --check` passed; final
scope inspection found no edits outside the writable allocation. No formatter,
commit or push ran. Historical round-112 raw evidence remains unchanged.

| File | Final lines | Added / deleted |
| --- | ---: | ---: |
| `owner-isolated-112-return.md` | 40 | +34 / -306 |
| `owner-limited-113b-return.md` | 625 | +625 / -0 |
| `owner-limited-113b-suite.txt` | 939 | +939 / -0 |
| `owner-limited-113b-focused.txt` | 525 | +525 / -0 |
| `owner-limited-113b-venv.txt` | 15 | +15 / -0 |
| `owner-limited-113b-test-root.txt` | 1 | +1 / -0 |
