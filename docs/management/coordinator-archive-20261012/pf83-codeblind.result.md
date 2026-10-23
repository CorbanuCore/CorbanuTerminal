I ran all eleven original cases with fresh code-blind GLM 5.2 executors against the pinned 0.1.42 package. Only three passed: F01, F02 and F06. F05 failed and the other seven are blocked. The independent evidence check passed on its third round (r1 and r2 failed on evidence defects, which I fixed). In the sprint file I ticked the evidence-check box and left the provisioning box open.

Package checked before every run: `corbanu` `e552180a…e758f6`, manifest `510a7286…`, frozen cases `c97bf701…`.

## Results per case
Evidence for each case is under `qa/initiative-control/management-bootstrap/pf83-codeblind-35/runs/<case>/`; the reasoning is in `dispositions.md` in the same folder.

| Case | Result | Why |
|---|---|---|
| F01 | PASS | |
| F02 | PASS | |
| F03 | BLOCKED | Attempt 1: the selection was made with the session idle, and the product's model refused the outside-workspace test write. Attempt 2: the executor pressed Enter on the default "Cancel" every time, so Full Access never applied. Its "failed" verdict is a misread. |
| F04 | BLOCKED | Attempt 1: the test message sent during the Full Access turn was pulled into that running turn, but it was a workspace-only write that needs no approval either way, so it proves nothing. Attempt 2: the turn had finished before the selection. |
| F05 | FAIL | A declined request shows both "You canceled…" and "Ran … (no output)". The core action (choosing Full Access while an approval is pending) was never done. Separately, typing `/permissions` into an open approval prompt approved it with "don't ask again". |
| F06 | PASS | |
| F07 | BLOCKED | No permission change was ever made during an active turn. |
| F08 | BLOCKED | Cancelling was only tried from the restricted level, and the failure branch was replaced by a refused command, which isn't the same thing. |
| F09 | BLOCKED | Every selection was made while idle. |
| F10 | BLOCKED | Attempt 2 showed every expected result, but its variants ran in one session, the restart was a forced kill rather than a normal quit, and the Esc recovery path was never tried. No product decision accepts those deviations. |
| F11 | BLOCKED | The `zai-anthropic` route fails every turn ("model glm-5.2 has no catalogued maximum output token limit"). The route switch happened after Full Access had already taken effect. |

F03, F04 and F10 were each replayed once with the same prompt and a fresh executor; both attempts are kept. Four earlier runs were aborted by my own harness mistake: background jobs were killed when the tool session ended, and one wiped a lane mid-run. No case step had been reached; they're kept in `runs-aborted/`.

## Isolation and negative probes
- **Accounts:** each executor ran as one of three hidden accounts (`pf83x1–3`). They have no password, no admin, staff or SSH access, and private homes. Launch was only through a root-owned launcher with a clean environment.
- **Network:** firewall rules per account allowed only a local proxy. That proxy holds the Z.AI key and passes only `api.z.ai` model calls, so executors never had the key. 1136 calls went through, all successful; GitHub and other hosts were refused. The firewall dropped about 2,000 packets per account.
- **Package and cases:** read-only and root-owned.
- **Probes:** before every attempt the probe set ran from the launcher, from inside the terminal session, and by the executor itself, using fake canary files. It covers repo listing, history, symlink escape, credentials, other sessions, unapproved network, package modification and child-process escape. Every attempt had zero failures.
- **Receipts:** all 11 cases have valid isolation receipts and all 259 hashes match.
- **Process visibility:** the three lanes ran at the same time, and executors could see each other's command lines via `ps`. One did run `ps aux`.
- **Shell path:** executors could still run the guest's system and Homebrew tools.
- **Original designer record:** never supplied, so `check.py` still exits BLOCKED on "missing designer".

## Reviewer verdict
The reviewer was a separate Opus 5.5 High session, read-only and limited to the record folder.
- **r1 failed:** seven evidence defects. It moved F07 to blocked and corrected F04, F05 and F11.
- **r2 failed:** four presentation defects. It moved F10 to blocked.
- **r3 passed**, with minor follow-ups; I applied the wording ones.

Those three rounds take the review count to 7 against a limit of 5. I recorded the extension under the September 12 delegation; it needs manager confirmation.

## Commit
`cc1af43542`, pushed fast-forward to `integrate/management-workstreams-20260911` from a fresh worktree, `worktrees/pf83-codeblind-20261005`. I didn't touch the main management worktree. No secrets in the commit: I scanned for the Z.AI key and the admin password and found no matches.

## VM restore
Everything is reverted: original firewall rules, executor accounts and group removed, `/opt/pf83` gone, home and socket permissions back, the host share remounted, and the proxy and canaries removed. After restore `api.z.ai` returns 301 and the three GitHub hosts fail to resolve. The log is in `common/restore-record.txt`.

Empty system temp folders for the deleted accounts couldn't be removed because macOS system protection blocks it. "macOS 2" is still stopped.

## Still open
- **F05:** a real failure on the pinned package.
- **Blocked cases:** need fresh executor runs that actually set up active-turn starting states, or product decisions on F08's failure branch, F10's deviations and F11's routes.
- **Lane setup:** next time run lanes one at a time or one per VM, so executors can't see each other's processes.
- **Original designer packet:** still missing.
- **Review-limit extension:** needs manager confirmation.
- **Provisioning checkbox:** still open, for the process-visibility gap and the scope amendment, which wasn't supplied or applied.
- **Product findings:**
  - typed text in an approval prompt can approve it;
  - "Enable full access?" defaults to Cancel with no message;
  - the `zai-anthropic` route is broken.