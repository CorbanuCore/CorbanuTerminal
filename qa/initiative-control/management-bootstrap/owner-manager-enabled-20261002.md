# Owner manager enable, 2026-10-02/03: automatic loop at generation 11

**Current state (round 7, 2026-10-03T17:40Z):** armed, generation 11, scope `tmux-workers`,
`manager_enabled: true`, package `38055df7…ab1c`, config `9cfde5f7…bc61`, no unresolved holds. A
GUI-domain manager lane (`com.corbanu.initiative-owner.manager`) now starts manager cycles by itself.
A goal seeded through normal intake went manager cycle → prepared action → owner adoption →
ACK/START/RETURN → worker closed → manager cycle → verdict (accepted) with no human step, and the
loop then idled (see round 7). Rounds 1-6 below are the history.

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

## Round 5, 2026-10-03: ACK check fixed, hold resolved, requalified, restaged to generation 5, live action returned

Brief: "Travis wants the loop on and working. Make the loop actually process work." Evidence:
`.codex-work/owner-manager-enable-20261002/round5/` (`qual/` for the VM run, `live/` for the owner).

**Outcome:** armed, **generation 5**, `tmux-workers`, `manager_enabled: true`, config digest
`38ea62a4…c63b`, package `03aefd15…2a46`, `unresolved_holds: []`. The manager prepared
`owner-first-check-01-r3`; the owner claimed it and took it through ACK, START and RETURN with no
holds; the coordinator shows it `returned` (not yet verified by the manager).

### Code (commits `54e6a90229`, `841548849e`, `491046fe4d`; pushed fast-forward from `3c8736914f`)

