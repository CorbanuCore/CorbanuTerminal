# Isolated model transport — Stage A findings

PF-80-S01 internal infrastructure under the September13 bootstrap allocation.
This is diagnostic evidence, not an admitted transport or functional-test pass.

Ampere returned four private files,650 total/328 non-test lines, and closed.
The original implementation/evidence remains at
`.codex-work/executor-route-audit.Nerv9u/model-only-v1/` under the Corbanu workspace.
The final suite `evidence-k77pww_t/suite.txt` ran16 tests in56.557s:12 passed,
4 failed,0 errors. All52 launches checked the exact0.1.41 package hash from the
allocation. Forty hostile function/custom-tool calls returned unsupported-call
responses. This is not a proof that every possible capability is unreachable.

Observed limitations:

- A synthetic global AGENTS canary reached the model request despite project-doc
  limits; malformed skill diagnostics demonstrated a startup skill read.
- An intentionally invalid effort override reached the wire. The normal builder
  fixes effort to High; general strict-config validation is not an effort allowlist.
- A503 fixture received eight requests. Those retries did not change the model;
  the one-request expectation failed. Retry/cost/cancellation bounds need an
  explicit disposition, not an unsupported claim of fallback or a passing test.

The parent inspected all four source files and original failures. Before choosing
the next enforcement unit, a scoped Fable5.1High review is checking the diagnostic
evidence and whether supported wrapper restrictions suffice. Its private review
copy `.codex-work/model-only-review.BjOu7t/` is byte-identical to the four frozen
files; no raw homes, auth stores or the large generated manifest were exported.
Review01 returned five findings. Parent accepted four diagnostic-evidence gaps:
marker-label limits, missing per-retry identity checks, drop/timeout attribution,
and pre-launch catalog/evidence write protection. The .git-accounting finding was
rejected: .git existed only in the review copy; the actual frozen manifest had
zero generated paths outside evidence directories. Existing RTX runner is unchanged.

Fresh Astra High Arendt corrected only the four allocated files and closed.
New size758 total/372 non-test; the original4670 generated files were hash-checked
unchanged. New suite18 tests in57.376s:14 passed, the same4 capability probes failed,
0 errors. Separate shell-start/write-positive controls prove actual catalog and
owner-evidence write denial; pre/post catalog integrity and all captured retry
identities are checked. No native retry-budget or startup-exclusion pass inferred.
The fresh freeze is `evidence-ycpb8f4q/freeze.json`; explicit QA is
`evidence-qa-1Yc084/qa.json`. Parent inspected the correction; Fable review02
completed clean (exit0, no findings) against the byte-identical four-file copy.
The structured result is retained at `.codex-work/model-only-review.BjOu7t/review02.json`.
This closes diagnostic review, not the four failed capability gates or transport
admission. No additional review of this unchanged candidate is needed. No new
source capability change or full mediator is allocated by this correction.

Direct RPC SSH was separately verified. Its bubblewrap0.9.0/Python3.12.3 host
failed a rootless namespace smoke with `RTM_NEWADDR: Operation not permitted`.
No host security policy was relaxed and no RTX acceptance was transferred to RPC.
Private dashboard browser access, Linux isolation and local transport admission
are separate qualifications. No live inference, product test or recurring launch
is enabled by this result; the bootstrap coordinator owns the next allocation.
