# PF-80-S01 supervised quiescence — frozen internal candidate, OFF

Read the Corbanu Terminal development skill, root AGENTS1.7, current initiative-delivery-control plan/PF-80-S01 sprint, complete slack-supervised-quiescence-allocation.md and private proposal before implementation. No nested AGENTS applies. The allocation governs; skill-directed canonical bookkeeping and the later isolated functional gate remain manager-owned. Product: **Internal delivery control — TO BUILD**, contextual Slack decisions and verified manager handoff, not a new initiative or completed sprint.

Only worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/slack-supervisor-20260912`, branch `workstream/slack-supervisor-20260912` was changed. HEAD remains `179172541dda91e4f9e185269a0fad95f3866bab`; accepted source base `a89a48548f644a64cbb8cd9da090b5b75578c922` is an ancestor. No checkout movement, old-candidate/private-proposal/shared source/state edit, dependency install, worker review, agent child, credential access, live send, service deployment or commit occurred. Test subprocesses and controlled local SDK endpoints are not live agents/services.

## Implemented owner adapter

`decision_manager.ManagedListener(store, binding, *, live=False)` exposes `start(*, seconds=60, ongoing=False)`, `stop()` and `quiesced()`. Start returns only `{state:"starting"}` after the child's exact runtime-owned frame; it is not connection, qualification or stopped evidence. Stop retains the actual Popen handle through graceful EOF/wait5s, terminate/wait2s, kill/wait2s. An unsuccessful wait/kill leaves that handle retained; no PID discovery/adoption or replacement process is used as proof.

`quiesced()` first stops/reaps outside store/feed/transport locks, then acquires the exact runtime guard nonblocking and holds it through the caller's complete inspection/repair. Its None-returning witness is root/binding/descriptor bound, rechecks the birth pin and flock, and expires with a unique context token even if a later descriptor number is reused. A failed or still-busy owner cannot admit repair. Order: manager operation → runtime guard → store → transport → nonblocking lifecycle owner; no stop/join under a store lock.

`decision_manager.supervise_listener(store, binding, *, live=False, stdin=None, stdout=None, now)` is registered by `main` as `supervise-listener --store <private-root> [--live]`. Existing decision-slack forwarding needs no shared control.py change. Initial owner-pipe JSON carries `{binding:...}`. Subsequent bounded frames carry operation `start` (seconds/ongoing), `stop`, `status`, `inspect-fence-loss`, `recover-missing-fence` (case_digest/evidence), or `exit`; results retain `{type:"result",result:...}`. Waiting between frames is ongoing foreground ownership, not a 20-second runtime limit or scheduler. Clean owner EOF requests shutdown. No arbitrary child command or PID is accepted.

Supported local sequence: use the foreground owner; stop, inspect after shutdown, then repair with that returned digest and retained manager evidence. The adapter invokes the existing recovery API inside its real scoped witness. Concurrent changes make the snapshot stale and require another inspection, not an unchecked retry. Local inspection/repair/status do not construct SDK clients or load credentials; start requires explicit live authority. Existing finite/ongoing `listen` uses the same ManagedListener and also stops on owner EOF. No detached service is installed.

At first transport initialization only, `.listener.runtime.lock` is created0600 and fsynced before transport publication; schema2 gains optional `runtime_guard={version:1,file:[device,inode]}`. Store.write supplies atomic journal publication and directory fsync. `slack_transport.runtime_file(store,binding)` never creates, migrates or adopts; it validates exact binding, birth pin, owner-only single-link regular file and no-follow open. `runtime_owned(fd,store,binding)` verifies the pinned descriptor and independent-open flock exclusion.

`Transport.listen(..., *, runtime=None)` now requires the internal `_ChildRuntime` capability before SDK construction. The fixed pinned-Python dedicated exec validates inherited guard/control descriptors, makes them non-inheritable and retains the guard until process death, never Session/SDK cleanup. Parent closes its duplicate without LOCK_UN. The dedicated child denies Python subprocess/fork/spawn/exec paths; the pinned SDK is threads-only. This is a local trusted-runtime contract, NOT a general OS sandbox or binary-only acceptance boundary.

The child watches the private control pipe and a finite monotonic deadline when applicable. EOF/stop sets the actual listener stop event; a five-second forced-exit deadline bounds SDK close or executor joins. Only actual process exit/reap proves all threads are gone. SDK3.44.1 may create executor/runner threads before Session persistence and close can block on workers; a lifecycle lease or stopped frame therefore cannot release this guard. Production clocks remain fresh per observation.

Missing/replaced/symlinked guard, missing pin, legacy initialized shape (including epoch0), partial birth, wrong identity/root/fd and unknown busy owner refuse quiescence without mutation. No retrofit/reset/fabricated clean root exists. A future legacy cutover needs separately classified positive external process-boundary proof. An owned stopped process may require the retained handle's forced kill; an unowned busy process is never signaled by guessed PID. A controller restart waits for verified guard availability, including orphan EOF-watchdog exit, rather than trusting lease expiry.

All existing loss incidents, requests, uncertainty, ingress/events, replies, native submissions/ACKs and consumed permits remain retained. Guard ownership proves bookkeeping quiescence only. Repair still leaves held/outage-gap, refuses historical alert revisions and requires the existing reviewed-gap, current session/identity/context, canonical CAS and exact ACK gates for fresh work. No replay, recovered answer, canonical cancellation, new authorization, recovery-send capability or status-schema change is added.

## Actual proof and retained failures

Every Python invocation below used this exact prefix from the worktree above; table suffixes complete each command. No source edit occurred during the final full export/source-guard run. Earlier focused suites are intermediate results, not the final-tree proof.

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control
```

