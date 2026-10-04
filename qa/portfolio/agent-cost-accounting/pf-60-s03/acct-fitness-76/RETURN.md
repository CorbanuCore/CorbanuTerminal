# RETURN

Allocation `acct-fitness-76`; `gpt-6-astra`, effort `high`.
Claim `9f4ebe2f-2a2d-4d30-abfe-9364b6bcb576`.
Allocation digest `bc72791cef6c2d90ecb4c001eb212a122783894939a65de9f53a3f0f13a27178`.
Brief read first; SHA-256 matched
`93682b155a87639f7d6d9026e759c11fd375d05c3b8124635931d1bc6ba92aa1`.
Clean launch HEAD matched `1aadfc123592530f657794489249e954f5e3885a`.

**Use the inspector only for estimates within a known session root: open P2
scope-zero renders “No recorded attempts in this day; collection coverage
unknown.” and “Known subtotal exact USD: 0” despite four priced attempts totaling
USD 0.00284 in other roots in the same store that day, inviting a false day-wide
zero conclusion; it is not fit for day-wide spend or complete-bill decisions.**

That sentence now leads the [owner acceptance document](../acct-acceptance-75/acceptance.md).
The P2 is open, with no code fix or owner waiver. S03 remains in_progress and
not accepted; developer-only activation does not authorize shipping the command.

Timeout status: **historical double timeout preserved, causal explanation not
established**. Exact test:
`codex-core::all suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`.
Round 66 feature TRY 1/2 timed out at 60.013s/60.012s, leaving 126/127 passed,
1 timeout and exit 100; default passed at 58.790s. Round 69 passed alone at
31.743s and in-lane at 31.335s. Current round 76 passed alone at **31.280s**,
in the full feature lane at **31.106s**, and in default at **31.080s**.
The `codex-rs` tree is unchanged from round 66:
`d1f029a6967f2212f85e98052a5567bcabe9e35c`.
Timings support load sensitivity; they do not identify a resource or exclude an
intermittent defect. No timeout relaxation, exemption, or causal fix is claimed.

Fresh guarded gates, prerequisites built first, shared dedicated target,
`NEXTEST_TEST_THREADS=4`, commands run from `codex-rs`:

| Lane | Passed/run | Skipped | Failed / timed out / flaky / leaky |
| --- | ---: | ---: | --- |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 / 0 / 0 / 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 / 0 / 0 / 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 / 0 / 0 / 0 |
| Affected test alone, feature enabled | 1/1 | 3671 | 0 / 0 / 0 / 0 |

**342/342 gate executions plus 1/1 alone replay**; overlapping core sets,
not unique-test counts. Current failure names: none. Lossless raw logs, commands,
timings, exact base and hashes are in [test-results.json](test-results.json)
and `gates-01/`. No native credential prompt or live-profile access was observed.

Owner asks now contain only bounded receiving acceptance, explicit product
exceptions/limited-testing decisions, eventual named acceptance, and any actual
future scope/spend authorization. No renewed permission is asked for ordinary QA.
Engineering performed now: [sprint-evidence reconciliation and explicit matrix](engineering.md),
[neutral designer intent](designer-intent.md), fresh guarded gates and targeted
comparison, inventory repair, and a reproducible local developer package build
with read-only binaries and [exact manifest](package-manifest.json).
Independent design/screenshots, an externally staged packet, enforced executor
enclosure, final-package execution/review and receiving-tree/canonical record
updates remain incomplete. Those are execution prerequisites, not approvals;
this QA-only worker cannot claim them from unrestricted host tools or write
outside the assigned subtree. No functional acceptance or packaged/native run
is fabricated. The local package is preparation, not an isolated qualification.

Inventory repair: [round-72 scope](../acct-controls-72/scope.json) rechecks all
55 entries and updates three changed by round 75: `RETURN.md` corrected coverage
claims; `acceptance-gap.md` gained the owner-document link; `summarize.py` now
derives counts from hashed evidence. [Round-75 scope](../acct-acceptance-75/scope.json)
rechecks 18 entries and updates the acceptance document changed here. The
[inventory delta](inventory-refresh.json) preserves old/new hashes, byte/line
counts and reasons; byte-identical original inventories are retained alongside
it. Historical membership/base/numstat are expressly labeled historical.

Validation: `finalize.py` verifies log hashes, isolation banners and nonzero
counts against raw summaries, plus all 73 refreshed inventory entries. Python
helpers parse; whitespace check passes. Plan checker: 3 active/3 limit; sprint
checker: 115 current/127 archived. The first inventory-helper attempt failed
with `KeyError: new_files_excluding_this_inventory` after round-72 refresh,
because round 75 uses `files`; the helper now supports both schemas and its
fresh replay passed. No Rust test or evidence result was overwritten by this
helper correction.

Changed lines: existing files **+103/−71**; exact new-file byte sizes, line counts,
SHA-256 and per-file tracked numstat are in [scope.json](scope.json); the inventory excludes itself
to avoid a self-referential digest. All changes are within the assigned QA
scope. No Rust/product edits, workspace formatter, commit, or push.

Brief precision: the substantive omissions and stale hashes are confirmed.
“Unqualified zero” describes the primary conclusion, not the entire page:
its title and secondary lines already name root/descendant scope. Round 69 had
already supplied alone/in-lane evidence and a qualified load-sensitivity
inference; the exact timeout cause remains unproven, so it is not resolved.
The request to finish engineering cannot be read as permission to alter runtime,
write outside the frozen QA scope, or impersonate an independent executor; the
remaining execution boundary is explicit rather than called passed.
