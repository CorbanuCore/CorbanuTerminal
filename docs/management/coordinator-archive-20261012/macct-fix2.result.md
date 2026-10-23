I fixed #425, #426 and #427 in **PR #431**. It merged at `2f95b4fc9a` after every check passed, and all three issues are closed with evidence comments. I didn't touch #428.

**What changed**

| Issue | Change |
|---|---|
| #425 | `--account <other-provider>:<name>` is now checked when the session starts. A missing account is refused with the same "not configured … `corbanu account add …`" text as the session-provider case. |
| #426 | The TUI checks the selected accounts before onboarding and exits with that text, including on `resume`. It no longer loops into setting up a default key. |
| #427 | Resuming a thread recorded on a named account now returns that account (new experimental `providerAccount` field). The TUI uses it, so a 401 names the right account. A resume with `named_accounts` off now says the thread's recorded account needs the feature and suggests `--account zai:default`; it no longer blames `--account`. |

**What a prefix for another provider means now** (written up in `--help` and a new `docs/provider-accounts.md`):
- It picks that provider's account for anything in the session that runs on it: spawned agents, TUI workers, and a TUI switch to that provider.
- `corbanu exec` can't switch providers, so it refuses such an account when no spawned agent could run there. That covers `agents.enabled = false` and an `agents.provider_allowlist` that leaves the provider out.
- `<provider>:default` is never refused.

The review also found a related fallback, which I fixed: on resume, passing another provider's `--account` dropped the thread's recorded account and ran it on the default. The recorded account (or a worker's parent's account) now has its own slot, and only an explicit account for the same provider replaces it.

**Gate evidence**
- **Regression tests:** new tests cover each fix (other-provider refusal and positive cases, the recorded-account precedence, the flag-off message, the resume 401 hint).
  - Core, exec, CLI and app-server suites: 7253 of 7258 pass. The 5 failures fail the same way on `origin/main`.
  - TUI suite: 4436 of 4459 pass. The failures come from this Mac's environment: the tmux socket path is too long, a status line is cut off by the long worktree path, and one snapshot already differs on `main`.
  - Most new tests rely on new code, so the "fail before" proof is the live before/after runs below.
  - Focused suites pass with developer-accounting too.
- **Linux clippy `-D warnings` on glitch:** clean, with and without developer-accounting, on the final merged head.
- **Review (Opus 5.5 High, installed `corbanu exec`, read-only):** round 1 requested changes, which I fixed. Round 2 approved with nits; I fixed most and wrote the rest up as limits. That's two rounds rather than one, because the round-1 fixes were substantial; both are in `REVIEW.md`.
- **Live tmux checks:** fresh debug binaries, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, GLM 5.3 Flash and Kimi. I proved which account answered by leaving the default credential fake or absent.
  - #425: an unconfigured other-provider account used to get a `pong` from the default key, in exec on both Z.AI and Kimi, and the TUI opened a chat on the default. All three are now refused.
  - Positive case: a Kimi child spawned under `--account kimi-code:kmain` answered and recorded `kmain`. The same spawn without the flag failed, because there is no default Kimi key.
  - The resume fallback the review found (with `exec`): before, it got a `pong` from the default key; now it stays on the recorded `fake` account and gets a 401.
  - #426 and #427 before/after are in the evidence folder too. A leak scan of the diff and captures against both real keys found nothing.
  - Everything is in `qa/provider-auth/pf-84/account-refusals-20261010/`.
- **CI:** the first run on the final head was cancelled. It re-ran on the same commit, and I merged only after all checks were green.

**Overlap with S04 (#430):** a trial merge conflicts in `core/src/config/mod.rs`, `thread_processor_tests.rs` and the precomputed schema export. That export needs regenerating with `just write-app-server-schema --experimental`. I left a comment on #430 with the details. I found no compile-level coupling.

**Cleanup:** the worktree, build target, scratch homes (including the ones with real keys), tmux socket and my files on glitch are removed. Git refused to delete the local branch `fix/pf84-account-refusals-20261010` with plain `-d`; I left it, since forcing it would need `-D`.

**Still open (low)**
- `exec` can't see the model's catalog. With both `multi_agent` features off, it accepts another provider's account even if the model offers no spawn tool. This is documented.
- With the default `multi_agent` v1 spawn tool, a Z.AI or Kimi parent can't move a child to another provider at all; it works through `multi_agent_v2`. So in plain `exec` that account mostly only matters for v2 spawns.
- On resume, `--account <name>` without a prefix is read against the provider you launch with, not the thread's (S03's design). The docs say to write `<provider>:<name>` for a thread on another provider.
- A TUI resume with a missing `[provider_accounts]` selection is refused up front, even if the thread would have run on its recorded account. Without that, onboarding loops; `--account <name>` gets past it.
- There's no app-server integration test for passing the inherited account through; it's covered by unit tests and live run R8.