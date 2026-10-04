# acct-bounds-16 worker return

## Provenance and scope

- Action: acct-bounds-16; claim: 62bebebd-6b99-43e1-a181-2384c6fd0f4e.
- Runtime: gpt-6-astra, high.
- Allocation digest: e22dca3c351d4dae053d1e9e6b06dee4aaaef5332f79f00ce2e2fb0b597a4d9c.
- Brief SHA-256 verified before other file reads: a479abe56ee3839f17b4f6cfd4c634aab0160db0cdd875545647464385beac6a.
- Clean launch HEAD: 1b1ea6912ef2a8ff2d005c202fba33c9256f04a6.
- Final four-file Rust diff SHA-256: 98c3773b33abc2be661bf808511b5fcb0f4fecd70a544d6d21be7edc72b28acb (git diff against that HEAD; no source changes after the passing state gate).
- Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915; branch: bootstrap/acct-inspect-20260915.
- Bounded correction within active PF-60 / in-progress PF-60-S03. Existing plan product citation: **Product measurement**, “No commercial performance numbers have been supplied.” Authorized outcome: inspect one run and descendants with explainable recorded totals.
- No schema/index changes, installation, collection activation, live profile, push, or release.

## Approach and bounds

The inspector replaces its call to the inherited whole-store retention planner with selected-owner/window validation, reusing the existing authority, original binding, quote, and retained-estimate validators.

Before decoding a selected attempt, SQL bounds its observation count at 4,096, each relevant payload/evidence value at 4 MiB, and its combined observation payload vector at 4 MiB; historical estimate versions and cross-day request siblings are read individually. The result still refuses above 512 selected attempts, the existing 4 MiB projected packet budget, or 10,000 selected ancestry entries. Unrelated-root ancestry is discarded before charging those budgets. Unknown ancestry remains separate, including unavailable-owner counts.

These are materialization/result bounds, not a constant bound on all SQL work or wall time. Ownership discovery still scans JSON rows without a new index; ancestor discovery may traverse unrelated paths before determining their relation. No truncation is used: keyset LIMIT 1 loops exhaust the relevant history.

## Regression and positive boundaries

- Unmodified production code: the new busy-host regression failed with **TooLarge** (0 passed, 1 failed; nextest retried once and both attempts failed). Setup succeeded. Its owner has one valid unpriced request; 12,000 unrelated valid attempts occupy 600 other same-day native CLI roots.
- After the failing baseline, the fixture was extended with explicit unknown-token/unpriced expectations and a later phase adding 4,097 valid observations to an unrelated attempt. The owner must remain unchanged; directly selecting that unrelated thread must refuse. The original phase also checks the bucketed day-range result against the direct day result.
- 10,000-row positive: exactly 10,000 valid retained attempt rows, 600 unrelated roots, and a Ready owner result containing its one recorded request. This is a retained-store-row boundary, not a claim that 10,000 requests fit the selected 512-attempt cap.
- Maximal packet positive: valid quotes sized to **4,194,303 bytes under the existing projection** (8,192 header allowance plus serialized quotes plus 2,048 per quote). This is exactly one byte below the 4 MiB ceiling, not a claim about a separately serialized wire packet. Two additional legal identity bytes must refuse. More than 4 MiB of valid older estimate evidence is also present and must not consume the packet budget.
- No additional standalone 512-attempt or 4,096-observation positives: the existing positives remain.

## Existing test changes and findings

`accounting_inspect_candidate_cap_precedes_unavailable_candidates` previously created the seven-owner tree fixture plus 512 distinct unknown owners with missing contributions, then asserted TooLarge solely because the global same-day candidate count exceeded 512. It retains its name and fixture. It now compares the complete Ready result with the original tree result plus exactly 512 unavailable unknown owners: known root/descendant totals and existing unknown amounts remain unchanged. The removed behavior—refusing all same-day candidates before relation/unavailability handling—is intentionally no longer covered or implemented.

One additional whole-store-bound dependency was found in `inspection_limit_edges`, called by `accounting_inspect_public_limits_no_truncation`: 10,000 malformed global rows formerly errored but 10,001 returned TooLarge; an ownerless malformed payload at exactly 4 MiB errored but 4 MiB + 1 returned TooLarge. Both sizes now remain malformed-ownership errors. Existing selected 513-attempt and 4,097-observation refusal checks remain. The new valid packet fixture covers the selected packet ceiling.

The brief's defect was reproduced, and its acknowledgment of existing 512/4,096 positives was accurate. One additional existing defect was found in the required UNKNOWN contract: an unparseable thread source with a surviving parent edge could be treated as a resolved descendant. The resolver now keeps it UNKNOWN, with a focused regression. No claim that a schema/index change is necessary.

## Verification and handoff

- `just test -p codex-state accounting`: **159 passed, 0 failed, 187 skipped**, exit 0, 22.591 seconds of test execution; run caa1c1dd-d346-4200-a300-c6d290d3f855. Log: `acct-bounds-16-state-2.log`.
- State run reported **1 leaky marker**: `runtime::accounting::pricing::storage::lifecycle::compact_values::tests::canonical_amounts_round_trip_at_all_required_scales_and_u128_max`. This is a preexisting test; the marker is recorded, not waived or attributed without evidence.
- All four added tests and the revised candidate-cap test passed. Busy-host before/after: **TooLarge → Ready**, with correct original unknown token/cost semantics and matching bucketed-range contents.
- `just test -p codex-tui usage`: **91 passed, 0 failed, 4,071 skipped**, exit 0, 0.389 seconds of test execution; run c52ac08a-1561-4d84-9be1-e7641d4bc897. No leaky marker. Log: `acct-bounds-16-tui.log`.
- `python3 docs/sprints/check.py`: passed (115 current, 127 archived). `git diff --check`: passed. No original accounting test function was removed.
- Final authored changed lines (additions + deletions): **358 tests; 287 production Rust; 54 QA receipt = 341 outside tests, 699 total**. This exceeds the 300-line non-test target by 41 including the receipt, and stays below the hard 450 limit. Generated test logs are excluded from authored-line counts. Final status contains only the four allocated Rust files and this receipt; no outside-scope path changed.

Intermediate state gate `acct-bounds-16-state-1.log` failed to compile the new integration fixture because Inspection has no Clone implementation. The assertion was changed to compare borrowed views; no production workaround or API change was made. The guarded state gate was restarted after scoped formatting.

The replacement inspector leaves the preexisting `inspect_quote_on_connection` helper unused, producing a dead-code warning. Its defining file is outside this allocation; this is disclosed rather than edited outside scope.

Automated logs are local artifacts in this directory (the repository ignores .log files). The original failed baseline and any intermediate failed gates are preserved, not overwritten. Only changed Rust files were formatted with rustfmt --edition 2024; no just fmt/fix was used.

This is an implementation return to Fable, not an unqualified human-test or release handoff. Independent review, receiving-tree checks, isolated code-blind execution/evidence review, true-TUI proof, and applicable live-repository qualification remain manager-owned sprint gates; none is claimed passed by this worker.
