The flake came from the test, not the product. #589 makes the tmux test helper robust against it and is merged into main (`056daf5ce8`) with all checks green. I couldn't get the natural failure to happen on glitch, but a forced reproduction gives exactly the CI screen.

**Cause**
- **What the CI artifacts show:** the test pressed Esc and then typed `/exit` right away. The input box ended up showing `› exit`, without the slash.
- **Why the slash disappears:** on a slow runner the app reads Esc and the next characters together. The terminal-input library treats that as Alt+`/`, and the input box ignores Alt+character keys by design. So the typed `/exit` never appears.
- **Ruled out:** the bubblewrap warning, the pane height and the long `/status` box played no part. The input box was visible and accepting typing. It isn't a product bug, and what the test proves is unchanged.
- **Already partly fixed:** #561, merged shortly after the failed runs, removed that Esc from this test. #568 and #569 had been running without it.

**What #589 changes**
- The shared test helper now waits after any bare Esc before sending more input: at least 300 ms, then until the screen stops changing (capped at 3 s).
- This also covers two existing tests that still press Esc and then type immediately: `memory_human_fixture.rs:575` and `provider_convergence.rs:136`.
- I added a test for the helper itself. It fails without the wait and passes with it.

**Other new tmux tests**
- #561 is fixed (it removed the Esc).
- #566's new test only presses Esc and then waits for the screen to change before typing, so it's safe.

**Gate evidence** (glitch, bubblewrap hidden from PATH like CI)

| Run | Result |
|---|---|
| Old test (Esc kept) | 20/20 pass; 40/40 with 4 copies on one CPU |
| Current main | 20/20 pass |
| This fix | 20/20 pass, plus 10/10 in the final gate |
| TUI paused while Esc and `/exit` arrive | input box shows `› exit`, the same as CI |
| Same keys read separately | input box shows `› /exit` |
| New helper test with the wait turned off / on | 0/10 / 10/10 pass |
| codex-tui tmux suite | 77 passed, 0 failed |
| Linux clippy, codex-tui tests, `-D warnings` | clean |
| PR's own Ubuntu tmux smoke | 82/82 passed |
| Opus 5.5 High review, read-only | APPROVE |

- **Before/after counts:** on glitch the old test never failed, so these runs don't show a before/after difference for the resume test. The forced reproduction (TUI paused) is what shows the cause.
- **Discarded run:** I also ran a heavy-starvation run, but two of my loops overlapped on the same CPU. Every failure there was a 30 s startup timeout, so it tells us nothing.
- **Review:** I acted on its only real point, softening a code comment, in a comment-only commit. The PR's CI passed on that final commit.

**#568 and #569**
- **#568:** already merged, after its tmux smoke rerun passed.
- **#569:** has merged main (including #561), and its tmux smoke was running again when I finished. I left a comment explaining the earlier failure, so I didn't trigger a rerun.

**Cleanup:** the glitch worktree, build folder and scratch files are removed. The Mac worktree and the local and remote branches are deleted.

**Still open:** the wait makes this failure much less likely but can't fully prevent it. If a runner stalls the app for more than about half a second right after an Esc, the keys can still be read together.