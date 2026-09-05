# Independent review ledger

Limit: five invocations for this lane; one used so far. No runtime/model substitution or automatic rerun.

## Review 1 — Astra High

- Candidate: `4f263ca73..12e4a6f25`.
- Engine: structured autoreview helper with `/Applications/ChatGPT.app/Contents/Resources/codex` 0.153.1, model `gpt-6-astra`, effort `high`, read-only inspection, no nested reviewers.
- Scope: `review-scope.md`, including explicit staged native-screening limitations and the potentially >1,000-token fragment bound.
- Result: two P2 findings, retained verbatim in `astra-review-1.json`.
- Accepted in-scope lifecycle issue: the actual session settings replacement path discarded the native registry. Fixed in `60dba1d32` by carrying the host-held registry only across matching thread identities. A regression invokes `Session::update_settings` and checks both pending source identity and admitted projections, plus cross-thread non-sharing. Remote scoped fix/full formatting and all 20 `pf_30_s01` Core tests passed before this review-triggered fix was committed/pushed.
- Accepted shared registration issue: the isolated lane lacked the new Core dependency edge in Cargo.lock. Integration owner already prepared that exact edge in `ff2922928`, alongside other lanes' lock edges. Isolated-lane extraction/registration remains coordinator-owned; it is not silently dismissed as a clean review.

Review 2, Fable 5.1 High through Corbanu in a private TMUX fixture, remains pending on the corrected frozen candidate. Production screening qualification and sprint completion are not claimed.
