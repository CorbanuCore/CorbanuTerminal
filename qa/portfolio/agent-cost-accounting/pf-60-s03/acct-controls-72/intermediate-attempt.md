# Preserved harness failure

`controls-01` stopped at mutation 106 (`refused-money:stale-estimate`).
The fixture flattened all selected rows into one string, which inadvertently
removed the standalone Refresh row. The auditor stopped at
`assert "Refresh" in lines` (line 140 at the time); the harness then raised
`JSONDecodeError` while expecting the semantic diagnostic on stderr's first line.
This was a fixture defect, not a passed mutation or a product failure.
The original plan, two baseline receipts and mutant directory remain locally.
Append mutations now retain the original rows and append a separate row.
Fresh `controls-02` passed. `controls-03` then caught a syntax error in a
stdout-only wording edit before either baseline ran; its raw process log is
preserved. The f-string quoting was corrected. Final-tree replay is `controls-04`.
The harness now preserves every invocation's stdout/stderr before checking it.
No original results were overwritten.
