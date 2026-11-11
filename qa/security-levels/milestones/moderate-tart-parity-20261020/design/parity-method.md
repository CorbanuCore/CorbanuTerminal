# Tart parity run (tart-par1): predeclared comparison method

Written and frozen before the first group's clone (review r1 required changes 5-9). Owner decisions D34/D68 (#864):
Tart becomes the milestone VM after one parallel milestone run matches UTM.

## What is compared

- **Tart run:** all 34 cases, packet v4p (the signed-off texts byte for byte; `packet-v4p/`), candidate main `8c73c8ea80`
  (`bin/corbanu` `15cb32e9…4c4ae6`), one fresh Tart clone of `corbanu-qa-base` per group, testers GLM 5.3 Flash on Z.AI;
  product model GLM 5.3 Flash, GLM 5.2 for MOD-09 and MOD-21 (amendments-v3 R-1).
- **UTM baseline, per case:** the latest UTM attempt in the signed-off record (`baseline.json`, built by
  `tools/make_baseline.py` from `moderate-targeted2-20261020/results.json`): record and group, candidate (run 1
  `df3d8c5d1c`/`abd3e6ad`: 17 cases; first targeted re-run `4ef3be8467`/`cb42614c`: 15 cases; second targeted re-run
  `8bf182de43`/`95ee54cb`: MOD-15, MOD-28), the amendment set it was judged under (v2, v3, v4), product model, executor
  verdict, disposition and owner decision. Known deviations recorded in a baseline record carry over to the comparison.
- **Unit of comparison:** each expected result (passed / failed / not exercised), then the case disposition. The
  baseline's outcome for each expected result is taken from that `utm_run_id`'s `result.md` and the baseline record's
  `dispositions.md` and evidence checks; `baseline.json` holds only verdicts and summaries. Wording or
  timing differences that no expected result judges are noted, not counted.

## Dispositions on Tart

Same rules as the UTM records: an expected result observed to fail makes the case failed; a starting condition not met,
or a product-model decline under G-1 (#447) after the one unchanged resubmission, makes it blocked; an executor verdict is
corrected where the evidence contradicts it. Owner dispositions apply only when the raw Tart outcome repeats the limit the
owner accepted: D39 (MOD-09, MOD-21: the product's model declined before any product protection was reached), D40
(MOD-28: no session approval exists to revoke), D41 (MOD-33: the PATH-aliases note), D43 (MOD-11 and MOD-06's git part:
`.git` read-only under Moderate). MOD-06's git part is not run on either machine, so its D43 disposition carries over and
the comparison covers the rest of MOD-06. A newly observed failure of any other expected result is "failed", not accepted.
MOD-11's run 1 results that D43 does not cover (item 2's Reason, the "don't ask again" offer, the final check) are judged
on their own.

## Classes

- **matched:** same disposition and the same outcome for every expected result.
- **differs**, with one attribution, each needing positive evidence:
  - **product change** since that case's baseline candidate: a named PR merged between the baseline candidate and
    `8c73c8ea80` whose described change matches the observed text or behaviour (or the tie-break below);
  - **environment** (Tart vs UTM): a Tart-specific mechanism (machine, network path, account layout, console state)
    shown by the record, or the tie-break below;
  - **tester variance:** the tester deviated from the case text (skipped, reordered or changed an action, misread a
    screen, ran out of budget);
  - **product-model variance:** the product's own model behaved differently (declined, chose another tool, did not
    attempt) where the case text leaves that to the model;
  - **undetermined:** none of the above is shown.

## Replays and tie-breaks (bounded)

- **Transport check (review r2):** product-model variance, and tester variance caused by a timeout or a slow reply, may
  be assigned only if the guest mediator log and the host relay log for that window show every upstream request
  completed normally (status 200, no `session_error` or timeout) and the clone's `tunnel.log` shows no `ssh exited` or
  reconnect. Otherwise the difference is environment, or undetermined.
- **Replay:** for a difference attributed to tester or product-model variance, or undetermined, at most one replay of
  that group with a fresh tester and the group's frozen prompt (the label only in the run name), in a fresh clone with its
  own evidence folder. Both attempts are kept. The first attempt is the one that counts. A variance attribution stands
  only if the replay matches the baseline; if the replay differs the same way, the difference is undetermined (or, with
  positive evidence, environment or product change).
- **Tie-break** for a difference not clearly a product change: run that case on Tart with its baseline candidate (the
  same signed binary, SHA-256 checked: `corbanu.prev-4ef3be8467`, `corbanu.prev-8bf182de43`, or run 1's candidate-v2
  package), the same packet, only the prompt's candidate line changed (`tools/make_tiebreak.sh`; the three tie-break
  packets `packet-v4p-tb-<commit>` are generated and hashed at the freeze; every candidate is 0.1.48). If Tart plus the baseline candidate reproduces the
  UTM outcome, the difference is a product change; if it reproduces the Tart outcome on `8c73c8ea80`, it is environment;
  otherwise undetermined. A re-run on UTM is not available to this run (UTM is not touched). A change on Z.AI's side
  since the baseline date would show up here as "environment"; that errs on the safe side and is stated where it applies.
- **Cap:** at most 6 replay or tie-break groups in total, chosen in this order: blocker cases first, then advisory.
  Differences left over count as undetermined.

## Parity rule (recommendation on D68's switch)

Tart matches UTM for the milestone if:
1. every clone passed its own isolation gate (`h/gate_ok.py`: 99 probes ok, `AGMS-SMOKE-OK`, relay control 200), every tester re-ran the
   probes with `ok: true`, and the native-prompt guard saw no `SECURITYAGENT-SEEN`;
2. no blocker case differs with the attribution environment or undetermined;
3. advisory cases that differ as environment or undetermined are listed and each is explained; the recommendation says
   whether any of them matters for milestone use.
Differences attributed to product change or to tester/product-model variance (after the replay rule) do not count
against parity. New product failures are filed as issues and reported as findings, not as parity results.

## Environment manifest (both machines)

macOS version and build, virtualization (UTM "macOS 3" vs Tart clone of `corbanu-qa-base`), guest `python3`, `git` and
`bash` versions, CPUs and memory, network path (UTM: guest mediator to `api.z.ai` with the key file in the guest; Tart:
guest mediator over an SSH reverse tunnel to a host relay that adds the key), console/login-window state and the guard
counts, per-group host memory pressure and model latency (mediator `seconds`), and run dates (Z.AI's models may have
changed since 2026-10-14). The UTM side comes from the earlier records; the Tart side from `common/T*/vm-start.txt`,
the gate and the mediator logs.

## Other record rules

- MOD-24's relay check: the integrator reads the guest mediator log for the case's window and cross-checks the host
  relay log; both slices are saved in the run folder.
- `keyscan.sh` runs on every exported folder before commit; product homes are not exported.
- Notes per group (`tools/standing-notes.json`, sources in `tools/standing-notes-sources.json`): each case gets the note
  its baseline group had, or none (run 1 and targeted groups without notes: T01-T06, T09, T11, T14-T16, T19, T20, T22,
  T23, T25, T26; run 1's G16 note "MOD-28 runs first, then MOD-19, in the same session" is dropped because MOD-28 now
  runs alone, as in both targeted re-runs); T07, T13 and T17 get scope or test-machine notes of this run.
