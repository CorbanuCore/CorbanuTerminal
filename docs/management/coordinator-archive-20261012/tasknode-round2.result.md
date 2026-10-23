Steps 1–4 are done. I didn't send any evidence, because all five new Task Node tasks are still **Proposed** and none has been accepted yet. Everything is pushed to `integrate/management-workstreams-20260911` (now at `aa273695db`, a plain fast-forward).

**Tasks created (A1).** I sent each request once. Task Node generated these tasks, and I didn't accept, refuse or move any of them.

| Task | Title | State |
| --- | --- | --- |
| `task_9d4fc2d8b21c3fd3ab2bc5ce6b18a188` | Record PF-80-S01 Bootstrap Deliverables With Commit Evidence | Proposed |
| `task_512bafc249df308083f7acacbc22382e` | Restore the Receipted Slack Alert Path and Send One Owner Alert | Proposed |
| `task_cf2cc91ff71a095a3baa93df1c3d91f8` | Install and Arm the Corbanu Owner Daemon Recurrence | Proposed |
| `task_80628c7accf770686bdd04fc4cb78bb5` | Ship PF-80-S01 Live Progress Event With Idempotent Re-Send | Proposed |
| `task_6d472c45a76bcd6cb5e554eda863a9a1` | Register First Owner-Bridge Allocation and Record QA Evidence | Proposed |

The map is `qa/initiative-control/management-bootstrap/tasknode-task-map.md`. It links each task to its request ID, commits, QA records and PF-80-S01 ledger entries, and says what each task does not claim. For example, the Slack reply half isn't proven, and the live event isn't sent.

**A2: the vault-backed send command.** Commits `ec88be0a06` and `b0222784b9`.
- `tasknode.py send --live` and `enroll --confirm-live` now read the Task Node session and the Corbanu API key from the vault at send time. Both stay in memory only and are never logged or written to disk.
- An independent Opus review found two P2s, which are now fixed: the debug-home environment variables could redirect the vault read, and the session's "unlinked" marker was ignored. The second review pass was clean.
- The Task Node tests pass 43/43, using only made-up tokens.
- In the full suite, 31 tests fail, all in `test_owner_daemon`. The same 31 fail on the untouched base commit `4da42a9b10`, so this change didn't cause them.
- **Incident:** before one fix, a new test reached the production enrollment endpoint once with the made-up token and key. It was rejected with HTTP 401; no real credential was involved, and that path is fixed.

**A3: the Corbanu API key.** I didn't copy anything from the RTX box: its vault has no Corbanu API key. This Mac already had one in the terminal home vault, under `provider/pfterminal_plan_api_key`. It resolves with exit code 0 and length 47, and the server accepted it during enrollment. The sender reads it from that home, so there's still only one copy. I didn't touch the wallet private key.

**A4: enrollment.** `corbanu-initiative-control` enrolled at 2026-10-03T09:58:15Z through the new command, and the server verified it. `enrollment.json` was written. `control.json` is unchanged, and batch posting is still off (`enabled=false`).

**Send and re-send receipts:** none. There's no accepted task to send against.

**Still blocked:**
1. **Your action:** accept one task in Task Node, most naturally `task_80628c7…`. After that, the manager points the `PF-80-S01` mapping in `control.json` at that task instead of the Rewarded `task_789a0f3…`. Then the send and the same-key re-send can run.
2. **Wallet key exposure:** the vault entry `CorbanuAPI`, which looks like the wallet private key you described, is saved as a "manual secret". The vault helper only refuses entries saved as private keys, seed phrases or keystores, so any agent can read this one. Please re-save it as a crypto private key. I didn't read it.
3. **Other worker's commit pushed:** my push also published commit `edc60bfb02` (B1) from the other worker in this worktree, because my commits sit on top of it. It was a fast-forward and nothing was rewritten.
4. **Command trap:** `~/.local/bin/corbanu` is a wrapper script that always switches to the terminal's own home folder, whatever you set. Round-1-style checks of `~/.corbanu` must call the real binary. The new sender refuses wrapper scripts for this reason.

The record `qa/initiative-control/management-bootstrap/tasknode-writeback-live-20261002.md` now has A1–A4 word for word and the round-2 results. The PF-80-S01 ledger is updated, and the sprint and plan checks pass.