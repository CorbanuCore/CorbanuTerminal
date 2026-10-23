MCP tools now work directly on the Chat Completions and Anthropic wires. PR #539 is merged into main (merge `8c45b5b`), with every PR check passing (29 passed, 9 skipped). I didn't build on PR #254: I opened a new PR and closed #254 with a note pointing to #539.

**Why not #254:** it came from a fork, and it only changed the tool converters in `client.rs`. Those never see the MCP tools on these wires, because `spec_plan.rs` drops them earlier. It also sent the tool's short name (`lookup` instead of `mcp__server__lookup`) with no way to route the call back to the MCP server.

**What #539 changes:**
- **Names:** where a provider has no namespace support, each MCP tool gets a flat name like `mcp__server__tool`. Every name fits all supported providers: 64 characters at most, `[A-Za-z0-9_-]`, starting with a letter or `_`. A name that is too long, invalid or clashes with another tool gets a short hash suffix instead. The same tools always get the same names.
- **Calls:** a call to a flat name is turned back into the original server and tool before it runs. Approvals, elicitation and the security checks therefore apply exactly as on the Responses wire. A built-in tool with the same plain name always wins.
- **Providers:** custom `wire_api = "chat"` providers now correctly report no namespace support. OpenAI and Bedrock are unchanged.
- **Tool limit:** a request carries at most 128 tools, the OpenAI Chat Completions limit. If a user has more MCP tools than fit, the ones that sort last by name are left out and a warning goes to the log. The model isn't told.
- **History:** after switching from a Responses provider, a past MCP call is replayed under the flat name the new provider is offered.

**Gate evidence:**
- **Unit tests:** cover the round trip, the name rules, clashes, ordering, the 128 limit and history replay.
- **Integration test:** `core/tests/suite/mcp_chat_completions.rs` uses a mock Chat Completions server and the rmcp stdio server. The flat call `mcp__rmcp__echo_tool` reaches `rmcp`/`echo-tool`, the approval prompt appears, and the result goes back to the model.
- **Focused `just test`:** 667/667 passed without developer-accounting and 797/797 with it. One earlier run with developer-accounting had 4 timeouts while the machine was under load; the same tests passed on rerun.
- **Clippy `-D warnings`:** clean on macOS and on glitch (Linux), both with and without developer-accounting.
- **Opus 5.5 High review:** round 1 asked for changes (history naming, the tool limit, test gaps), all fixed; round 2 approved. Of its three small leftover points, I fixed one test and left the other two (below).
- **Live tmux checks** (fresh debug binary, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, rmcp stdio server, code mode off):
  1. GLM 5.3 Flash on zai called `mcp__rmcp__echo` directly, the approval prompt appeared, and after Allow it showed `Called rmcp.echo`.
  2. Claude Fable 5.1 on claude-plan, which uses the Anthropic wire, gave the same result.
- **Videos** (SOP, leak scan passed, recorded on the final code):
  - [GLM on zai](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-515-issue515-mcp-tools-on-chat-wire-add027c7be7a-2026-10-11.mp4)
  - [Claude on claude-plan](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-515-issue227-mcp-tools-on-anthropic-wire-add027c7be7a-2026-10-11.mp4)
  - Index: `qa/demos/index/issue-515.md`

**Issues:** #515 closed when the PR merged, and I added an evidence comment. I closed #227 with the same evidence.

**Cleanup:** I removed my build target, the worktree and everything I created on glitch. The local branch `fix-515-mcp-chat-wires` is still in the main checkout, because `git branch -d` refused to delete it.

**Still open (follow-ups, not regressions):**
- Only MCP tools are flattened. Other grouped tools, such as V1 sub-agent tools on generic chat providers, are still dropped on these wires, as before.
- Flat tools don't carry the MCP server's description.
- The tool-limit warning repeats every turn.
- If a built-in tool takes an MCP tool's exact flat name, past calls to that MCP tool can show up in history under the built-in's name. Calls still run correctly.
- The first round of videos (commit `b1e1ac1`) is still on the `demos` release; only the final ones are in the index.