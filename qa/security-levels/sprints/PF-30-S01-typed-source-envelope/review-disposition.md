# Independent review ledger

Limit: five invocations for this lane; two used so far. No runtime/model substitution or automatic rerun.

## Review 1 — Astra High

- Candidate: `4f263ca73..12e4a6f25`.
- Engine: structured autoreview helper with `/Applications/ChatGPT.app/Contents/Resources/codex` 0.153.1, model `gpt-6-astra`, effort `high`, read-only inspection, no nested reviewers.
- Scope: `review-scope.md`, including explicit staged native-screening limitations and the potentially >1,000-token fragment bound.
- Result: two P2 findings, retained verbatim in `astra-review-1.json`.
- Accepted in-scope lifecycle issue: the actual session settings replacement path discarded the native registry. Fixed in `60dba1d32` by carrying the host-held registry only across matching thread identities. A regression invokes `Session::update_settings` and checks both pending source identity and admitted projections, plus cross-thread non-sharing. Remote scoped fix/full formatting and all 20 `pf_30_s01` Core tests passed before this review-triggered fix was committed/pushed.
- Accepted shared registration issue: the isolated lane lacked the new Core dependency edge in Cargo.lock. Integration owner applied the exact isolated-lane edge in `7b884e477`; the locked CLI build subsequently passed.

## Review 2 — Fable 5.1 High

- Candidate: `4f263ca73..7b884e477`.
- Structured autoreview using the coordinator's Corbanu wrapper, model `claude-fable-5-1-plan`, effort `high`, inside a private TMUX session. This is a structured exec review in TMUX, not an interactive slash-command review.
- Verified both Astra fixes and the explicit >1,000-token fragment bound. One P2 remained: default realtime WebSocket startup directly connects without the ModelClient WebRTC admission guard. Independent output and TMUX transcript are retained in `fable-review-2.json` and `fable-review-2-tmux.txt`.
- Classified as the same in-scope alternate-transport admission bug. The coordinator approved the exact realtime implementation/test files in `03ec8a6f1` (lane `112878848`) before editing. The second review-remediation cycle adds the shared effective-policy guard before both realtime transports, per-operation live-policy checks, and no transcript-tail flush after denial. Focused remote proof and review 3 are pending; no clean overall review is claimed.

Production screening qualification and sprint completion are not claimed.
