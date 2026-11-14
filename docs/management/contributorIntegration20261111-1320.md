# Contributor integration — November 11, 13:20 UTC

> Historical November 11 checkpoint, preserved from PR #2126. For current decisions, merged PRs, runtime availability and remaining gates, see the [November 14 management handoff](managementHandoff.html#update-20261114). Statements below describe their original timestamp, not current instructions.

**The approved main-branch merge queue is integrated; the overall work is not finished.** This dated checkpoint supersedes the [10:26 snapshot](contributorIntegration20261111-1026.md) for this contributor batch. It is not a release, installation or feature-acceptance announcement. [Tracker #2106](https://github.com/CorbanuCore/CorbanuTerminal/issues/2106) records subsequent outcomes.

## Main-branch integration

Main was verified at `493f370ccf328840c4e052ceeffcda4e8dd132e9`. Since the preceding snapshot:

| Work                                                                                          | Merge / evidence                                                                                                                                        | Remaining limits                                                                                                                                                                                                               |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Provider recovery Stage A [#2115](https://github.com/CorbanuCore/CorbanuTerminal/pull/2115)   | `3b4721b0d4`; all 48 checks passed or skipped. Combined API 249, Core unit 10 and integration 9 tests passed.                                           | Thanks **goodalexander**, with original #88 attribution retained. No credential-store restoration was implemented. Stage B and independent functional acceptance remain open.                                                  |
| Rustix advisory repair [#2118](https://github.com/CorbanuCore/CorbanuTerminal/pull/2118)      | `325ef32c29`; all 51 checks passed or skipped, including four V8 builds.                                                                                | No new audit exception. Sync propagation remains separately staged in #2119.                                                                                                                                                   |
| Axum update [#1961](https://github.com/CorbanuCore/CorbanuTerminal/pull/1961)                 | `49f2b4c167`; all 54 checks passed or skipped after the rustix prerequisite. Combined locks match the previously tested integration join byte-for-byte. | Transport passed 152 cases with a private short temporary path; the original two long-path failures remain recorded in #2108. Additional combined campaigns passed 124 remote-control, 222 exec-server and 27 code-mode tests. |
| Chroot classification [#1837](https://github.com/CorbanuCore/CorbanuTerminal/pull/1837)       | `493f370ccf`; all 59 checks passed or skipped, including native Windows and four V8 builds. Reviewed candidate passed 407 classifier tests.             | Thanks **citadelculture**. Original contribution history is retained, with maintainer repairs separately recorded. Eleven independent functional cases and the demo remain unexecuted.                                         |
| Dated management checkpoint [#2125](https://github.com/CorbanuCore/CorbanuTerminal/pull/2125) | `a0d22084c0`; independently reviewed 10:26 snapshot.                                                                                                    | Historical evidence, not a claim that later work was already complete.                                                                                                                                                         |

The chroot merge tree was clean against main after both dependency updates. Its changed source/plan/test files matched the exact reviewed head; its Cargo/Bazel locks matched the qualified combined dependency join. The merged classifier-test file is 479,784 bytes, below the 512,000-byte ceiling. Current-main differences were checked before each merge; no force push or branch-protection bypass was used.

GitHub dismissed an axum approval after a merge-base event. The merge guard stopped; unchanged head, base and combined lock contents were rechecked before the independent review was reaffirmed. The subsequent normal, SHA-guarded merge passed all gates.

Full post-merge CI passed at `a0d22084`. Newer post-merge runs were still pending at this checkpoint; do not transfer that earlier green result to the latest main. No binary was installed or release published.

## Contributor credit

Earlier integrated contributions remain listed in the preceding snapshot. Named acknowledgments and factual Task Node provenance were posted on contributor threads, including **citadelculture**, **local-maxi** and **goodalexander**. No invented task IDs, payments or external Task Node completion are asserted. **bradsbots** received an update distinguishing #2101's three passing discovery-timeout regressions from inherited sync baseline failures.

## Larger follow-ups: implemented, but not fully integrated or accepted

- **Upstream MCP/CI repairs:** #2119, #2120, #2123 and #2124 retain their independent reviews and focused evidence. Typed-401 recovery: 15 tests passed. Synthetic-keyring MCP library: 280 enabled tests passed, one existing skip; native-store denial control passed. Scoped test Clippy passed without disabling production privacy guards or redacting away test leak detection.
- **Formatter draft #2121:** independent review found that formatting captured evidence changed bytes pinned by historical checksum manifests. The repair restored 362 frozen files exactly, preserved the original G17 manifests, and excluded only frozen artifact packets from formatting while retaining live source selection. Four regression tests, a real Ruff selection check and 256 Linux script tests passed. The remaining 530 mechanical Python changes preserve ASTs; the formatter implementation and its new regression test are explicitly semantic changes, not included in that mechanical claim. Review found no remaining bounded repair issue. **The PR remains a draft**, not full-tree-approved.
- **CLI-helper draft #2122:** held until the upstream CLI is resolved. No eager full-CLI build was introduced merely to make isolated package tests pass.
- **Whole sync tree:** unresolved Rust conflicts still prevent whole-tree formatting/build acceptance. Scoped results do not waive these gates, finish #2101, or complete the upstream migration. S2a #2100's broader interfaces/conflicts remain separate work.
- **WinGet #1958:** still held over the inherited `OpenAI.Codex`/`openai-oss-forks` publisher. Do not publish or invent Corbanu publisher ownership without resolving that prerequisite.

PF-23-S04 remains in progress for functional evidence and its demo. PF-84-S07 remains draft under the explicit shared-owner/shared-file handoff, preserving Stage A evidence and the original credential-preservation contract. Reactivation must satisfy the plan/sprint allocation rules; source merges alone do not complete either sprint.

## Owner actions and acceptance gates

1. **88-B — unresolved:** Recommend fail-closed behavior and explicit sign-in for ambiguous damaged/deleted external Claude credentials. Guaranteed automatic preservation needs all-writer coordination or identity-safe isolation. The original preservation requirement has not been waived.
2. **New #2114 — unresolved:** Recommend accepting **citadelculture**'s socket-path fixture fix. Its 153 tests, independent review and CI passed; it is not automatically covered by the earlier batch approval.
3. **Optional execution prerequisite:** Restore Opus 5.5 High execution access for independent functional testing and demo work. Earlier authentication/cooldown failures remain a blocker; no substitute execution model was selected. Coordinator implementation and Astra High code review are not independent functional acceptance.

True-TUI workflows, applicable live-repository qualification, human acceptance and demo reels remain required where specified. Frozen designs and supporting unit tests do not replace those artifacts. Repeat unanswered owner items in five-minute updates while coordinating; during 01:30–09:30 PST, choose productive work that does not require an answer. No unattended continuation is implied by this handoff.