| Suffix / standalone command | Actual result |
| --- | --- |
| `-p 'test_slack_transport.py' -k ongoing` | Exit0,2tests/4.552s. |
| `-p 'test_decision_manager.py' -k supervisor`, initial | Exit1,2tests/7.420s,1error: fixture socketpair blocked SDK close beyond wait5s. Added the fixture receive timeout matching the SDK connection; production stop bounds unchanged. |
| Same supervisor suffix, expanded | Exit1,4tests/22.319s,1failure: macOS SIGTERM ended the SIGSTOP fixture before kill, contrary to the test's kill expectation. Fixture now explicitly ignores SIGTERM to exercise the real forced-kill branch; no mocked successful wait. |
| `-p 'test_slack_transport.py' -k repair` | Exit0,4tests/3.089s during real-hook migration. |
| `-p 'test_decision_manager.py' -k controller` | Exit0,1test/6.875s, orphan controller death/EOF watchdog. |
| `-p 'test_slack_transport.py'` | Exit0,45tests/58.562s. |
| `-p 'test_decision_manager.py'` intermediate | Exit0,36tests/82.999s before the final two methods. |
| `-p 'test_decision_manager.py' -k test_controller_crashes` | Exit0,1test/9.873s, four actual controller-crash boundaries. |
| `-p 'test_decision_manager.py' -k test_dedicated_exec` | Exit0,1test/0.529s, wrong root/fd rejected before readiness. |
| `-p 'test_decision_manager.py' -k test_supervisor_sigstop` | Exit0,1test/7.694s, injected wait/kill failure retains exclusion, then actual same-handle forced kill/reap. |
| `-p 'test_*.py'`, final frozen | Exit0,261tests/189.695s. All251 baseline methods/assertions retained;8 manager and2 transport methods added. Existing simulated HTTPError500/429 cleanup ResourceWarnings remain disclosed. |
| `node scripts/initiative_control/test_facilities_js.cjs` | Exit0, Facilities UI regression passed,7 assertions. |
| `python3 docs/plans/check.py`; `python3 docs/sprints/check.py` | Both exit0:3 active plans;115 current/126 archived sprints. |
| `git diff --check`; `git rev-parse HEAD`; `git merge-base --is-ancestor a89a48548f644a64cbb8cd9da090b5b75578c922 HEAD` | All exit0; unchanged HEAD/ancestor above. |

Real pinned SDK queue/executor proof retains an old ingress descriptor after actual Session release and emits a deliberately false stopped frame. Concurrent repair stays excluded and journals unchanged until the retained process is killed/reaped; only then can the real hook repair. Other actual subprocess cases cover finite close, graceful EOF, hung close/watchdog, controller SIGKILL before readiness/after renewal/during stop/after reap, SIGSTOP, competing controllers, wait/kill failures, pipe/spawn/framing failures and descendant-inheritance refusal. Wrong root/fd dedicated launches exit1 with empty stdout/stderr before readiness.

