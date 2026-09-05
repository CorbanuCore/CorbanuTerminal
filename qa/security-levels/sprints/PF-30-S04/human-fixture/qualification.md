# Human memory fixture qualification — in progress

Routine QA allocation `9ceac3333`, base `b64d7390d`; branch
`test/security-human-memory-fixture`. This does not reopen archived PF-30-S04 or
change the Corbanu product binary. No human acceptance claimed.

## Contract

Ignored manual test requires explicit opt-in, selected case, new evidence
directory, and an expected immutable candidate hash. Each case creates a fresh
synthetic home, SQLite source and two loopback fake providers. Inherited
environment/credentials are cleared for the child. No real key, private history,
native keychain or arbitrary existing TMUX session is used.

The test runner owns the providers/home/TMUX session for at most ten minutes;
manual pending responses have a 120-second window. The existing cleanup watchdog
is unchanged. The operator attaches to the owned session, sends real keys, and
can cancel via the fixture's own `cancel` marker. Pending-exit relaunch waits for
explicit `restart`. Results report source-specific DB output and endpoint/model
routing; ordinary chat is not proof of memory execution.

Automated rehearsal uses the same state machine, shortened synthetic delay,
actual `/model` keys, success/owner-exit/restart and cancel/timeout cleanup.
Rehearsal is not named-human sign-off. No session is to be left running until
the coordinator deliberately arranges the human package.

## Required final gates

- [x] Coordinator suite registration and bounded manual nextest profile.
- [x] RTX scoped fix/fullfmt, new support tests and existing TMUX regressions.
- [x] Actual-key rehearsal: startup, fake-provider switch, pending exit/restart.
- [x] Cancel and timeout remove owned home/socket and leave no server.
- [x] Pin test artifact and immutable product binary; runner operates without
      shared build lock during operator waits.
- [x] Verify final diff contains only allocated tests/QA and root registration.
- [ ] Review #3 Astra High and #4 Fable5.1High via Corbanu/private TMUX, reusing
      the original memory budget. Review3 findings are remediated; review4 pending.
- [ ] Publish tested operator commands and exact evidence.

## Checkpoints

`0ca14bb8d`: first synthetic fixture, owned attachment and cleanup assertion.
Not qualified; no tests/build/review have run at this checkpoint.

`69bd039d3` plus final formatter: registered fixture compiled after correcting
one OsStr reference comparison in the attachment helper. Nine existing/new
TMUX support tests passed (including abnormal child termination and owned socket
cleanup). First full rehearsal completed startup but stopped at provider switch:
a fresh custom provider B had no selectable model preset. Source inspection
shows custom presets sync on provider-manager access; this is not a claim of a
new product regression or authorization to modify product UX.

`d70972fd3` attempted to redirect established OpenAI/Z.ai catalog identities.
The fixture exited before readiness (rehearsal failed; support tests 9/9 passed).
Source inspection confirms built-in overrides accept retry/timeouts, not the
endpoint/auth replacement this test requires. That attempt does not prove
loopback routing or a usable operator flow. It has been replaced in `516ce8142`
with custom loopback providers and explicit `/providers` catalog initialization
before the real `/model` journey. Failed rehearsal status/captures persist before
propagating the test error. Final qualification is still pending.

## Pre-review source qualification (superseded below)

Runtime `77200387f` (test-only source): RTX source `a86de72bd` plus exact
formatter delta, verified identical to the committed tree. Product remains
`6a6bb029d` / codex0.1.38, not latest combined PF20 runtime; integration owner
will requalify the combined candidate separately.

Scoped `just fix -p codex-tui`, complete `just fmt`, then
`just test -p codex-tui --test all -E 'test(memory_human_fixture_rehearsal) | test(support::tmux::tests)' --retries 0 --test-threads 1`:
**10/10 passed**, run `82424229-36d7-49cc-859e-8de3e4d3b49b`,69.248seconds.
The five-case actual-key fixture completed in65.708seconds. Supporting cleanup
tests include abnormal child termination/watchdog cleanup. No local builds.

Results: startup A request/output1/1; explicit `/providers` replacement then
`/model` distinct synthetic B model/effort produced a foreground request at
B/`memory-fixture-b`, old memory provider-change denial, source output0;
pending owner exit/restart remained1/0; cancel/timeout removed home and socket.
Previous monolithic source also passed10/10 at
`00600edd-c1d6-43f3-810a-1358f6c44ea4`; final evidence is the extracted tree above.

