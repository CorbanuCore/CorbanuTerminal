# RETURN — owner-headless-59

Allocation digest: `4b2c6654bd2dea13328198154626087fec3b00f3cecc5d4228e63fcea2f4e789`.
Claim: `1e713057-dca5-4760-bd56-925bc623120b`.
Runtime: gpt-6-astra, high. START received before reading or changing files.
Base: `a4a4c9302cab5d676cee0b8ca9d6043a7b33da83`.
Brief SHA-256 verified:
`61662bd9f6336590575a387470276d3b149575b4ca18cbab4f6e4535083f892a`.

Bounded correction under **Internal delivery control — TO BUILD**, requirement
excerpt “durable event dispatch, acknowledgments and watchdog”.
Existing initiative-delivery-control / PF-80-S01 context is manager-owned.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`;
branch: `bootstrap/owner-recurrence-20260917`.
No authorization, credential, financial or disclosure boundary is expanded.
This is an internal installer revision; no TUI change or independent functional
acceptance is claimed. The manager retains the later recurring-owner live
acceptance gate; this record does not waive it. No release or push.

## Correction

`owner_daemon.service` now recognizes exit 125 plus the exact recorded line
`Could not print domain: 125: Domain does not support specified action` only
for a GUI descriptor. It returns `domain_absent` with the diagnostic, leaving
activation to permit that observation only for the sibling.
The existing exit-112 exact domain/UID diagnostic remains supported.
Exit 113 with the queried label's missing-service diagnostic remains `absent`.

`activate.owner_activation` uses one `is_clear` predicate for install conflict
checks and post-bootout checks. It explicitly admits `domain_absent` only for
the sibling. The common observation function rejects a missing selected domain
both initially and after bootout with `service_observation_unavailable`.
The latter leaves the plist and receipt intact; bootout may already have occurred.

Strict decode errors are converted from `UnicodeError` to
`LaunchError(service_observation_unavailable: <domain>/<label>)`.
Invalid labels still retain their own validation error; unknown diagnostics,
permission failures and timeouts are not absence evidence.

## State table

All “permit” entries remain subject to the existing receipt, pins, ownership,
configuration and phase checks; they are not independent activation authority.

| Observation | Fresh install | Uninstall |
| --- | --- | --- |
| `absent` | Permits selected or sibling | Permits; receipt required |
| Service absent: exit 113 plus the exact queried label | Same `absent` state, not domain absence | Same `absent` state |
| `domain_absent`: supported exit 112; GUI-only exact exit 125 | Permits sibling only; selected refuses | Permits sibling only; selected refuses, including after bootout |
| Present/conflicting label | Refuses either domain | Foreign path or digest refuses before bootout |
| Present/owned receipted instance | Selected idempotent reinstall only; loaded sibling still conflicts | May boot out owned instances in either domain, then must observe clearance |
| Permission denied or unrecognized output | Refuses | Refuses |
| Timeout or undecodable output | Refuses | Refuses |

## Regression evidence

The two transcript constants retain the manager's stderr strings verbatim,
including `Bad request.` and the UID-503 missing-service line.
The raw recorded label `com.corbanu.absent` is outside the owner-label namespace.
The test first verifies that it cannot prove absence for another queried label,
then substitutes only the label for a valid owner-service absence check.
No label validation was weakened.

Four newly added tests were run against the unchanged base production code:
4 run, 0 passed, 1 failure, 3 errors, exit 1, 0.149s.
All names are in `test_owner_daemon.RecurrenceTests`:

- ERROR: `test_headless_recorded_transcript_distinguishes_gui_domain_and_user_service`
- ERROR: `test_headless_gui_sibling_allows_install_reinstall_and_uninstall`
- FAIL: `test_selected_domain_vanishes_after_bootout_preserves_installation`
- ERROR: `test_undecodable_service_output_refuses_install_and_uninstall`

The first two errors are the old matcher's `service_observation_unavailable`
on the exact exit-125 GUI diagnostic. The third receives `service_still_present`
instead of observation-unavailable. The fourth leaks `UnicodeDecodeError`.
The final invalid-byte fixture uses `bytes([255])`; the initial red run used
a literal backslash-x byte sequence in the synthetic exception. Both inject the
same exception type; the original failed attempt is retained, not overwritten.

Final owner run: 71 tests, 71 passed, 0 failures/errors/skips, exit 0, 9.157s.
This includes install/reinstall/uninstall with the exit-125 GUI sibling,
receipt diagnostic retention, selected-domain disappearance, decode refusal,
and wrong-code/wrong-descriptor/modified-line/permission-denied rejection.

Full isolated suite: 753 tests, 751 passed, 2 failures, 0 errors/skips,
exit 1, 447.923s. Exact failures:

- `test_fable_launcher.RealTmux.test_auth_failure_inactive_provider_and_stale_screen`:
  receipt error was `launcher_failure`, expected `timeout`; shutdown reported
  clean with launched=false.
- `test_owner_tmux.TmuxTests.test_bridge_refuses_when_too_little_budget_remains_to_collect`:
  fixture timed out waiting for READY, with a blank captured pane.

The three `test_decision_manager.ManagerTests` tmux tests named in the brief
all passed. The two observed failures are in unchanged files; causation by load
or pre-existence at the base is not established by this run. No out-of-scope
fix was attempted. A separate isolated replay is recorded below; it does not
replace or turn the failed full-suite run into a pass.

Isolated replay: 2 tests, 1 passed, 1 failure, 0 errors/skips, exit 1, 21.747s.
The launcher test passed. The same owner-tmux READY/blank-pane timeout persisted
in `test_owner_tmux.TmuxTests.test_bridge_refuses_when_too_little_budget_remains_to_collect`.
This remains unresolved; there was no further retry or out-of-scope fix.
Test invocations with nonzero exit: 3 (intentional pre-fix regression run,
full suite, isolated replay). Final owner-only invocation exited zero.


## Environment and artifacts

Environment E: `/private/tmp/owner-headless-59-env.MORzzW`.
From the repository root, created a disposable venv with
`env -i HOME=E PATH=/opt/homebrew/bin:/usr/bin:/bin /opt/homebrew/bin/python3 -m venv E/venv`.
Installed only `scripts/initiative_control/requirements.txt` with venv Python
and pip `--isolated --disable-pip-version-check --no-input` under `env -i`.
Pinned dependencies: markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1.

Test commands use `env -i`, HOME and all three profile aliases set to E,
checkout-only PYTHONPATH, PYTHONDONTWRITEBYTECODE=1, TMPDIR=/private/tmp,
and venv/Homebrew/system PATH. No operator profile or credential read.
The environment also set `CODEX_TEST_DISABLE_NATIVE_KEYRING=1`; this is not
the repository's Rust guard variable and is not claimed as an enforced native
credential boundary. These are Python synthetic-fixture tests, not Rust/native
credential qualification. No native prompt occurred.

Commands:
`E/venv/bin/python -B -m unittest -v test_owner_daemon`;
`E/venv/bin/python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'`.
Logs: E/pip.log, E/pre-fix.log, E/owner-tests.log, E/suite.log,
E/isolated-replay.log. The replay targets exactly the two full-suite failures.
Read-only native evidence: E/host-probes.json.

