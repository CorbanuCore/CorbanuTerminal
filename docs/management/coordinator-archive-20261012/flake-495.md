You are a bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux
work on glitch, `ssh glitch`, max -j10), then issue #495 and its comments (`gh issue view 495 --comments`).

The macOS credential canary intermittently fails
`credential_broker::isolated::tests::pf_27_s04_pf_27_s01_externally_terminated_broker_is_detected_and_cleaned_up`
(codex-rs/network-proxy/src/credential_broker/isolated_tests.rs ~826-850): after SIGTERM to the broker and its socket
directory disappearing, a route made 200 ms later SUCCEEDS instead of returning IsolatedBrokerUnavailable. It hit 2 of ~6
macOS runs today (#489, #490). Pull the failing job logs (`gh api repos/CorbanuCore/CorbanuTerminal/actions/jobs/<id>/logs`)
and the uploaded failure artifact if present.

First determine the real cause, because this is the credential broker: can a route ever succeed against a broker that was
externally terminated (e.g. a stale client connection, an accepted-but-unprocessed request, a silent relaunch, socket
removed before exit)? If product behaviour can inject credentials after the broker is killed, fix the product (fail closed
once the broker's death is observable) and keep the test strict. If it is purely a test race, replace the fixed sleep with a
real synchronization point (wait, bounded, for the pid to exit / the client to observe death) without loosening what is
asserted. Reproduce locally on the Mac by looping the test (e.g. 200 iterations, also under CPU load) before and after;
report the counts.

Gate: focused network-proxy tests; macOS clippy and Linux clippy -D warnings on glitch; one independent Opus 5.5 High
read-only review via installed `corbanu exec` (iterate to APPROVE). PR off origin/main in a new worktree under
/Volumes/CorbanuDrive/Corbanu/worktrees/, "Fixes #495". Merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f; never pkill broadly). If the fix changes
product behaviour of the broker (not just the test), do NOT merge: title it "— needs security sign-off" and stop. Remove
build targets when done. Report as in sec-common.md.
