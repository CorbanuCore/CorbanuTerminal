# RETURN

Allocation `acct-derive-95`; model `gpt-6-astra`; effort `high`.
Claim `3adfa0c1-f921-463d-af4d-4be772c8b553`.
Allocation digest `a640ae3e164c208c29882ab4739e1de50996568dcbdf806325cc0dd20bde9b2b`.
Brief SHA-256 verified before work:
`f978b3e2cd9aca6e444324c6602079b7307379cf29a675c9375c2480556be254`.
Assigned/observed base `b2fa4f7276baa19b33d84732dacac9e494d9943d`;
initial worktree clean. ACK preceded START and every tool call.

## Delivered corrections

Removed `additional_unverifiable_claim_defects_found` from both
[recheck.py](../acct-inventory-92/recheck.py) and its regenerated
[recheck.json](../acct-inventory-92/recheck.json). No replacement typed verdict
was added. The [round-92 record](../acct-inventory-92/RETURN.md) now identifies
its qualitative finding as a manual read by the named `acct-inventory-92`
`gpt-6-astra` high worker, claim `7646a61f-2dc2-4079-b1be-59ceb49a796a`.
It distinguishes that agent judgement from derived checks and human sign-off.
No human reader was evidenced, and none has been invented.

Moved the complete exit table and execution-error caveat in
[acceptance.md](../acct-acceptance-75/acceptance.md) after the manual replay
introduction, command and expected-output explanation. Corrected its directional
reference from “below” to “above.” Content byte/line totals are unchanged;
refreshed affected inventories and preserved historical membership/change counts
and prior digests. Expected stdout and its content digest remain unchanged.

[OPERATOR.md](OPERATOR.md) is the one-page re-verification note: clone supplied
repository with full history; pin the candidate in a disposable copy; run the
verifier, controls and separate replay; interpret output and exits; name the
three missing binaries and their storage exclusion; run guarded Rust gates;
preserve disagreements rather than refreshing expectations to conceal them.
All linked files exist. Source-build heading was checked against the linked doc.

## Checks and exact gate counts

The corrected retained audit passed, normal and optimized baseline both exit 2;
[actual audit stdout](recheck.stdout.json) records derived checks only.
The unchanged expected output ends in
`RESULT agreement=20 disagreement=0 unavailable=3 exit=2`.

[Fresh replay results](replay-checks.json) cover simulated candidate landings:
normal/optimized helper exits 0/0 with matching exact output; deliberately wrong
content-bound references exit 1/1. Every helper invocation runs the verifier
(exit 2) and all 13 controls (exit 0). [Raw streams](replay-streams/) retain
four outer stdout/stderr pairs and sixteen nested streams with verified receipt
hashes. Local synthetic commits and absolute scratch paths are provenance;
the source integration ref was not moved. This is not a claim of actual landing
or automatic CI enforcement.

Prerequisites built first; all fresh Rust tests ran from `codex-rs` via guarded
`just test`, sharing `acct-activation-33/feature/target`, with
`NEXTEST_TEST_THREADS=4`. [Logs/receipts](gates-01/) and independently parsed
[gate results](test-results.json) retain commands, elapsed times, isolation
banners and raw-log SHA-256 digests.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Core accounting default | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| Core accounting developer-accounting | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| TUI usage | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

Total 342 passing executions across overlapping filters. The historically
intermittent auxiliary-scope/event-parity test passed at 32.192s default and
31.532s feature; its earlier timeout remains unresolved historical evidence.
No native credential prompt or live-profile access was observed. No product/Rust
source edits, workspace formatter, raw Cargo tests, source commit or push.

## Changed lines and brief limitations

Existing-file diff against the assigned base: **+71/-45 across 8 files**.
[scope.json](scope.json) records exact per-file changes and all new artifact
line/byte counts and SHA-256 digests, excluding itself. All writes and scratch
outputs stay within the assigned QA subtree. Historical scope inventories retain
original change counts and label their round-95 current-content refresh.

Both P3 findings were confirmed. The brief's reference to a “human read” cannot
be substantiated: the round-92 record identifies an Astra agent, not a human.
This correction uses the removal option and records the actual reader; a named
human re-read remains unprovided if that wording was intended as a separate gate.
The supplied REVIEWED/RECEIVED status is manager context, not an approval made by
this worker. The retained acceptance document still says S03 itself is not
accepted or ready for unqualified human testing; “lane complete” does not close it.
No other brief error was found.

Routine evidence maintenance under PF-60 / PF-60-S03, cited product heading
**Product measurement**, subsection **Measurement targets**, excerpt
“No commercial performance numbers have been supplied.” Independent functional
design/execution, true-TUI, live-repository and benchmark runs are N/A for this
internal evidence edit; final-package functional qualification remains due for
S03. Integrator acceptance of that N/A and canonical sprint/plan updates remain
with Fable. No new human approval, qualification, waiver or release is claimed.
