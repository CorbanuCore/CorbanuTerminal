I merged PR #500 (`a564900b`), which fixes #495, and the issue is closed. It was a race in the test, not a product bug, so only the test changed and no security sign-off is needed.

**Can a route succeed against a killed broker?** No credential can be injected once the broker has removed its socket directory. When it gets SIGTERM, the broker first revokes every reference. Revoking clears its stored credentials and env stash and bumps the run generation. Requests already in flight are then refused or cut off. Next it stops accepting connections, then removes its socket directory, and only then exits.

So the process can outlive its socket directory for a few hundred ms on a loaded machine. During that gap a route can still come back as `Brokered`, but that's only an authorization decision. Actually forwarding the request opens a new connection to the socket that's gone, so it fails as unavailable. Nothing relaunches a dead broker.

A diagnostic run showed the broker process still alive (state S or R, not zombie) in every failure, and it exited roughly 100–300 ms after the old 200 ms sleep.

**Fix:** the fixed 200 ms sleep is replaced with a wait of up to 10 s for the client to see the broker exit. The test keeps the strict `IsolatedBrokerUnavailable` assertion. It also now checks that the broker wasn't replaced (same client, same pid).

**Repro (Mac, looping the single test):**

| | idle | under load (48 busy loops, 24 cores) |
|---|---|---|
| before | 200/200 pass | 167/200 pass (33 fail) |
| after | 200/200 pass | 200/200 pass |

I also ran the whole `credential_broker` suite 3 times under the same load: 58/58 passed each time.

**Gate evidence:**
- **Tests:** `just test -p codex-network-proxy` passed 321/321.
- **Linux (glitch):** the `credential_broker` tests passed 58/58, and looping the test 100 times passed 100/100.
- **Clippy:** `-D warnings` is clean on macOS and on Linux (glitch, `-j10`). Both show one warning from the existing `clippy.toml` (`log::set_boxed_logger` entry), which isn't from this change.
- **Review:** Opus 5.5 High via `corbanu exec` (read-only) gave APPROVE on the first round. It agreed with the diagnosis and found nothing blocking. The full review is in `.codex-work/workers-20261002/fix-495-review.result.md`.
- **CI:** all 31 checks passed (9 skipped), including the macOS and Windows credential canaries. I merged with `--match-head-commit 9385585c`.
- **Cleanup:** I removed the Mac target dir, the worktree, the temp binaries and my glitch checkout and target. The local branch `fix-495-broker-termination` is still there: `git branch -d` refused, and I didn't force-delete it.

**Open (optional, not blocking), from the reviewer:**
- A test for that exit gap, asserting that forwarding fails and the upstream never receives the token.
- Make the early-error exits in `run_broker` revoke too. They currently skip it, but they come from a broken control channel, not from SIGTERM.
- The `pid_for_test` assertion repeats what the same-client check already proves. I kept it.