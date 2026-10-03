# Owner manager enable, 2026-10-02/03: promoted to tmux-workers (generation 3), first action held

**Current state (round 4, 2026-10-03T12:21Z):** armed, generation 3, scope `tmux-workers`,
`manager_enabled: true`, ticking on the interval. Its first action, `owner-first-check-01-r2`, is
held with `wrong_ack` (see round 4). Rounds 1-3 below are the history.

Authority: `owner-manager-enable-20261002`, revision 1. Travis (owner), 2026-10-02,
answered **yes** to "Should I turn the dashboard manager on (let it dispatch work
by itself)?" That covers moving the live owner from `fixture-only` to
`tmux-workers` with `manager_enabled: true`. It does not cover pushes, extra
Slack sends, credential-store reads or product-source changes.

**Outcome: not promoted.** The mandatory round-94 preflight
([runbook](owner-promotion-94-preflight.md)) refused four of its five items. The
[promotion recipe](owner-handoff-80-promotion.md) requires that preflight to pass
before it disarms anything. I stopped before the first destructive step. The
owner is still **armed at generation 1, scope `fixture-only`** and ticks normally.
Nothing was disarmed, uninstalled, restaged, reconfigured, handed off or armed.

## State before (and after, unchanged)

Installed-runtime `--activation-status`, read while no owner-lane dispatch was in
flight. Hand workers were running outside the coordinator (see item 5).

| Field | Value |
| --- | --- |
| state / generation / next | `armed` / 1 / 2 |
| scope | `fixture-only` |
| decision | `owner-recurrence-domain-20260917` rev 3, activation digest `ef41fa04…0616b` |
| config digest | `cd07b20b0196278e1fd6b39894bf1d0bb25e5151e9ae4ebfdec9d8cb5b40d6f5` |
| installed package digest | `84c09899ca50fd1acfe8843fe770d24ce00d40a8180fc3f5c2866a1dc3108625` |
| candidate package digest (704650ccd8) | `8021fc53b7dd04b69e46cdec0886a7b7745c0497491dd8a7fa7d9cdc33cf2065` |
| recovery required / unresolved holds | false / `[]` (candidate read) |
| coordinator | revision 2911, enabled, manager `null`, `dispatch_control` `null` |
| actions | 4 accepted, 5 failed, 1 running (`slack-receiver-02`, stall_reported, legacy hand claim); **0 prepared** |
| coverage (candidate read) | `covered=0, excluded=0; manager=covered; new default actions=covered; hand stall detection=covered by this watchdog.` |

Ticks after the refusal: `ticks` 42967, `hold: null`, `firing: interval`,
`consecutive_errors: 0`, launchd `runs` 33763, last exit 0.

## Preflight result (exit 2)

| Item | Result | Reason |
| --- | --- | --- |
| 1 manager_bridge_authority | UNMET | `manager_preparation_bridge_not_adopted` |
| 2 fresh_allocations_and_decisions | UNMET | no prepared worker action exists to select; replacement evidence absent |
| 3 publication_pending | PASS | no `*.pending` publication |
| 4 isolated_real_worker_qualification | UNMET | no real ACK/START/RETURN qualification or Travis named-limitation acceptance for this binary/package |
| 5 quiescent_live_audit | UNMET | depends on item 2. Hand dispatch was also live (`workers-20261002` tmux workers) |

The "yes" is a valid activation decision, but it does not satisfy items 1, 2
or 4. Writing a "Travis Good" limitation record myself would not count; the
runbook forbids exactly that.

## What enabling would and would not do

In this code, `tmux-workers` only dispatches actions that a handoff assigns to
`owner`. New actions default to `hand`. `manager_enabled` only changes the
fixture tick's `manager` label to `deferred`. The owner never creates actions or
runs manager inference. Even after a successful promotion, the owner would not
dispatch work by itself unless a manager prepares actions through
`freeze_worker_inputs` and they are handed to `owner`. The owner should be told
this before he answers again.

