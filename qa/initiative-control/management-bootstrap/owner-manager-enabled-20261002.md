# Owner manager enable, 2026-10-02: stopped by the promotion gate, still armed fixture-only

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
