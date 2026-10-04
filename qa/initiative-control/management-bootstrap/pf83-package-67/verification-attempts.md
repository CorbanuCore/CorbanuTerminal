# Package verification attempts — pf83-package-67

No packaged binary or functional case was executed by these checks.

1. `python3 -B qa/initiative-control/management-bootstrap/pf83-package-67/verify.py` exited **1**. Source comparison completed with all 34 files matching. The packet-map check then found that `G17-old-id--macos-arm64--existing--isometric.json` was labeled only F07 in the newly added column, whereas the actual harness map assigns both F07 and F10. The following chained `prepare_pin.py` command did not run. This was a new evidence-inventory defect, not a package or product test failure.
2. Regenerated the entire `all_mapped_cases` column from the actual `fixtures.py:MAP` and corrected both prose claims to include G17. The same `verify.py` command exited **0**: 34/34 source hashes, 288/288 packet hashes and complete case attributions, exact four-binary package inventory/hashes/modes, clean before/after pinned source, and static Mach-O/dependency inspection all passed.
3. `python3 -B qa/initiative-control/management-bootstrap/pf83-package-67/prepare_pin.py` exited **0**. The live verifier rejected the new package with `source pin mismatch`. Evaluating the proposed commit/tree patch with the same verifier accepted the exact package. The live file was asserted unchanged. This is evidence for a proposed patch, not an applied harness update.
4. Read-only `git apply --check` passed for the manager's sprint correction and the live harness pin patch. Neither patch was applied. The first sprint patch draft lacked context and its apply check exited 1; the corrected contextual patch passed.

Build and regression-lane commands/outcomes are recorded separately in the return receipt. Raw build/test logs remain under ignored `artifacts/`, inside the authorized QA prefix.
