# Independent review ledger

Limit: five invocations for this lane; all five used. No runtime/model substitution or automatic rerun. Both requested engines are clean on the final bounded segmentation delta; no sixth review is planned.

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

## Review 3 — Fable 5.1 High

- Frozen candidate: `4f263ca73..e592cf75a`, same structured Corbanu/TMUX route and model/effort as review 2. Private fixture exited only after its output was captured.
- Scoped fix/full formatting and all 88 combined Core provenance/realtime tests passed before the remediation commit. Final locked CLI build, formatter check, actual-key TMUX `/status`/`/exit`, and plan/sprint checks then passed.
- Reviewer verified the realtime startup/live-operation/transcript-tail fix and both Astra fixes. One P2 remains, retained verbatim in `fable-review-3.json` and its TMUX transcript: `codex-memories-write::runtime::stream_stage_one_prompt` independently constructs an unbound ModelClient and streams rollout contents.
- Caller verified at `memories/write/src/runtime.rs:251`, from `phase1.rs:323`, started by the app-server turn processor. The trace-summarize endpoint guard does not cover this separate stage-one route. Narrowed the README accordingly.
- Scope classification: real adjacent alternate-route gap, outside this lane's literal write scope and across the Core public API/memory owner boundary. After two remediation cycles, the autoreview scope governor pauses code changes pending coordinator disposition. No speculative public API expansion or fourth review was started. This is not a clean overall review and does not qualify all memory paths.

## Review 4 — Astra High, rolling segmentation continuation

- Coordinator authorized this new bounded continuation after assigning the memory-dispatch follow-up to PF-30-S04 and transferring its Core file ownership. The five-invocation ledger was not reset.
- Frozen HEAD `38e1d6e85`, base `72f5de657`; tested source `2a4fb5857`. Structured helper, app-bundled Codex CLI 0.153.1, model `gpt-6-astra`, effort `high`, and `segmentation-review-scope.md`; no nested reviews.
- Exit 0, findings empty, patch correct (confidence 0.94). Exact JSON/text retained in `astra-review-4.*`. Reviewer verified complete-input source/digest/count, atomic refusal, original-index reassembly and unchanged provider shaping. It explicitly noted the unchanged potentially >1,000-token projection and did not qualify production screening or unfinished source coverage.
- No source changes followed. This is a clean review of the segmentation delta, not completion of PF-30-S01 or erasure of earlier findings. Coordinator has authorized Fable 5.1 High external review 5 as the requested second-engine coverage; it has not been replaced by an extra Astra run.

## Review 5 — Fable 5.1 High, external segmentation closeout

- Frozen HEAD `ae699a7e9`, base `72f5de657`; source remains tested `2a4fb5857`. The only addition since review 4 was its QA evidence. Structured autoreview through the approved Corbanu wrapper in private TMUX session `pf30-fable-high`, model `claude-fable-5-1-plan`, effort `high`, with `segmentation-review-scope.md`. This is structured exec review inside TMUX, not an interactive slash-command review.
- Invocation: `python3 /Users/Neo/.codex/skills/autoreview/scripts/autoreview --engine codex --codex-bin /Volumes/CorbanuDrive/Corbanu/.codex-work/security-round5/review-fable-high --model claude-fable-5-1-plan --thinking high --mode branch --base 72f5de657 --prompt-file qa/security-levels/sprints/PF-30-S01-typed-source-envelope/segmentation-review-scope.md`, plus the frozen-source/limits prompt and output paths. No nested reviewers.
- Exit 0, findings empty, patch correct (confidence 0.90). Exact JSON/text and completed private TMUX capture retained in `fable-review-5.*` and `fable-review-5-tmux.txt`; the fixture was closed with Enter after capture.
- Reviewer inspected the actual native callers and ScreeningSession contract and confirmed complete-source/digest/count admission, original-index reassembly, atomic withholding and stable provider bytes. It explicitly relied on recorded RTX test results rather than rerunning tests, excluded production classifier qualification, and retained the unchanged >1,000-token projection warning.
- No findings to accept or reject, and no source changes after review. All five invocations remain recorded; PF-30-S01 stays `in_progress`, with production producer/finer-origin coverage unfinished and the earlier memory-dispatch finding separately owned by PF-30-S04. These two clean continuation reviews do not qualify the whole sprint or a release.
