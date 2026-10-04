# RETURN

Allocation `acct-reference-88`; model `gpt-6-astra`; effort `high`.
Claim `b37d9043-e2ec-49ea-85c1-99343f351a8d`.
Allocation digest `1c9cfbcadbcaa90fad9e4f3f698f343a13f24184266b9cca5af9a42e48efaa06`.
Brief SHA-256 verified before work:
`a22e1dce7573e88812773f9809d228b7aa8c963b7e33db085a3b3b16ac9122ac`.
Source HEAD remains `e38b4ceec04c50d14699aa8b97f32274f87e66a5`.

Routine evidence-tool and acceptance-document correction under active PF-60 /
in-progress PF-60-S03. Product heading **Product measurement**, subsection
**Measurement targets**, excerpt “No commercial performance numbers have been
supplied.” No product/Rust behavior changed. Independent functional design,
execution, true-TUI, live-repository and benchmark qualification are N/A to this
internal correction; the independent final-package functional gate remains due
for S03. The manager owns canonical records and acceptance of that N/A.
No human acceptance, sprint completion, release or review approval is claimed.

## Reference that survives integration

Chose a **content-bound reference**, not a historical commit pin or a receive-time
update. The helper reads [expected.stdout.txt](expected.stdout.txt) and its
[SHA-256 manifest](reference.json) from the resolved integration candidate.
It checks the content digest and compares every stdout byte. Verification never
generates or refreshes the expectation. Thus the document/inventory changes and
their reviewed expected output travel together in the same change, with no need
to know the future integration commit ID and no self-reference through an
inventory of the new reference artifacts.

Reference SHA-256:
`7c0be95732e992bdb92da90ec1868df60a102fb83a35109b4717d03825135190`.

[simulate_landing.py](simulate_landing.py) used a disposable full local clone,
applied this QA revision and committed it on the clone's simulated integration
branch, then added an unrelated tracked evidence file and committed again:

| Stage | Commit | Helper exit | Acceptance exit | Output |
| --- | --- | ---: | ---: | --- |
| Before applying change | `e38b4ceec04c50d14699aa8b97f32274f87e66a5` | — | — | Source base |
| Landed change | `8928f8c44db55ec50e8c00ed6790845187403299` | 0 | 2 | 20 agreement / 0 disagreement / 3 unavailable |
| Later tip | `07bf69ed7afae349aa71842d8f6121bdaba8bc55` | 0 | 2 | Identical bytes and reference digest |

Both replays resolved the named integration ref afresh, used clean full clones,
and asserted unchanged tips, empty stderr and clean checkouts. Each ran all
**13/13** verifier controls successfully. No package launched and no network
was used. Source-worktree refs were not moved; these are simulation commits,
not actual integration or receipt by Fable.

[Landing receipt](landing-demonstration.json), [landed raw evidence](landed/receipt.json),
[advanced raw evidence](advanced/receipt.json) and each directory's `controls/`
retain commands, hashes, stdout/stderr and individual control attempts.
[Final-tree check](final-tree-check.json) verifies the proposed input hashes still
match the simulated landing and final verifier output still equals the reference.

## Durability condition and owner caveat

Immediately beside the baseline, [acceptance.md](../acct-acceptance-75/acceptance.md)
now states: “This baseline reproduces only while every edit to an inventoried
evidence file is accompanied by an inventory refresh.” The refreshed inventory
and its correction receipt preserve earlier refresh history and membership.

The acceptance document itself explains what Fable would accept: bounded saved-
evidence reproducibility, not S03 acceptance or shipment. It states what keeps
that true and what breaks it: unrefreshed edits, missing evidence/history,
changed claims or verifier contracts. **There is no calendar expiry, but there
is still a conditional shelf life.** The exact-output reference additionally
requires a reviewed expected-output/digest update when intentional changes alter
output, including inventory totals. Refreshing hashes or replacing expectations
does not establish the truth of a changed claim or remove qualification gaps.

## All five receipt assertions

[check_receipt.py](check_receipt.py) accepts the actual advanced-tip receipt, then
isolates each violation. [receipt-controls.json](receipt-controls.json) records
**1 positive and 9 negative controls**, all passed:

| Assertion | Rejected mutations |
| --- | --- |
| Output matches reference | Match flag false |
| Integration tip stable | Source tip differs from tested commit |
| Checkout clean | Before dirty, after dirty, both equally dirty |
| Expected command exits | Wrong acceptance exit; wrong controls exit |
| Empty command stderr | Acceptance stderr flag false; controls stderr flag false |

Each negative verifies its specific rejection reason, rather than accepting any
exception. Exit and stderr assertions are now separate so neither masks the other.

## Required fresh gates

Read `docs/development/test-isolation.md` before testing. Prerequisites built
first, exit **0**. All lanes ran from this checkout's `codex-rs`, with the same
dedicated `acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`, build
jobs two and debug assertions enabled. Rust tests used only guarded `just test`.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 |

**342/342 executions**, overlapping filters; exact failure names: **none**.
The historically intermittent auxiliary-scope test passed at **31.619s** default
and **31.243s** feature. Earlier unexplained timeouts remain open.
[Test results](test-results.json) and `gates-01/` retain commands, isolation
settings, exact counts, hashes and compressed raw logs. Isolation banners were
present. No native credential prompt or live-profile access was observed.
No auth/token files were read. Python syntax and `git diff --check` passed.

## Changed lines and brief corrections

Four existing files changed, **+64 / -19**:

| File | Added | Removed |
| --- | ---: | ---: |
| `acct-acceptance-75/acceptance.md` | 24 | 0 |
| `acct-acceptance-75/scope.json` | 7 | 7 |
| `acct-inventory-79/inventory-correction.json` | 19 | 5 |
| `acct-selfcheck-84/check_current_tip.py` | 14 | 7 |

New scripts, content reference, this return and raw evidence have exact per-file
line/byte counts and hashes in [scope.json](scope.json); that inventory excludes
itself. All changed paths and disposable clones are within the assigned QA
subtree. No workspace-wide formatter, source-worktree commit or push ran.

The substantive findings are supported. One wording correction: the historical
commit object is immutable; this round does not change that commit. It changes
inventoried evidence in the future candidate, making the old expected stdout
stale. The new mechanism fixes that landing mismatch, not all possible future
evidence drift. Manager-supplied review score and “NOT received” are brief
provenance, not independently established status.
