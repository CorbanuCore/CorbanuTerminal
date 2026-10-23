Both of your decisions are now on main, plus one related follow-up: #453, #456 and #459 are merged with all checks green, and #443 and #445 are closed with evidence. In the live tmux runs, `corbanu vault list` is refused straight after confirming Aggressive. With the kill switch on, a question gets a "Not sent: …" message instead of an answer.

**What shipped**
- **#453 → #443:**
  - **Vault refusal:** from the moment Aggressive is confirmed, agent `corbanu vault …` commands are refused, including wrapped forms like `bash -c '…'`. This covers the normal shell tool and `exec_command`. Before the fix, `corbanu vault list` ran until restart (usage error, exit 2) exactly as in the milestone.
  - **Home reads:** these were already denied from confirmation. The live run confirms it.
  - **Screen 07, the review and the `/security` list** now say what is active now and what starts at the restart: sandbox, approvals, network, environment, model keys and child agents. Commands already running keep their earlier permissions. "This session stays Permissive" is gone.
- **#456 → #445:**
  - **Turn refusal:** with the kill switch on, a turn is refused before anything is sent, with a message naming the kill switch and the `/security` steps to turn it off. The refused prompt is not kept, so it is not sent later either.
  - **Every other model request path** also refuses.
  - **Which switch counts:** only the kill switch a person turned on. A session started from an unreadable security state, or a resumed child agent, still gets its turns.
  - **Review texts** match the new behaviour.
- **#459 (follow-up to #443):** a command the sandbox stopped as it started (the AGG-04 home read) now shows on screen as "Ran …" with the denial. Before, the person never saw it. This also changes two upstream tests that asserted the old silence.

**Checks per PR**
- **Regression tests** reproduce the milestone observations, fail before and pass after. The kill-switch loop test counted 73,280 refusals in 2 seconds before its fix.
- **`just test`** passes on the focused sets, with and without developer-accounting.
- **Linux clippy `-D warnings` on glitch** is clean, plain and developer-accounting, at each final head.
- **tmux before/after runs** with GLM 5.3 Flash in a disposable home are recorded under `qa/security-levels/fixes/issue-443-20261011/`, `issue-445-20261011/` and `issue-443-shown-20261011/`. The kill-switch run was also done after a restart into Aggressive, as in AGG-22.
- **Reviews:** each PR had one Opus 5.5 High review, and all three asked for changes; every finding was fixed before merge.
  - #453: screen 07 now names commands already running.
  - #456: a refused turn could loop forever on waiting agent messages, and sessions where the switch was never turned on could be locked out.
  - #459: a lint rule had moved onto the wrong function, which would have failed clippy.
- **Videos:**
  - [vault refused before restart](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-443-issue443-vault-refused-before-restart-03f165ac28d5-2026-10-11.mp4)
  - [kill switch refuses turns](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-445-issue445-kill-switch-refuses-turns-405b34fed76d-2026-10-11.mp4)
  - [denied command shown](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-443-issue443-denied-command-shown-492a40066b48-2026-10-11.mp4)
- **Leaks:** the evidence was scanned against the real Z.AI key (no hits), and the demo tool's own scan passed.
- **Overlaps:** agg-fix-b also edited the picker and the kill-switch view, and agg-fix-a edited `session/turn.rs`. #456 had one conflict with #453 in `client.rs` and the test list, resolved and re-tested. I checked #453 merged with agg-fix-b's merged changes locally before merging.

**Decisions for Travis**
1. **Restart-only items:** the sandbox, approval for every command, network off, environment stripping, the model-key broker and the child-agent rules still start only at the restart; screen 07 names them. They could be applied the moment you confirm, but that is a larger change. My recommendation is to keep them at restart for this milestone.
2. **Home listing:** listing the Corbanu home still shows file names, both before and after a restart. Reading the protected files is denied. Should listing be denied too?
3. **Kill switch and Claude panes:** Claude panes are not stopped by the kill switch, and the text says so. Should they be?
4. **Kill switch and a reply in progress:** a reply already streaming when the switch goes on finishes, and its commands still run. Should turning it on also interrupt the current turn?
5. **Waiting agent messages:** messages from other agents that arrive while the switch is on wait until the next turn after it is turned off. Turning it off does not start a turn on its own.

**Still open**
- **#459 gaps (named in its evidence):**
  - A command whose network check also fails is still not shown.
  - On Linux, a sandbox that fails to start shows as "Ran" with the error output.
  - A denied read-like command shows as an "Explored" line rather than a full command.
- **AGG-37's restart banner** staying visible for 15 seconds was not in my brief and is not fixed. It may belong with #444.
- **Problems that were already there:**
  - The picker test `security_confirm_level_file_edited_by_an_agent_is_caught` fails locally because of our long temp path; it passes in CI.
  - The two `client::skills_*` tests pick up my user skill folders.
  - The compiler crashed once in incremental mode; building with `CARGO_INCREMENTAL=0` got around it.

**Process notes**
- **Frozen design:** a broad text search over `qa/`, looking for how earlier reviews were run, showed part of the frozen design file `design/designer-events.jsonl`. The fixes were already written by then and it did not change them. I made no further searches there.
- **Real binary in tests:** early test runs ran the freshly built test binary for `corbanu vault list/get` against a throwaway home. Debug builds keep the keychain off, so the real keychain was not reached. The tests now use fake binaries.

The Anthropic content filter did not stop me. The build targets and worktrees on the Mac, the glitch worktree, target and logs, and the binaries and run folders in my scratch space are all removed.