## Prepared and left in place (no live effect)

- **Binary:** `/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-worker-bin/704650ccd8/corbanu`,
  0500, copied from the integration dev package (commit 704650ccd, corbanu 0.1.48).
  sha256 `7b8c77a613e48214e9e6b29d70941d634d05222175140f7e57d5cfcba36dd77c`;
  `codesign --verify --strict` passed.
- **runs_dir:** `/Users/Neo/.local/state/corbanu-owner-runs`, 0700, on the
  local disk. The tmux socket path is 55 bytes, under the 100-byte limit.
- **Transport:** `private/transport.json`, rewritten with the new binary and
  runs_dir, sha256 `6520870d4806bbcb234dd9a5f02c272510a3d41d74a7ea5b8210b49d7b33e2b1`.
  `owner_tmux.validate` passes. `tmux` and `auth_link` are unchanged. The
  auth file was never opened. The stale Sept-17 file is kept as
  `private/transport-archive-20260917.json`. The live `config.json` does
  **not** reference the transport yet.
- **Evidence:** `.codex-work/owner-manager-enable-20261002/`, containing both
  status JSONs, the preflight manifest and result, the activation-authority
  record, the candidate extracted from commit 704650ccd8 and the sync log.

## Dashboard

`sync-source.sh` exit 0. It published 380 documents for
`integrate/management-workstreams-20260911 @ 704650ccd8`, tree `81e93e8d…0d0e`.
`health.json` `collected_at` is now `2026-10-02T23:46:02+00:00`, previously
2026-09-27.

**Disclosure:** the sync did not refuse the dirty checkout. It exported the
**working-tree** bytes of the four uncommitted files: `facilities.js`,
`facilities.py`, `facility_control.py` and `test_control.py`. Their hashes match
the working tree, not HEAD. The live dashboard source is therefore HEAD plus
another session's uncommitted changes, labeled 704650ccd8. Also,
`decision_feed.assessed_at` is still 2026-09-20 and `owner_recurrence` shows
`unknown / observation-stale`. Republishing does not refresh the feed.

## Needed to proceed

1. A manager change that adopts `owner_tmux.freeze_worker_inputs` with an explicit
   provider and `--yolo` policy authority (item 1).
2. At least one fresh prepared worker action, replacing a cancelled one, with an
   accepted manager decision (item 2).
3. Either a real isolated ACK/START/RETURN qualification of binary `7b8c77a6…`
   with package `8021fc53…` and independent review, or Travis's own named
   limitation acceptance with allowed scope and remaining proof (item 4).
4. A quiescence window: hand and raw tmux dispatch stopped, and publishers quiet,
   within 300 s of the gate (item 5).
5. Settling `slack-receiver-02`. A transport handoff can partition a legacy hand
   claim to `hand`, but it stays overdue and unwatched.

After that, run the recipe as written. Its activation becomes generation 3,
because disarm consumes generation 2. Add `manager_enabled: true` to the
replacement config as well: the preflight's plan copies the live value, which
is `false`.

## Round 2, 2026-10-02/03: bridge adopted, still not promoted

Travis answered "Run the test first" to the named-limitation question, so item 4
needs a real isolated worker run. The owner is unchanged: **armed, generation 1,
`fixture-only`**, config digest `cd07b20b…d6f5`, `recovery_required: false`, no
holds (installed `--activation-status` at 2026-10-03T05:59Z). Ticks 43,688,
`hold: null`, `firing: interval`, 0 consecutive errors. Nothing was disarmed,
reinstalled, handed off or armed. Evidence: `.codex-work/owner-manager-enable-20261002/round2/`.

### Item 1: PASS

- Commit `66a5b66840` (fast-forward from `98f905d306`): `manager_preparation.prepare_worker`,
  exposed as `coordinator_cli.py prepare_worker`. It requires an explicit provider,
  policy `--yolo` and an authority reference, calls `owner_tmux.freeze_worker_inputs`
  **before** `put_allocation`, and records the bridge in the allocation evidence.
