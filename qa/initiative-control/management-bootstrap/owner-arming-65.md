# owner-arming-65 — operator arming and revocation

Allocation digest: `02fe592309aebc95e8de07089d2404a31726b8211960220e198ef7e7549a743c`.
Claim: `086d2091-b813-4825-a10c-e9213e252e47`.
Brief SHA-256 verified before reading:
`5eafcb8694551ad03cace56f6738482f2d37e677fece3f68ae8b29fdd9480f31`.
Base/initial HEAD: `d53c177ca4fbf0ed5aa05bb3f8e611ae76d1746b`.
Worker: gpt-6-astra, high. Worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.

Product initiative within active `initiative-delivery-control`, PF-80-S01
(`in_progress`), under the manager's frozen bootstrap allocation. Product heading:
**Internal delivery control — TO BUILD**, “durable event dispatch,
acknowledgments and watchdog”; “initialize and rehearse all three workstreams
before enabling recurring operation”. The registered plan includes this worktree;
its historical registered base is `08db99fff46fd22c582fbea0240fa379a42242e7`.
The present dispatch supplies the newer exact base above. Shared sprint/plan
coordinates and Done/Remaining updates remain manager-owned, outside writable scope.
Sprint checker: 115 current, 127 archived, exit 0.

## Entry points

All commands below target this candidate's `owner_daemon.py`.
No live launchd installation, live activation or live coordinator writes were performed by
this worker. Every execution used disposable synthetic state.

```text
python owner_daemon.py --activation-status --config /absolute/private/config.json
python owner_daemon.py --arm --config /absolute/private/config.json --authority /absolute/private/decision.json
python owner_daemon.py --disarm --config /absolute/private/config.json --generation CURRENT
```

Status is read-only and returns current/next generation, actual package digest,
canonical config digest, transport scope, stored metadata and pending recovery.
It does not assert that configuration validation or live qualification passed.
The existing config and owner database must already have been initialized for
this package; these commands never initialize or silently rebind them.
Final source package digest:
`8dba564b3e1079457680f4e2e0fff0592ee49b472ae7947e4d95c29cd08f96c5`.
The digest of configuration is `coordinator.digest(parsed_config)`, not the
SHA-256 of its on-disk whitespace. Package digest uses the existing five-file
package definition, including the changed owner daemon itself.

The caller supplies a private 0600 JSON file with exactly these seven fields:

```json
{
  "decision_id": "operator-recorded-decision-id",
  "revision": 1,
  "authority": "actual authorizing person and decision reference",
  "scope": "fixture-only",
  "generation": 1,
  "config_digest": "EXACT_CONFIG_DIGEST_FROM_STATUS",
  "package_digest": "EXACT_PACKAGE_DIGEST_FROM_STATUS"
}
```

Use the reported next generation, not a copied example's 1. Scope is exactly
`tmux-workers` when transport is present and `fixture-only` otherwise.
Authority is an explicit same-UID operator attestation, not a verified signature
or independent proof that the named person approved it. The entry point does not
invent authority, consult a decision feed or grant additional product scope.

Arm requires OFF state and exactly current generation + 1, writes activation.json
and the four binding columns plus requested_mode under both owner locks, and
returns ARMED only after durable publication. Disarm requires the current
generation, commits OFF and advances generation by one. It retains activation.json
and its metadata as revoked evidence. A subsequent arm needs another new
generation. Disarm does not stop already-running workers, resolve operation
holds, change coordinator enablement or clear schedule HOLD. It remains available
after package/config-content drift if the original store is still identifiable
and its schema/files are safe.

SQLite and a JSON file cannot participate in one physical atomic transaction.
This implementation provides atomic **admission**: locks exclude concurrent ticks,
a fsynced activation-transaction.json precedes file/SQL writes, FULL-synchronous
SQL commit precedes intent removal, and admission rejects any remaining intent
or partial-file marker. A crash can leave partially committed storage, but never
an admissible partially committed activation. Explicit disarm consumes a generation
and clears these markers after committing OFF. It does not automatically repair
a SQLite hot journal; that remains `owner_recovery_required`, requiring separately
reviewed offline recovery. No power-loss/filesystem durability qualification is claimed.

## Refusals and deliberately wrong inputs

These are the activation decision/transition reasons. Each row maps to a separate
`ArmingTests.test_refuses_*` case; tests assert the exact error and unchanged
database/file bytes. No valid-input-only assertion substitutes for these cases.