Five existing listener tests now execute their original assertions inside genuine guard-owning child processes with controlled SDK seams, not an unguarded listen flag. Existing process listener fixtures also hold the real guard. Read-only AST comparison against accepted a89a48548 found all145 manager/252 transport baseline assertion calls literally retained (zero missing); baseline30/43 methods become38/45. All seven accepted repair crash boundaries—prepared, cancelled, partial, staged, linked, synced, restored—now enter through ManagedListener.quiesced; restart and repeated repair preserve the same incident and original evidence. Legacy epoch0 refusal includes actual SDK threads created before Session. Partial initialization uses a real private store/fsync failure; missing/replaced/symlink guard and binding drift remain held.

Dependency verification used the same Python/PYTHONPATH with `-c 'from importlib.metadata import version; print({name:version(name) for name in ("slack-sdk", "markdown-it-py", "mdurl")})'`: exit0,3.44.1/3.0.0/0.1.2. No install. `shasum -a 256 /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/slack_sdk-3.44.1-py2.py3-none-any.whl` exit0: `d6f20a0fbe3fecf9cac955c99d686301b48a645b7045472c3a0cdd186d7c42b2`.

## Frozen scope and acceptance limits

Incremental source/test count is714 additions/25 deletions =739total/268non-test against accepted a89a48548, unchanged by the docs-only launch. This75-line receipt yields789 additions/25 deletions =814total/343non-test:114/3 above target700/340, with36/57 headroom below STOP850/400. These are this unit's limits, not inherited allowances; target overage was reported before receipt expansion. The manager accepted a narrow above800 ceiling for inseparable process-lifetime proof; no tests were compressed or extra path added.

| Exact path | Added/deleted | SHA256 |
| --- | --- | --- |
| scripts/initiative_control/decision_manager.py | 193/8 | ef7869717397d8f9092577f82253131062d65f4f3d59b7b1262094da2a932851 |
| scripts/initiative_control/slack_transport.py | 65/2 | 3788735618d4572cfcab83a04b6fca4690936e6c0b7a85b8dcf851474c773eb6 |
| scripts/initiative_control/test_decision_manager.py | 326/3 | e92b3ee955f0c29c8422efe2b471fc6f3db33c5d682353eba3b5b1ecbd54cff4 |
| scripts/initiative_control/test_slack_transport.py | 130/12 | 934c4e406a0f83d75916df6231de417e7b881f1c818c2a238c8507f6dfa7eaa8 |

Fifth/only other write: `qa/initiative-control/pf-80-s01/decision-projection/slack-supervised-quiescence-receipt.md`. Final verification uses `git diff --numstat a89a48548f644a64cbb8cd9da090b5b75578c922 --` with the four table paths, `wc -l` on this receipt, `git status --short`, and `shasum -a 256` on all five paths in table order followed by the receipt. Pipe those exact five shasum output lines into `shasum -a 256` for the ordered manifest; receipt/manifest hashes are returned separately to avoid self-reference. Tracked whitespace check exits0; new-file `git diff --no-index --check /dev/null <receipt-path>` exits1 with no diagnostics. Facilities/governance are rerun after receipt only.

Review history remains unchanged: live01–05 failed, combined06 clean, recovery07 clean; earlier offline reviews and all prior size/failure dispositions remain in slack-live-receipt.md/slack-fence-recovery-receipt.md and allocations. Recovery526/242, receiving251tests/144.128s, previous worker2675/1024 and combined3131/1314 are historical, not this unit's budget. Parent owns new08 and necessary corrective09; no duplicate review07 or worker self-review occurred.

Root1.7/qa/code-blind-functional/isolated-execution.md applies. Manager explicitly accepted internal-only/default-OFF N/A for this increment's integration, NOT operator-ready or plan-wide acceptance. Before advertising the supervisor/recovery usable, the manager must provision exact opaque/binary-only boundaries, independent fresh executor, real executor/child denial probes and positive controls, schema2 execution receipts and separate evidence review. These source-readable unit/process/loopback tests are supporting evidence only; historical tests are not upgraded and no harness was duplicated.

Actual Slack authentication/private-channel identity/UI qualification, phone reply/edit/delete/restart, native-manager exact ACK, overnight reception and three-initiative handoff remain unrun. The existing approved credentials/scopes and manager-owned qualification are prerequisites, not fabricated missing-credential/user-approval blockers. Next operation: parent reviews this exact frozen five-file manifest under08, classifies findings before09, then performs combined receiving proof and the separately isolated/actual-service gates. No live enablement or sprint completion is claimed.
