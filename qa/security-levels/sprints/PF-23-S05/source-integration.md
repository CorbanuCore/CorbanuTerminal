# PF-23-S05 source integration evidence

Updated 2026-11-12. This is source qualification, not functional acceptance or a release.

## Candidate and attribution

- Contributor: @araupooo, [#2133](https://github.com/CorbanuCore/CorbanuTerminal/pull/2133), original commit `63ed7ba534c3bda6ce190e816320ebb68cc053d8` retained in ancestry.
- Reviewed source head: `48644d9be74108d684eddbb007fc1c91387f3f3d`.
- Complete tree: `70d8b05660cb6aaa4d13486882ab5106a2f5bad0`.
- Combined replay started at `b0b7d2706981ec683c65e9c79e97e6b01deb0d7a`; final head is a metadata-only merge with the identical complete tree. This does not independently establish the binary's embedded commit as the final head.
- Includes merged MCP contribution [#2132](https://github.com/CorbanuCore/CorbanuTerminal/pull/2132) by @bradsbots and governing/design stage [#2135](https://github.com/CorbanuCore/CorbanuTerminal/pull/2135).
- All 46 final CI checks passed or were explicitly skipped; merged normally at 09:12 UTC as `23ac70e1f16561df98a4d198b5df9d531ca9b683`. Fresh main merge-tree equaled the reviewed/tested tree; no bypass. [Named thanks and factual Task Node acknowledgment](https://github.com/CorbanuCore/CorbanuTerminal/pull/2133#issuecomment-6819561454) preserve the distinction between contributor and maintainer work.

## Repairs and independent review

The initial contribution addressed conditional/subshell assignment provenance. Maintainer revisions preserve possible literal values for protected reads, bound overlapping group analysis, and cover pipeline/function/conditional-brace/newline/background contexts. Later review found comment continuation, redirected asynchronous groups, piped keyword conditions and quoted/escaped hash gaps; these were reproduced as incorrectly quiet Bash/zsh judgments and repaired. Independent reaching, comment-free reaching and possible-value interpretations now retain their strongest protection.

Four initial review lenses, independent code-blind design pass5 and repair/integration passes6–11 are retained without resetting the budget. Each extension was coordinator-authorized before dispatch under the owner's delegation. Pass9 approved repaired code; pass10 approved integration; pass11 attested unchanged tree and single main merge-base. Exact-head security approval [5871295071](https://github.com/CorbanuCore/CorbanuTerminal/pull/2133#pullrequestreview-5871295071) contains the full head SHA in body and commit metadata. Static approval does not waive CI, functional or release gates.

## Final local source-tree results

All Rust tests used `just test`, native-credential isolation and four test threads after scoped fix/fmt. Disposable session fixtures use a local harmless executable and do not run host Git or protected operations.

| Evidence                                | Result                                                                             |
| --------------------------------------- | ---------------------------------------------------------------------------------- |
| Full classifier suite                   | 414 passed, no retries or leaks in final replay                                    |
| Assignment mock-session integration     | 16 passed: 13 denial contexts, fresh approval, approvals-off and quiet controls    |
| Combined MCP suite                      | 241 passed, 6 explicit skips                                                       |
| Review6/7 Bash/zsh regression judgments | 12/12 protected as `Some(UnseenCode)`                                              |
| Retained corpus                         | 5,397 cases / 10,794 verdicts; zero quieter, newly asking or changed-kind outcomes |
| Bounded processing                      | Existing work-counter coverage and new group-linearity regression passed           |

Local raw evidence is retained under `/Volumes/CorbanuDrive/Corbanu/.codex-work/`:

- `contrib-2133-combined-classifier.log`, `contrib-2133-combined-agent.log`, `contrib-2133-combined-mcp.log`.
- `contrib-2133-corpus-comparison-final.json` and `contrib-1837-sweep-corpus-XX-2133-final.tsv`/`.log` (11 batches).
- `contrib-2133-review6-red.tsv`, `contrib-2133-review7-red.tsv`, `contrib-2133-review7-green.tsv` and corresponding logs.
- `contributor-2132-2133-review-ledger.md`: full findings, prior failures, authorization and review history. Public authorization checkpoints remain on #2133.

These are local artifact locations, not publicly downloadable evidence links.

## Failed attempts retained

Initial full-suite failures exposed lost outer substitution state; a subsequent high-concurrency run needed two timeout retries. Fresh four-thread replay passed without retries. Session fixture corrections addressed an incorrect denial expectation and a read-only marker destination. Expanded sessions in one test exhausted file descriptors because the shared helper retains mock servers; parameterized cases isolate each process instead of raising limits. A later source-instrumentation test caught a missing work-counter step, repaired before final replay. Corpus comparison initially split embedded control separators incorrectly; newline-only parsing fixed the comparator, retaining raw outputs and the failed attempt. None of these earlier attempts is relabeled as a pass.

## Remaining acceptance gates

PF-23-S05 stays `in_progress`. The frozen 19-case code-blind design has **not** been executed by an independently sealed executor. Opus recharge remains the acknowledged prerequisite; no execution-model substitution was made. Independent evidence review, true-TUI keys, applicable live-repository workflows and demos remain open. Resolve ambiguous synthetic-repository wording explicitly against the required disposable real-repository worktrees; do not silently rewrite frozen cases. No named-human acceptance, install, benchmark completion or release is claimed.