- The five owner-package modules are untouched; `package_digest()` stays
  `8021fc53…2065`, so the staged candidate binding is unchanged.
- Tests in a clean worktree at the commit, fresh venv from `requirements.txt`,
  disposable HOME/profile aliases, `CORBANU_TEST_NO_NATIVE_KEYRING=1`:
  `scripts/initiative_control` discovery **886 OK** (512 s), owner QA suites **103 OK**.
  A first discovery attempt in the shared venv errored on 5 imports (`slack_sdk`
  missing from that venv); it is retained, not counted.
- Real use: allocation `owner-first-check-01` (PF-80-S01, `review`, resource
  `owner-readonly-check`, 1800 s, read-only repository check in this worktree at base
  `66a5b66840`) was registered through the bridge with provider `openai` and `--yolo`,
  revision 2911 → 2912, audit seq 3413. Worker block
  `openai / gpt-6-astra / high / --yolo`; allocation digest `285010e0…6897`.
- Preflight item 1 passes with the authority (`provider-policy-authority.md`,
  `f5adb567…`) and adoption (`manager-preparation-adoption.md`, `fc77dbf8…`) records.

### Item 2: UNMET — the verified manager cycle cannot run

The gate needs a prepared action from an accepted manager decision, replacing a
cancelled one. The only verified manager path is `manager_cycle.run_cycle` through
`fable_launcher`, which reads `CLAUDE_CODE_OAUTH_TOKEN` from an operator-provisioned
0600 regular auth file. No such file exists (the old `/private/tmp/fmgr.Q1SIYZ` is
gone), and writing a token into a file is outside what this worker may do. I did not
substitute an unverified decision path or write a receipt. The allocation waits for
the manager.

**Needed:** the operator provisions the manager auth file; then the manager runs a
verified cycle (action A1 on `owner-first-check-01`), replaces the allocation through
`prepare_worker` with `replace: true` (for example a refreshed base commit; this
cancels A1 with `owner_cancellation`) and runs a second verified cycle (A2). A2 plus a
replacement record is item 2.

### Item 4: UNMET — the guest lane cannot host the run today

Guest `agent@192.168.64.3` (macOS 26.2, UTM "macOS 3") is reachable with the pf83 key.
`/etc/codex/config.toml` already points at the broker tunnel (`127.0.0.1:18443`). Blockers:

1. **Disk.** Container free space 625,037,312 bytes; the staged binary is 611,885,600
   bytes, before runtime, worker HOME, rollouts and logs. About 10 GB is held by
   pf83 case directories (Sept 16 and Oct 2), which are another track's evidence.
   Filling the disk would also hit `neo2`'s active console session.
2. **Isolation posture changed for pf83.** The guest now holds a subscription
   `auth.json` and a Z.AI key under `/Users/agent/.pf83-auth` (readable by `agent`, the
   only account I can reach), and its packet filter passes direct 443 egress to pinned
   OpenAI and Z.AI addresses. An `agent`-UID run could not honestly claim
   `isolated_profile` or `mediated_inference`.
3. `macOS 2` is stopped and needs a graphical login before sshd starts.

No broker, tunnel, guest staging or worker launch was started. No independent review ran:
there is no qualification evidence to review.

**Needed:** the pf83 owner frees at least ~2 GB on the guest (or a separate guest is
provided), and the test runs under an account that cannot read real credentials (a
dedicated guest user provisioned with the `neo-vm` admin credential, or the guest sealed
again). Then run recipe steps 1–11 of `owner-limited-113b-return.md` (broker in
subscription mode on host loopback, reverse tunnel, unchanged `TmuxAdapter`) and send the
evidence plus the procedure to the independent `claude-opus-5-5-plan` reviewer.

### Item 5