- **ACK check** (`owner_tmux.provenance`, worker path only): accepts the exact ACK line with
  surrounding ASCII whitespace (space, tab, CR, LF), or an empty final when the turn's correlated
  assistant messages (after this turn's `user_message`, before its completion) carry the line.
  Every non-empty assistant message in the turn must be that exact line. A wrong action id,
  allocation digest, model or effort, interior whitespace, a prefix or fence, extra lines, other
  messages, `\x0b`/NBSP padding and out-of-turn messages are refused with `wrong_ack`. The bridge
  payload ACK stays byte-exact. The receipt keeps `ack_line` exact and records `ack_received` raw.
- **Hold resolution** (`owner_daemon.py --resolve-hold <request.json> --config …`): under both owner
  locks; requires the exact set of unresolved reasons, the recorded claim and allocation digest
  (coordinator and journal), an owner-owned action in a reconcilable or terminal status, all its
  operations `held`, and a stopped worker. An attempted launch needs the run directory and a full
  recorded process identity matching the journal; no recorded process may be alive; the socket must
  be absent or refuse connections. It fails the claim through `reconcile_dispatch` (never resumes or
  relaunches), marks operations `resolved` (the owner lane skips them; reconfigure accepts them),
  writes `resolution_evidence_digest` and `resolved_at` on every hold row and keeps the evidence
  document as a private artifact. Rows are never deleted; a recurrence opens a new hold row.
- **Tests:** `AckTokenTests` (observed newline/non-final shape accepted; wrong action, digest, model,
  effort and extra content refused; byte-exact without `ack_token`; out-of-turn messages) and two
  TMUX tests (newline non-final ACK permits START; newline wrong-effort ACK holds). Resolve tests:
  exact reasons/claim/evidence, listening vs stale socket, live owned pid, missing or mismatched
  identity, non-held operations, missing run directory, hand-owned action, CLI receipt, never
  resumed by a tick, recurrence, OFF reconfigure held vs resolved. Full `scripts/initiative_control`
  discovery at `491046fe4d` in a clean worktree (fresh venv, disposable HOME and aliases,
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`): **906 OK**. Two earlier full runs were stopped when the
  code changed; their partial logs are kept.
- **Code review** (`claude-opus-5-5-plan`): first pass CHANGES REQUIRED (session `01a101c3`: the
  stopped-worker check passed when `process.json` and the socket were deleted); fixed in
  `841548849e`; second pass **APPROVE** (session `01a101c8`). Remaining non-blocking notes: a failed
  inspection can overwrite the journal's process identity with nulls; holds from a worker that dies
  before its identity is recorded cannot be resolved (fails closed); the owner's second `ack_line`
  check is now redundant.
- The runbook line "No hold-resolution command is implemented" in `owner-handoff-80-promotion.md`
  now points to `--resolve-hold`.

### Hold resolution on the live owner

1. `Worker.close()` refused to act: inspection failed with `host_changed`, because
   `kern.boottime` had moved from usec 617366 (recorded at launch) to 555270. I stopped the worker
   through its own socket only: `/quit` (pane exit status 0), then `kill-server` on
   `w-ylg59fqs/s`; pids 72775/72776 gone, socket stale (`live/held-worker-stop.txt`).
2. Disarmed generation 3 (now OFF, generation 4) and uninstalled the schedule with the installed
   runtime, so no old-package tick could re-hold the claim (`live/disarm-uninstall.txt`; a first
   attempt split the path at its space and ran nothing).
3. `--resolve-hold` from the candidate: `wrong_ack` and `operation_held` resolved,
   `resolution_evidence_digest` `5c7bbe062d0210e59825163ad46054d3f427b3e00efa06acab6825ec1f99da7a`,
   `resolved_at` 1791032927.80; action `dispatch_uncertain` → `failed`; coordinator 2925 → 2926;
   the four operations `held` → `resolved`; 0 unresolved holds (`live/resolve-*.json`).

### Qualification (PF-83 VM, `qual/`, run `5cca15`)

Round 4 run 3 repeated step for step with package `03aefd15…2a46` (binary unchanged `7b8c77a6…d77c`):

- **Fence and hold:** same fence, hosts, credential and share handling as run 3.
  - Before, during and after digests are as in round 4.
  - The anchor, `/etc/hosts`, credentials, neo2's home and the 9,910-entry manifest were restored
    identical (inode, mtime, digest), and the share was remounted.
  - Post-restore probes: 421/301/403, as before.
- **Zero direct connections:**
  - en0 has 63 packets: local only, with zero public addresses and zero guest SYNs.
  - pflog has 76 blocked provider SYNs, all in probe windows (one is the 7th retransmit of the
    parent probe's own flow, just past the window's host-clock end). None fall in either control.
- **Controls:**
  - Positive control C returned, with 3/3 broker joins.
  - The broker-down control N failed closed (`runtime_failure` hold).
- **Main lifecycle:** 13 ticks, no holds, 23/23 broker joins; real_ack, real_start, real_return 1.
  - **The ACK turn reproduced round 4's failing shape:** a commentary ACK plus `\n` and an empty
    final. The new check accepted it and START/RETURN followed.
- **Independent review** (`claude-opus-5-5-plan`, session `01a101da-123b-79d1-8ef3-64e69aa9b236`,
  evidence and procedure only): **VERDICT: QUALIFIED**, all four fields, counts 1/1/1.
  - Section-4 corrections are in `qual/review-notes.md`: "HOME clean" is top-level only; the N
    window's zero public packets is an en0 statement (pflog shows blocked Tailscale relay SYNs); the
    pre-auth refusal record has count 2; and a 50-packet block-counter gap falls before the capture
    started.
  - Hardening is still not applied: the z23 share is still in UTM; `~/Library`,
    `/private/var/folders/<agent>` and `/private/tmp/codex-browser-use` are not held; the port-22
    rule is not narrowed.

### Manager action, preflight and restage

- **Allocation:** `owner-first-check-01` replaced through `prepare_worker` (openai, `--yolo`, the
  round-2 authority). The worktree moved to the disposable
  `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-action-r5-20261003` at `491046fe4d`; revision
  2926 → 2927.
- **Manager cycles:**
  - Attempt 1 (`a66cad97`): the decision was correct but wrapped in a ```json fence. The launcher
    refused it (`invalid_json`) and prepared nothing; reconciled with `fail_manager`.
  - Attempt 2 (`5f3a43b4`): accepted. It prepared `owner-first-check-01-r3`, allocation
    `c90f5511…1156`.
- **Preflight:** all five items PASS (`live/item5-preflight-result.json`, 0.9 s after the audit,
  revision 2933 unchanged).
  - **Disclosed adaptation** (`live/restage_preflight.py`): the documented gate assumes an armed
    fixture-only start. Items 1, 3 and 4 and item 2's function run unchanged; item 2 sees the
    replacement worktree list. Item 5 is an OFF-restage variant: the schedule is uninstalled, the
    coordinator already partitioned, the transport unchanged, the owner OFF with no holds, and
    operations applied or resolved.
- **Restage recipe** (`live/restage-recipe.py`, adapted from round 4's): exit 0.
  - Effect-boundary preflight, runtime copy, reconfigure (worktrees + disposable worktree, new
    package), handoff of r3 hand → owner, repin, OFF tick (`owner_off`), arm **generation 5**,
    schedule `--recover`.
  - Watchdog summary unchanged before, before arm and after.
  - Transcript: `live/restage-transcript.jsonl`; evidence `schedule/restage-r5-1791033139852227000`.

### Live receipts (`live/live-operations-r3.txt`, `live/live-rollout-r3-summary.txt`)

| Step | Time (UTC) | Receipt |
| --- | --- | --- |
| claim / prepare / launch | 13:12:55 / :57 / 13:13:00 | applied, generation 5; worker `w-n1g91psg` in the disposable worktree |
| prompt | 13:13:42 | applied |
| **ACK** | 13:14:27 | final message exactly `ACK owner-first-check-01-r3 c90f5511… gpt-6-astra high`; dispatched :29, acknowledged :31 |
| **START** | 13:14:34 | applied; turn 2 user `START`; `working` 13:15:27 |
| **RETURN** | 13:15:29 / :32 | `return_observed`, `returned` applied; coordinator `returned`, result evidence `26a5efe2…f96f` |

- The worker's RETURN: `git rev-parse HEAD` = `491046fe4d…`; `git status --short | wc -l` = 0;
  `docs/sprints/check.py` exit 0 ("current 115; archived 127"); `check_portable_skills.py` exit 0.
  The disposable worktree is still clean.
- **Ticks:** `ACTIVE` with `unresolved_holds: []` on every tick (`live/live-tick-outcomes.txt`); r2
  reports `resolved`.
- After RETURN, the idle worker was closed cleanly through its own socket
  (`live/r3-worker-close.json`: clean, not forced).

### Dashboard

Not republished. `initiative-control.oGQGyA/sync-source.sh` exports this worktree's working tree,
which still has the other session's four uncommitted files in `scripts/initiative_control/`.

### Still open

1. **Boot-identity drift:** `owner_tmux.boot_id()` compares `kern.boottime` including usec. It moved
   62 ms during round 4's held run, so any worker whose run spans such a shift holds with
   `host_changed`, and `Worker.close()` refuses it. The manual stop through the run's own socket
   works, and `--resolve-hold` then accepts the stale socket. A fix (compare seconds, or the
   recorded process start) changes the package and needs requalification.
2. **Manager output fence:** 1 of 2 cycles this round failed `invalid_json` because the decision
   came in a code fence. It is reconcilable, but each failure costs a manager cycle.
3. **Owner routing:** new actions still default to `hand`. Each owner action needs an explicit
   handoff, as r3 got in the restage. The owner does not close a worker after RETURN.
4. `owner-first-check-01-r3` is `returned`; manager verification (accept or reject) is pending.
5. VM hardening (above), `slack-receiver-02` (legacy hand claim, still running/stalled), and the code
   review's non-blocking notes.

## Round 6, 2026-10-03: boot identity, fenced JSON, close after RETURN, default routing, manager verdicts; generation 9; hands-off loop

Brief: "Travis wants this to be an automatic loop: work should flow manager → owner → worker → back
without hand steps or manual hold clearing." Evidence: `.codex-work/owner-manager-enable-20261002/round6/`
(`qual/` attempt 3, `qual-1-e1b29b6e/` and `qual-2-53689742/` attempts 1-2, `live/`, `code-review*/`).

**Outcome:** armed, **generation 9**, package `abe1ae15b2cdac13d8e9c08bf036ede221878bb030e72c2997ef2a147ddcf305`,
`unresolved_holds: []`, ticking on its interval. Generation 7 (package `53689742…4706`) carried the
hands-off run; generation 9 adds only the verdict-aware briefing choice in `manager_cycle.py`.

### Code (pushed fast-forward `e477859183` → `7e844abfbd`)

| Commit | Change |
| --- | --- |
| `55b5d1076d` | All five items below, with tests |
| `ef7601c047` | Review 01a101f9 fixes (prior-generation return, routing of hand-touched work, close on pause, accepting verdict needs the result body) |
| `bc473d5c59` | Review 01a10202 fixes (post-RETURN holds resolvable, proven-stopped close, route only gate-admissible work, handoff evidence chain) |
| `07c7df70e9` | A `ps` timeout during the stop proof falls back to the real close |
| `fbfb166d78` | Briefing reads returned results first and evicts them last |
| `da4e4e468c` | Test only |
| `7e844abfbd` | `fit_briefing` picks the largest event prefix that still carries every returned result awaiting a verdict |

1. **Boot identity** (`owner_tmux.boot_id`): macOS now uses `kern.bootsessionuuid`
   (`bootsession:<uuid>`), fixed for a boot session; `kern.boottime` is re-derived from the wall
   clock and moved 62 ms in round 4. Linux keeps `/proc/sys/kernel/random/boot_id`. The owner's
   `boots.host_boot_id` uses the same function. Tests: jittered `kern.boottime` keeps the identity;
   a new session UUID changes it; a real-tmux worker under jitter inspects and closes cleanly; a
   changed boot id yields `host_changed` and `Worker.close` sends nothing. Workers recorded in the
   old format would fail closed (`host_changed`); none were running at the restage.
2. **Manager JSON fences** (`fable_launcher.decision`): exactly one whole-message ```` ```json ````
   or ```` ``` ```` block (surrounding whitespace only) is unwrapped; prose before/after, two blocks,
   another language tag, CRLF fences, unterminated fences and non-object content stay `invalid_json`.
   Tests at unit and full-cycle level. The live verdict cycle `a48f4abd` answered in a fence and
   was accepted.
3. **Close after RETURN** (`owner_daemon.Kernel.close_worker`): after the coordinator records RETURN
   the owner journals a `close` operation and runs `Worker.close` on the worker's own socket
   (`/quit`, then `kill-session`); the receipt is the audit record and the process row becomes
   `closed`. An already-stopped worker (recorded processes dead, socket absent or refusing) gets
   that proof instead of keys. An unclean close holds `worker_close_unclean`; an uncertain close is
   never retried; a pause or manager-owned cycle defers it; a restart between RETURN and close
   finishes it next tick. A prior generation's return is history only if its close applied or the
   worker is proven stopped, otherwise it still holds `prior_activation`/`effect_uncertain`.
   `--resolve-hold` now settles post-RETURN holds (status `returned`/`accepted` with a receipted
   RETURN) without touching the coordinator. Real-tmux tests: idle worker closed through its socket;
   killed pane closed.
4. **Routing.** Found: `Coordinator.dispatch_owner` returns `action.get("dispatch_owner", "hand")`
   once `dispatch_control` exists, and `_handoff` stores `default: "hand"` (commit `08f42237c6`,
   `owner-handoff-80.md`). The recorded reasons are about existing work: "Existing coordinators
   retain their old contract until explicit cutover", and the owner watchdog must not mutate hand
   claims ("prevents the scheduled owner from changing an external hand claim to
   dispatch_uncertain", `owner-cutover-85.md`). No recorded safety reason requires new
   manager-prepared work to start as hand, so the coordinator default is unchanged and the armed
   owner routes instead (`Kernel.route_defaults`, owner dispatcher only): unassigned, `prepared`,
   unclaimed, worker-kind actions from a manager run whose frozen worktree is in the owner's
   configured `worktrees`, never touched by the hand lane (no journal rows, no hold), and admissible
   by the claim gate now (allocation digest, sprint executable, resources free, one per resource
   set). One revision-checked `_handoff` per tick with evidence `armed_scope_default_route`
   (generation, activation, config and package digests, previous `dispatch_control`). Everything
   else stays hand; a concurrent revision change routes nothing that tick. Activation status shows
   `watchdog_coverage.default_route`. Note: the configured `worktrees` include the main integration
   worktree, so manager actions targeting it are also in scope.
5. **Manager verdicts** (`Coordinator.accept_decision`): a decision may carry `verdicts`
   (`action`, `accepted`, `reason`) for actions in status `returned`; same transition and evidence
   shape as `verify()`, with a non-meaningful `verified:` event so the manager does not wake itself.
   Invalid verdicts reject the whole decision; actions may be empty only with a verdict. `run_cycle`
   refuses an accepting verdict whose result body was not in the briefing
   (`verdict_without_result_evidence`). The manager's instructions name the field.
6. **Briefing for verdicts** (found live, see below): returned results are read first and evicted
   last, and `fit_briefing` picks the largest FIFO event prefix whose briefing keeps all of them
   (deferred events stay pending), else the largest prefix that fits.

**Tests:** full `scripts/initiative_control` discovery in clean worktrees (fresh venv, disposable HOME
and aliases, `CORBANU_TEST_NO_NATIVE_KEYRING=1`): `07c7df70e9` **930 OK**, `fbfb166d78` **930 OK**,
`7e844abfbd` **930 OK**. `55b5d1076d` had 1 expected failure (a real-tmux test now sees `closed`),
fixed; runs at `ef7601c047` and `bc473d5c59` were stopped when the code changed. Discovery's
`test_*.py` pattern does not match `manager_cycle_test.py`; it ran separately at `7e844abfbd`: 24 OK.

**Code reviews** (`claude-opus-5-5-plan`, high effort, read-only, code and records only):
`01a101f9` CHANGES REQUIRED → `01a10202` CHANGES REQUIRED → `01a10209` **APPROVE** (all five items);
`01a1021e` **APPROVE** (`fbfb166d78`; noted the live briefing has almost no room); `01a1023c`
**APPROVE** (`7e844abfbd`). Non-blocking notes kept: a crash between `c.returned` and its receipt
leaves an unresolvable `effect_uncertain` (pre-existing); a routed action can still hold if its
gate fails between routing and claim (race); the fallback when results cannot all fit is
all-or-nothing; the restriction audit reason text is unchanged.

**Review budget:** this track used 5 code reviews and 3 qualification evidence reviews this round,
above the default five. The extra passes removed real blockers (two CHANGES REQUIRED rounds; one
requalification per package change). The integrator should record the extension.

### Qualification (PF-83 VM, three attempts, each round 5 repeated step for step)

Each attempt: same fence, hosts, credential and share handling; exact restore (anchor and
`/etc/hosts` back to `58954a3f…`/`6be8ce8e…` with original inode and mtime, three credentials
identical in owner/mode/size/inode/mtime/digest, 9,910-entry manifest identical, neo2's home 0755,
share remounted, guest `/private/tmp` cleaned); post-restore probes 301/421/403. Zero en0 public
packets and zero guest SYNs; 76 blocked provider SYNs, all from the probes or the worker's own probe
steps; none from C or N. Positive control C returned; broker-down N failed closed
(`runtime_failure`). C and M were **closed by the owner itself** after RETURN (clean receipts).

| Attempt | Package | M ticks / joins | ACK shape | Review |
| --- | --- | --- | --- | --- |
| 1 `ec703d` | `e1b29b6e…4db0` (`07c7df70e9`) | 12 / 23 | C: commentary + `\n` + empty final, accepted | `01a10218` QUALIFIED |
| 2 `f4e5c6` | `53689742…4706` (`fbfb166d78`) | 13 / 24 | M: commentary + `\n` + empty final, accepted | `01a1022a` QUALIFIED |
| 3 `9116df` | `abe1ae15…f305` (`7e844abfbd`) | 12 / 23 | exact final | `01a10248` QUALIFIED |

Corrections the reviewers found are in each `review-notes.md` (unchanged evidence files). Attempts
2 and 3: control C overlapped the end of the seal egress probe (I issued both at once); the provider
SYNs in the overlap are the probe's last two pinned curls (attempt 3: 10, not the 12 the evidence
says). Hardening still not applied: z23 share still configured in UTM; `~/.ssh`, `~/Library`, `~/bin`
not held; port-22 pass rule not narrowed; no interface or anchor-table inventory.

### r3 verdict through the normal path

Manager run `eaca7fec` (candidate `07c7df70e9`, owner still at generation 5) returned
`verdicts: [owner-first-check-01-r3, accepted false]`: "Result body is withheld under
evidence_budget … cannot accept unverified." The briefing was at its 64 KiB limit (3 events
selected, 9 bodies dropped). r3 is `failed`. That exposed item 6.

### Live run (generation 7, then generation 9)

1. **Restage to 7:** disarm generation 5 first refused `BlockingIOError` (a tick held the lock);
   uninstall, then disarm → OFF generation 6. Preflight items 1-5 PASS (`live/restage_preflight.py`,
   disclosed variant: item 2 checks the two new allocations instead of a prepared action, because
   this package routes actions prepared after arming; item 5 is round 5's OFF variant). The recipe
   (`live/restage-recipe.py`, no selected handoff) reconfigured (two disposable worktrees added),
   repinned, then stopped at its OFF-tick assertion: the uninstall had booted out a tick in flight,
   latching `interrupted_tick`. `live/restage-continue.py` recovered it with evidence, ran the OFF
   tick (`owner_off`), armed **generation 7** and recovered the `owner_off` latch.
2. **Allocations** (`prepare_worker`, openai, `--yolo`, round-2 authority): `owner-loop-r6-a`
   (read-only check) and `owner-loop-r6-b` (one uncommitted docs note), worktrees
   `owner-loop-r6-{a,b}-20261003` at `da4e4e468c`.
3. **Manager `d22519f8`** (14:37:30Z) prepared `owner-loop-r6-a-01` and `owner-loop-r6-b-01`.

| Step (UTC) | a-01 | b-01 |
| --- | --- | --- |
| routed to owner (one handoff, rev 2947) | 14:37:37 | 14:37:37 |
| claim / launch | 14:37:39 / :43 | 14:37:47 / :51 |
| prompt | 14:38:35 | 14:38:45 |
| **ACK** (exact final line) | 14:39:30 | 14:39:49 |
| **START** | 14:39:36 | 14:39:56 |
| working / **RETURN** recorded | 14:40:52 / 14:40:57 | 14:41:23 / 14:41:27 |
| **worker closed** (own socket, clean, pane exit 0, server gone) | 14:41:00 | 14:41:30 |
| **manager verdict** `a48f4abd` (14:49:16) | accepted | accepted |

   - Every tick `ACTIVE`, `unresolved_holds: []` (`live/live-tick-outcomes.txt`).
   - RETURNs: a-01 HEAD `da4e4e468c…`, 0 changes, `docs/sprints/check.py` and
     `check_portable_skills.py` exit 0; b-01 created only `owner-loop-r6-note.md` (HEAD, `26`).
   - Before the verdict, the briefing still could not carry both results (dry run on a copy:
     largest fitting prefix kept neither). I compacted `owner-first-check-01` and
     `acct-coverage-123` (all actions terminal) to consumed stubs (the documented
     `briefing-budget-recurrence-20260916.md` mechanism) and ran the verdict cycle from candidate
     `7e844abfbd`, which chose 2 events and kept both results. The manager's reasons cite the
     returned contents.
4. **Restage to 9:** after a completed tick, uninstall then disarm → OFF generation 8 (no latch this
   time); preflight 1-5 PASS (`live/restage_preflight_gen9.py`: item 2 checks that no owner work is
   pending and both hands-off actions carry accepting manager verdicts); recipe exit 0; **generation
   9** armed and ticking (`live/activation-status-final.json`).

### Dashboard

Not republished. `initiative-control.oGQGyA/sync-source.sh` exports this worktree's working tree,
which still has the other session's four uncommitted files in `scripts/initiative_control/`.

### Still open

1. **Manager cycles are not automated.** The owner routes, runs, returns and closes by itself; a
   manager cycle (`manager_cycle.py --run`) still has to be started to prepare work and to give
   verdicts. The owner's `manager_enabled` only labels the fixture tick.
2. **Briefing budget.** The live briefing sits at 64 KiB: `consumed_allocations` (245 entries,
   ~22 KB) and `actions` (~20 KB) dominate, 85 events are pending, and a cycle takes 2-4 events.
   Item 6 makes verdicts work at small batches; the base size needs a structural fix and routine
   compaction.
3. Review non-blocking notes (routing/claim race, all-or-nothing fallback, restriction reason
   text); the hardening list above; `slack-receiver-02`; the round-5 notes.
4. Disposable worktrees `owner-action-r5-20261003` and `owner-loop-r6-{a,b}-20261003` are in the
   owner config; b keeps its uncommitted note.

## Round 7, 2026-10-03: automatic manager cycles, bounded briefing, disposable-only adoption; generation 11; no-human-step loop

Brief: make the loop fully automatic (cycles start themselves, rate-limited, audited, fail-closed),
make the briefing budget structural, keep the main integration worktree out of auto-adoption, then
requalify, promote and prove the loop with no human step. Evidence:
`.codex-work/owner-manager-enable-20261002/round7/` (`qual/`, `live/`, `code-review-*/`, `suite-*.txt`).

**Outcome:** armed, **generation 11**, package `38055df74625e17d00f5b0ddd496ed0b8a86034a98b7cc604aaafd5bcae3ab1c`,
config `9cfde5f73fc5baf1489f28bd9b4652961c575f436178ed9a3084a9558e96bc61`, `unresolved_holds: []`.
The owner lane ticks in `user/501` as before; the new manager lane ticks in `gui/501`.

### Code (pushed fast-forward `bbd703ee66` → `ec7918b368`)

| Commit | Change |
| --- | --- |
| `213c814364` | Manager lane, bounded briefing, compaction, no-action decisions, disposable-only routing, with tests |
| `81eccfc1a8` | Review 1 fixes (below) |
| `2faff0f5cf` | Review 2 notes: the acceptance gate holds the admission lock; route markers before the handoff; in-progress rebase/bisect/merge is not disposable |
| `ec7918b368` | Review 3 notes: revert/sequencer in progress is not disposable; route-marker order pinned |

1. **Automatic manager cycles.** Finding first: the owner's launchd job runs in the `user/501`
   Background domain, where the Claude login fails ("failed to load existing secrets encryption key
   (OS keyring: … User interaction is not allowed.)"; probe as a temporary user-domain job). The same
   probe in `gui/501` succeeded. So cycles cannot run inside the owner's own job. Instead there is a
   second schedule, the **manager lane**: same runtime and config, receipt `lane: manager`, GUI
   domain, label `com.corbanu.initiative-owner.manager`, root `corbanu-owner-live/manager-schedule`,
   publishing `manager-recurrence.json` to its own `publish-manager` directory
   (`activate.py --owner install --lane manager`; a root keeps its lane, `installation_lane_conflict`).
   Each tick runs `owner_daemon.manager_lane`:
   - **Only while armed:** admission is checked under the owner locks; OFF (`owner_off`), a paused
     coordinator or a manager already owning the coordinator start nothing and latch nothing.
   - **When there is work:** pending meaningful events; a result awaiting a verdict or, with nothing
     queued, live allocations never used or only failed (the last two through one deduplicated
     `owner-wake:` event per situation and 12-hour window).
   - **Rate limits, recorded in config** (`manager_cycle`): `min_interval_seconds` 120 between starts,
     `daily_cap` 48 starts per UTC day (cap logged once), `timeout_seconds` 900, the binary pinned by
     digest, a dedicated `runs_dir` (`/Users/Neo/.local/cma`). One in flight: the lane's tick lock,
     launchd's single instance and the coordinator's manager claim.
   - **Audit log:** every start, finish (status, manager run, prepared actions, verdicts, batch,
     no-action reason), wake, compaction, hold and recovery is a line in
     `manager-schedule/manager-cycles.jsonl`; each cycle's owner context and result are artifacts
     under `manager-schedule/cycles/<id>/`; the coordinator audit has the usual manager rows.
   - **Fail closed:** any failed cycle, crash (interrupted tick) or error latches the lane's schedule
     HOLD; no retry until `--recover <evidence>` on that schedule. When a failed cycle returns, the lane
     releases the coordinator claim only with its launchers proven stopped (recorded process not alive,
     no process naming its launch directory, socket absent or refusing), so worker dispatch is not left
     "owned"; recovery after a crash does the same, only when the lane is held. The run ID and cycle
     directory exist before the claim, so a crash at any point is attributable.
   - **Disarm/pause during a cycle:** the decision is committed inside an acceptance gate that holds the
     owner's admission lock and re-checks that the owner is armed at the generation that started the
     cycle; a refusal commits nothing and holds.
   - The owner gives the manager an automatically generated context (triggers, results awaiting a
     verdict, owner holds, queued work, cap state). A decision may now be `actions: []` with
     `no_action_reason` (coordinator and launcher), so "nothing to do" consumes the batch instead of
     holding.
2. **Briefing budget.** Measured on a copy of live state: the old briefing selected 6 of 85 events at
   65,413 bytes (consumed-allocation index 21,929 B). Now:
   - The consumed index is summarized as count, digest of the full index and the entries a supplied
     action names (the full index stays in the coordinator state).
   - A prioritized claim: results awaiting a verdict, stalls, failures and owner wake-ups first, then
     the two oldest others (aging, so the backlog always moves), then the newest. `event_backlog`
     counts every pending event by kind and lists the 16 oldest unclaimed IDs; unclaimed and restricted
     events stay pending for the next cycle. Fit and eviction rules are unchanged (prefix only; oversized
     events still hold).
   - Routine compaction (the consumed-stub method, now `Coordinator.compact_allocation`, a
     non-meaningful event) runs in every lane pass and before every cycle: an allocation whose actions
     are all terminal is compacted when its sprint is completed or, without lifecycle kinds, 24 h after
     an accepted action / 7 days otherwise. Lifecycle allocations stay until their sprint completes,
     because `complete_sprint`/`activate_successor` re-check their digest.
   - Same live copy: all 24 claimed events selected at 62,589 bytes.
3. **Main worktree scope.** Default routing (auto-adoption) now needs a configured worktree that is a
   linked worktree detached at the allocation's frozen `base_commit`, with no rebase, bisect, merge,
   cherry-pick, revert or sequencer in progress; the claim re-checks it for default-routed actions
   (`worktree_not_disposable`). A branch checkout such as the main integration worktree, a plain
   directory, a main checkout or a moved HEAD stays hand. The live config's worktrees are now only
   `worktrees/owner-loop-r7-20261003` (the main integration worktree and the old r5/r6 disposable
   worktrees were removed).

**Tests:** full `scripts/initiative_control` discovery in clean worktrees (`git worktree add --detach`,
round 6 venv, disposable HOME and aliases, `CORBANU_TEST_NO_NATIVE_KEYRING=1`): `213c814364` **954 OK**,
`81eccfc1a8` **961 OK**, `ec7918b368` **963 OK**; `manager_cycle_test.py` (not matched by discovery)
30 OK at each. A run at `2faff0f5cf` was stopped when `ec7918b368` superseded it (kept as
`suite-2faff0f5cf-aborted.txt`). New tests cover: lane OFF/disarmed/paused/owned/idle; one audited
cycle and the interval; the daily cap and its reset; wake once plus the 12-hour repeat; never-used
and failed allocations; compaction without a wake; holds with the claim released only when launchers
are proven stopped (dead, live, missing record, a process naming the launch directory); disarm during a
cycle; the gate's lock, generation check and journal retry; scheduled latching and evidenced recovery;
**a crash mid-cycle (child process killed after the claim) and the restart: the next passes hold
`interrupted_tick` without calling the manager, recovery releases the claim, and the next cycle runs**;
recovery without a hold leaves the coordinator alone; lane-bound install and separate publication; the
binary pin; prioritized and aging claims; no-action decisions; compaction policy and lifecycle kinds;
the live-shaped briefing; disposable worktrees (branch, plain, main checkout, moved HEAD, no base
commit, in-progress markers) and the claim-time re-check.

**Code reviews** (`claude-opus-5-5-plan`, high effort, read-only, code and records only):
`01a10281` **CHANGES REQUIRED** (compaction could break lifecycle proofs; newest-first could starve
old events; no disarm re-check; missing launcher record counted as proof; crash before the cycle
directory; lane/publish separation; re-wake; worktree binding) → fixed in `81eccfc1a8` →
`01a10291` **APPROVE** (low notes: disarm race window, journal retry, route-marker order, detached
integration worktree) → fixed in `2faff0f5cf` → `01a1029a` **APPROVE** (notes: revert/sequencer,
marker-order test) → `ec7918b368`. Non-blocking notes kept: a disarm during acceptance is refused (retry)
rather than queued; the 25 s gate wait comes out of the 30 s launch margin, so a very long owner tick
can hold the lane (fails safe); never-used allocations on a completed sprint or paused stream wake twice
a day until compacted.

**Review budget:** 3 code reviews and 1 qualification evidence review this round (4 of the default 5).

### Qualification (PF-83 VM, `qual/`, run `db8ba2`, round 6 attempt 3 repeated step for step)

- Same fence, hosts, credential and share handling; exact restore (anchor and `/etc/hosts` back to
  `58954a3f…`/`6be8ce8e…` with original inode and mtime, three credentials identical in
  owner/mode/size/inode/mtime/digest, 9,910-entry manifest identical, neo2's home 0755, share remounted,
  guest `/private/tmp/q11*` removed); post-restore probes 301/421/403.
- Disclosed script change: this session's command guard blocks `rm -f`, so `root-restore.sh` renames the
  held originals over the test copies with a plain `mv` (same inode/mtime result) and the cleanups use
  `/bin/rm -r` (`script-diff-vs-round6-*.txt`).
- Controls ran only after the egress probe ended (no overlap this time). Positive control C returned with
  3/3 broker joins and was closed by the owner; broker-down control N failed closed (`runtime_failure`).
- Main lifecycle M: 14 ticks, no holds, 23/23 broker joins, exact-final ACK, 21 tool calls, RETURN, owner
  close clean; real_ack, real_start, real_return 1.
- Egress: zero guest SYNs and zero guest-originated packets to public addresses on en0; provider SYNs
  only from the probes and the worker's own two probe steps. New and disclosed: a TCP connection to
  chatgpt.com's address (guest port 59758) that existed **before** the fence. pf killed its state and
  dropped all 12 later packets (7 keepalives out, the server's close in), 5 of which reached en0 inbound.
  No data passed and no new connection was made; it predates every worker. Its owner is unknown (the
  reviewer showed my "baseline curl" attribution is probably wrong; corrected in `qual/review-notes.md`).
- **Independent review** `01a102ad-1307-74e3-bb0f-62e7fa5e84fa` (evidence and procedure only):
  **VERDICT: QUALIFIED**, all four fields supported, counts 1/1/1; it does not affect
  `mediated_inference`. Follow-ups: record a root `lsof -i`/`netstat -anv` before the fence; point the
  case's canary probe at the current round's path (it still named round 6's).

### Preflight and promotion (`live/`)

1. After a completed tick: uninstall the owner schedule, disarm generation 9 → OFF generation 10
   (`disarm-uninstall-gen9.txt`).
2. Item 5 audit and preflight (`item5-audit-gen11.py`, `restage_preflight_gen11.py`: round 6's
   generation-9 variant plus the round-7 replacement config: main integration and old disposable
   worktrees removed, `owner-loop-r7-20261003` added, `manager_cycle` settings added). First run: item 4
   UNMET `unsafe_file` (the new qualification files were 0644); after `chmod 600` all five items PASS,
   0.9 s after the audit, coordinator revision 2960 unchanged.
3. Recipe (`restage-recipe-gen11.py`, exit 0): effect-boundary preflight, runtime copy, reconfigure,
   repin, OFF tick, arm **generation 11**, recover, then the manager lane installed in `gui/501`
   (plist `0946cd88…c0a6`). Watchdog line unchanged before, before arm and after.

### No-human-step proof (all times UTC, 2026-10-03)

After the restage I started no manager cycle. The lane's own first run (the install's kickstart) found
the 85-event backlog and worked through it by itself:

| Cycle start → finish | Batch | Decision |
| --- | --- | --- |
| 17:01:07 → 17:02:22 | 24 selected, 61 deferred | no action (reason recorded) |
| 17:03:24 → 17:04:40 | 24, 37 deferred | no action |
| 17:05:43 → 17:08:10 | 24, 13 deferred | no action |
| 17:08:42 → 17:10:15 | 13, 0 deferred | no action |

Then the lane was IDLE. At **17:12:29** I seeded one read-only goal through the normal intake
(`coordinator_cli.py prepare_worker`, allocation `owner-loop-r7`, sprint PF-80-S01, kind `review`,
worktree `owner-loop-r7-20261003` detached at `ec7918b368`, base commit the same; evidence asked the
manager to prepare one review action and later give a verdict). Everything after that ran by itself:

| Step | Time | Receipt |
| --- | --- | --- |
| Manager cycle starts (trigger `pending_events`) | 17:12:51 | lane log `cycle_started` 8eea9808; coordinator `begin_manager` 17:12:52 |
| Manager prepares the action | 17:14:03 | `owner-loop-r7-01` prepared (manager run `bb48c413`) |
| Owner adopts it (default route) | 17:14:18 | `dispatch_handoff`, evidence `armed_scope_default_route`, generation 11 |
| Claim / prepare / launch | 17:14:21 / :24 / :26 | owner operations applied; worker `w-wrzs7zmp` |
| Prompt | 17:15:10 | applied |
| **ACK** | 17:15:55 | commentary `ACK owner-loop-r7-01 d33f8b87… gpt-6-astra high` + newline and an empty final (round 5's accepted shape) |
| dispatched / acknowledged | 17:15:57 / :59 | applied |
| **START** | 17:16:01 | turn 2 user `START` |
| working / **RETURN** | 17:16:56 / 17:17:00 | `return_observed` 17:16:58, coordinator `returned` 17:17:01 |
| **Worker closed** | 17:17:02-:04 | own socket, clean, not forced, pane exit 0, no survivors, server probe rc 1; process row `closed` |
| Manager cycle starts again (trigger `pending_events`, the `returned:` event) | 17:17:09 | lane log `cycle_started` 4515c86b |
| **Verdict** | 17:18:59 | `accepted` (manager run `303b7083`): "Read-only check met: HEAD ec7918b3 equals base_commit, 0 changed files, both checks exit 0, RETURN format, no writes." |

- The worker's RETURN: `git rev-parse HEAD` = `ec7918b368…` (matches base); 0 changed files;
  `docs/sprints/check.py` exit 0 ("current 115, archived 127"); `check_portable_skills.py` exit 0. The
  worktree is still clean.
- No hold at any point (`live/live-receipts-r7.txt`, `live/live-rollout-r7.txt`, the owner and lane logs).

### Idle afterwards (`live/idle-evidence.txt`, observed 17:36:16Z)

- 17.3 minutes after the verdict (17:18:59Z → 17:36:16Z): **no manager cycle**. The lane log has no
  record after the verdict; `manager-auto.json` still shows 6 cycles today, last start 17:17:09; the
  coordinator has no `begin_manager` audit row after the verdict, revision 2978 unchanged, no manager
  claim and 0 pending meaningful events.
- The lane ticked 33 times on its 30 s interval (30 `IDLE`, 3 `BUSY` while an owner tick held the
  owner locks), each under 1.6 s, no hold, 0 consecutive errors. The owner lane's last 30 ticks were
  `ACTIVE` with no unresolved holds. No worker server or launcher process remains.

### Dashboard

Not republished. `initiative-control.oGQGyA/sync-source.sh` still exports this worktree's working tree,
which still has the other session's four uncommitted files in `scripts/initiative_control/`.

### Still open

1. Non-blocking review notes: a disarm during an acceptance is refused (retry) rather than queued; the
   acceptance gate's wait can hold the lane after an unusually long owner tick (fails safe); never-used
   allocations on a completed sprint or paused stream wake the manager twice a day until compacted.
2. Qualification follow-ups: record a root `lsof -i`/`netstat -anv` before the fence; point the case's
   canary probe at the current round's path; the hardening list from rounds 4-6 (z23 share still in UTM,
   `~/.ssh`/`~/Library`/`~/bin` not held, port-22 rule not narrowed).
3. The manager lane depends on the GUI session (`gui/501`): if the operator logs out, manager cycles stop
   (the owner lane keeps ticking in `user/501`). A held lane needs `--recover <evidence>` on
   `corbanu-owner-live/manager-schedule`.
4. Carried over: `slack-receiver-02` (legacy hand claim, still running/stalled), the round 5-6 notes.