| Reason | Wrong-input test suffix |
| --- | --- |
| `invalid_activation` | `invalid_activation_keys` (extra key) |
| `activation_decision_id_required` | `invalid_decision_id` (list) |
| `activation_authority_required` | `blank_authority` |
| `activation_revision_required` | `invalid_revision` (boolean) |
| `activation_generation_required` | `invalid_generation`; `disarm_boolean_generation` |
| `activation_scope_mismatch` | `wrong_scope_without_transport`; `wrong_scope_with_transport` |
| `activation_config_mismatch` | `wrong_config_digest` |
| `activation_package_mismatch` | `wrong_package_digest` |
| `activation_generation_mismatch` | `stale_generation`; `skipped_generation`; `disarm_stale_generation` |
| `owner_already_armed` | `already_armed` (any mode other than OFF is refused) |
| `activation_recovery_required` | `pending_activation` |
| `state_drift` | `state_drift` (stored config digest changed) |
| `package_drift` | `package_drift` (configuration package pin changed) |
| `owner_recovery_required` | `owner_journal` |
| `schema_drift` | `schema_drift` (missing required table) |
| `BlockingIOError` | `busy_owner_and_admission_locks` (each held lock) |

The existing configuration/private-file/SQLite/coordinator/transport guards still
apply unchanged in addition to these cases. They retain their own named errors:
`invalid_config`, `config_drift`, `journal_mode`, `coordinator_recovery_required`,
`missing_worktree`, `unsafe_existing_database`, `symlink_path`, `unsafe_file`,
`unsafe_runs_directory`, `lock_identity_changed`, `invalid_file`,
`unsafe_private_file`, `file_limit`, `duplicate_json_key`, `invalid_json_number`,
`invalid_json`; transport validation adds `invalid_tmux_config`,
`absolute_transport_path_required`, `binary_drift`, `tmux_missing`,
`invalid_auth_link`. Coordinator validation can additionally return its existing
`Rejected` reasons. Filesystem/SQLite failures return their exception class name,
not paths or file contents. CLI misuse is argparse exit 2; runtime refusal is JSON
`{"state":"REFUSED","reason":"..." }` and exit 2. The generic scheduled-tick HOLD
behavior is preserved. Kernel authority checks now share the precise reason codes;
the previous wrong-scope expectation was updated accordingly.

## Transport and manual-dispatch coexistence

`tmux-workers` enables real prepared worker actions: claim, allocate a private
run/socket, launch the pinned binary, send the assignment, verify completed ACK,
send START, collect correlated RETURN and update the coordinator. It can spend
inference capacity and edit allocated worktrees with the recorded `--yolo`
policy (danger-full-access / approval never). It does not launch the management
model merely because `manager_enabled` is true; that routing remains deferred.
Presence of transport chooses this worker adapter, independent of that flag.

Configuration must carry exactly `kind: "tmux"`, absolute `binary`,
`binary_sha256`, absolute `tmux`, private `runs_dir`, and absolute `auth_link`
whose filename is auth.json. The binary must match its digest; the TMUX executable
must exist; the run directory must pass UID/mode/no-link checks. The transport
validator does not hash TMUX or authenticate the auth-link target. Legitimate
live use also requires the operator's approved inference/profile authority,
reviewed allocations/worktrees/runtime/policy and actual packaged transport proof.
Arming itself never creates the auth link or reads its target. Synthetic transport
tests used /bin/echo and a nonexistent fixture auth target.

**Do not treat live transport arming as safe while hand dispatch shares the same
work.** A common coordinator atomically arbitrates claims/resources, so a compliant
manual claim and daemon claim cannot both win the same prepared action. However:

- A hand-claimed active action has no owner operation receipt. The daemon refuses
  adoption with `unowned_claim` and records an operation HOLD. Its watchdog can
  also report overdue manual claims and mark a still-dispatching claim uncertain.
- If a person sends prompt/START/quit to a daemon-owned pane, the manual path does
  not acquire the daemon locks or update its delivery receipts. The daemon can
  send its own keys, encounter unexpected rollout turns, or find the worker gone.
  Locks/unique sockets do not establish exclusive supervision against manual keys.
- Hand launch outside the common coordinator, or through a separate coordinator,
  has no shared resource arbitration and can create two workers editing the same
  worktree. Separate TMUX sockets prevent accidental socket-name reuse, not this
  duplicate work.

Use an explicit supervised handoff/drain with reconciled claims before enabling
transport against shared work. Fixture-only avoids worker launches, but still
runs the existing fixture-event/watchdog path and thus is not a read-only mode
for a live coordinator. No live coordinator/profile inspection was attempted.

