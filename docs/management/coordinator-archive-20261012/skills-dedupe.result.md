The fix is in and PR #160 is open against `main`. All 35 checks pass at head `10ff5949ac`; 9 were skipped. It is mergeable and not merged.

**Root cause:** for non-OpenAI providers, `retain_latest_contextual_developer_fragments` kept only the newest developer section per tag. Four producers write `<skills_instructions>`:
- the executor catalog (World State section `skills`);
- the host catalog (World State section `host_skills`);
- the thread-context catalog (orchestrator and bundled skills);
- the per-turn catalog the extension writes when the host catalog isn't in World State. The reviewer found this one.

Core's older thread-start host catalog also writes the tag. Whichever copy was newest replaced the others, so the model lost a catalog. Every other tag in `CONTEXTUAL_DEVELOPER_PREFIXES` has only one producer.

**Discriminator:** the dedupe now keys on tag plus producer.
- **Producer IDs:** World State fragments carry their section ID automatically. The thread catalog declares `thread_skills`, and every host-catalog variant shares `host_skills`, so they still replace each other.
- **Where the ID lives:** a new local-only field, `context_fragment_sources`, on the existing message metadata (`InternalChatMessageMetadataPassthrough`). It holds one entry per content item.
- **Why there:** the text the model sees doesn't change, and the ID is saved in rollouts so it survives resume. The four catalogs render identically, so the text itself can't tell them apart. A new field directly on the message type would have meant editing about 415 construction sites.
- **Kept off the wire:** it is removed before every provider request (Responses HTTP, websocket, compaction, Ambient) and from `rawResponseItem` notifications. A test confirms OpenAI receives byte-identical requests. The app-server schema fixtures are unchanged.
- **Older rollouts:** a section with no producer behaves exactly as before and replaces every older section with its tag.

**Tests before and after:**
- **App-server test:** I moved `orchestrator_skill_can_read_referenced_resource_without_an_executor` back to the default (non-OpenAI) provider. On `main` it fails at `mcp_resource.rs:248` (catalog line found 0 times, expected 1); it now passes.
- **Two tests left on OpenAI:** `git_attribution_follows_authenticated_workspace_policy` and `host_skill_catalog_refreshes_once_when_skills_change`. They count superseded copies from a single producer, which the non-OpenAI path is meant to collapse, so they weren't moved because of this bug.
- **New unit tests:** in `client_common_tests.rs`, two sources sharing a tag are both kept, repeated copies of one source collapse, and old unattributed items behave as before. Smaller tests cover protocol, codex-api, World State and the skills extension.
- **Linux (RTX box, fresh clone, `TMPDIR` on ext4):** the touched crates passed all 1,033 tests. `codex-core` and `codex-app-server` failed only on the same tests that fail on `main`: 3 PF-60 accounting tests and 6 paginated-thread tests.
- **After merging #155:** #155 merged while I was working, so I merged `main` into the branch. All 2,548 `codex-core` lib tests now pass on macOS, as do the four affected app-server tests.
- **Housekeeping:** `just fmt` and clippy are clean. The Linux clone is deleted; I copied its logs to `/tmp/skills-dedupe-linux-logs/` on the Mac.

**Review:** I ran two passes with claude-opus-5-5-plan.
- **First pass, "request changes":** the major finding was the unattributed per-turn host catalog. I fixed it along with the minor findings.
- **Second pass, "ready to merge":** two remaining minors were adding tests (done) and keeping alignment in remote-compaction truncation. That truncation never sees developer messages because they're dropped before it, so I left it alone.

**CI:** the first Windows credential canary timed out after 15 minutes, while the same job on the branch's push run passed. The rerun and the run after the merge both passed.

**`integrate/management-workstreams-20260911`:** I made no change there. A dry-run merge of this branch into it is clean. That branch doesn't contain #155 yet, so this fix will reach it with the next merge of `main`.

PR: https://github.com/CorbanuCore/CorbanuTerminal/pull/160