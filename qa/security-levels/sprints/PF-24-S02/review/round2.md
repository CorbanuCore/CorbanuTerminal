**Verdict: request changes.** The fixes for round-1 findings 1, 2, 4–8 and 10 hold. The restart-argument fix (round-1 #9) is wrong, though, and the new `catch_up` and propagate-on-`Unchanged` code introduces three smaller problems.

This was a read-only review of the merge-base diff (`4f09d7af99..HEAD`); I didn't build or run anything.

**Checked and fine**
- **Session tree:** `live_controller(home, thread)` finds the right tree, matching on the root or agent thread. The `AuthorityEpoch` in the basis also ties the review to that tree's runtime nonce, epoch and generation.
- **Lock ordering:** the order is `security_confirm.lock` → `COMMITS` → `security_state.lock`, and nothing takes them the other way round. `live_controller` takes `TREES` then a tree's read lock, and no path holds a tree's write lock while taking `TREES`. No deadlock.
- **Save failures:** a failed restrictive save now reports only its cause. The mismatch branch that restores when Core's `next_start` differs from the target is effectively unreachable outside races, and correct as a guard.
- **Tamper check:** a record that can't be read (`Inaccessible`) is no longer treated as tampering; corrupt content or an Aggressive record still is. `save` checks the file with `load_file`, before the tamper check, which is correct during a downgrade.
- **Aggressive lock:** it's now a `Mutex<Option<File>>` and is released before the restart. On Unix the file is closed on exec anyway.

**Findings**

1. **Medium — restart drops the wrong argument and re-sends the prompt.** `tui/src/security/restart.rs:17-20,33-37`
   - **Cause:** clap's indices don't line up with positions in argv. A value written as `--opt=v`, `-mv`, `-ofz` or `-i a,b` occupies one argv entry but takes up extra clap indices. I confirmed this in the clap_builder 4.5.58 source (`push_arg_values` and `react`) and its `index_of` docs.
   - **Example:** `corbanu --model=x "do it" --sandbox=read-only` gives the prompt clap index 3, and argv[3] is `--sandbox=read-only`. That flag is dropped, and the restarted process sends "do it" again without the read-only sandbox.
   - **Images:** the disposition's claim that "the TUI has no image argument" is wrong. `SharedCliOptions` has `-i/--image` (comma-delimited), and `create_initial_user_message` sends images on their own after the restart.
   - **Parse failure:** if parsing fails, every argument is kept, prompt included.
   - **Fix:** fail closed. Either have the new process ignore any initial prompt and images via a marker variable such as `CORBANU_RESTART_NO_INITIAL_INPUT=1`, or drop arguments only after checking each one equals the parsed prompt value, re-parse, and confirm that `prompt` and `images` are gone and every other argument's value is unchanged. Otherwise don't offer `r`. Add tests for `--model=x`, `-mx`, `-i a,b` and `--sandbox=read-only`.

2. **Low/Medium — `catch_up` raises the live session even though the person chose Permissive.** `core/src/security/level_change.rs:229-234`, `tui/src/bottom_pane/security_level_picker.rs:1047-1053`, `tui/src/security/confirm.rs:323`
   - **What happens:** before the downgrade, this session takes the stored stricter level now, without the activation preflight, along with any stored revocations including the kill switch.
   - **Wording:** the review says Core's level "stays Aggressive in this session" when the session is actually rising from Permissive.
   - **On failure:** if the commit then fails (lock timeout, a `config.toml` rewrite error, `Changed`), the screen says "Nothing changed" although the session was already raised. This errs strict, but the claim is false.
   - **Better fix:** leave the session alone. Pass the stored level the person reviewed into the downgrade's merge, so a stored level no higher than that is accepted (`stored_level > max(from, reviewed_stored)` → `StoredLevelChanged`).
   - **Minimum fix:** word the review "rises to X now" and mention the kill switch if one is stored; make the failure text say the session took the stored level.

3. **Low/Medium — propagating on `Unchanged` doesn't end "for session" approvals.** `core/src/security/transition.rs:340,567`; review text at `security_level_picker.rs:1032`
   - **What happens:** other trees raised by an `Unchanged` commit go through `adopt` (grants dropped) but are never notified, because `closes_channels` is false. Their "for session" network approvals and brokered credentials survive. In this session the event is `None`, so its approvals survive too.
   - **Wording:** the Aggressive review promises "grants and 'for session' approvals end" in this session and the others. When the session is already Aggressive (for example, a project's config sets it), it also says "becomes Aggressive".
   - **Fix:** in `propagate`, call `notify(&tree)` when the level actually rose. Notify this tree's sinks for an `Unchanged` commit that changes the saved level. Say "already Aggressive" when `basis.in_force == Aggressive`.

4. **Low — the request isn't bound to the epoch that was reviewed.** `level_change.rs:219,236`
   - **What happens:** after the basis check, the request is built from a fresh `authority_epoch()`. If another tree in this process propagates in between, the request binds to state the person never saw. A downgrade could then lower an Aggressive level that was just saved. The window is narrow and, in practice, needs another in-process raise, but it reopens round-1 #7.
   - **Fix:** use `reviewed.epoch` when there's no `catch_up`. With `catch_up`, do the epoch check and the catch-up under one write lock and return the new epoch.

5. **Process — the branch is behind `origin/main`.** A plain `git diff origin/main` shows unrelated reverts (`git_paths.rs`, `bwrap.rs`, landlock tests). Rebase and re-run the affected tests before merging.

Finding 1 blocks. Findings 2 and 3 should land with tests: a session that is still below the stored level after a failed downgrade, and sinks notified on a raise propagated by `Unchanged`. The disposition record should correct its round-1 #9 statement about images.