## This machine and activation verdict

Read-only probes found macOS 26.6.2, UID 501, managername Background, both
gui/501 and user/501 readable (exit 0), and a fresh random absent owner label
returning exit 113 in each domain. Domain stdout was discarded without being
printed; no live job, profile or credential was inspected. The manager's
macOS-26.2 UID-503 transcript is supplied evidence, not a native trial performed
by this worker. Background managername alone does not establish GUI absence.

Yes: I consider the corrected guard safe for the manager to use for a controlled
migration to user/Background on this machine, following the reviewed procedure:
receipted GUI uninstall, verify clearance, retain history, then install a fresh
private user-domain root with reviewed runtime/config pins. This is a judgment
about safely attempting the migration, not a claim that it has succeeded. The
current live-label ownership/receipt and pins were not inspected; successful
live bootstrap, kickstart, recurring owner work, publication and restart/login
recovery remain unproven by this revision. The full-suite run is not clean;
this verdict does not claim overall qualification. The exact exit-125 lifecycle is covered
by the supplied transcript and deterministic tests, not a native no-GUI lifecycle
on this machine, which has both domains. No activation was attempted.

## Brief corrections and limits

The reported P2 and both P3 findings are valid. “Every refusal precedes any
write” is too broad: the installer can create installation.lock before observing
domains, and post-bootout refusal necessarily follows attempted job removal.
The correction preserves those semantics and protects the receipt/plist on
post-bootout observation failure. The historical owner-ownership-56 record
already limits its GUI-absence probe to unused UID 2147483646; it did not prove
a current-user session-less install. No further factual disagreement established.

## Final-tree hashes and scope

- activate.py: `7154a9543ebcada98728f283a3c9f89fc1c4c4ce28addb8b999b7fb88d15969a`
- owner_daemon.py: `9bbf5eefc4c384fd015106e2f1d813c8bfcb3c5919f823cb4210b99dc868e3af`
- test_owner_daemon.py: `078cf4809ff3aeb9109d5060d67ea6c17065c66c19b538e460ddecfae4cb53c5`
- E/pre-fix.log: `d1b78aa61898fea18eda569f14c044d533cf0e92fac021beeb28dfd2c468111d`
- E/owner-tests.log: `4c7263b11cc47fae50ac715ff72fa7bf82a29c59fe8cb823f9f99bf5de97c537`
- E/host-probes.json: `b60e3ebf4515105f1034d7dfeb87a9ea8cdbf5c68beda56fec97ad220efbd727`
- E/suite.log: `728f5bb893f8f9ab7f8cebdda40f12088b0858c297868546665ba43a27444ba3`
- E/isolated-replay.log: `6e7e17c4329a38276ed4e81f97dcb704e21b39a10955e574aec74a7520908c4d`

Code/test changed lines (additions + deletions): activate.py 8+5=13;
owner_daemon.py 10+2=12; test_owner_daemon.py 105+1=106.
Production subtotal 25, test subtotal 106, code/test total 131.
Evidence record: 179 added lines; total across all four changed files: 310 lines.
No workspace formatter, Rust tests, native credential prompt, activation or push.
Only the three assigned Python files and this assigned evidence record changed.
