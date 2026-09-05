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

- [ ] Coordinator suite registration and bounded manual nextest profile.
- [ ] RTX scoped fix/fullfmt, new support tests and existing TMUX regressions.
- [ ] Actual-key rehearsal: startup, fake-provider switch, pending exit/restart.
- [ ] Cancel and timeout remove owned home/socket and leave no server.
- [ ] Pin test artifact and immutable product binary; runner operates without
      shared build lock during operator waits.
- [ ] Verify final diff contains only allocated tests/QA and root registration.
- [ ] Review #3 Astra High and #4 Fable5.1High via Corbanu/private TMUX, reusing
      existing memory budget 2/5. No review invoked for this support yet.
- [ ] Publish tested operator commands and exact evidence.

## Checkpoints

`0ca14bb8d`: first synthetic fixture, owned attachment and cleanup assertion.
Not qualified; no tests/build/review have run at this checkpoint.