At 2026-10-03T05:59Z the coordinator's worker tmux server
(`workers-20261002/tmux.sock`) was not running, so no hand workers were live. The gate
still reports `item_2_required`. The quiescence record must be rewritten within 300 s of
the real check once items 2 and 4 exist.

### Preflight rerun (exit 2)

Candidate from commit `66a5b66840`, live schedule and staged transport, selected action
`owner-first-check-01`: item 1 PASS, item 2 UNMET
(`replacement_evidence_for_each_action_required`), item 3 PASS, item 4 UNMET
(`qualification_candidate_mismatch`; no qualification record), item 5 UNMET
(`item_2_required`). The promotion recipe was not started.

### Dashboard

`sync-source.sh` exit 0: `integrate/management-workstreams-20260911 @ 66a5b66840`,
tree `0b6ed62b…f5cc`, 380 documents, verified at 2026-10-03T06:00:20Z. Same disclosure as
round 1: the export includes the working-tree bytes of the four uncommitted files in
`scripts/initiative_control/` that belong to another session.

### Owner-broker credential risk to know before item 4

`qualification_broker.py` in subscription mode refreshes in memory when the access token
is within 300 s of expiry and never writes the rotation back. A refresh during the run
would leave `~/.codex/auth.json` holding a used refresh token. Start the run only after the
host's own client has refreshed the file recently.

## Round 3, 2026-10-03: items 1, 2, 3 and 5 pass; item 4 blocked on guest egress, not promoted

Travis's decisions, 2026-10-03:

- **B1:** the manager reads its Claude login from the vault, not from a file.
- **B2:** reuse the PF-83 test VM "macOS 3" (`agent@192.168.64.3`, pf83 key, pinned known_hosts).
- **B3:** "These VMs are expensive and annoying to create. Just programmatically erase data as
  needed to support fresh tests."

**Outcome: not promoted.** The preflight rerun passed items 1, 2, 3 and 5 and refused item 4.
The independent review found the guest can still reach the public OpenAI endpoints, and closing
that needs a guest egress change this round was told not to make. The owner is unchanged:
**armed, generation 1, `fixture-only`**, config digest `cd07b20b…d6f5`, no recovery required;
44,193 ticks, `hold: null`, 0 consecutive errors. Evidence:
`.codex-work/owner-manager-enable-20261002/round3/`.

### B1: vault-backed manager login (commit `edc60bfb02`)

- `fable_launcher` / `manager_cycle` take `--auth-vault-home` instead of `--auth-file`. The
  launcher and its TMUX child each resolve the Claude login at use time and keep it in memory;
  only its SHA-256 is recorded, so a rotation before exec is refused (`vault_auth_changed`).
- `corbanu vault auth-helper` refuses the managed label `provider/claude-code-oauth-token` by
  design ("can only be used by its provider integration"). The launcher therefore runs the
  claude-plan provider's own command-backed helper (`corbanu internal-claude-oauth-token`) with
  the vault home. The /providers screen shows the managed vault credential is the current Claude
  source. If Travis wants the generic auth-helper instead, a copy under an operational label is
  needed.
