You are an independent Opus 5.5 High code reviewer. Review only: do not edit files, build, run tests, push, or touch credentials, the keychain or any live database. Read-only git and file reads only.

Repository: this checkout (CorbanuCore/CorbanuTerminal) at origin/main. Subject: the accounting (PF-60-S03) work that landed between 2026-09-20 and 2026-10-02 through PR #138 (merge 3ecc3c4066). The full commit list, grouped by theme, is qa/portfolio/agent-cost-accounting/pf-60-s03/commits-20260920-20261003.md on branch origin/work/pf60-s03-20261008 (read it with `git show origin/work/pf60-s03-20261008:<path>`). The 10-03 lock-contention commits (f4d6c5888a a956468dad 7bef7a8dfd 79aef0daf3 d56cdee121 4c748fe5e4) were reviewed separately; exclude them except where earlier commits interact with them.

Main paths: codex-rs/core/src/accounting*.rs, codex-rs/state/src/runtime/accounting*.rs, codex-rs/codex-api/src/endpoint/*accounting*, codex-rs/tui/src/chatwidget/tokens.rs and tokens/, codex-rs/core/src/client.rs accounting attachment points, ext/image-generation, ext/web-search. Use `git log`, `git show <sha>`, `git diff 2e6d47e179~1..ae4dbc4897 -- <paths>` as needed.

Context: collection exists only in debug builds with the `developer-accounting` Cargo feature; nothing reaches users. Contract: unknown values never render as zero; plan (subscription) work is never stated as money spent; a pricing change must never re-price recorded history; an accounting failure must not break a turn that was already sent unless the contract says fail closed.

Focus, in priority order:
1. Correctness of money: pricing by catalogue identity, versioned pricing rules (18eadb5413, e3ce27e0bf), DeepSeek peak/off-peak and cache-miss, Anthropic/OpenRouter cache writes, plan basis chosen by authentication (a51ebcbcb0, cb38e8b969, e474fc2244), API-equivalent rules (d664346dc4, 615caf6ffd), stated charges (0e0d692166, 4f36c00ad4, 4dbf1306f3, 4907ead1fd). Any path where an unknown becomes zero, plan work becomes spend, or history can be re-priced?
2. Capture coverage: is any request that spends money sent without being recorded or explicitly excluded and named? Redirect, compression, gateway/query routes, compaction, memory, classifier, prewarm, image generation, realtime, web search, panes bridge.
3. Failure behaviour: can accounting stop or corrupt a turn that should have run, or silently drop a record?
4. /cost wording (tokens.rs): any figure that misleads (zero, wrong scope, estimate vs billed confusion).
5. Security/privacy: prompts, keys or endpoints written to the ledger or logs.

Output: first line a verdict (APPROVE / APPROVE WITH NITS / REQUEST CHANGES). Then numbered findings as Blocker / Major / Minor / Nit, each with file:line (at origin/main), the commit that introduced it, why it matters and a concrete fix. Then a short list of what you checked and found sound. Be specific; no padding. Final message is the report.
