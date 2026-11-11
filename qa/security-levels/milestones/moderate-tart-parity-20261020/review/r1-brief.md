# Review round 1 of 5 (packet/plan review) — tart-par1, Moderate milestone parity run on Tart

You are an independent, read-only reviewer (code-blind QA method). Read only files in this folder. Do not run VMs.

## What this run is
Owner decisions D34/D68 (#864): Tart becomes the milestone VM after one parallel milestone run matches UTM. This run
re-runs the FULL Moderate milestone (34 cases MOD-01..34) on Tart (one fresh clone of `corbanu-qa-base` per group,
macOS 26.6.2) with the installed signed build main `8c73c8ea80` (bin/corbanu SHA-256 `15cb32e9…4c4ae6`), code-blind,
GLM 5.3 Flash testers on Z.AI, and compares each case's outcome with the UTM record (signed-off totals: 30 passed, 4
accepted; `baseline-dispositions-targeted2.md`, `owner-dispositions.json`). Differences are to be attributed to
environment (Tart vs UTM) or product change since `8bf182de43` (the signed-off candidate; changes since include #926
post-command .git check, #986, #1014, #1019/#1021/#1026/#1053 stale-approval/grant fencing, #1024, #1025/#1050,
#999, #1042, #1033).

## The design is frozen: no new case text
`packet-v4p/` = the signed-off executor packet v4 of `moderate-targeted2-20261020` with design-cases.md, amendments.md
(v2), amendments-v3.md, amendments-v4.md, executor-prompt.md and all three screen folders byte-identical
(`packet-vs-v4.diff` shows the only differences: navigation.md and prompts/). Built by `make_packet_par.py` (same prompt
template as qual-mod3's make_packet4.py).
- navigation.md: v4 text with two factual wording corrections (amendments-v4 is "the second re-run's" amendment; the
  screens-v3/v4 screens are of earlier candidates, not the one under test) and one added paragraph: where a case names
  "UTM macOS 3" as the test machine (amendments-v4 MOD-15), read it as this run's test machine.
- Groups (`groups.json`, T01..T26): run 1's 25 groups, with MOD-28 split from MOD-19 (as in both targeted re-runs, since
  amendment v4 rewrote MOD-28). Standing notes (`standing-notes.json`, sources in `standing-notes-sources.json`) are
  copied verbatim from earlier runs: the latest one for each case (Q05, Q09, Q11, Q13 of the first targeted re-run;
  G10, G12 of run 1). Run 1's replay notes (G18b, G21b) are not used; they are superseded by Q09/Q11.
- MOD-09 and MOD-21: run per amendments-v3 R-1 (product model GLM 5.2), the text in force at sign-off (D61, on the
  targeted2 record). Not per amendment v5 (local model, qwen-1052 on Tart after the sign-off; its outcome is pending owner
  decisions), because the brief says "case texts and dispositions in force after the sign-off" and v5 has no UTM
  baseline to compare with. R-1's "single re-run A(13) allows" sentence is read as limiting re-runs to unblock these
  cases; this is a repetition of the whole milestone, and G-1 still allows one unchanged resubmission.
- MOD-11 (not re-run since run 1; accepted under D43) and MOD-06's git part are run as written; their raw outcome is
  compared with UTM run 1 and the disposition follows D43 if the failure is the same `.git` read-only limit.
- MOD-33 counts passed by D41 on UTM (its note still showed); it is run as amended in v3 and compared on the raw outcome.

## Harness (the UTM runs' guest files, ported to Tart as in qwen-1052/acc-tart1)
`harness/` + `harness-vs-utm.diff` (vs the UTM targeted2 harness) + `tart-run-vs-qwen1052.diff`.
- agms-launch, agms-product, agms-tui, agms-mcp-notes, provision.sh, product-config.toml, capture-policy.sh,
  kill-lane-procs.sh, keyscan.sh, tmux.conf: identical to the UTM harness (agms-launch/product/tui reverted from the
  qwen-1052 variants: no local-model route, 90-minute budget as on UTM).
- Changed for Tart, from qwen-1052 unchanged: setup-agent.sh (no real key in the guest; synthetic placeholder key file),
  agms_mediator.py (forwards over the SSH reverse tunnel to a host relay that adds the Z.AI key; one request per
  connection; an inert qwen.agms.test target with no upstream: no tunnel port 18602 in this run and no product route),
  agms-probe (99 probes: 85 UTM ones + tunnel and qwen-path denials). seal-guest.sh (main's scripts/qa/tart) builds the
  sealed-guest baseline. host_zai_relay.py runs on the host (127.0.0.1:11237).
- tart-run.sh: one fresh clone per group: up (lock, guard_slot) / seal / tunnel / stage / gate (99 probes + smoke turn
  AGMS-SMOKE-OK) / group (wipe, preflight, executor, cleanup, collect, wipe) / down (logs, delete synthetic test data and
  placeholder key and leaf key, verify absent, stop tunnel, delete clone, release lock). Memory gate: pause before a
  clone while `memory_pressure` free% < 20.
- Shakedown (no executor, before freeze): `shakedown/` — gate ok, 99 probes, smoke ok, erase verified, clone deleted.

## Method after this review
Freeze (SHA-256 of packet, tools, harness, candidate) and push before the first clone; run T01..T26 sequentially, one
VM at a time; MOD-24 relay check from the guest mediator log by the integrator; dispositions per case with the same
rules as the UTM records (an expected result observed to fail = failed; starting condition not met / model decline per
G-1 = blocked; owner dispositions D39/D40/D41/D43 applied as on UTM); a parity table per case: UTM outcome (latest UTM
record) vs Tart outcome: matched / differs (environment | product change since 8bf182de43 | tester variance). Evidence
checks use rounds 2-5.

## Your task
Check: (1) the packet is faithful to the signed-off texts and the groups/notes add no new expectations or bias;
(2) the MOD-09/21 choice and the other case-specific choices above are sound for a parity run; (3) the harness port
keeps the UTM isolation and method (anything that would make Tart results not comparable); (4) the parity/attribution
method is sound and anything missing that the record must capture (e.g. per-case UTM baseline source, how to treat
tester variance, run-1-only cases). Be concrete; list required changes separately from optional ones.
End with exactly one line: `VERDICT: APPROVE` or `VERDICT: CHANGES`.
