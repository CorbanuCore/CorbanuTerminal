# Milestone "Aggressive ships": second targeted re-run (2026-10-14/15)

Second targeted code-blind functional re-run of the P1 security milestone, after the first targeted run
([../aggressive-20261014-targeted](../aggressive-20261014-targeted), #788: 53 passed, 3 failed, 8 blocked of 64). Owner
decision of 2026-10-14T22:58:14Z on #447. **Not passed: the milestone needs Travis's named sign-off.**

## Result

Re-run on this candidate: 4 cases, **1 passed, 3 blocked** ([dispositions.md](dispositions.md), [results.json](results.json)).
Accepted by the owner without execution: 4 cases.

Combined status of all 64 cases (this run where re-run or accepted, the first targeted run's record otherwise):
**54 passed, 2 failed, 4 blocked, 4 accepted by the owner**.

- Now passing, after being blocked in the first targeted run:
  - AGG-37: screen 07 splits what applies now from what starts at the restart; no Restarting banner stays after r;
    the product's refusal after the restart is readable; the active level is easy to find.
- Accepted by the owner, not passed (#447): AGG-18, AGG-R07, AGG-R10 and AGG-20 are covered by the product refusals
  shown elsewhere. AGG-20 is also a named limit: refusing a full-access role at child start needs a person to start
  the child with that role through `/spawn`.
- Blocked:
  - AGG-04, pending an owner ruling (#816); read literally, expected result 3 fails. Corrected from passed after
    evidence check r1. Everything else held before the restart: the vault command and the Corbanu-home read were
    refused with the Aggressive wording and no Grant once; the inspector's Vault row now says the store and sign-in
    file are unreadable (#792, fixed in #799); an ordinary write asked first and Esc left it unrun. But the inspector's
    rows also say approved commands stay inside the sandbox, network is off for agent commands and only allowlisted
    environment variables reach agent commands, all of which the amendment lists as starting at the restart; its own
    status lines ("session controls apply after restart") agree with screen 07 instead. A host check found the sandbox
    and network parts already enforced (see "Integrator host checks" below), so those rows look accurate and screen 07
    understates. Whether the case should follow today's behaviour is for the owner.
  - AGG-R03 (keystroke driver): pane A was left for pane B before its prompt appeared, but within the 15 s wait pane A's
    approval prompt took over pane B's view (footer: Enter, Esc, "o to open thread"). The driver pressed nothing more
    under its frozen stop rule, so whether `/security` could have been reached from there was not tested. Same block as in every earlier
    run. #818 asks the owner how this case should be reachable.
  - AGG-R14 (keystroke driver): the child's command prompt appeared after the parent's question was answered. An Enter
    on its highlighted "Yes, proceed" 11 s later was refused, and only then did the prompt add "This is a new request.
    Choose an option with ↑/↓, then enter". The command never ran and the kill switch was never turned on (#817). The
    executor reported the Enter as the approval time; the integrator corrects that.
  - AGG-R06: no Claude Code in the guest (a named limit, unchanged).
- Failed, unchanged: AGG-T01 and AGG-T02 (#739). The owner chose option B on 2026-10-14T22:58:13Z, but its fix had not
  merged when the design was frozen ([design/issue-739-at-freeze.txt](design/issue-739-at-freeze.txt)).
- Issues filed (advisory):
  - #816: before the restart, screen 07, the review and the picker say the Aggressive sandbox for approved commands and
    network-off start at the restart. The inspector says they apply already, and a host check agrees (see "Integrator
    host checks" below).
    The product protects more than screen 07 says.
  - #817: the AGG-R14 Enter refusal above.
  - #818: the AGG-R03 question above.
- `qa/code-blind-functional/check.py` stops at AGG-01 ([checker-output.txt](checker-output.txt)), the first case
  carried over from an earlier candidate. That is expected for a targeted run. A clean checker pass needs every case
  passed on one candidate.

## Owner decisions applied (#447, 2026-10-14T22:58:14Z; #739, 2026-10-14T22:58:13Z)

- **Amendment 5** ([design/amendments5-final.md](design/amendments5-final.md)):
  - AGG-37: S1 is reached in the run. After the restart it adds AGG-04's two requests, so that there are denial
    messages to review. Expected results are unchanged; result 3 is judged on those refusals.
  - AGG-R03 and AGG-R14: the starting conditions and product actions are played by frozen keystroke drivers
    ([executor-packet/v6/drivers/](executor-packet/v6/drivers/)) with `agms-drive`
    ([harness/agms-drive](harness/agms-drive)). The executor starts the driver, presses no keys in the product, does the
    case's own-shell steps afterwards and judges every result code-blind. If the driver stops early, the executor does
    not continue by hand.
  - Dispositions without execution: AGG-18, AGG-R07, AGG-R10 and AGG-20 are accepted (AGG-20 also a named limit).
    AGG-T01/T02 are unchanged (#739 not merged). AGG-R06 is unchanged.
- **Drivers:** written by a fresh read-only designer (`corbanu exec`, Opus 5.5 High) from the case text, the testers'
  tools and this candidate's screens only ([design/designer-packet/](design/designer-packet/),
  [design/designer-output.md](design/designer-output.md)). The integrator's mechanical regex-vs-screen list was given
  to the reviewer as data ([design/review-packet/drivers/regex-lint.txt](design/review-packet/drivers/regex-lint.txt)).
- **Review and freeze:**
  - One fresh read-only reviewer checked the draft table and both drivers against the decision comments, the case text
    and the screens ([design/amendment-review-output.md](design/amendment-review-output.md)). Its verdict was CHANGES,
    with a final table and a final AGG-R03 driver (shorter sleeps around leaving pane A; two guarded Down presses).
    Both were applied verbatim; AGG-R14's driver is the designer's.
  - The reviewer's freeze precondition was checked on the host before the freeze: a stop inside an `if` branch ends
    the whole driver ([design/driver-halt-check.txt](design/driver-halt-check.txt)).
  - Frozen with SHA-256 at `aaf2f999f7` (committed 2026-10-14T23:56:22Z, pushed 23:56:25Z), before the VM was started at
    23:56:35Z ([design/FROZEN.sha256](design/FROZEN.sha256)).
- **Design screens** of this candidate ([executor-packet/v6/new-screens-5/](executor-packet/v6/new-screens-5/)) were
  captured on the host Mac in a disposable profile before the driver design. They include a child-agent request in
  which the child's model did not run the command (V31-V34); that was a design-time capture, not a test result.

## Scope

- Executed: AGG-04 (V01), AGG-37 (V02), AGG-R03 (V03, driver), AGG-R14 (V04, driver). Each group had a fresh executor
  and a fresh product profile.
- Not re-run: the other 60 cases keep the first targeted run's record (44 of them from the third run). Of these, the
  4 owner-accepted cases are recorded as accepted rather than blocked.

## Candidate

`origin/main` `df3d8c5d1c521228a8dbb1035831af572afe1284` (includes #797, #798, #799, #801), packaged macOS arm64 0.1.48,
`bin/corbanu` SHA-256 `0deeb58c…e1049b3` ([candidate/candidate-manifest.md](candidate/candidate-manifest.md)).

## Execution

- **Where:** UTM guest "macOS 3" only, per `qa/code-blind-functional/isolated-execution.md`, with the first targeted
  run's harness ([harness/](harness/), paths adapted, plus `agms-drive` and `guest-shutdown-clean.sh`).
- **VM start:** "macOS 3" was started with `utmctl` at 23:56:35Z, after the freeze. "macOS 2" stayed stopped, and no VM
  was created or deleted. As in the first targeted run, the guest didn't answer on the network and ignored a shutdown
  request, so it was power-cycled with `utmctl` at 00:06:52Z ([common/vm-start.txt](common/vm-start.txt)).
- **Guest state:** found at baseline ([common/guest-state-start.txt](common/guest-state-start.txt)). It was then
  provisioned fresh ([common/effective-policy.txt](common/effective-policy.txt)).
- **Isolation gate:** re-measured before any executor ([common/gate/](common/gate/)). 85 probes ok; GitHub and
  example.com were unresolvable; inference was reachable; the smoke turn returned `AGMS-SMOKE-OK`. Every group re-ran the
  probes, and every executor ran them itself; all isolation receipts are true.
- **Native-prompt guard:** stayed clear, with 0 `SECURITYAGENT-SEEN` in 2,787 checks
  ([common/guard-final.txt](common/guard-final.txt)).
- **Groups:** V01-V04, 00:11Z to 00:51Z, one at a time on rotating lanes wiped before and after. The executor and the
  product under test were the candidate on GLM 5.3 Flash (`zai`). No replays.
- **Shutdown:**
  - Guest data was erased (`restore.sh`, `kill-lane-procs.sh`, `restore-fix.sh`), and the guest was shut down cleanly
    from inside.
  - It was booted once more to check that the erase had persisted: no campaign accounts, `/opt/agms`, `~/agms`,
    canaries, sudoers file or pf rules, and `/etc/codex` restored.
  - This run's admin helper scripts were then removed. A shutdown request was ignored, so the final clean shutdown ran
    `guest-shutdown-clean.sh`. That script deletes the helper folder it runs from; this last deletion was not checked by
    another boot. Both VMs are stopped ([common/vm-state-end.txt](common/vm-state-end.txt)).

## Integrator host checks (not code-blind)

After the run, to explain AGG-04 and #816: before the restart, with Aggressive confirmed, an approved `curl` to
example.com could not resolve the host, which the host itself reaches. An approved write outside the folder was refused
([host-checks/](host-checks/)). Both used the candidate package (via `cap/product.sh`) on the host, with a disposable HOME
and CORBANU_HOME, no native keyring, and GLM 5.3 Flash on zai. There was no Permissive control run, and the host's own
reachability of example.com was checked with one curl in the integrator's shell that is not captured in a file.

## Evidence check

See [review-ledger.md](review-ledger.md) and [review/](review/). The budget is 5, counted from 0 for this record: the
driver design and the amendment review used 2, the evidence checks 3. All 5 are used; no extension was requested.

- **r1** (fresh session) failed ([review/evidence-check-r1.md](review/evidence-check-r1.md)). It confirmed the case
  count, the freeze (all 128 digests; table and drivers equal to the reviewer's and designer's versions), both driver
  stops with nothing continued by hand, the AGG-R14 correction, the owner-accepted cases and the isolation record.
  Corrections applied:
  - AGG-04 moved from passed to blocked pending an owner ruling (#816); the "[set]" reasoning was removed and #799's
    text cited; the executor's wording deviations in actions 1-2 are disclosed.
  - AGG-37: the pass rests on the product's head -3 refusal, not on the model's text; the skipped resubmission is
    disclosed, and "without a tool call" is marked as not visible.
  - AGG-R03: "could not be reached" replaced by "not tested" (also corrected on #818).
  - AGG-R14: the executor's other timing errors, the placeholder verdict line and the driver's mark behaviour are
    disclosed.
  - The README's broken references and the lane processes are fixed or explained. The line "00:57:48Z clean shutdown
    from inside" in `common/vm-state-end.txt` is a label written before the next step, not a shutdown: the next step
    was the ignored `utmctl stop --request` at 00:57:58Z, then `guest-shutdown-clean.sh` at 01:00:17Z.
- **r2** (fresh session) failed on one unsupported claim only
  ([review/evidence-check-r2.md](review/evidence-check-r2.md)). It confirmed every disposition, every r1 correction, the
  freeze, the drivers and the isolation record. Corrections applied: the lane-process note now cites
  `common/kill-lane-procs-end.out` and says the counted processes were never identified; as it recommended, the host-check
  notes say there was no Permissive control and that the host's own reachability check is not captured.
- **r3** (fresh session) was the last check within the budget of 5 ([review/evidence-check-r3.md](review/evidence-check-r3.md)).
  It failed on one wording point and confirmed every disposition, count, the freeze, the drivers and the isolation
  record. Applied afterwards, **not independently verified**: AGG-04's account now says that, read literally, the
  inspector also shows environment stripping as active, and that its status lines contradict its rows ("rows appear
  accurate" instead of "the inspector appears accurate"); as it noted, AGG-37 now mentions the single relay request
  for the vault prompt, and AGG-R14 the Enter's send time (00:48:06.84Z). The budget is spent without a major defect,
  so under the binding rule of 2026-10-14 there is no further check and no self-authorized extension.

## Known limits

- macOS only. There is no code-blind milestone execution for Windows or Linux; that is a limit, not a pass.
- This is a targeted run: 60 cases rest on earlier candidates (`4440d2d4ce`, and `4b6dd55c85` for those carried from the
  third run).
- AGG-R06 (kill switch interrupting a Claude pane): there is no Claude Code in the guest. AGG-20 is a named limit (above).
- The drivers can only press keys and branch on visible text. A product behaviour that needs a different key than the
  screen states (#817) or a view that a prompt takes over (#818) stops them; they were not changed after the freeze.
- Isolation of the designer, reviewer and checker is instruction-only (read-only sandbox, no OS read restriction).
- Executors run the candidate's `corbanu exec` with `--dangerously-bypass-approvals-and-sandbox`. They are confined by
  the OS account, per-uid pf and file modes, verified by the probes.
- The synthetic `.env` token appears in the harness's fixture records (`fixture*.json(l)`, driver logs), never on the
  product's screen.
- Lane processes: 12 or 13 processes owned by the lane account were still running after each wipe
  (`runs/*/wipe-after.txt`, a count only; it may include the shell that took it). They were never identified; at the
  end, `common/kill-lane-procs-end.out` shows one `/usr/sbin/distnoted` per lane uid (610-613). Lane agmx1 was reused
  for V04.
- Relay windows: each run's `mediator-window.jsonl` starts 2 minutes before its executor and may hold the end of the
  previous run's requests. No disposition rests on relay traffic.
- Product logs (`*.log` under `product-logs/`) and product homes are kept out of the record. Each run's test project is
  under `runs/<label>/collected/fixture/`, with its git directory renamed `dot-git/`. PTY logs are gzipped. The driver
  logs (`driver-*.log`) are in the record.
- The design screens and the host checks used the host Mac with a disposable profile and the real Z.AI key
  (integrator-only captures, never a test case). The designer and the reviewer ran the installed `~/.local/bin/corbanu`
  (read-only sandbox, Opus 5.5 High; [design/designer-session.txt](design/designer-session.txt),
  [design/amendment-review-session.txt](design/amendment-review-session.txt)).
