# Pass 8 — final independent evidence review

Agent: `/root/final_evidence_review`. Model: Astra High. Fresh context; instruction-only
code-blind isolation. No source inspection, reruns, account operations or new design.
The following is the reviewer’s response, preserved without substantive editing.

Verdict: targeted repairs are supported; neither platform is ready for unqualified handoff. The conservative case dispositions are justified.

- All 26 original cases are retained in order: 23 blockers and three advisories. None is promoted to a whole-case pass. The proposal, brief and three screenshot hashes match `design.json`. The referenced `designer-session.md` is absent, so its access-record hash cannot be checked. References: `original-proposal.md`, `design.json`, `dispositions.md`.

- Execution counts agree with the supplied final receipts: 226 selected unit tests, 32 TMUX/support checks, and one separate Apps-origin check passed. The 32 include two support checks; they are not 32 complete user journeys. References: `linux/unit-final-results.txt`, `linux/tmux-tests-results.txt`, `linux/origin-test-results.txt`.

- Named-provider rejection guidance and subsequent same-process replies are visible for synthetic managed-key and OpenAI recovery. The duplicate-slug capture identifies `pf55-duplicate-b` in the footer; corresponding exact-provider tests pass. Mac Claude menu captures show one current row. These substantiate the targeted warning/footer/catalog repairs within their exercised scope, without completing F12 or F15. References: `linux/success-captures/pf54-reauth-key/scrollback.txt`, `linux/success-captures/pf54-reauth-openai/scrollback.txt`, `linux/success-captures/pf55-duplicateslug/scrollback.txt`, `mac/existing-live/models-0.txt`.

- Both package receipts identify Escape and successful recovery; associated captures show an interrupted stream followed by `STREAM_RECOVERY_OK`. Four Mac existing-profile captures show replies across two launches per provider, and both live receipts contain identical before/after configuration hashes. These support the stated smoke/live results, but provide no native prompt count or Applications-launch evidence. References: `{linux,mac}/package-tmux/{result.json,stream-recovery.txt}`, `mac/{existing-live,existing-openai}/{result.json,live-0.txt,live-1.txt}`.

- Main executable and code-mode-helper hashes agree across the manifest, package receipts and supplied staging/hash receipts. All 32 Linux capture hash receipts match the named Linux candidate. The Mac stage receipt records successful signature verification. However, `linux/candidate.sha256` lists four executable paths, not all package resources or an ACP executable; it does not establish the manifest’s “complete” package-hash claim. `mac/mac-build.log` is absent. Full build provenance, resource completeness and strict verification command flags are not independently established here. References: `candidate-manifest.md`, `linux/candidate.sha256`, `mac/mac-stage.log`.

Remaining prerequisites are substantive: fresh live authentication and billing-route journeys, invalid-credential remote rejection, complete recovery/isolation/selection matrices, persisted disabled-provider and supported migration coverage, Linux detach/reattach, native consent denial/cancel/retry and prompt-count confirmation, and advisory UI/race coverage. Environment-owned credentials requiring restart remain an unresolved F12 boundary against its same-process expectation.

F22 remains open: the supplied Apps capture shows an off-origin 403, two requests without authorization attached, and healthy chat—not trusted credential expiry and recovery. F08 likewise remains open: invalid input is stored as configured with explicit “not verified” guidance. References: `linux/success-captures/pf54-apps-origin-boundary/{scrollback.txt,service-requests.json}`, `{linux,mac}/input-probe/{result.json,invalid-submitted.txt}`.

One bounded, code-blind review completed within the supplied directory; no reruns, live-account operations or source inspection.

## Coordinator disposition (after review)

Accepted. Full readiness stays blocked. Corrected the manifest's overbroad
“complete” Linux hash-list claim; it is an executable list, not a resource inventory.
The build log exists in the source evidence but was intentionally excluded from
the code-free packet. Its absence from the packet is not a failed build or a
reviewed provenance proof. No further review is authorized. No source changes
were made after this review. Human/native confirmations remain pending.