- Tests: focused 69 OK; full `scripts/initiative_control` discovery **887 OK** in a clean
  worktree at the commit (fresh venv, disposable HOME and aliases,
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`). The first full attempt had 7 errors because `TMPDIR=/tmp`
  is a symlink that the decision fixtures reject; the attempt is kept, not counted.
- `fable_launcher.py` and `manager_cycle.py` are package modules, so the candidate package digest
  is now `146e64e9…eb04c` (was `8021fc53…2065`). The commit reached the remote with the Task Node
  session's push (`aa273695db`).

### Item 2: PASS

- Real manager cycles on the live coordinator with the vault login (Opus via claude-plan):
  - A1: run `206e06c8`, action `owner-first-check-01`.
  - Replacement: `prepare_worker` with `replace: true`, base commit set to `edc60bfb02`,
    revision 2918 → 2919. This cancelled A1 with `owner_cancellation`.
  - A2: run `d9a192f6`, prepared action **`owner-first-check-01-r2`**, allocation digest
    `7c990e19…3b98`.
- One earlier attempt was refused before launch (`tmux_socket_path_too_long`). It was
  reconciled with `fail_manager` and used no inference.
- A2 is still prepared in the live coordinator for the owner handoff. No hand dispatcher should
  claim it.

### Item 4: UNMET — real run completed, mediated_inference not supported

- **Guest cleanup:**
  - Deleted 21 `~/pf83-cases-*` folders and `pf83-cases-29.Y0k1xE/cases.uPHy00`. Their evidence
    is sealed on the host.
  - Kept the `Y0k1xE` package, manifest, packet and runtime.
  - Free space went from 532,192 KiB to 9,428,532 KiB (+8.5 GiB).
- **Credential isolation:**
  - As root through `neo2` (vault `neo-vm` at use time; password through askpass and sudo stdin,
    never written), moved these into a root-only directory: `/Users/agent/.pf83-auth` and a
    52-byte `~/qual/probe/home/auth.json` of unknown origin.
  - A UTM VirtioFS share of host `/Volumes/FASTDRIVE2/z23` (personal files, no Corbanu source)
    was also unmounted for the run.
  - Afterwards everything was restored with identical inode, owner, mode, size, mtime and
    SHA-256: `db552572…f256`, `4ac7ce27…d9f7`, `ec7ab237…eff2`. The share was remounted.
- **Run:**
  - Setup:
    - Binary `7b8c77a6…d77c` plus its `codex-code-mode-host` (`61a15c4c…b711`).
    - Package `146e64e9…` (same digest on host and guest).
    - Broker in subscription mode, reverse tunnel, and the pre-existing system config
      `f4db2b3d…`.
  - Owner lifecycle: a fresh guest fixture owner (step-8 initializer) ran real ticks:
    ACK → START → RETURN in 7 ticks with no holds.
  - Result: `real_ack`, `real_start` and `real_return` = 1. All 11 rollout response IDs join
    broker `relay_completed` records (gpt-6-astra, high). Both the worker's child shell and the
    parent shell ran negative probes against the host checkout, a host canary, GitHub and broker
    routes. The broker-down control failed closed.
- **Broker defects (fixed in `36b6ef77e5`, 64 broker tests OK):** on its first live use the
  ChatGPT Codex backend sent `x-oai-request-id` instead of `x-request-id`, and streamed SSE with
  no Content-Type. The failed attempts (journals 2–5) are kept.
- **Deployment finding:** the live transport binary directory lacked `codex-code-mode-host`, so
  every worker shell call would have failed closed. The signed helper from the same build is now
  installed there.
- **Blocking finding:** the guest packet filter passes 443 to the pinned OpenAI and Z.AI
  addresses.
  - The capture shows two short TLS connections to chatgpt.com at every client start, including
    the worker launch and the broker-down control. None carried inference or credentials.
  - `api.openai.com` answers 401.
- **Independent review** (`claude-opus-5-5-plan`, session `01a10141…`, evidence plus procedure
  only):
  - Verdict: **NOT QUALIFIED**.
  - `isolated_transport`, `isolated_profile` and `negative_access_probes` are supported, with
    gaps.
  - `mediated_inference` is not supported, because public-endpoint bypass is not denied.
  - Counts verified at 1/1/1. Credential restore exact.
  - It also flagged evidence-record inaccuracies, listed in its review file. The recorded
    qualification sets `mediated_inference: false`.
- **Needed:** authorization for an egress fence (outside the guest, or the guest pf pinhole
  removed during the test) that drops everything except the SSH tunnel. Then repeat the
  broker-down control and one owner lifecycle with zero public flows, followed by a fresh review.

### Item 5: PASS

- **Quiet state:**
  - The coordinator's hand-worker TMUX server was not running.
  - No coordinator or publisher callers were running.
  - The facility-control job was identified as unrelated.
  - The owner launchd job was idle.
- **Preflight:** ran 1.7 s after the observation. Before and after: job runs counter 34,989
  (unchanged), no PID, coordinator snapshot unchanged at revision 2922.

### Preflight rerun (exit 2)

Candidate `36b6ef77e5`, action `owner-first-check-01-r2`: items 1, 2, 3 and 5 PASS; item 4
UNMET (`isolated_transport_profile_and_probe_evidence_required`). The promotion recipe was not
started, and the dashboard was not republished; republishing would again export the other
session's uncommitted files.

## Round 4, 2026-10-03: qualified on the third run, promoted to generation 3, first action held

Travis's decisions (2026-10-03), verbatim:

1. "Yes": block the VM's direct OpenAI access during the isolated worker test. Temporarily remove the OpenAI
   addresses from the PF-83 guest's pinned egress (/etc/pf.anchors/corbanu.pf83 `inference` table and the matching
   /etc/hosts entries) using the `neo-vm` admin login from the vault at use time. Run the test, then restore both
   files byte-for-byte and reload pf. Record digests before, during and after. Z.AI entries must also be absent
   during the test if they would let the worker bypass the broker. Prove zero direct connections.
2. "Yes": the manager may keep using `corbanu internal-claude-oauth-token` for its Claude login.

**Outcome:** item 4 qualified on the third isolated run (independent review **QUALIFIED**). The
preflight passed all five items. The promotion recipe ran to completion: the owner is **armed,
generation 3, `tmux-workers`, `manager_enabled: true`**, ticking on the interval with no tick-level
hold. Its first real action then **held** (`wrong_ack`), so "no holds" is not met (see the end of
this section). Evidence: `.codex-work/owner-manager-enable-20261002/round4/`.

### Round-3 write-up corrections

`round3/item4/qualification-evidence-corrections.md` corrects the round-3 evidence file without
changing it: the full kept `Y0k1xE` contents, the broker-down control's two public flows, which
file shows which ENOENT, how the worker server and tcpdump were stopped, the real capture window
and drop count, the failed first initializer attempt, and where the owner operations came from.

### Egress fence and restore (the same in all three runs)

- As root through `neo2`, with the vault `neo-vm` login at use time, the anchor and `/etc/hosts` were
  moved into a root-only hold (same filesystem, so inode and mtime are kept). Test copies were then
  installed: the anchor with the `inference` macro and its 443 pass rule commented out, and
  `/etc/hosts` without its six provider lines (chatgpt.com ×2, auth.openai.com, api.openai.com,
  api.z.ai ×2). The anchor was reloaded, pf states to the eight addresses were killed, and the DNS
  cache was flushed. Z.AI was removed too, because a `--yolo` worker could call any reachable
  endpoint.

| File | Before | During | After restore |
| --- | --- | --- | --- |
| `corbanu.pf83` | `58954a3f…975d`, 3035 B, inode 2387569, mtime 1790982939 | `b4432fbb…7ac6`, 3220 B | identical to before |
| `/etc/hosts` | `6be8ce8e…cba5`, 369 B, inode 17599, mtime 1790982939 | `c7dd0e2e…e2da`, 213 B | identical to before |
| `/etc/pf.conf` | `fb9fe39d…b53b` | unchanged | unchanged |

- After each restore, the anchor table again holds exactly the eight pinned addresses. The probe
  results, as agent:

| Endpoint | Before the fence | During the fence | After restore |
| --- | --- | --- | --- |
| api.openai.com | 421 | no DNS; pinned address times out | 421 |
| api.z.ai | 301 | no DNS; pinned address times out | 301 |
| chatgpt.com | 403 | no DNS; pinned address times out | 403 |
| github.com, example.com | do not resolve | do not resolve | still do not resolve |

  PF-83 reruns work as before.
- Credentials (`.pf83-auth/auth.json` `db552572…f256`, `zai_api_key` `4ac7ce27…d9f7`, qual probe
  `auth.json` `ec7ab237…eff2`) were held root-only for every run. Owner, mode, size, inode, mtime
  and SHA-256 were identical after each restore. The VirtioFS share was remounted each time.

### Runs

| Run | What changed | Result | Review |
| --- | --- | --- | --- |
| 1 (`d23da0`) | Fence plus credential hold. | Attempt A held `wrong_ack` (the ACK came with a trailing newline). Attempt B ran ACK → START → RETURN, with 17/17 broker joins. Zero public packets. | **NOT QUALIFIED**: the controls used `corbanu exec`, not the adapter; prior artifacts were only named; the probes had no positive control; several write-up errors. |
| 2 (`fcfd9a`) | Adapter-level controls. Prior PF-83/qual artifacts held. neo2's home set to 0700. Host-only IPC and broker secret canaries. | Lifecycle 24/24 joins. The share **could be remounted by agent** (UTM maps it to host `/Volumes/FASTDRIVE2/z23`, writable). `~/.corbanu` was unexamined. | **NOT QUALIFIED**: the share, `~/.corbanu`, the `~` probes, and the case expectation weakened after the probe. |
| 3 (`6ef76a`) | `~/.corbanu` also held. The share device stayed mounted by root inside the hold, so agent's `mount_virtiofs` fails "Resource busy". Absolute-path probes; CONNECT sent with the bearer. | Adapter positive control: 3/3 joins. Adapter broker-down control: failed closed (`runtime_failure` hold). Lifecycle: 13 ticks, 23/23 joins. | **QUALIFIED** |

Every run was restored to the same digests. Each run's corrections file and review are kept in `round4/item4/`.

### Item 4 evidence (run 3)

- **Pins:** binary `7b8c77a6…d77c` (codesign ok); 33/33 runtime files byte-identical to
  `577526daab`; package `146e64e9…eb04c` on host and guest; broker `05935e97…476e`; system config
  `f4db2b3d…b8a0` unchanged.
- **Zero direct connections:**
  - The en0 capture (11:40:32-11:50:13Z, everything except established SSH with the host, every
    SYN kept, 0 dropped) has 39 packets, all ARP, router advertisements, mDNS, or host SYNs to
    guest port 22. **No public address appears, and the guest originated no SYN.**
  - pflog shows 76 blocked SYNs to provider addresses, all inside deliberate probe windows. None
    falls in the adapter controls or in the lifecycle outside the worker's own two probe commands.
- **Lifecycle:** 13 ticks, 11:44:49-11:48:29Z, no holds, all owner operations `applied`.
  - The ACK turn's final line is exactly the expected line.
  - The START turn ran 21 tool calls; its first line is `RETURN`.
  - **real_ack 1, real_start 1, real_return 1.**
- **Probes:** run from both the worker's child shell and the parent shell, against host targets
  confirmed present first.
  - The host checkout and its `qa/` are absent, and a symlink to them dangles.
  - The source search finds only the toy repo `.git`.
  - Prior artifacts are absent; neo2's home and `/private/var/root` return EACCES.
  - The share remount is refused.
  - The host canary times out, and the host-only IPC socket is absent.
  - Direct provider addresses fail: no DNS, and pinned addresses time out.
  - With the synthetic bearer, the broker refuses `/v1/models`, the `file://` URL and CONNECT.
  - The secret canary value appears nowhere.
- **Independent review:** `claude-opus-5-5-plan`, session `01a1019e-d234-77e0-adab-f7724df76f6f`,
  given only the evidence and the procedure. **VERDICT: QUALIFIED**, with all four fields supported
  and counts 1/1/1.
  - Disclosed limits:
    - The share is blocked by a root-held mount, not removed in UTM.
    - Fence and capture are in-guest.
    - There is no separate case designer or blind executor.
    - There is no explicit spend record for the host subscription.
  - Hardening follow-ups:
    - Remove the z23 share in UTM.
    - Hold `~/Library`, `/private/var/folders/<agent>` and `/private/tmp/codex-browser-use`.
    - Restrict the anchor's port-22 rule.

### Preflight and promotion

- **Item 5 audit:** run at 12:15:06Z on coordinator revision 2922. No hand-worker TMUX server and
  no coordinator or publisher callers; the Task Node session had finished. The preflight then ran
  1.4 s later on candidate `888d6dd1ad` (clean `git archive`, not the dirty worktree) with action
  `owner-first-check-01-r2`. **All five items passed.**
- **Promotion recipe** (`round4/promotion-recipe.py`, `owner-handoff-80-promotion.md` with one
  disclosed change):
  - The change binds `manager_enabled: true` in the replacement config and recomputes only the
    activation's config digest, as round 1 required. The stored plan was left unchanged, so the
    effect-boundary comparison still checks it.
  - It exited 0. Disarm (generation 2), uninstall, runtime copy, reconfigure and handoff (revision
    2923) followed, then repin, the OFF tick, arm and recover.
  - Watchdog lines before and after:
    - BEFORE: `covered=0, excluded=0; … new default actions=covered`
    - BEFORE ARM and AFTER: `covered=0, excluded=0; … new default actions=excluded; hand stall
      detection=manager responsibility`
  - Transcript: `round4/promotion-transcript.jsonl`. Evidence:
    `schedule/promotion-80-1791029717562719000`.
- **State afterwards:**
  - `--activation-status`: `armed`, generation **3** (next 4), scope `tmux-workers`, config digest
    `8dcf657d…9ead` (with `manager_enabled: true` and the transport), package `146e64e9…eb04c`,
    `recovery_required: false`.
  - `owner-first-check-01-r2` is owner-owned; every other open action is hand-owned (the default).
- **Ticks:** interval firing since 12:15:50Z. At 12:21Z the count was 44,428, with
  `hold: null`, 0 consecutive errors, and launchd runs 11 (last exit 0).

### First live action held: `wrong_ack`

- **What happened:** the owner claimed `owner-first-check-01-r2` and launched worker `w-ylg59fqs`.
  At 12:17Z it held the action with `wrong_ack`, followed by `operation_held`. The worker replied
  with the correct ACK line, but sent it as a commentary message ending in `\n` and then an empty
  final answer. `owner_tmux.provenance` requires the completed turn's final message to equal the ACK
  exactly, so it refused and never sent START.
- **Frequency:** the same behavior from gpt-6-astra caused round-4 attempt A. It has happened on 2
  of 7 real first turns this round.
- **Current state:**
  - Coordinator revision 2924; the action is `dispatching`; the owner reports `state: HOLD` for it.
  - `unresolved_holds` lists `wrong_ack` and `operation_held`.
  - The worker's TMUX server (`~/.local/state/corbanu-owner-runs/w-ylg59fqs/s`) is still up and
    idle; no START was sent.
  - The 1,800 s action timeout will make the watchdog report it as a stall.
- **Why it was left this way:** the owner is armed and fail-closed. A hold-resolution command is
  not implemented. Disarming would revoke both dispatch lanes and consume generation 4 without
  clearing the hold. Unresolved holds also block any later reconfigure.
- **Needed:**
  - A decision on the ACK check: either accept a single trailing newline, or prompt the worker to
    answer in the final message. Either one changes `owner_tmux.py`, which changes the package
    digest and requires re-qualification.
  - A supported way to resolve this hold.
- **To revoke admission now:** `owner_daemon.py --disarm --config … --generation 3`.

### Dashboard

Not republished. The wrapper `initiative-control.oGQGyA/sync-source.sh` exports from this worktree,
which still has the other session's four uncommitted files in `scripts/initiative_control/`.
Publishing from a clean clone would change the declared receiving checkout, so it was skipped.