RTX evidence root `/home/travis/security-round5/evidence/human-memory/`:
`final.log`, `rehearsal-final/{Startup,ProviderSwitch,PendingExit,Cancel,Timeout}`,
and `manual-profile.json` (human-memory profile parses and lists ignored entry).
No reviewer or named human has accepted this helper yet.

Frozen test runner copied under shared build lock after checking it was not
newer than the just-produced nextest manifest:
`candidate-77200387f/all`, SHA256
`8d0cd04d817b37b57db1d4641401239ef1b40b2d54a40dc85ed55944e0274ccd`.
The product binary is
`/home/travis/security-round5/evidence/integration/6a6bb029d/candidate/codex`, SHA256
`90d6a1f7f72c5397ff858583c038b2615c8fb034f57a890d6595d6b98afccd4f`.
Final main fixture519lines (including entries/rehearsal), support118lines;
small TMUX accessor stage plus separate111line pinned-entry driver. Review
stages and scope limits are explicit in review-scope.md.

The frozen runner's ignored manual entry was exercised outside `build.lock`
using `rehearse-pinned.py`: startup completed with actual keys, and a separate
manual cancellation returned the expected nonzero/not-complete outcome. Both
removed their exact disposable home and owned socket. Evidence:
`pinned-entry-77200387f.log` and `pinned-entry-77200387f/{startup,cancel}`.
`finished.json` reports `Ok(Complete)` / `Ok(Cancelled)` respectively, with
`human_acceptance:false`. No fixture process was left for an operator.

## Independent review ledger

Original memory reviews1–2 remain counted. This support reuses, not resets, that
budget. Review3 is now invoked against the frozen branch diff from `b64d7390d`:
bundled CodexCLI `/Applications/ChatGPT.app/Contents/Resources/codex`, requested
`gpt-6-astra` with High effort, structured autoreview helper, branch mode and
review-scope.md. Result recorded below; no nested reviewer. Review4 Fable5.1High remains
conditional on disposition of review3. At most5 total absent continuing critical
issues; the limit is not a target.

Review3 completed with three accepted in-scope P2 findings: unpinned source
catalog; cancellation rehearsal that accepted unrelated nonzero failures; and
pending-exit counts without an enforced pending deadline/failure check. Original
structured result/text are preserved under review-3-astra.*. No product
authorization boundary changed. Remediation checkpoint `36c98ca93` is WIP:
bundled catalog API; exact final cancellation outcome validation and negative
Python regression; first observed exit proof before the delayed response
deadline, with a conservative polling margin and source failure rejection.
The new pending-proof test rejects missing/expired/failed/completed evidence.
Final affected tests and pinned runner were regenerated as recorded below.

## Remediated final source qualification

Test-only runtime `673c378b6` equals RTX `a4ac1f072` plus the exact formatter
delta. Scoped fix/fullfmt preceded the final tests. Python cancellation-outcome
regressions passed2/2; nextest selected `test(suite::memory_human_fixture) |
test(support::tmux::tests)` passed11/11, run
`e12c8b9a-bff1-4e1f-ab49-193aab81b10f`,69.144seconds. The five actual-key
scenarios passed, including provider replacement, cancellation/timeout cleanup
and restart. Pending owner exit was proven at520ms against the30-second delayed
response window, no source output/failure; `pending-exit.json` records the proof.
The manual nextest profile parses and lists the ignored opt-in entry.

RTX artifacts: `remediated.log`, `remediated-profile.json`,
`rehearsal-remediated/{Startup,ProviderSwitch,PendingExit,Cancel,Timeout}` under
the evidence root above. New runner `candidate-673c378b6/all` was copied while
holding the shared build lock, after a nextest-manifest mtime comparison. SHA256:
`dabbac2b5c54386424183f9fce9ee6e7df807f4ecaa3cc2ae3eebb82cb4d7ad1`.
Product candidate/hash is unchanged. The regenerated frozen manual-entry
rehearsal passed outside the build lock: exact `Ok(Complete)` startup and
`Ok(Cancelled)` cancellation outcomes, owned home/socket removed. Evidence:
`pinned-entry-673c378b6.log` and `pinned-entry-673c378b6/{startup,cancel}`.
This supersedes the old
runner for human testing, without erasing the original failed review evidence.

Review4 is allocated and now being invoked against this frozen runtime using
the structured helper in a private TMUX, approved Corbanu `review-fable-high`
wrapper, requested `claude-fable-5-1-plan` / High. This is Corbanu exec under
TMUX, not a claim of an interactive TUI model review. No nested reviewer;
four of the original maximum five memory-track invocations consumed. Result
pending; human acceptance remains unperformed.
