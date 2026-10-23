Both issues are fixed. PR #472 is merged into main (merge `3daac07f2d`) at head `de5e5c05ba` with all 27 required checks green. Issues #461 and #462 are closed, each with an evidence comment.

**#461 (security-relevant)**
- **Fix:** while any onboarding text field has focus, every printable key is now treated as text, even when the field is empty. That covers the API key, the Claude token or code, and "Add another account"'s name and masked key. Ctrl+C and Ctrl+D still exit, and `q` still quits from lists.
- **Empty fields:** these used to let `q` quit on purpose. I removed that, because a key starting with `q` would quit at its first character and leak the rest.
- **While saving:** the same rule applies while a value is being saved, so a pasted value with a newline can't quit partway through.
- **On exit:** the terminal now discards any typed input the TUI never read. This only catches keystrokes already waiting; anything arriving after exit can still reach the shell. The real protection is that `q` no longer quits.
- **/providers:** I checked every text and masked field and none needed changing; they already take every key except Ctrl+C, Ctrl+R and Ctrl+U.

**#462**
- **/status:** a home whose only credential is a Z.AI key now shows "Z.AI API key configured" instead of "OpenAI API key configured".
- **Onboarding loop:** when config already names an API-key provider such as Kimi or Z.AI, saving the key now continues to chat instead of looping.
  - **Cause:** setup was started for OpenAI (or not at all), so the saved key was ignored.
  - **Why it was intermittent:** it only worked when a background provider check happened to finish after the save.
- **Related fix:** onboarding no longer silently saves the key to the first listed provider while the screen names a different one.

**Gate evidence**
- **Focused tests (`just test -p codex-tui`):** 153 passed, both with and without developer-accounting. The 10 failures are the environmental ones already recorded in the S04 report for this drive: socket paths too long and the wallet daemon not starting. All 10 are wallet or env-alias onboarding cases in `multi_provider_onboarding`.
- **New tmux tests:** one types a key containing `q`; the other covers the config-named-provider loop. Both fail against an `origin/main` build and pass on the fix. There are also unit tests for each field, the saving states, Ctrl+C, the two loop entry points and the /status wording.
- **Linux clippy `-D warnings` on glitch:** clean on the final merged head, with and without developer-accounting.
- **Review (Opus 5.5 High, read-only):** approved with 7 non-blocking notes. I fixed 4 of them (saving states, the wrong-provider fallback, a test through the real entry point, and a flaky wait). The PR states the stdin-flush limitation it raised.
- **tmux before/after:** I reproduced both issues on debug binaries in disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- **Live check:** one GLM 5.3 Flash turn with the real Z.AI key. Onboarding continued to chat, the model replied "pong", and /status showed "Z.AI API key configured".
- **Evidence:** in `qa/provider-auth/pf-84/fix-461-462-20261011/`.
- **Video:** [#461 fixed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-461-onboarding-q-f575311571d1-2026-10-11.mp4). The leak scan found no hits for the synthetic values.
- **CI:** the Windows credential canary failed once on an unrelated memory test in `codex-network-proxy` and passed on rerun.

**Still open / notes**
- **Clean-up:** my build folder, temp homes, tmux servers, worktree, and the glitch clone and build folder are removed. Only small clippy logs remain in `~/corbanu-rtx/` on glitch.
- **Leftover branch:** the local branch `fix/pf84-461-462-onboarding-text-fields` is still in the main checkout. It is harmless and can be deleted.
- **Side observation, not fixed:** a fresh home that has the Z.AI key only as an environment variable shows Z.AI as "Credential needs attention" and refuses turns. Onboarding with the key works, so the live check used that path. I left it alone as out of scope.

Nothing needs a decision from you.