## Evidence and limits

Read docs/development/test-isolation.md before testing. Venv created under
`env -i` at `/private/tmp/owner-arming-65.Fcrobu/venv`; only requirements.txt
packages installed: markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1.
Runs used empty inherited environment, disposable HOME and all three profile
aliases, TMPDIR=/private/tmp, PYTHONDONTWRITEBYTECODE=1, checkout-only PYTHONPATH
and the venv/bin plus system/Homebrew PATH. No native credential prompt occurred.

- Initial tests-before-implementation: 26 tests, 6 failures and 25 errors (subtests
  count separately), exit 1. Raw: owner-arming-65-red.txt.
- First implementation run: 26 tests, 1 failure and 2 errors, exit 1. Two fixture
  defects: string passed to a Path digest helper; SQL snapshot recovered the
  deliberately bad journal before testing its refusal. Raw:
  owner-arming-65-green-first.txt. Corrected without weakening expectations.
- Corrected refusal cases against the frozen base module: 20 tests, 21 errors,
  exit 1. All fail because the public arm/disarm APIs are absent; the extra error
  is the second lock subtest. Raw: owner-arming-65-baseline-refusals.txt.
- First owner regression: 97 tests, 1 failure, exit 1: existing wrong-scope test
  expected activation_authority_required instead of the new precise
  activation_scope_mismatch. Raw: owner-arming-65-owner.txt.
- Final owner regression: **98 tests passed, zero failures/errors/skips**, exit 0,
  9.782s. Raw: owner-arming-65-owner-final.txt. Includes 26 new arming tests and
  one new real-kernel/scheduled-tick recovery test (launchctl is mocked).
  Five actual child-process exits cover before/after intent publication,
  before/after activation-file publication, and after SQL commit. A partial
  pending-file recovery case, CLI arm/status/disarm, transport-no-side-effects,
  stale-generation rejection after revoke, and arm/tick/disarm/rearm are included.
- Full required suite: **780 tests passed, zero failures/errors/skips**, exit 0,
  435.883s. Command from the repository root in the isolated environment above:
  `python -m unittest discover -s scripts/initiative_control -p 'test_*.py'`.
  Raw: owner-arming-65-suite.txt. Failure names: none, including the flagged
  load-sensitive ManagerTests/RealTmux/owner_tmux cases. ResourceWarnings and
  printed ERROR/TimeoutExpired JSON are fixture output, not unittest failures.

Raw transcripts named above live alongside this record as six untracked .txt
artifacts (1,008 lines total) for inclusion in the manager's evidence bundle.
Earlier attempts are retained separately. These are implementation tests, not an
independent functional acceptance handoff or a release qualification. The manager
owns independent review, code-blind design/execution/evidence acceptance and shared
ledger updates. No new independent review or human sign-off is asserted.

Before live arming: qualify the exact deployed runtime and its new package/config/
state pins, reconcile supervisor ownership and outstanding claims, perform required
independent functional/transport evidence, and record operator authority. Updating
owner_daemon.py changes package_digest; existing installation/runtime/config/state
pins cannot silently be reused. Migration/reinstallation is separate manager work.
The external-volume launchd failure described in the brief was not investigated
or re-qualified; local storage behavior and the live firing count were not verified.

One correction to the brief's operating interpretation: after its first refusal,
the schedule latches HOLD; later firings only record skipped probes, not fresh
kernel admissions. Arming alone cannot clear that HOLD. After verified arming,
the operator separately uses the existing command:
`python owner_daemon.py --schedule S --recover "actual inspection evidence"`.
This does not clear per-action holds. No claim is made that fixture-only arming
does useful worker work or qualifies transport. No live arming, live store access,
live coordinator mutation, inference, push, release or broad formatter occurred.

## Final tree identity and scope

Source changes: owner_daemon.py +149/-9; test_owner_daemon.py +218/-1.
Total code/test changed lines: 377 (367 additions, 10 deletions).
activate.py is unchanged. All seven evidence artifacts are inside the allocated
management-bootstrap directory. No out-of-scope changes, commit or push.
`git diff --check` passes. Final source SHA-256 values:

- owner_daemon.py: `bccb100955af7ff4d260a6f78b11c67f7bcf85d37ba4c3eb8ed9a4ecfb6088bd`
- test_owner_daemon.py: `9566d8b4485e5dc8c7871538f7186e0972886c362ada97490c7c93a8a40ab2dd`
