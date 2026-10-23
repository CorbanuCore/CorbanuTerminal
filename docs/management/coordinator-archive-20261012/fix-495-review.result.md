I found no real problem. The diagnosis holds, and the test change is a sound fix. I only read the code; I didn't run the tests here because the sandbox is read-only.

**1. Root cause: correct, and no credential is sent after shutdown.**
- **Shutdown order (`server.rs`, end of `run_broker`):** after a stop signal, the broker calls `broker.revoke()`, then `accept.abort()`, then `drop(endpoint)`, which removes the socket directory. Only then does `block_on` return and `std::process::exit` run. So the process can briefly outlive its socket directory.
- **Route check (`credential_broker.rs:529`):** the client's `is_alive()` relies on `try_wait`, so it returns true until the process has actually exited. A route made in that gap returns `Brokered`, as the worker described.
- **Revoke happens before the directory disappears.** `revoke()` does three things under the state lock: bumps the run generation, clears `credentials`, `env_stash` and `replay`, and wakes in-flight requests (`send_replace`). `authorize()` holds the same lock and checks the generation (`StaleGeneration`) and then the credential (`UnknownCredential`). So once revoke has run, no request can pick up a credential.
- **Connections already accepted.** Their `serve_connection` tasks keep running after `accept.abort()`, but they go through `authorize`, so they are refused. A request authorized just before the revoke is either refused at `borrow_and_update` or cut off by `generation.changed()` in the upstream `select!` and the revocable response body. That is the existing, intended behaviour for requests already in progress.
- **Forwarding in the gap.** The client opens a new Unix-socket connection for each request. The socket path is already unlinked, so `forward` fails with `Unavailable`. Even if it connected, the broker holds no credentials any more.
- **No relaunch.** `live_client` doesn't replace a dead broker, `fail()` never touches `mode.client`, and the crash test (around line 697) already checks this.
- **Early `?` error exits** (control read/write error inside the loop) skip the revoke and the abort. But they come from a broken control channel, not a stop signal. `endpoint` is still dropped on return and the process exits right away. Not relevant to #495.
- **Before the broker handles the signal**, requests are served normally, so a request racing the signal itself can still get a credential. That is inherent and not something this test claims to cover.

**2. The test change doesn't loosen anything and is a correct synchronisation point.**
- **What changed:** the fixed 200 ms sleep only assumed the process would exit within 200 ms; it wasn't a security property. The real requirement is "once the process has exited, routing fails". The new code waits for exactly that, within the same 10 s deadline, asserts it happened, and keeps the strict route assertion.
- **The new no-replacement checks are good.** `current_isolated_client()` returns the raw stored client without filtering on liveness, so `Arc::ptr_eq` really does show the client wasn't replaced. `pid_for_test()` stays `Some(pid)` because `is_alive` reaps the process with `try_wait` but doesn't remove the stored child.
- **One small coverage shift:** the test's own `is_alive()` poll now does the exit detection (it flips `alive` and cleans up), so `route` sees the cached flag rather than calling `try_wait` itself. That code path is still covered by the crash test, where `kill_for_test` is followed by `route`. Acceptable.
- **New flake risk is negligible.** It would need more than 10 s between SIGTERM and process exit. Windows isn't affected because the test is `cfg(unix)` and the change touches only the test.

**3. Other notes (none blocking):**
- `assert_eq!(current.pid_for_test(), Some(pid))` repeats what `ptr_eq` already proves. It would break for no good reason if exit detection ever started removing the stored child. Optional to drop it.
- Optional follow-up test: in the gap where the directory is gone but the process is alive, assert that `forward` fails and the upstream never receives the token. That would turn the "fails closed" reasoning above into a tested guarantee.
- Optional hardening: call `broker.revoke()` on the early-error exits too, for example with a drop guard, so every exit path revokes. This existed before this change.
- Pre-existing, out of scope: on Unix, the data-socket client doesn't verify the peer the way the Windows squat check does. Only signed references cross that socket, never raw credentials.

VERDICT: APPROVE