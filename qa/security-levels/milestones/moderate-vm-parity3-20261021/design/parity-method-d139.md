# UTM vs Tart, three attempts each (utm-par3, owner decisions D139, D140, D141): predeclared method

Written and frozen before the first VM starts. It follows the template of the frozen Tart parity method
(`moderate-tart-parity-20261020/design/parity-method.md`), the same-build rule of `vm-parity-method.md` (D97), and the
settling check the UTM same-build record proposed (`moderate-utm-samebuild-20261021/README.md`, decision 1(a): "switch
if MOD-22 (with amendment v6) matches and MOD-21's spread is alike on both"). The owner's rule (D93/D139): switch
milestone runs to Tart if these cases behave on Tart as on UTM.

## What runs

- **Build, both machines:** the installed signed build when this run was prepared: main `9750861c2a`
  (`corbanu --version` 0.1.48; `bin/corbanu` SHA-256 `0de41e09…2d263cb5d7`; `INSTALL.json` commit `9750861c2a`;
  Developer ID K5N2T25C23), which contains #1171 and #1175. Private copy, in run 1's helper package (only
  `bin/corbanu` differs); one stage (`package-files.txt`) for every group on both machines.
- **Case texts, both machines:** packet `v4d` = utm-par2's packet `v4c` (the signed-off texts plus amendments v6 and
  v7) plus `amendments-v9.md` (D140, frozen in its own record before this method), with the prompts' candidate and
  amendment SHA-256 lines updated, the D114 tester text of `tester-packet-rules.md` in every prompt's Environment
  section and in `executor-prompt.md`, and T17's group note naming v9 (`tools/make_packet_v4d.py`). The same packet
  is used on both machines, byte for byte.
- **Groups:** T17 (MOD-21; product and child model GLM 5.2, amendments-v3 R-1) and T18 (MOD-22), three attempts each
  per machine, in this order on each machine: T17-1, T18-1, T17-2, T18-2, T17-3, T18-3 (labels `utm1..3`, `tart1..3`).
  Then, on Tart only, one T09 (MOD-11, D141, label `tart-d141`).
- **Machines, one at a time, never both:** UTM "macOS 3" first (provisioned once; each group gets a wiped lane, a fresh
  profile, project and tester; `restore.sh` reverses the provisioning at the end; a verification boot; stopped). Then
  Tart: one fresh APFS clone of `corbanu-qa-base` per group, deleted after its synthetic test data is deleted and
  checked. "macOS 2" is never touched.
- **Harness:** utm-par2's, unchanged except: (1) the guest mediator also logs, for each product request, flags for the
  messages that hold MOD-21's task (role; whether the task sits inside a `corbanu_untrusted_data` or an
  `inter_agent_message` element; no text is logged), the product-side record of what the model was sent (utm-par2
  decision 2); (2) one guest-file set (`h/`) and one stage for both drivers. Same network path on both machines (guest
  mediator, SSH reverse tunnel, host relay that adds the key; no real key placed in any guest).
- **Testers:** GLM 5.3 Flash on Z.AI (the candidate's `corbanu exec`), one fresh tester per group; product model GLM
  5.3 Flash (T18, T09), GLM 5.2 (T17).

## Dispositions (per attempt)

As in the earlier Moderate records: an expected result observed to fail makes the case failed; a starting condition
not met, a G-1 model decline after the one unchanged resubmission, or a v9 "message instead of a turn" makes it
blocked; an executor verdict is corrected where the evidence contradicts it. Owner dispositions apply only where the
raw outcome repeats the accepted limit: D39 (MOD-21: the product's model declined before any product protection was
reached), D43 (MOD-11: `.git` read-only under Moderate). MOD-22 is judged with amendment v6's row-by-row comparison.

## Comparison (D139)

- **Unit:** per attempt, the outcome of each expected result (passed / failed / other: blocked, not exercised or
  inconclusive) and the disposition (passed / failed / blocked).
- **Excluded attempts:** an attempt is excluded from the counts, and named, only if (a) the group did not run (gate or
  preflight failed, VM or harness failure, no `result.md`), or (b) the tester departed from the case text in a way the
  record shows changed an outcome (a skipped, reordered or changed action, a misread screen, an exhausted budget). For
  (a) one re-run of that slot is allowed, at most two per machine, labelled `-rerun`; (b) has no re-run.
- **Alike:** for a case, for each expected result and for the disposition, the number of attempts in each outcome class
  differs by at most one between the machines (for example 3 vs 2 passed is alike; 3 vs 1 or 2 vs 0 is not). If
  exclusions leave fewer than two attempts on a machine, the case is **undetermined**.
- **Not alike**, attributed with positive evidence only:
  - **environment:** a machine-specific mechanism shown by the record (machine, network path, accounts, console
    state, transport);
  - **undetermined:** otherwise. Product-model variance is not an attribution here: three attempts per machine are
    there to absorb it.
- **Transport check:** for each attempt, the guest mediator log and the host relay log of its window must show every
  upstream request completed (200, no `session_error` or timeout) and no tunnel reconnect. An attempt whose outcome
  depends on a failed or slow request (a rate limit followed by a success is noted, not counted) is excluded under (a)
  and re-run.
- Wording and timing differences that no expected result judges are noted, not counted. Run-to-run spread within one
  machine is reported per case.

## Recommendation rule (D68/D139)

1. Every group on both machines passed the isolation gate (99 probes ok, `AGMS-SMOKE-OK`, relay control 200), its
   preflight and the tester's own probe run reported `ok: true`, and the native-prompt guard saw no
   `SECURITYAGENT-SEEN`.
2. MOD-21 and MOD-22 are each alike.
- If 1 and 2 hold: **recommend switching** milestone runs to Tart.
- If a case is not alike as environment: recommend **not switching** and name the mechanism.
- If a case is not alike as undetermined, or undetermined: recommend **not yet**, and say what the record shows.
The recommendation may add an integrator's reading, marked as a departure from this rule, for the owner to decide.

## Separate from the comparison

- **#1175 (D95/D140):** for each T17 attempt, how the task reached the child: from the product's session files (a user
  message with origin `human`, or a mailbox assignment; integrator excerpts, not exported) and from the mediator flags
  (whether the request that first carries the task, and any later one, carries it inside `corbanu_untrusted_data` or
  `inter_agent_message`). Confirmed for an attempt if the task sent with "Send task to" is a user message and no
  product request carries it inside either element. Reported per attempt and per machine; not a parity result.
- **MOD-11 (D141):** one attempt on Tart, judged as in utm-par2's part 2 (`rerun-method-d94d95.md`): #1171's message
  or a prompt naming the level for items 1-2, D43 for the approved hook write, every other expected result as observed.
  One re-run is allowed only if item 8 is not reached because of a tester deviation or a transport failure (the first
  attempt still counts).
- New product failures are filed as issues and reported as findings, not as parity results.

## Environment manifest and record

As in utm-par2: per machine `vm-start.txt` (macOS version, `csrutil`, `spctl`, clocks, tool versions, console state,
CPUs, memory), per group host memory (no group starts below 25% free; the driver pauses), mediator latency, run dates
and times (the two machines run hours apart; a change on Z.AI's side in between cannot be told from model variance and
is named where it could apply). Record under `qa/security-levels/milestones/`: evidence per group, product homes and
logs not exported (listed by size and SHA-256), `pty.raw` and tester events gzipped, each fixture's `.git` renamed
`dot-git`, key scan on everything exported.
