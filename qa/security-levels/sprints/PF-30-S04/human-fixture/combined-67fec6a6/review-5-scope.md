# Review5 — narrow pending-exit fixture correction

Requested Astra High, last of original maximum five memory-track reviews.
No nested reviewer, product changes, broad architecture work or extra review.
Base `673c378b6` is the Fable-reviewed fixture runtime. Review only the new
test-only delta, not the entire original fixture or unrelated product features.
Source delta is in memory_human_fixture.rs and its support sibling; other
changes are QA records including preserved earlier findings and failed evidence.

The combined67fec6a6 qualification hit the documented pending-exit false-negative
at14/15. It had1request/0outputs and29seconds remaining, but no raw failure
reason survived the old helper. Exact original cause remains suspected, not
retrospectively proven. Coordinator authorized this narrow remediation.

Source-specific failure logs are mapped to bounded labels in persisted status;
no raw log/provider body/secret is copied. Only an exact whole reason matching
Core StageOneMemoryDenial::OwnerTerminated is allowed for pending owner exit.
Other/mixed/provider-change reasons, extended text, expiry, missing request or
persisted output remain denial. This is not a product cancellation change or
weakening of protected-memory behavior. Owner/other-source and mixed reason
regressions are authored; existing expiry/output tests remain.

Final combined proof uses67fec6a6 plus the exact formatter-equivalent fixture
patch from branch runtimeea158a210; product remains immutableb12e32db3. RTX
scoped fix/fullfmt preceded Python2/2 and Rust16/16 (actual-key fixture5cases,
memory-policy,security2,slash1,TMUX support and targeted units). A new runner is
pinned for strict ignored-entry startup/cancel outside shared build lock.

Check exact classification, false-positive/false-negative evidence and bounded
diagnostic ownership. No dependency/schema/public API change is authorized.
Report concrete blocking findings if any; don't demand adjacent provider-picker
fixes or restart a review panel. Original review4 finding is now remediated, not
silently reclassified as a clean original run.
