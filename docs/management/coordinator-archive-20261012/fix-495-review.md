You are an independent, read-only security reviewer (do not modify files). Review branch `fix-495-broker-termination`
in this worktree (commit 9385585c; diff against origin/main: `git diff origin/main...HEAD`). It fixes issue #495: the macOS
credential canary intermittently failed `credential_broker::isolated::tests::pf_27_s04_pf_27_s01_externally_terminated_broker_is_detected_and_cleaned_up`
(codex-rs/network-proxy/src/credential_broker/isolated_tests.rs): after SIGTERM to the isolated credential broker and its
socket directory disappearing, a route made after a fixed 200 ms sleep returned Ok (Brokered) instead of
Err(IsolatedBrokerUnavailable).

Worker's diagnosis (verify it, do not trust it):
- Server shutdown order (network-proxy/src/credential_broker/isolated/server.rs, end of run_broker, and run_credential_broker_main_with):
  on SIGTERM the control loop breaks, `broker.revoke()` bumps the run generation and clears credentials/env stash/replay
  and wakes in-flight requests, `accept.abort()`, `drop(endpoint)` removes the private socket dir, then block_on returns and
  `std::process::exit`. So the process can outlive its socket directory.
- Client `is_alive()` (isolated/client.rs) uses `child.try_wait()`; `route_request_credentials` (credential_broker.rs ~529)
  returns Brokered while the pid has not exited. A Brokered route only authorizes; `forward` connects to the unlinked socket
  path and fails closed (Unavailable), and the broker holds no credentials after revoke. No relaunch path exists
  (`live_client`: "A dead broker is not replaced").
- Repro on the Mac: test loop, 200 iterations idle 200/200 pass; under 48 busy loops on 24 cores 167/200 pass (33 fail).
  Diagnostic (`ps -o stat=` before route) showed the broker pid in state S/R (alive, not zombie) in every failure; it
  exited within ~100-300 ms after the sleep.
- Fix (test only): replace the fixed sleep with a bounded wait (same 10 s deadline) for `client.is_alive()` to become false,
  assert it did, keep the strict route assertion, and additionally assert the broker client was not replaced (Arc::ptr_eq,
  same pid). After: 200/200 idle, 200/200 under the same load.

Questions to answer:
1. Is the root cause right? Can any route/forward inject a real credential after the broker received SIGTERM and removed its
   socket dir (stale client connection, accepted-but-unprocessed request, relaunch, socket removed before exit, early
   `?` error paths in run_broker)? If yes, this must be a product fix instead.
2. Does the test change loosen what is asserted, or is it a sound synchronization point? Any new flake risk (e.g. Windows,
   which uses named pipes; the test is cfg(unix))?
3. Anything else wrong with the diff.

End with exactly one line: `VERDICT: APPROVE` or `VERDICT: REQUEST_CHANGES` followed by the required changes.
