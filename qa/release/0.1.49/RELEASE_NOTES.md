# Corbanu Terminal 0.1.49

## Upgrading from 0.1.48

Back up your complete Corbanu home before upgrading, with Corbanu stopped.
Keep the configuration, databases (including any SQLite sidecar files), vault
and its key storage, sessions and history together.

The provider-request salt migration makes the upgraded state database
incompatible with 0.1.48: that version refuses to open migration 53 and may
call the database damaged. This is a version mismatch, not evidence of
corruption. To return to 0.1.48, restore the matching pre-upgrade backup along
with the old package. Do not delete migration records to bypass the check.
Conversations created after the backup will not be present in that backup.

## Protocol change: `turn.completed` usage can be `null`

- **`corbanu exec --json` reports `"usage": null` when exec receives no
  matching usage report** (#290). Unknown usage is no longer reported as an
  all-zero usage object, so consumers can distinguish "not reported" from a
  measured zero. When usage is reported, the object is unchanged.
- **A non-null usage does not prove the provider reported this turn.** On a
  resumed thread, if this turn's response reports no usage, `usage` still
  holds the thread's earlier totals. When only some of a turn's requests report
  usage, the totals include only those.
- **Usage includes observed descendant work** (#290, #1601, thanks to
  @white-guy-01). Repeated cumulative reports are counted once. Every
  history-forked descendant makes the total `null`, as do a descendant with no
  report, a failed or timed-out ancestry read, or dropped events (`Lagged`).
  The selected thread's own totals are not a difference from the start of the
  invocation, so its first report needs no inherited baseline: a resumed
  thread whose first report follows a compaction estimate keeps a known total
  (#1637). A history-forked subagent that is not saved to disk yet is now also
  recognized as a fork (the app server's `thread/read` returns its
  `forkedFromId`), so it too makes the total `null` (#1637).
  A resumed (non-fork) descendant whose first report carries in earlier usage
  (its total differs from its last request) makes the total `null` too
  (#1637, #1750, thanks to @citadelculture).
  Detached children can incur further usage after this snapshot.
- **Default multi-agent spawns copy history.** In MultiAgentV2, `spawn_agent`
  defaults to `fork_turns: "all"` when omitted, so these default multi-agent runs
  report `usage: null`. A non-null total can be larger than in earlier versions
  because it includes descendant work rather than only the root's own total.
- **Context estimates are not provider reports.** Compaction, rollback and
  full-window estimates keep the last measured report, or `null` if none exists.

### Compatibility

- **Consumers that read token counts from `turn.completed.usage` without a
  null check** will fail on turns with no reported usage and must be updated.
  The TypeScript SDK in `sdk/typescript` handles `null` from this release:
  `TurnCompletedEvent.usage` is `Usage | null`, and `Thread.run()` returns
  `usage: null` when exec receives no usage report.

## Protocol change: `refused` command status

- **A refused command now says "Refused", not "Approval declined".** When a
  policy or setup check stops a command without anyone being asked to approve
  it, the command's cell shows `└ Refused: <reason>` (the transcript says
  `✗ Refused`), and `corbanu exec` prints `refused:` followed by the reason
  (#822, #851, #900).
- **New status value.** The app-server protocol's `commandExecution` item can
  now have `status: "refused"` (alongside `inProgress`, `completed`, `failed`
  and `declined`); `corbanu exec --json` reports `"refused"` too. `declined`
  now means only that an approval request ended without permission (denied,
  timed out or cancelled). `refused` usually means the command did not run,
  but a network-policy denial can arrive after it started, so don't assume it
  never ran. The refusal reason is in `aggregatedOutput` (`aggregated_output`
  in `exec --json`); the exit code is a placeholder, not a process exit. File
  changes (`fileChange`) have no `refused` status.

### Compatibility

- **App-server clients and SDKs generated before this change** don't know
  `refused` and can fail to read an `item/completed` that carries it. Regenerate
  from this release's schema (`codex-rs/app-server-protocol/schema`). The
  TypeScript SDK in `sdk/typescript` (which reads `corbanu exec --json`)
  already includes it.
- **The Python SDK (`sdk/python`) does not include it.** It stays on its pinned
  upstream runtime (`openai-codex-cli-bin==0.144.4`), which has no `refused`.
  A Python client pointed at a Corbanu binary (`CodexConfig.codex_bin`) can
  fail on a refused command.
- **Saved history.** Default sessions don't save command status, so older
  builds resume them as before. Threads started with the experimental
  app-server `historyMode: "paginated"` (and their subagents) do: an older
  build can't read the history of such a thread once it has a refused command
  (`thread/resume` with turns, `thread/turns/list` with full items,
  `thread/items/list` and `thread/turns/items/list` fail with "failed to
  deserialize stored thread item"). Newer builds read older history unchanged.

## Slash commands typed while the session starts

- **A slash command entered during startup always waits for it** (D376,
  #1748). While the header still reads `model: loading`, a command such as
  `/model` is queued and runs once startup completes, like an ordinary prompt.
  Before, it was queued only when its keys arrived in one quick burst; typed
  a little more slowly (so the command list opened before Enter), it ran at
  once and was refused with "Model selection is disabled until startup
  completes."
- **`/quit` and `/exit` don't wait** (D390, #1833). Typed during startup,
  they exit at once instead of joining the queue, so a startup that hangs can
  still be left without Ctrl+C. Every other command still waits.

## `/cost`

- **Codex Auto Review is named as unpriced.** On an OpenAI API key, requests
  from the automatic approval reviewer still show `no price available` (no
  estimate, never $0), and the next step now adds "OpenAI doesn't publish a
  price for Codex Auto Review." (#913).
- **Editing the first message counts as a reopen.** Editing the first message
  of a saved conversation (Esc Esc) now sends its warm-up as a reopen warm-up,
  as editing a later message already did: `/cost` lists it as "Reopen
  warm-up", Corbanu says once it has been sent, and `skip_reopen_warmup = true`
  skips it (#1038).
- **Realtime-only threads can skip the warm-up.** `skip_startup_warmup = true`
  (in `config.toml`, with `-c`, or in `thread/start`'s `config`) sends no
  startup warm-up, new or reopened. App-server clients that start a thread
  only for `thread/realtime/start` should set it: the warm-up on the default
  model could cost several times the realtime session (#1045).
- **Realtime sessions show their tokens.** A realtime WebSocket session now
  records the input, cached-input, output and total tokens OpenAI reports at
  the end of each response, and `/cost` shows them. Its cost stays unknown
  (`no price available`, never $0): Corbanu has no realtime price table, so
  no rate is applied even when the session's model has a catalogue price.
  Sessions recorded earlier, and WebRTC calls, still show their tokens as
  unknown (#889, D223).
- **A day with an unrecorded request says its totals may be incomplete.**
  When accounting fails to record a request (the turn already warns, for
  example with a clock far behind the ledger), Corbanu now keeps a small
  marker for that conversation and the UTC days the request could have been
  sent on, and `/cost` for such a day says "Totals for this day may be
  incomplete" under its heading (a range or period that includes it says so
  too). A home's ledger moves to format 7 the first time this happens, so
  older builds then treat it as written by a newer build and leave
  collection off. A home with no ledger yet gets none: its `/cost` already
  shows as unavailable (#314, D228).

## Moderate: waiting on and starting agents after untrusted content

- **Waiting on an agent never asks** (D320, #1653). On Z.AI and other
  providers without namespace tools, `wait_agent_v1` used to ask (or, with
  approvals off as in `corbanu exec`, was refused) as "a tool the security
  level has not classified" once untrusted content had entered the session,
  so a parent couldn't wait for its child. It now goes ahead, as `wait_agent`
  already did.
- **Starting an agent asks, on every route.** After untrusted content under
  Moderate (and Aggressive), `spawn_agent` (and `spawn_agent_v1`,
  `spawn_agent_plaintext`) asks for your approval, and the prompt says it is
  "a request to start a new agent". With approvals off it is refused. This is
  new on the OpenAI Responses route, where `spawn_agent` used to go ahead.
- **The `_v1` agent aliases act like their usual names** (D334, #1702). On
  Z.AI, Chat Completions and Claude-style routes, `send_input_v1` and
  `close_agent_v1` now go ahead after untrusted content, as `send_input` and
  `close_agent` do; before, they asked (or were refused with approvals off) as
  unclassified tools.
- **Restarting a finished agent asks, like starting one** (D334). After
  untrusted content, `resume_agent` (and `resume_agent_v1`) asks for your
  approval on every route, and so does a MultiAgentV2 `followup_task` (and
  `followup_task_plaintext`) to an agent that isn't running, since it starts a
  new turn there; the prompt says it is "a request to restart a finished
  agent", and with approvals off it is refused. A follow-up to an agent with a
  turn running goes ahead. Before, `resume_agent` and `followup_task` went
  ahead on every route that offers them, and `resume_agent_v1` asked as an
  unclassified tool.
- **`send_input` to an agent that isn't running asks too** (D388, #1832).
  After untrusted content, V1 `send_input` (and `send_input_v1`) to a
  finished or idle agent starts a new turn there, so it now asks like
  `followup_task`; with approvals off it is refused. To a running agent it
  still goes ahead.
- **Interrupting an agent that isn't running no longer wakes it** (D404,
  #1796), at every security level. `interrupt_agent` on a finished or idle
  agent only reports its status; before, the interrupt notice started a new
  turn there, with the caller's reason as its message.
- **Interrupting a just-started agent stops its first turn again** (D435).
  For an agent `spawn_agent` just started, whose first turn hasn't begun,
  `interrupt_agent` sends the interrupt, which stops that first turn, but not
  the notice that would start another. Since D404 the interrupt was not sent
  at all, so the first turn ran, and the reply wrongly said the agent "was not
  started". A first turn held back by the agent limit, the kill switch or a
  signed-out account is not cancelled and runs later; the reply says so. The
  interrupt can also cancel that agent's MCP server startup, so a held-back
  first turn may later run without its MCP tools. An idle agent reloaded
  without a turn still only gets its status reported.
- **Known limit** (D389): a follow-up or `send_input` to an agent that is
  running when checked goes ahead; if that turn ends just before the message
  arrives, the message starts a new turn without asking. Likewise, at any
  level, if an agent's turn ends just before an interrupt reaches it, the
  interrupt notice can start a new turn.
- **The same rules apply under Aggressive** (D335).
- **An interrupted agent message no longer comes back when the agent is
  reloaded** (D485, #1952), at every security level. If an interrupt stopped
  a turn before it had recorded the message it was working on (for example a
  just-started agent's first message), the message stayed marked as handed
  over, and reloading the agent later (after it was unloaded, or on quit and
  resume) handed it over again and started a turn on it, even from a
  `send_message` that never starts one. Such a message is now cancelled when
  the turn stops. Messages the turn did record were already never replayed;
  recovery after a crash is unchanged.
- **An agent message that arrives while a turn is stopping is no longer
  lost** (#2010). A turn that was being stopped (interrupt, kill switch,
  guardian, a replacing turn, or quit) could still take a queued agent
  message in its last moments without noting it, so the message was neither
  delivered nor cancelled. It now stays queued for the next turn, or for
  recovery after quit and resume.
- **A turn stopped by automatic approval review no longer takes the next
  turn's input** (#2046). Guardian stops a turn in the background and waits
  briefly for it to finish. If you started a new turn in that moment, the
  stopping turn could take the new turn's steering messages and queued agent
  messages. Only the turn that is running now takes them.

## Agent tools act only on the caller's own agents

- **A sub-agent's `list_agents` shows only itself and its own agents**
  (D320, #1756, #1767, thanks to @citadelculture). With MultiAgentV2,
  `list_agents` used to list every live agent, including the caller's parent
  and siblings and other root threads' agents; it now lists the caller and
  the agents it started (and theirs), loaded or not. This is the intended
  tightening of D320. `close_agent` and `send_input` (and their `_v1`
  aliases) likewise refuse an agent the caller didn't start, directly or
  through its own agents.
- **`wait_agent` waits only for the caller's own agents** (#1744, #1701,
  thanks to @citadelculture). `wait_agent` (and `wait_agent_v1`) could wait
  for any loaded agent, including another root thread's, and return its final
  status. It now refuses an agent the caller didn't start, directly or
  through its own agents, with "wait_agent can only wait for agents spawned
  by this agent or by its sub-agents". With MultiAgentV2, `wait_agent` used
  to report every live agent with its last result; it now reports only the
  caller's agents.
- V1 `close_agent` and `send_input` check the saved ownership of registered,
  unloaded agents before closing or loading them. A loaded thread's actual
  parent is checked again before input or interruption.
- V2 `list_agents` no longer mistakes another session's `/root` entry for the
  caller. V2 `wait_agent` includes the caller's unloaded agents in its summary,
  as `list_agents` does.
- Agent shutdown leaves hidden internal threads with their lifecycle owner.

## `/security` wording

- **Revoke all names its real scope.** After "Revoke all active authority",
  the confirmation now says it was revoked "in every session of this Corbanu
  Terminal on this Corbanu home", as its review says, instead of "in this
  session". A saved kill switch, on or off, also says that other Corbanu
  Terminals on the home take it within about a second (#1035).
- **A stale inspector view is not called degraded.** When the inspector's facts
  are more than 30 seconds old and nothing else is wrong, its top line reads
  "◌ Moderate set; this view is stale (status observed 33s ago)" instead of
  "▲ Moderate degraded"; press `r` to refresh. A stale view still never shows
  the green "Protected" badge (#1036).

## `/security`: windows without a session yet, and remote app servers

- **Moderate before the window's session starts.** Confirmed in a window whose
  session hasn't started, Moderate's Saved screen no longer says the session
  stays Permissive until a restart: that session starts under Core's Moderate
  rules; the credential broker still waits for the restart (#1047).
- **Remote app server windows.** In a window whose sessions run on an app
  server outside Corbanu Terminal (a remote server or the local daemon),
  `/security` no longer says the session starts here, says the level reaches
  that server only if it runs on this Corbanu home, and the level picker
  doesn't offer "restart now", since Corbanu Terminal starts against such a
  server only under Permissive (#1047).
- **Revoking there no longer offers a restart that is refused.** After "Revoke
  all active authority" or a kill-switch change in such a window, the grants
  and kill-switch view no longer offers `r restart now`, and says why:
  `Restart now is not offered: this window's sessions run in an app server
  outside this Corbanu Terminal.` (#1284, #1094, thanks to @citadelculture).

## Aggressive: contained Claude panes and the macOS keychain

- **Contained Claude panes follow the stricter level right away** (#807). A
  contained pane is now kept away from the macOS keychain services as soon as
  Aggressive is saved in `/security` or takes effect in the live session,
  before any restart, as agent commands already were. On Linux, a pane's
  user-namespace rule (#891) follows the live level the same way.
- **Approved commands stay in the sandbox under the keychain rule.** On macOS
  the keychain rule by itself now keeps escalated and allow-listed commands in
  the sandbox. Aggressive already kept them there through its other rules, so
  nothing you see changes. Permissive and Moderate are unchanged.
- **Claude panes and `/permissions` also follow Core's live level** (#1468).
  Claude panes (outside containment) and permission changes are refused while
  the live session runs a protected level (Aggressive, for `/permissions`),
  not only when it is active or saved. On macOS, `/status` no longer says
  approved commands can leave the sandbox when a permission profile keeps the
  keychain away (they can't).

## Named accounts (behind `named_accounts`)

- **App servers in homes with only the default login no longer wait for a
  conversation.** With named accounts on, a standalone app server or daemon
  held back its own ChatGPT requests (apps, plugins, rate limits, the model
  catalog) until a conversation started, even in homes without named
  accounts. It now does that only when the home has a named OpenAI sign-in
  account or `[provider_accounts]` selects one. A server started without one
  starts holding them from the first conversation on an account added while it
  runs, and never goes back to the default login (#1085).
- **`/status` names the account only where named accounts exist.** With named
  accounts on, the provider line read "OpenAI · account default" (or
  "Z.AI · account default") in every home. It now names the account only when
  that provider has a named account in the home, or the session is on or used
  one; otherwise the line is what it was without the feature (D86).
- **`/status` no longer reads the vault itself.** The summary of a named
  ChatGPT sign-in account (kind, email hint, plan) is now read in the
  background and cached. Until it is known, `/status` shows just
  `account <name>` and doesn't treat the account as removed. A sign-in change
  made by another process shows on the next `/status` (each one starts a
  refresh) (#1572, #1251).
- With `[features] named_accounts = true`, `corbanu account` can list, add,
  rename and remove accounts for a provider. Secret values for `account add`
  are read from stdin. The account list shows names and kinds, not credentials.
  `/providers` also exposes named-account management (#409, #430).
- By default, resuming a conversation preserves its recorded provider and
  account. An explicit `--account` for the same provider overrides the recorded
  account, including a removed account. Explicit model/provider overrides
  (`-m`, `-c model_provider`, or a profile model) use the session's selected
  provider. Without an override, a missing or removed named account requires
  recovery rather than silently using the default login (#566, #1299).

## `corbanu login status`

- **No part of an API key is shown.** With an API-key login it used to print
  the key's first 8 and last 5 characters. It now prints
  `Logged in using an API key - API key configured`, plus
  `(fingerprint <12 hex>)` when the home's vault has its fingerprint salt. That
  is the same per-home fingerprint a named OpenAI account holding the key
  shows (REG-02).

## Custom providers on OpenAI's regional API hosts

- **Multi-agent turns work on more OpenAI hosts.** A custom
  `wire_api = "responses"` provider whose `base_url` is
  `https://<region>.api.openai.com/v1` (`ae`, `au`, `ca`, `eu`, `gb`, `in`,
  `jp`, `kr`, `sg`, `us`) or `https://api.openai.com./v1` no longer gets every
  turn refused with HTTP 400 `collaboration.followup_task is reserved` (#863,
  #820). Its provider id, credentials and billing are unchanged. Proxies and
  gateways in front of OpenAI still get the provider-neutral tools.

## Task Node evidence receipts

- **An evidence receipt that confirms nothing now fails.** If Task Node answers
  `corbanu tasknode task evidence` or `verification respond` with success but
  no event id, transaction hash or CID, the command exits 1 with
  `task_evidence_receipt_unconfirmed`, the original receipt and
  `evidenceGateOpen` (whether the task still accepts that evidence). It used to
  exit 0 (#1280, reported by @0xzoz).
- **Confirmed receipts name what was recorded.** `pfterminalLifecycle` now has
  `submissionRef` (event id, transaction hash and/or CID), reports an empty
  `currentVerificationRequest` (`{}`) as `null`, and only says the initial
  evidence gate is closed when the refreshed task says so. If a task stays
  submitted with no verification request, report the task id and
  `submissionRef` to Task Node operators rather than cancelling it.

## Moderate and Aggressive: other git worktrees' `.git` files stay read-only

- **Agent commands can no longer rewrite another linked worktree's own `.git`
  file** (#675). The protected-path rules already kept every linked worktree's
  folder in the shared repository read-only (#650). Each worktree's own `.git`
  file, which says where its git folder is, now stays read-only too, so it
  can't be pointed at a planted git folder whose config would run code in your
  later git commands there. Git in the current worktree and in the main
  repository works as before. `git worktree repair` of another worktree now
  fails for agent commands as well, as `git worktree move` and `remove` already
  did. Permissive is unchanged.
- **The same holds for a submodule's linked worktrees** (#1468): the `gitdir`
  in `.git/modules/<module>/worktrees/<name>` and the `.git` file it names stay
  read-only too, and a `gitdir` spelled `.GIT` counts where the volume ignores
  case.

## GLM 5.2 on Z.AI and Ambient: "medium" now runs as Standard

- **A GLM 5.2 session set to `medium` now runs at Standard, as its label
  says** (#1266, #1270, thanks to @citadelculture). On the Z.AI (`zai`)
  `glm-5.2` and Ambient `z-ai/glm-5.2` routes, an explicit
  `model_reasoning_effort = "medium"` (the catalogue's **Standard**, chosen in
  Spawn or set in config) used to be rewritten to `xhigh` when the config
  loaded or a session resumed, so it quietly ran as Deep. It now stays
  Standard. Deep and an unset effort behave as before. A session that already
  ran as Deep under an earlier build resumes as Deep; pick Standard again. The
  Corbanu Plan route for GLM 5.2 still maps `medium` to `xhigh` (#1476).

## OpenRouter Owl Alpha is retired

- **`openrouter/owl-alpha` no longer works** (#128, #1486). OpenRouter lists no
  endpoints for it (checked 2026-10-28), so a request to it is expected to fail
  at OpenRouter. A config that saved this ID still resolves to its own hidden
  entry and is never switched to another model; its description now says it is
  retired. Pick a current model with `/model`. The earlier "explicit legacy IDs
  remain compatible" note no longer covers this ID. The other retained IDs
  (Grok 4.6 and 4.5, DeepSeek V4 Pro and V4 Flash 0731 on OpenRouter, and
  GPT-5.5 on OpenAI) are still served and unchanged.

## OpenRouter Tencent Hy3 Free is retired

- **`tencent/hy3:free` is no longer listed in `/model`** (#1658, #1662, thanks
  to @bradsbots). OpenRouter lists no endpoints for it (checked 2026-11-07), so
  a request to it is expected to fail at OpenRouter, and it is no longer
  allocated automatically. A config that saved this ID still resolves to its
  own hidden entry and is never switched to another model; its description now
  says it is retired. Pick a current model with `/model`. The paid
  `tencent/hy3` is not in the catalogue.

## Remote work: every network refusal ends the command as failed

- **A network request the remote environment's proxy refuses on its own now
  ends the command as failed**, with the reason, as it does locally (#722,
  D278). This covers blocked sites, local or private addresses, requests
  limited mode doesn't allow, and requests to sites not on the allowlist when
  no managed network rules are set. Before, these were refused but the
  command could still end normally.
- **Only stricter:** the remote side can only report a refusal; nothing it
  sends can allow a request. Remote environments running an older version
  don't send these reports, and behave as before.
- **Hidden characters can't disguise a remote site's name** (#722): bidi
  controls and zero-width characters are removed from the site named in a
  remote refusal, and a remote request whose site name contains them is
  refused without asking.

## Chat Completions and Anthropic: earlier MCP calls keep their own name

- **An MCP tool's earlier calls no longer replay as a built-in's** (#688). On
  providers without tool namespaces, a call recorded for an MCP tool that the
  request no longer offers (its server was removed, or the 128-tool limit left
  it out) used to replay under its bare name, such as `shell`, which a
  built-in tool can share. It now replays under the MCP tool's flat name
  (`mcp__server__tool`), or a hashed form if another tool has that name.
  MCP servers configured without the `mcp__` prefix get the same treatment
  (`server__tool`); only built-in tool namespaces keep the bare name.
- **MCP servers without the prefix can't take a built-in's namespace** (#1655).
  With `features.non_prefixed_mcp_tool_names`, a server that would be exposed
  without `mcp__` under a built-in tool namespace (`web`, `clock`, `skills`,
  `collaboration`, `image_gen`, `memories`, `multi_agent_v1`, or the configured
  multi-agent namespace) is now rejected when the config loads and by
  `corbanu mcp add`, with a message saying how to rename it. Skill dependency
  install skips such a server, and plugin servers keep the `mcp__` prefix.
  This includes servers from a trusted project's `.codex/config.toml`: with the
  feature on, such a project's config doesn't load until the server is renamed
  or left out of `server_names`. Disabled servers are skipped unless they are
  listed in `server_names`.
- **Nor can they take a namespace the Responses API reserves** (#1666). The
  same check now also refuses `api_tool`, `browser`, `computer`, `container`,
  `file_search`, `functions`, `multi_tool_use`, `python`,
  `python_user_visible`, `submodel_delegator`, `terminal` and `tool_search`,
  the names `features.multi_agent_v2.tool_namespace` already can't use.
  App-server config writes and MCP imports from other agents apply it too.
  `corbanu mcp add` now suggests another name to add (#1679) instead of
  `corbanu mcp remove` for a server that isn't configured. The warning for a
  skipped skill dependency now says which kind of reserved namespace it would
  take and no longer suggests "limiting" `server_names`.
  An MCP import from another agent that skips such a server now says it was
  not imported and suggests adding it under another name, instead of
  `corbanu mcp remove` (#1681, #1738, thanks to @citadelculture).

## `NO_PROXY` IP ranges apply to WebSocket connections

- **WebSocket connections now honour IP-range entries in `NO_PROXY`** (#722,
  D247), such as `172.16.0.0/12`. A connection to an IP address inside a
  listed range now goes direct, as HTTP requests already did, instead of
  through `HTTP_PROXY`/`HTTPS_PROXY`. Host names are matched by name only (they
  are not resolved to check ranges), as before.

## A timed-out network prompt for remote work closes

- **The approval prompt for a remote network request now closes when the
  request times out** (#722, D279). An unanswered request is still refused
  after 10 minutes (plus the hook timeout) and the command still ends as
  failed; before, its prompt stayed on screen and answering it did nothing.
- **App-server clients:** the server resolves such a pending
  `item/commandExecution/requestApproval` request itself and sends
  `serverRequest/resolved`, as it already does when a turn ends. Any command
  approval whose waiter in the core goes away unanswered is resolved the same
  way.

## Interrupting a tool on a busy machine keeps its elapsed time

- **The model now always sees how long an interrupted tool ran** (#1467,
  #1448). On a loaded machine, an interrupted turn could miss its 100 ms grace
  period. The interrupted tool's output (`Wall time: N seconds` /
  `aborted by user`) was then replaced by a bare `aborted`. That output is now
  recorded in this case too, timed to the moment of the interrupt.
- **A tool that had already finished keeps its real result** (#1536, D244).
  In the same rare case, a tool that completed just before the interrupt (for
  example a patch that was applied) was reported to the model as aborted. Its
  real output is now recorded instead.

## Windows sandbox: commands get a temp folder of their own (D377, #304)

- **The default (elevated) Windows sandbox no longer lets sandboxed commands
  write anywhere in your `%TEMP%`.** Each Corbanu session makes a folder of
  its own there (`corbanu-sandbox-tmp-…`), open only to you, SYSTEM and
  Administrators, and its sandboxed commands get it as `TEMP` and `TMP` (and
  as `TMPDIR` when that named `%TEMP%`). Only that folder is granted to the
  sandbox. Before, the sandbox's accounts kept lasting write access to all of
  `%TEMP%`: other programs' temp files, and other sessions'.
- **Upgrading needs no setup rerun.** The first time a session uses
  `%TEMP%`, it removes the grant older versions left there, as you (no UAC
  prompt). This also removes the non-admin sandbox's entry for that Corbanu
  home from `%TEMP%` itself (it adds it back when it next runs). Where you
  cannot change `%TEMP%`'s permissions, the old grant stays and is logged.
  Running an older version again puts it back.
- **A sandboxed command cannot turn its temp folder into a link** to make
  a later setup grant the sandbox somewhere else: the session keeps a file in
  the folder open, so it cannot be emptied or renamed, and checks the folder
  is still the one it made before each command.
- **Cleanup.** The folder is removed when the session ends; one a crashed
  session left behind is removed by a later session.
- On native Windows CI, in an administrator's and a normal session, Python
  (temp files, venv and pip), cargo, Node and npm, Windows PowerShell and
  PowerShell 7, git and Git Bash, and MSBuild work as before.
- **What older versions' sandboxed commands left in `%TEMP%` becomes yours**
  (#1871). The sandbox's accounts owned those files and folders, and an
  owner can change their permissions: a command could give its account,
  the sandbox's group or its workspace entries there that later commands
  carry, and keep changing them after the upgrade (shown on native Windows
  CI). In the same upgrade step, before the old grant is removed, the
  session, as you (no UAC prompt), makes you their owner and removes every
  allow entry other than yours, SYSTEM's, Administrators' and OWNER
  RIGHTS' (deny entries stay; a "no DACL" becomes one inherited from the
  folder). Nothing is deleted. One whose permissions stop you from doing
  that is renamed in place (`….corbanu-sandbox-leftover-…`), so programs
  that look for it by name make a new one of yours. The counts are logged
  in the Corbanu home's `.sandbox` log. A typical session leaves several
  hundred (Node's `node-compile-cache`, `MSBuildTemp`, temp files tools
  kept); on CI all were taken in under a second.
- Not changed, and accepted (D444): the non-admin sandbox still uses
  `%TEMP%` itself; sandboxed commands can still read (not write) other
  sessions' temp folders; and running an older version puts the old grant
  back until the next session of this version removes it. `docs/sandbox.md`
  now lists these limits.

## Windows non-admin sandbox warning

- **The warning now matches what the non-admin sandbox does** (#1506, D264). It
  no longer says commands can delete or move files you own outside your
  workspace; since #705 they can't, except in low-integrity places. It now says
  what it doesn't stop: reading your files, network access by programs that
  ignore proxy settings, and a few ways of starting programs outside its job.

## `corbanu resume --last` reopens the conversation you quit from

- **`resume --last` now opens the conversation that was on screen when you
  last quit** (#1412, D243). Opening an older conversation with `/resume`
  doesn't change its "updated" time until you send a turn, so `resume --last`
  used to reopen the conversation with the newest turn instead. Corbanu now
  records the conversation on screen when you quit and reopens it, as long as
  it is in the same scope (folder, provider, not archived) and no conversation
  there got a newer turn after you quit. Otherwise it still picks the newest
  turn, as before. The `/resume` list order and `fork --last` are unchanged.

## Corbanu API with no key: `/status` and first-run setup

- **`/status` no longer claims a missing Corbanu API key.** Under Moderate or
  Aggressive, with Corbanu Plan chosen but no key, `/status` said
  `Corbanu Plan API key configured` while `/providers` said not configured. It
  now says `Corbanu API not configured (set it up in /providers)` (#1563).
- **First-run setup for a chosen Corbanu Plan opens the provider list.** It used
  to open an OpenAI-worded key prompt that ended in
  `Provider API-key selection is no longer available`. The list now starts on
  **Corbanu API**, whose wallet and API-key setup runs after **Done**.
- **The same holds with `forced_login_method = "api"`.** With Corbanu Plan as
  the provider, setup now opens on the **Corbanu API** row (only that row and
  **Done**) instead of the OpenAI key prompt (#1584, #1571, thanks to
  @bradsbots).

## A force-stopped turn still reports its file changes

- **The turn's file-change summary is sent when an interrupt force-stops a
  turn** (#1604, D282). When a turn misses the interrupt grace period and is
  stopped, it now sends the summary of the files it changed, as a turn that
  ends normally does, so the status line stays current.
- **A turn stopped while recording a search result still marks the
  conversation as using outside content** (#1536). The check now runs before
  the result is recorded instead of after it.

## Telegram `/status`: token estimates are not a thread total

- **`/status` no longer shows unknown usage as a free thread.** After
  compaction or rollback on a thread whose provider never reported usage, it
  said `Thread total: 0 tokens (0 in, 0 out)`. It now says
  `Thread total: not reported by the provider`, and `Context` shows the token
  estimate (#1570, #1586, thanks to @sendoeth).
- **A full context window is shown as context, not as the thread's cost.**
  When the window is exceeded, `Context` shows the full window and
  `Thread total` stays the provider-reported usage, computed from the
  provider's counters, which the fill now keeps (#1583; #1586, thanks to
  @sendoeth). When the first update the bridge sees for a thread is a fill
  (for example after a bridge restart), `Context` shows the full window if
  the thread had already reached it (#1638), and otherwise what the fill
  added.
  After compaction, `Context` shows core's token estimate, and `/new` starts
  the new thread without the previous thread's total.

## Usage totals survive a full context window

- **A full context window no longer resets the thread's usage totals**
  (#1583, #1577, thanks to @bradsbots). When a request or a compaction hit the
  context window, the thread's input, cached-input, cache-write, output and
  reasoning counters were reset to zero, so `corbanu exec --json` usage, the
  TUI's `/status` and the app server's `thread/tokenUsage/updated` totals then
  counted only usage after the fill. They now keep the earlier reports. The
  total token count is still set to the full window at a fill, and the context
  indicator is unchanged.

## Chat Completions: replies stay messages

- **A JSON answer that mentions `"function_call"` is a message** (#1582,
  #1581, thanks to @bradsbots). Such a reply used to be read as a broken tool
  call and fail with `malformed function-call JSON as assistant text`, then be
  retried, even after it had been shown. Only JSON whose top-level `type` is
  `function_call` or `custom_tool_call` counts as a call.
- **A reply that opens with such JSON and goes on in prose is a message**
  (#1659, #1654). It used to fail with `trailing characters` after five
  retries.
- **A reply that opens with JSON-like text that does not parse is a message**
  (#1728, #1660, thanks to @bradsbots), for example `{"function_call": auto} …`.
  It used to fail with `malformed function-call JSON` after five retries. A
  broken reply that names a tool-call `type` still counts as a broken call and
  is retried.
- **Text already shown stays a message** (#1656, #1608, thanks to
  @bradsbots), even if it turns out to parse as a tool call.
- **A `null` usage counter no longer drops its chunk** (#1589, #1588, thanks
  to @bradsbots). When that chunk was the last one, the reply lost its last
  text, its finish reason and its usage, so a reply cut off at the output limit
  ended as if it had finished normally. A `null` counter now reads as 0 for display.

## Vercel GLM 5.3 output caps

- **`zai/glm-5.3` and `zai/glm-5.3-flash` on Vercel now cap output at
  128,000 tokens**, Z.AI's documented maximum, as the Z.AI and OpenRouter rows
  already do (#1585, #1567, thanks to @bradsbots). They were 1,000,000 and
  131,000.

## Agents and reviews

- **A message handed to a running agent is delivered once** (#1579, #1545,
  thanks to @bradsbots). If recovery ran just after a live sender handed a
  message over, it could send the same message again.
- **A review cancelled before it starts stays cancelled** (#1574). A `/review`
  or automatic approval review whose cancellation came before its session
  started could still start it.
- **A detached review runs with its conversation's current security
  settings** (#2015). It used to start from the settings the app server
  loaded when it started, so it could miss changes made since: a Windows
  sandbox set up later, a sandbox or approval policy chosen for the
  conversation or changed during it, or a stricter `/security` level confirmed
  during the session. It now uses the sandbox, approval policy, Windows
  sandbox and security level the conversation is running with, as an inline
  review already did.

## Moderate and Aggressive: more command lines ask after untrusted content

After the agent has read untrusted content, these lines used to run without
asking and now ask first:

- `rg --hostname-bin=PROG`, which runs a program (#1600, #1573, thanks to
  @bradsbots).
- zsh's `=name`, which names the program `name` on `PATH`, as a redirection
  target (`echo x >> =mytool`; #1602, #1573, thanks to @citadelculture), and
  as a writer's operand (`tee -a =mytool`) or in `eval`, `trap` and `alias`
  text (#1651, #1620).
- Writes to a relative file such as `git log --output=.zshenv` or
  `git config -f .zshenv …` when the folder it lands in can't be known: after
  `cd "$X"` or another `cd` the check can't follow, after a wrapper's folder
  option (`sudo -D ~`, `env -C ~`, `docker exec -w ~`), or in a function, trap
  or loop that runs after a later `cd` (#1599, #1309, thanks to @bradsbots;
  #1651, #1606, when an option value spelled `git` comes before the wrapper's
  folder option).
- `git grep -O<pager>` and `--open-files-in-pager`, which run a program on the
  matching files (#1527, #1516). On Windows, also git named by its full path
  (#1527) and git's dashed helpers (`git-grep.exe`, `git-clone.exe`) (#1598,
  #1528).
- `-O`, `-u`, `-c`, `--open-files-in-pager`, `--exec`, `--template` and
  `--config` after a git subcommand that isn't one of git's own commands,
  since an alias or an autocorrected typo can be `grep`, `clone`, `init` or
  `push` (#1598, #1528, thanks to @Holic101, with parts of
  @wizbubba1's #1597), and program-running options after built-ins that older
  gits don't have, such as `refs` or `hook` (#1647, #1635).
- git commands cut short by an unquoted bracket the shell reads as text
  (`git -C { -c core.pager=sh log`; #1525, #1515), and git options that brace
  expansion produces (`git clone r d {-q,--upload-pack=./x}`; #1644, #1636).
- zsh's `>!`, `>>!`, `&>!` and `>&!` clobber redirections, read as writes
  (`echo x >! ~/.zshrc`; #1640, #1607).
- Commands in `eval`, `trap` and `alias` text, which now get every check
  the same command gets on a line of its own (D341, #1698, #1685): writes
  to shell start-up files (`trap 'tee -a ~/.zshrc' EXIT`,
  `eval 'tee -a ~/.zshrc'`), output options (`curl -o`, `time -o`), sending
  a file (`curl -d @f`), recursive reads of your home, scripts it runs
  (`./x.sh`, `source`, `python3 x.py`), a wrapper's folder option
  (`eval 'sudo -D sub ./run.sh'`), and, in an alias, git settings that run
  code. Before, only some of these were checked there. A `cd` in such text
  isn't followed: the folder it leaves counts as unknown. Ordinary text such
  as `trap 'rm -f x' EXIT` or `alias ll='ls -l'` stays quiet.
- Later uses of variables set in such text
  (`eval 'F=~/.zshrc'; tee -a "$F"`), settings there of variables that run
  code (`eval 'export GIT_PAGER=sh'`), and code written into the shells'
  tables of code run by name (zsh's `functions`, `commands` and alias
  tables, bash's `BASH_ALIASES` and `BASH_CMDS`), which is now checked as a
  line of its own (`functions[f]='tee -a ~/.zshrc'; f`; D341, #1698,
  #1685, #1704). Ordinary settings such as `eval "x=1"` or
  `aliases[ll]='ls -l'` stay quiet.
- A word the check can't see into (`$o`, `$(cat f)`) where a command that
  can write code into those tables still reads its options:
  `o=$(cat f); zstyle $o :x y functions`, `zstyle -a $(cat f)`,
  `zselect -t 0 -r 0 -A $n`, `zsystem flock -t 0 -f $n file` (#1704).
  `print`, `printf` and `getopts` are now read this way too. So an
  unquoted variable, `$(…)` or pattern where `print`, `printf`, `set`,
  `read`, `mapfile`, `vared` or `getopts` read their options or the
  name they write can now ask (`print -r $(cat f)`,
  `set ?A commands ls ./x`, `read -d $(cat f)`,
  `getopts $(cat f)`), and so can a quoted one with more words after it
  (`printf "$fmt" x`). `print -r -- $x`, `set -- $(cat f)`,
  `printf "$fmt"`, `print -l *.png` and `wait $pid` stay
  quiet.
- zsh's `stat` builtin, which can write those tables (`stat -A functions
  x`), now asks only when the same action may load it: a line of it runs
  `zmodload` or names `zsh/stat` (`zmodload zsh/stat; stat -f %m $(cat f)`
  asks, before or after the `stat`), holds a brace range, or runs bash's
  `enable` or a command named by a variable, command output or a pattern.
  zsh starts without it. Elsewhere `stat` is the system program, so macOS
  `stat -f %m $f` stays quiet. A module loaded by your zsh start-up files,
  or by an earlier command in the same long-running shell session, isn't
  seen (D410, #1704). `zstat` is always checked.
- A program named by a path, or a shell or interpreter, run after a leading
  redirection (`>log ./x`, `2>log sh x`, `2>&1 ./x`, `<<EOF ./x`,
  `sudo >log ./x`, `env >log ./x`), also in `eval` text (`eval '>log ./x'`),
  where the check used to read the redirection as the command (#1719,
  #1482, thanks to @darkmatter-sudo). `ls >log` and `2>/dev/null ls` are
  read as before.
- `sudo -A`/`--askpass`, `sudo -e`/`--edit` and `sudoedit`, which run a
  helper or editor named by the environment, also in `eval` text
  (`eval sudo -A ./x`) (#1719, #1482, thanks to @darkmatter-sudo).
- A command given as an argument list whose elements up to the program
  don't each read as one word (`["true ./x"]`, `["true;./x"]`,
  `["env", "A=1 ./y", "./x"]`) (#1719, #1482, thanks to @darkmatter-sudo).
  A shell's `-c` script is read as before. This doesn't apply under Windows
  path rules, where program paths hold spaces.

One line asks less: `git add src/{a,b}.rs && git commit -m "msg"` no longer
asks just because a quoted message follows the brace word (#1580, #1514,
thanks to @grapemarketer). A line with many `sed` commands and long output
names is now checked within a fixed size budget; past it, its output files
count as one unknown file, the cautious reading (#1587, #1132, thanks to
@0xzoz).

These lines also ask now:

- A pattern the shell expands to file names, where `tar`, `sed`, `rsync`,
  `rg` or `sort` still reads options (#1920, #1912). A file named like an
  option, such as `--checkpoint-action=exec=sh x`, turns `tar cf a.tar *`
  into a command that runs a program, so these lines now ask:
  `tar cf a.tar *`, `sed -i 's/a/b/' *.md`, `rsync -a * dest/`,
  `rg x *`. For `rg` and `sort` only a pattern that ends in a wildcard
  counts (`rg x *.rs` stays quiet). Patterns that can't start with an
  option name stay quiet (`./*`, `src/*`, `--exclude=*.log`), and so does
  a pattern after a `--` that ends the options (`tar cf a.tar -- *`; in
  `tar cf -- *` the `--` is the archive's name, so that line asks).
- `git fetch` and `git pull` now ask before using an unresolved remote after
  an option containing literal, quoted or escaped braces. Those option values
  could previously hide the remote from the approval check (#1975, #2005).
- The same runners (`tar`, `sed`, `rsync`, `rg`, `sort`) given a file name
  the line doesn't show, in two forms (D484, #2006, #1912): a loop variable
  taken from such a pattern (`for f in *; do tar rf a.tar "$f"; done`, also
  `select` and zsh's `for f (*)`), and `xargs` fed by `ls` or `find`
  (`ls | xargs tar cf a.tar`, `find . | xargs tar cf a.tar`). These lines
  now ask. `for f in ./*`, `find … -print0 | xargs -0` and a `--` before the
  names (`xargs tar cf a.tar --`) stay quiet. Other values the line doesn't
  show, such as `$(git ls-files)` or `while read`, are left as before.

## Moderate and Aggressive: no git `ext::` transport after untrusted content

- **Commands run after untrusted content get git's `protocol.ext.allow=never`**
  (D447, #1876, #1910). It is set in the command's environment
  (`GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>`, `GIT_CONFIG_VALUE_<n>`), after any
  entries you set there yourself, and `ext` is dropped from a
  `GIT_ALLOW_PROTOCOL` list. It outranks every git config file, so an
  `ext::` remote stored in `.git/config` or `.gitmodules` can't run its
  command even where the repository's own config allows `ext`. It is an
  extra layer under the command checks, and needs git 2.31 or later. It
  doesn't stop `git remote-ext` or `--upload-pack`, which run a program
  without the `ext` transport (the checks from #1897 ask for those), nor
  config git reads after it: `git -c protocol.ext.allow=...` or
  `--config-env` on the line, or `GIT_CONFIG_PARAMETERS`. A
  `GIT_ALLOW_PROTOCOL` list that held only `ext` is left empty, which
  allows no git transport. A command session started before the untrusted
  content keeps its environment. Git variables that only your shell's
  startup files set (and so only the shell snapshot restores) aren't seen:
  a `GIT_ALLOW_PROTOCOL` set there keeps `ext`, and the entry may take the
  place of a `GIT_CONFIG_KEY_<n>` set there.

## Bare git repositories a command plants

- **New advice in `docs/sandbox.md`** (#1775). A sandboxed command can make a
  bare repository in any folder it may write to, and git run there or in a
  folder below it later runs what its config names. Corbanu Terminal
  doesn't check for these, on Windows or elsewhere. The guide recommends
  `git config --global safe.bareRepository explicit` (git 2.45 or later)
  and explains what it doesn't cover.

## Windows sandbox

- **The default (elevated) sandbox's commands can't break away from their
  job** (#1639, D256, #1507). Cancelling or a timeout now ends every process
  they start in their job; a program that asks to break away gets "access
  denied" unless it retries without the flag. The non-admin sandbox already
  worked this way.
- **Default (elevated) sandbox: each command gets its own identity** (D324,
  #1717, #1648), as non-admin sandbox commands already do. Commands running
  at the same time, for example in different projects, can no longer write
  to each other's processes or the helper processes that start them, or
  start programs under them. Projects are still not fully kept apart: a
  command can still borrow another command's access and write in its
  folders (#1560), or, by racing another command's start-up, run code
  outside its own sandbox restrictions (#1721), and can still read and end
  the other's processes. So still avoid running commands
  in several projects at once (see `docs/sandbox.md`).
- **Commands in the default sandbox can no longer read a non-admin sandbox
  command's memory** (D429, #1803). A default (elevated) sandbox command
  could read the memory of a non-admin sandbox command running at the same
  time, such as secrets in its environment, its output and what was typed at
  its prompts. Other accounts (except SYSTEM and administrators using debug
  rights) can now only wait for a non-admin command's processes. One
  non-admin command can still read another's memory (#1838), so still avoid
  running commands in several projects at once.
- **Profiles the Windows sandbox can't enforce are refused, and say why**
  (D331, #1762). A profile with a writable folder inside a read-only one is
  now refused for the agent's commands in the non-admin sandbox too, as it
  already was in the default sandbox and for app-server `command/exec`; in
  our measurements this changes the outcome only under Permissive. On both
  sandboxes the message now names both folders, notes that the protected
  `.git`, `.codex` and `.agents` folders count as read-only, and says to
  change the profile so that no writable folder lies in a read-only one. In
  the non-admin sandbox, `command/exec` (and the exec server) now refuse a
  profile that blocks or limits reading files with a clear message that
  says how to switch to the default sandbox, instead of an unclear error
  from the sandbox.
- **Non-admin sandbox: a working folder reached through a junction no
  longer bypasses a read-only folder** (#1780, #1792). With the working
  folder given as a junction, a read-only folder given by its real path was
  writable, and so was a read-only folder spelled through a junction inside
  a working folder given by its real path. Read-only paths are now matched
  to writable folders by their real (link-resolved) path too. When one
  profile marks the same folder read-only under one spelling and writable
  under another, read-only wins.
- **A `\\localhost` network working folder gets a clear error** (#1863).
  `command/exec` refused a working folder given as a `\\localhost\...`
  network (UNC) path with an unclear "not valid on this host" error (debug
  builds crashed). It still refuses it, in both sandboxes, now saying to use
  the folder's local path instead. Other network paths are unchanged: the
  command starts sandboxed as before.
- **A writable folder inside a read-only one is refused through links too**
  (#1801, D331). D331's refusal compared folders by spelling, so a
  writable folder that lay inside a read-only one only through a junction
  or folder symbolic link (on either folder, or on the working folder)
  wasn't refused, and the non-admin sandbox let commands write there. It
  now compares real paths too (`subst` drives included), for the agent's
  commands in both sandboxes and for `command/exec`. The same folder
  marked read-only and writable under two spellings is still not refused;
  in the non-admin sandbox read-only wins (#1780). Newly refused for the
  same reason: a protected `.git`, `.codex` or `.agents` folder that is a
  link to a folder holding another writable folder of the profile. Replace
  the link with a folder; see "A writable folder inside a read-only one" in
  `docs/sandbox.md`.
- **Non-admin sandbox: `command/exec` runs from a `subst` drive, an 8.3
  short name or a differently cased path** (#1766, D398).
  It (and the exec server) refused such a working folder with "cannot enforce split writable root
  sets directly", because it compared the folders by spelling. It now
  compares their real paths, so these commands run as they would from the
  real path, with the same read-only folders. A working folder that leads
  out of the workspace is still refused. The agent's own commands and the
  default (elevated) sandbox weren't affected.
- **Non-admin sandbox: `command/exec` refuses a working folder on a network
  with a clear message** (D446). A mapped network drive or a UNC path
  (`\\server\share\...`), as the command's or the sandbox's working folder,
  is refused with "... is a network folder (a UNC path or a mapped network
  drive)", for any sandboxed permission profile, read-only included. In
  #1866's measurement the non-admin sandbox's commands couldn't write
  through a network path, so such a command ran as if the folder were
  read-only. Use a folder on a local drive. `\\localhost` paths were
  already refused (#1863). The agent's own commands and file tools, the
  exec server and the default (elevated) sandbox are unchanged.
- **Non-admin sandbox: a writable folder inside a protected `.git`, `.codex`
  or `.agents` folder is refused for older workspace-write profiles too**
  (#1810, D331). A profile in the older workspace-write format, for example
  with a writable root at the workspace's `.git\hooks`, skipped D331's
  check, so the agent's commands and `command/exec` ran and could write
  there. Such profiles are now refused, as the default sandbox already
  refused them. Naming `.git` itself writable is still allowed.
- **Non-admin sandbox: nested `.codex` and `.agents` folders are watched**
  (#1642, D283, #1541). One that appears or is replaced deeper in your
  workspace while a command runs isn't loaded until you confirm it, like one
  at the top. Commands can't rename, move or delete existing ones or the
  folders above them, so a `git checkout` or `npm ci` that does is refused
  there.
- **Non-admin sandbox: a `.git` that appears or changes is reported**
  (D332, #1682). When a `.git` at the top of a writable folder is new, or
  was replaced, when a command ends, Corbanu Terminal warns at startup and
  the TUI asks you to confirm it once you have checked its hooks and config:
  git runs them outside the sandbox. Commands can still create a `.git`
  (`git init` works). A `.git` that is a link is reported after every
  command. `.git` folders deeper inside aren't checked.
- **Non-admin sandbox: possibly planted folders are reported where you see
  them** (D424, #1775). A `.git`, `.codex` or `.agents` folder a command may
  have planted is now named in a warning right after that command ends (for
  one that leaves processes running, after a later command), not only at the
  next start. At startup, a flagged `.git` is shown (and the TUI
  asks about it) when it lies in any folder commands may write to,
  `%TEMP%` included, or below the working folder; before, only one in the
  working folder or a folder above it was.
- **Non-admin sandbox: nested and bare repositories are watched** (D420,
  D421, #1775). Repositories deeper in a writable folder (a nested `.git`,
  or a folder that is a bare repository), and a writable folder that is
  itself a bare repository, are reported like the top-level `.git` when they
  are new or replaced, or when their config, hooks or other files git runs
  programs from change (compared by content, submodules and worktrees
  included). The warning says what changed. A new repository with exactly
  the shape `git init` or `git clone` leave isn't reported, so cloning a
  project to look at it doesn't ask; later changes to it are. Nothing is
  refused: agents can still commit and run `git config` there. Going back
  to an older build drops these records the next time it saves its own.
- **Non-admin sandbox: `.codex`/`.agents` below a folder whose name isn't
  valid Unicode count as planted** (#1877). The folder holding a folder with
  such a name is flagged after every command, so what lies below it isn't
  loaded until you confirm it; before, nested folders below it weren't
  recorded at all. (When that is the workspace itself, its own
  `.codex`/`.agents` count too until the folder is renamed or removed.) Such a
  folder now gets one "isn't valid Unicode" note, not one per entry, so many
  such names no longer push earlier warnings out of the post-command notice.
- **Non-admin sandbox: the repository checks cost about half as much**
  (#1877). The checks for planted `.codex`/`.agents` folders and nested
  repositories that run at each command's start and end now use several
  threads. On a 38,000-file tree with 100 repositories (native Windows CI,
  4 CPUs) each side went from about 80-90 ms to about 40 ms. They find
  exactly the same things, in the same order.
- **Non-admin sandbox: hooks folders a git config names are watched** (D422,
  #1775). A `core.hooksPath` folder in the project (Husky, `.githooks`), an
  included config file, or a script a program key names, when it lies in a
  writable folder, is recorded by content; a new or changed file there is
  reported like a changed repository. Nothing is refused: Husky still
  installs inside the sandbox.
- **Default (elevated) sandbox: possibly planted folders are reported too**
  (D425, #1846). The `.codex`, `.agents` and `.git` folders, repositories
  and hooks folders the non-admin sandbox watches are now watched in the
  default sandbox as well: when a command ends, what it created or changed
  there gets the same warning right after the command, at startup and in the
  TUI prompt. Nothing is refused. Processes a command leaves running aren't
  waited for.
- **Default (elevated) sandbox: a change inside an existing nested `.codex`
  or `.agents` folder is reported too** (#1896). The non-admin sandbox
  keeps such a folder (below the top of a writable folder) unchanged; the
  default sandbox doesn't protect it, so it now records what is inside each
  one when a command starts and reports the folder, like a planted one,
  when its contents differ after the command. Nothing is refused. A folder
  too large to compare (over 20,000 entries or 64 MiB) is reported after
  every command.
- **Default (elevated) sandbox: a `.git` made for a read-only `.git\hooks`
  is protected from the first command** (#1758). At the top of a writable
  folder (other than `%TEMP%`) with no `.git`, a configured read-only
  `.git\hooks` makes the sandbox create `.git` before the command starts.
  The command that caused this could still write `.git\config`, `HEAD` or
  `objects`; only later commands found `.git` protected. Now that first
  command can't either, as in the non-admin sandbox. The same holds for a
  `.codex` or `.agents` folder made this way. A deeper one, such as
  `sub\.git` for `sub\.git\hooks`, still has only `hooks` protected.
- **Default (elevated) sandbox: starting a command no longer holds up
  other work** (#1896). Its preparation, which includes walking the
  writable folders for the planted-folder records (about 0.06–0.08 s on a
  38,000-file repository, 0.5–0.7 s on a `repo` client with 100 projects) and
  the credential watch's first scan, ran on the thread that drives other
  tasks; it now runs on its own. What is flagged is unchanged.
- **Non-admin sandbox: starting a command no longer holds up other work**
  (#1909). Its preparation (the command's token, the folder permissions
  and labels, and the planted-folder walk) and the start of the command
  itself ran on the thread that drives other tasks: in our measurements
  each command held it for about 2–4 s on a 38,000-file repository and
  4–8 s on a `repo` client with 100 projects. They now run on their own,
  in the same order; other tasks wait at most a few tens of milliseconds.
  What is flagged is unchanged.
- **Both sandboxes: the planted-folder watch is a little faster in a `repo`
  client** (#1909). Each git folder in `.repo\projects` is now looked
  through once per command start or end instead of twice (once as a
  repository, once through its work tree's `.git` file), with the same
  result: about 3% off each (0.51 s to 0.49–0.50 s with 100 projects).
- **Default (elevated) sandbox setup no longer fails now and then where
  Windows requires complex passwords** (#1776). Setup gives the sandbox's
  user accounts a fresh random password, and about one setup in 4,500 drew
  one without the mix of characters that Windows' "Password must meet
  complexity requirements" policy needs (the Windows Server default, and
  common under domain policy), so setup failed with
  `helper_user_create_or_update_failed`. Passwords now always hold
  upper-case and lower-case letters, digits and symbols.
- **Non-admin sandbox: links can't undo a read-only path** (#1645, D293,
  #1542; #1670, #1667). A command doesn't start when a configured read-only
  path, or a folder above it, is a link (junction, symbolic link or mount
  point) inside a writable folder, which a command could replace; the message
  says to configure the link's target instead. A read-only path spelled
  through a writable folder's link gets the same denies (#1645). Cloud
  placeholder folders (OneDrive) at or above a read-only path, or as one, now
  get the same denies as plain folders, and a writable root that is a
  placeholder gets its label back (#1671, #1668).
- **Non-admin sandbox: a configured read-only path that doesn't exist is
  created and protected** (#1687, #1686, #1705; D333, D342), as the default
  sandbox already did. Before the command starts, it's made as an empty
  folder, so a command can't create it and fill it. A path meant to be a
  file (`.env`, say) therefore becomes an empty folder, and a read-only
  `.git\hooks` in a folder with no `.git` creates a protected `.git` with
  only an empty `hooks` folder, which git doesn't treat as a repository. To
  keep a file there, create it before you start a session. See "Read-only
  paths that don't exist yet" in `docs/sandbox.md`.
- **Non-admin sandbox commands start faster in large workspaces** (#1612,
  D285). Their delete protections are now written without rewriting the
  permissions of every file below (each is still checked, and anything
  unexpected falls back to the old method). In the PR's measurement on a
  101,000-file repository, a command took about 20 s instead of 31 s.
- **Non-admin sandbox: the faster protection walks don't start at a link**
  (D352, #1754, #1722). When the folder a walk starts at is itself a
  junction or symbolic link, or is swapped for one as the walk starts, it
  now leaves that folder to Windows' own walk, so the result always matches
  what Windows does. Corbanu doesn't normally start a walk at a link, so
  nothing changes in normal use.
- **Known limitation, now documented: the non-admin sandbox breaks tools that
  use named pipes** (#1797, #1561, #1778). Its commands can't write to
  named pipes they create, so Node.js child processes with piped output
  (`EPERM`), Git Bash (`couldn't create signal pipe`), a local `git clone`
  and Python `multiprocessing` fail there. This is not new in this release.
  The default (elevated) sandbox runs all of them; see "Known limitations"
  in `docs/sandbox.md`.
- **Known limitation, now documented: non-admin sandbox commands can write
  in `C:\ProgramData\Microsoft\DeviceSync`** (#1770). Windows labels that
  folder Low and lets everyone write to it. This is not new in this
  release, and we accept it; don't keep files there that you rely on.
- **Known limitation, now documented: default (elevated) sandbox commands
  can read other projects' running commands** (#1803). A command can read
  the memory of another project's command running at the same time
  (secrets in its environment, what it prints) and, for one run in a
  terminal, its console host (what you type and what it prints). It can't
  change them. This is not new in this release. A change awaiting security
  sign-off (#1800) stops the memory read, and a follow-up stops the console
  read; until then, avoid running commands in several projects at once.
- **Default (elevated) sandbox: a command that leaves a process running
  returns when it exits** (#1830). A process started with the command's
  output handles (PowerShell's `Start-Process -RedirectStandardOutput`, for
  example) kept the command "running" until that process exited too, with no
  time limit. Now one-shot commands and unified exec sessions in the default
  sandbox finish at most about 2 seconds after the command exits, and the
  process it started keeps running; output that process writes later is not
  shown. The non-admin sandbox's one-shot commands behave this way since
  #1826, and its unified exec sessions now do too (#1861).
- **A default (elevated) sandbox command now starts while a non-admin
  sandbox command is running** (#1819). Before, when the non-admin command's
  writable folders held the Corbanu home (for example `%TEMP%`, or a project
  folder above it), the elevated command failed after 15 seconds with
  `timed out after 15000ms connecting runner pipe-out`. The folder with the
  sandbox's helper programs now keeps its normal (Medium) integrity level.
  If that folder is a link, it is left alone and the sandbox log says so. If
  copying the helper fails and the one next to `codex.exe` is used instead,
  a Low label on it is fixed the same way, or the log names it as the cause.
- **Windows PowerShell's `Start-Process` works in both sandboxes** (#1805).
  Corbanu passes commands your environment, where Windows spells the search
  path `Path`, and both sandboxes added a second copy spelled `PATH`. Windows
  PowerShell 5.1's `Start-Process` then failed with "Item has already been
  added", and the copy that counted could miss folders added by
  `shell_environment_policy.set`. Each variable now reaches a command once,
  and a value you set wins over the inherited spelling; no folder is added
  to or removed from what you configured. Where Corbanu sets a proxy
  variable in both cases (`HTTP_PROXY` and `http_proxy`), sandboxed
  commands get only the capitals, so tools that read only the lower-case
  name (MSYS2 or Git Bash `curl`) no longer see that proxy.
- **Remote and exec-server commands on Windows also get each variable once**
  (#1817). The exec-server added Corbanu's variables beside the ones it
  inherited with exact case, so an inherited `Term` next to Corbanu's `TERM`
  reached the command twice and either value could count. Corbanu's value
  now replaces every inherited spelling, as in the sandboxes since #1805.
- **The planted-folder watch reads only the file it checked** (#1909, both
  sandboxes). When the watch records a file's content (inside a nested
  `.codex` or `.agents` folder, in a repository's git folder, or a file a
  git config names), it now opens the file it checked itself, so a link
  swapped in after the check is not followed. Before, a process a command
  left running could swap the file for a symbolic link in that moment, and
  Corbanu would read through it, for example from a network share. Such a
  swapped file is now recorded as unreadable, so the change is flagged. A
  link that is already in a git folder is still read through, as git
  reads it. A file Windows can't read as itself (for example a placeholder
  whose provider isn't running) now counts as changed after every command.
- Ctrl-C now reaches legacy Windows sandbox terminal commands even when Corbanu inherited Ctrl-C suppression from its launcher, including elevated sessions. A sandboxed console helper resets only the command's inherited state; tokens, access controls and Corbanu's own signal handling are unchanged.
- **Non-admin sandbox: confirming a flagged folder or repository covers
  only what the prompt showed** (D486, #1877 item 5). If another command
  planted or changed the same `.codex`/`.agents` folder, `.git` or
  repository while the startup prompt was on screen, confirming used to
  clear that later change too. Now confirming it is refused with a note,
  it stays flagged, and Corbanu Terminal asks about it again.
- **Non-admin sandbox git watch: fewer false alarms, bounded cost, and
  `\\?\C:\` paths in configs** (D486, #1877). A project inside `%TEMP%` no
  longer gets "links to a folder too large to check" after every command
  for links back into the project itself (pnpm's `node_modules`). A single
  huge folder behind a link into `%TEMP%` is no longer listed in full: past
  20,000 entries it counts as too large, as several folders already did.
  Git configs over 16 MB in all count as changed after every command,
  like configs past the other limits, and nothing more is read past any
  of those limits. A hooks folder, include or script a
  config names as `\\?\C:\...` is now watched as `C:\...` (`\\?\UNC\`
  network paths are still never opened).
- **Non-admin sandbox git watch reads a global git config on a network
  share** (D517, #1877). When your home folder, and so your `.gitconfig`,
  is on a network share (`\\server\share`), the hooks folders and scripts
  it names are now watched too. For a config on a share or mapped network
  drive, the sandbox waits at most about 2 seconds per check; if the share
  doesn't answer in time, or the read fails, the configs count as changed
  after every command instead of holding the command up longer. Include
  paths such a config names on a share or mapped network drive are never
  opened. Configs on this machine are read as before.
- The planted-folder watch stops reading a git file if it grows past the size
  observed on its open handle and flags it as changed. Unchanged files keep
  their existing fingerprints.
- Cancelling a non-admin command while it starts now terminates and waits for
  its job using the normal exit cleanup before restoring folder labels.
- **The planted-folder watch lists only the folders it checked** (#1909,
  both sandboxes). The watch now opens each folder below the writable
  folders, and each folder and file inside a nested `.codex` or `.agents`
  folder, through the folder it was found in, following no link. Before, a
  process a command left running could swap a folder for a junction or
  symbolic link after the check, and Corbanu would list what it points to,
  which could be a network share. Such a folder is now recorded as
  unreadable, so it is flagged.
- **Default (elevated) sandbox: the planted-folder watch has a total limit
  across nested `.codex` and `.agents` folders** (#1909). It already stopped
  looking through one such folder past 20,000 entries or 64 MiB of files. It
  now also stops past 100,000 entries or 256 MiB across all of them at a
  command's start or end. As with the per-folder limits, the folders it
  can't finish are flagged after every command; the command itself is never
  blocked.
- **Non-admin sandbox: a command cancelled while starting no longer leaks
  handles** (#1909). If a command was stopped while its preparation or start
  was still running, Corbanu kept the command's token open and, once the
  command had started, its process and thread handles, until Corbanu exited.
  The command itself was already ended. These handles are now closed too.
- **Default (elevated) sandbox: a test now covers a runner that can't
  start** (#1909). It checks that the error is reported and that the
  planted-folder watch ends on its own thread, not on the one that drives
  other tasks. No behaviour change.
- **Checked: the planted-folder watch doesn't follow a file re-marked as a
  link just before it reopens it** (#1909, both sandboxes). On Windows
  Server 2022 and 2025 and on Windows 11, reopening a file from its handle
  refuses to follow a link, so the opening raised in review isn't there. A
  new test keeps it that way. No behaviour change.
- **With network access off, git uses only local `file` transports** (D475,
  #1921). Sandboxed commands now get `GIT_ALLOW_PROTOCOL=file`, replacing
  any value they had, so git refuses `https`, `ssh`, `git://` and `ext::`
  remotes before connecting, while clones and fetches from a local folder
  or `file://` URL still work. The setting was misspelled
  `GIT_ALLOW_PROTOCOLS` before, which git ignores; the other no-network
  measures still applied.
- **Local-file submodules remain allowed** (D550 (b), #2155). The same
  allow-list permits recursive clones and submodule updates to copy local
  repository objects from files the sandbox can already read. Adding
  `protocol.file.allow=user` does not narrow it: git checks the allow-list
  before config (tested with Git 2.54.0.windows.1). This accepted limit keeps
  direct local clone, fetch and push available; #2156's alternative would
  refuse them too. See [Windows: git with network access off](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/docs/sandbox.md#windows-git-with-network-access-off).

Sandbox launches that need integrity or planted-project records now refuse existing records owned by an untrusted account, including files left by older elevated sandbox commands. The elevated watcher no longer silently disables tracking for this refusal. Keep the records for inspection and follow the recovery instructions in `docs/sandbox.md`; changing their owner does not make their contents trustworthy. D599 retains the approved `tsx-*` cache naming policy.

- The elevated setup helper refuses redirecting links in deny-write paths. It creates missing folders relative to checked handles and applies ACLs to the opened object, so a junction swap cannot redirect creation or the deny to its target. Non-redirecting reparse points, such as OneDrive folders, remain supported (#1953).
- The one-time temp-cache cleanup leaves ordinary reparse-point files in place. Only directories and links that redirect name resolution count as caches; the existing cache-name list is unchanged (#1953, D456).
- **Refused typing shows in the security inspector** (#1976 follow-up). On
  Windows without a Windows sandbox, Moderate and Aggressive refuse typing
  into a running command; the security inspector now lists that refusal like
  the refused commands and file accesses.
- **Moderate and Aggressive refuse agent commands and file tools on Windows
  without a Windows sandbox** (#1976, D491). Commands Windows treats as
  read-only, such as `Get-Content`, and file tools, such as image viewing,
  used to run there with no sandbox and could read the files these levels
  protect. Now each is refused, and so is typing into a running command,
  with a message naming the level and how to set the default sandbox up
  (`/permissions`, a mode that uses the sandbox, "Set up default sandbox");
  a notice says so when the session starts. The session still
  starts, so you can set the sandbox up or switch to Permissive. This applies
  however the level was set: any config layer, a level saved with
  `/security`, or Aggressive enforced because the saved state could not be
  read. Permissive, the non-admin sandbox's existing refusal and the default
  sandbox are unchanged.

Nested `.codex` and `.agents` folders in a workspace on a volume mounted inside another folder no longer count as changed after every command solely because of that mount. The watch recognizes mounts and junctions to an entire local volume by its GUID, then checks the recorded volume and file identity before reading metadata contents. Changed files and replaced folders are still flagged. Junctions to other paths, symbolic links and anything that cannot be verified retain the conservative warning.

- **"Set up default sandbox" now applies to the session you are in** (#2000).
  After setting the Windows sandbox up from `/permissions` in the TUI, the
  running session used to keep running agent commands without it until you
  started a new one with `/new`. Now it uses the sandbox from your next
  message, child agents started after that included, and under Moderate or
  Aggressive the "needs the default Windows sandbox" refusal no longer
  applies. A turn still running finishes as it started, and commands started
  before the setup keep running without the sandbox; under Moderate and
  Aggressive typing into them stays refused. The server reads the sandbox
  mode from its own config, never from the client, and a running session's
  sandbox can only get stronger: a weaker one (for example the non-admin
  sandbox after the default one) applies from the next session. If the
  session can't switch, the TUI says so.

Windows elevated-sandbox launch failures with error 267 now explain that the working directory is invalid or inaccessible to the sandbox account. The error remains a launch failure, with no credential refresh or retry outside the sandbox. Documentation covers the full-access limitation at Moderate/Aggressive; this change does not fix that limitation (#2021).

- **Moderate and Aggressive refuse work on a remote Windows machine without a
  Windows sandbox** (#2050, D510). #1976's refusal only looked at the
  computer Corbanu Terminal runs on, so a Mac or Linux computer driving a
  remote Windows machine (an exec-server) with no Windows sandbox ran
  commands and file tools there unsandboxed. The exec-server now reports its
  operating system and whether the default Windows sandbox is set up there,
  and under Moderate or Aggressive agent commands, typing into running
  commands and file tool access on such a machine are refused, naming the
  level, the environment and the setup steps for that machine. An
  exec-server too old to report is refused as well unless it clearly runs on
  Linux or macOS (an older Linux one whose default shell is PowerShell is
  refused too; update it). Permissive, remote Linux and macOS machines, and a Windows
  computer sending work to a remote Linux machine (#2049) are unchanged.
- Sandbox setup now waits for confirmation before reporting “Sandbox ready”
  for the running conversation. A failed or unconfirmed switch gives `/new`
  recovery guidance, and an error is no longer hidden by an unconfirmed retry.
  The message clarifies that already-running work keeps its earlier sandbox.
  A disabled sandbox is described as confirmed settings, without claiming readiness,
  and rejected updates show the server's reason.
- Selecting a weaker sandbox explains that it applies from the next session,
  without attributing the restriction to an unspecified configuration source.
- Detached reviews refresh the missing-sandbox notice for the security level
  inherited from their conversation.
- Moderate and Aggressive now carry the Windows sandbox refusal through
  executor skill/plugin discovery and skill resource reads, including full-disk
  permission profiles and catalogs captured before the live level changed.
  Remote discovery checks the executor before sending roots. The receiving
  executor also rejects marked file operations when the requested default
  Windows sandbox is not set up. Ready remote environments without a sandbox
  also get a startup notice explaining the refusal and setup steps. Permissive
  behavior is unchanged.
- When the non-admin sandbox can't give its records folder
  (`%LOCALAPPDATA%\CorbanuTerminalSandbox`) its private integrity label, the
  refusal now names the Windows error and what the folder needs (Full control,
  including `WRITE_OWNER`, which a user's own `AppData\Local` grants), instead
  of only "label … Medium". Commands are still refused in that case (#2584).

## Windows sandbox: capability SIDs are recorded under a lock (#1871)

- **Two Corbanu processes on one Corbanu home no longer lose each other's
  sandbox capability SIDs** when they record them at the same time. Each
  update of the home's `cap_sid` file now holds a lock (`cap_sid.lock`) from
  reading the file to replacing it, and the sandbox audit no longer rewrites
  the file. Before, the later write could drop a SID the other process had
  just recorded and granted, and two first uses could each make a different
  set.
- An update waits at most 5 seconds for the lock, then goes ahead unlocked,
  as all updates did before. So a program that holds the lock (anything that
  can read the Corbanu home) can delay updates by 5 seconds and bring back
  the old race, but cannot stop them.

## Windows sandbox tests no longer pile up entries on `%TEMP%` (#1890)

- **Running the `codex-windows-sandbox` and `codex-core` Windows tests no
  longer adds lasting permission entries to your `%TEMP%`.** Each test that
  ran a non-admin sandbox command left one there for good (about 25 per run
  of the `codex-windows-sandbox` tests), and every file later made in
  `%TEMP%` inherited all of them. The tests now give those commands a temp
  folder of their own, and a `codex-windows-sandbox` test fails if one still
  adds an entry to the real `%TEMP%` or `%TMP%`.
- Not changed: the non-admin sandbox itself keeps one lasting entry on
  `%TEMP%` per Corbanu home (measured: one after many sessions in several
  projects). On a machine that ran the tests before, `icacls "%TEMP%"` lists
  the old entries as bare `S-1-5-21-…` SIDs with `(OI)(CI)(M)`;
  `icacls "%TEMP%" /remove:g *S-1-5-21-…` (with the `*`) removes one. Remove
  only such bare-SID entries. If one was your Corbanu home's, the sandbox adds
  it back the next time it runs.

## Security

Linux sandbox cleanup reads cached process-root identities without requesting a filesystem sync. It preserves successful-read ordering and post-read identity checks, and scans mount views without deduplication when an identity is unavailable.

`corbanu login` and `corbanu account` usage errors no longer repeat an
unexpected value that might be a key or token accidentally supplied on the
command line. Supply credentials through stdin; the diagnostic also reminds
you that an accidentally typed key may remain in shell history (#1301).

Protected-home listing explanations preserve paths containing a colon and space without attributing unrelated file denials to protected credentials or suggesting an unnecessary security downgrade.

- After untrusted content, additional shell-generated output targets and git options require confirmation: zsh `=name` writes through launchers and output-file options, targets made by braces, empty variables or default values, and literal git options made by quoted parameter replacements. Existing prompts remain, including the stricter checks for bash.

Protected-action classification now also inspects commands launched directly through
`chroot /`, and requests approval when a root, command or option cannot be judged.
Existing finder/sudo and brace-reading protections are retained. This does not
claim complete coverage of commands launched indirectly by other programs or
replace operating-system sandboxing. Thanks to @citadelculture for the contribution.

- Typed or pasted text cannot approve a pending request. A decision shortcut
  highlights its choice for confirmation with Enter; ordinary typing and
  pastes activate the typing guard (#187).

After untrusted content, ask for approval when quoted or substituted text is misread as a shell assignment that could hide an unseen git remote. Existing protected-action findings remain in force.

After untrusted content, inspect the shell code stored by `zstyle -e` before it can run during a style lookup. Deferred styles also count as possible loaders of zsh's `stat` builtin when guarding writes to shell code tables.

The command classifier recognizes brace groups next to shell control keywords
while preserving existing approval decisions (#2018).

- Update rustix 1.1.x to 1.1.6 to address RUSTSEC-2026-0486 (zero-length extended-attribute buffer initialization). The unaffected 0.38.x dependency and existing audit policy remain unchanged.

Update the network proxy's DNS dependency to patched Hickory while retaining existing Rama interfaces, resolver selection and network authorization policy. Remove two obsolete Hickory advisory exceptions; no new exception is added.

Post-taint shell checks no longer trust assignments that might be skipped or run
outside the parent shell. Conditional, pipeline, deferred-function and background
forms retain protected-action checks; possible literal protected-file paths remain
checked too. Nested background-group analysis uses bounded interval processing.
Thanks to @araupooo for the contribution and regression cases (#2133).

Command rules now also check commands run through `sudo`, `chroot`, and `env`, including nested wrappers. The strictest outer or inner decision applies, and unrecognized wrapper options require approval. This fixes wrapped commands bypassing prompt or forbidden rules (#2165, D553).

A saved allow rule for a wrapped command no longer bypasses the sandbox unless its inner commands are also explicitly allowed. Wrapped commands no longer offer an automatic “always allow” rule; existing rules can still apply.

- After untrusted content, Moderate and Aggressive now ask before shell commands, MCP file tools, or the image reader access project `.env` credential files. This restores the existing Moderate policy; Permissive is unchanged, and approval does not lift Aggressive's protected-file denials.

After untrusted content, shell commands no longer treat text following a comma
or non-shell whitespace as a reliable assignment when resolving a later command
argument (#2192). Thanks to @white-guy-01.

With secretless agent launch on, an executor that Corbanu starts over stdio from `environments.toml` now gets only the allowed environment (plus its own configured `env`), not Corbanu's full environment, so MCP servers and commands it starts can't read your keys and tokens from it. An executor started before secretless launch was turned on during a run is refused until you restart Corbanu. An `ssh`-based executor can't reach your SSH agent (`SSH_AUTH_SOCK`) while it's on (#2200).

- Command, file-change, and network approvals now refuse answers the prompt did not offer. Custom clients cannot turn a one-call approval into a session grant or substitute a broader command or network rule. Offered approval choices keep their existing behavior.
- Image generation now asks before reading protected reference files after untrusted content. Model-selected local image and audio attachments sent to child agents use the same approval and filesystem protections. Moderate requires approval; Aggressive retains its protected-file denials after approval. Permissive is unchanged.
- The existing forced-`rm` protection now recognizes abbreviated `--force` options (such as `--forc`) and ANSI-C or translated quoted flags (such as `$'-rf'`) at every security level. Ordinary non-forced removal keeps its existing behavior.
- Shell snapshots now omit variables containing protected credential values, including aliases and concatenations created by shell startup files. Filtering happens before the snapshot file is written; credential-name filtering remains in place.

With protected launch enabled, MCP servers started on an executor now apply the same inherited environment allowlist as local MCP servers. Host and remote `env_vars` cannot forward provider keys, Corbanu credentials or credential helpers. Servers that need a credential can still receive it through their own explicit `env` configuration.

With secretless agent launch on, MCP servers no longer inherit an environment variable whose value embeds a URL password (for example `HTTPS_PROXY=http://user:pass@proxy`), even when the name is allowed; other URLs, including ones with only a user name, pass through unchanged. This covers local stdio servers and servers started through an executor. Values in a server's own `env` setting are still passed as written. An older remote executor that cannot filter these values is refused for MCP servers while secretless launch is on; update it to the current version (#2324).

Secretless agent launch now also drops a variable whose value holds a proxy password without a scheme (for example `HTTPS_PROXY=user:pass@proxy:3128`) or a URL password in any URL of a list, not just the first. Values with no password, such as `user@host`, `host:port`, IPv6 addresses, email addresses, `git@github.com:org/repo` and `mailto:` addresses, still pass through. This applies to every protected launch: agent commands, hooks, executor launches and MCP servers. An older remote executor keeps the previous rule for values it reads itself until it is updated (#2395).

Command rules now also check commands run through `unshare` (for example `unshare -r chroot / …`), `nsenter`, `setpriv`, `timeout`, `nice`, `nohup`, `stdbuf`, `setsid`, `ionice`, `chrt`, `taskset`, `flock`, `doas`, and `runuser -u`, alone or chained with `sudo`, `chroot`, and `env`. A forbidden rule for the inner command now refuses it before any approval is offered, including the Moderate user-namespace approval, and a prompt rule's reason is shown in that approval. Unrecognized launcher options, and forms that run a shell string (`flock … -c`, `runuser` without `-u`), require approval (#2455, D670).

As with `sudo`, a saved allow rule for one of these launchers no longer approves the command it runs, and these commands no longer offer an automatic “always allow” rule.

Command rules now also check commands run through the shell builtins `command` and `exec`, `time` (`/usr/bin/time`), `caffeinate`, `systemd-run`, `pkexec`, `bwrap`, `firejail`, and `busybox` or `toybox` applets. A forbidden rule for the inner command refuses it, and a prompt rule asks with its own reason. `command -v`/`-V` only print, so they run nothing. Unrecognized options, `systemd-run --shell` or an `Exec…=` property, and `bwrap --args` require approval, as do `firejail` commands (older firejail runs them through a shell) and `systemd-run` commands with an abbreviated option or a `$` (#2455, D670).

As with `sudo` and `unshare`, a saved allow rule for one of these launchers alone (for example `time`) no longer approves the command it runs, and such commands no longer offer an automatic “always allow” rule.

Command rules now cover `xargs`, `su`, `sg` and `watch` (#2455, D676–D678). `xargs` applies rules to the command it runs: a forbidden rule that matches its fixed words (for example `xargs -n1 git push`) refuses it, and it requires approval whenever a forbidden or prompt rule could match once `xargs` adds words read from its input (for example `xargs git` or `xargs -I{} git {}`). `su` with a command (`su -c`, `--command`, `--session-command`, `-s`, or words passed to the shell), and every `sg` and `watch` command, require approval even when an allow rule covers them. Unrecognized `xargs` options require approval too. Allow rules alone do not trigger the extra input-dependent prompt (D686). `xargs sh -c`, `xargs env` and `xargs command -v` still require approval even without a prompt or forbidden rule (D687); input that could complete a forbidden rule asks for a human decision, while fixed words that already match it are refused (D691).

Shell snapshots omit copies of keys held by the model credential broker, including keys removed from the environment. If model credential protection is unavailable, including legacy scoped credential routes without a safe inspection operation, snapshot creation is refused.

Large shell snapshots can now be checked against isolated credentials without exhausting the old scan budget. The matcher shares work for keys of the same length and reuses exact windows already found clean, while preserving copied-key detection and refusing when protection is unavailable. The bounded scan supports 3 MiB snapshots with eight ordinary credential lengths.

On macOS and Windows, command rules now see through launchers whatever their spelling (#2503, D688). These systems find programs without regard to letter case, so `SUDO`, `/usr/bin/Env`, `XARGS` or `Su -c` run the same launcher as `sudo`, `env`, `xargs` or `su`. Rules for the command such a launcher runs now apply as they do for the lower-case name, for every launcher command rules cover (`sudo`, `env`, `chroot`, `unshare`, `timeout`, `nice`, `xargs`, `su`, `busybox` and the others). Windows also reads `sudo.exe` and `C:\Tools\env.exe` as these launchers. On Linux, where `SUDO` is a different program, nothing changes.

On macOS and Windows, `forbidden` and `prompt` command rules now match the program name in any letter case (D692, #2510). These systems find programs without regard to case, so `RM -rf x`, `Rm x` or `/bin/RM x` run `rm`, and a rule that refuses or asks for `rm` now applies to them, including behind launchers and inside shell scripts. `allow` rules still need the exact name, so they never approve more than before. Arguments still compare exactly. On Linux, where `RM` is a different program, nothing changes.

On macOS and Windows, command rules now also apply inside shell scripts whatever the shell's spelling (#2510, D688). `BASH -lc '…'`, `/bin/Sh -c '…'`, `ZSH -c '…'` and, on Windows, `Bash.exe -c '…'` run the same shell as `bash -lc '…'`, so rules for the commands in the script apply, also behind a launcher such as `SUDO`. Program names that could be a launcher or shell spelled in a way Corbanu cannot read with confidence now ask for approval: on macOS, names with invisible characters that older HFS+ disks ignore; on Windows, short names such as `BUSYBO~1.EXE` and stream names such as `sudo::$DATA`. On every platform, rules now apply to the command run by a `chroot` whose path looks like a shell assignment or comment (`a=b/chroot / git push`). On Linux, shell and launcher names are otherwise unchanged.

Command rules now see into a shell script nested in another shell script (#2531, D553): `bash -lc "bash -c 'git push'"`, `sh -c "zsh -c '…'"` and chains mixing launchers and shells such as `sudo bash -c "env bash -c '…'"` are checked against the rules for the commands inside, like `bash -lc 'git push'`. Up to eight nested scripts are read below each command; a command nested deeper asks for approval. No "always allow" is offered for such an outer script, since it could not approve the commands inside.

On Windows, rules now also see into PowerShell scripts whatever the spelling of the shell's name (#2524, D688): `PWSH -c`, `PowerShell.EXE -Command` and `C:\…\POWERSHELL.EXE -Command` are read like `pwsh -c` and `powershell.exe -Command`.

Command rules now read literal quoted words in shell scripts (#2543, D553, D688): `bash -lc "'git' push"`, `"git" push`, `g'i't push`, `'/usr/bin/git' push` and `echo hi; 'git' push` are checked like `bash -lc 'git push'`. Scripts that are more than plain commands (redirections, substitutions, loops, functions) are now read for prompt and forbidden rules too, as are `dash -c` and `ksh -c` scripts. Where a command is only known once the script runs (`$cmd push`, `"$(echo git)" push`, `/usr/bin/gi? push`), or a shell gets a script rules cannot read (`bash -c '…' name`, `fish -c '…'`), the command asks for approval whenever a prompt or forbidden rule could match it; with no such rule, nothing changes. Allow rules still never approve a script through this partial reading. Run-time launcher words such as `env X="$PWD/y" cmd` can also require approval; with approvals disabled, commands that need approval are refused. This is not complete shell interpretation: code supplied through `eval`, shell stdin, here-documents or sourced files remains a follow-up (#2548), as does a POSIX script passing a run-time word to PowerShell (#2585). See [command rules](../../../docs/execpolicy.md) for examples and limits.

On macOS and Windows, the built-in dangerous-command check now matches program names in any case (`RM -rf x`, `/bin/Rm -f x`, `SUDO rm -rf x`), as those systems run them (D692); the safe-command list is unchanged.

On Windows, command rules no longer start the PowerShell a command names to read its script (#2546, D553). A command such as `C:\work\pwsh.exe -Command "…"` used to have its script parsed by that same `pwsh.exe` before approval and outside the sandbox, so a file planted under that name ran unasked. Scripts are now read by the PowerShell installed on the system (`%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe`, `%ProgramFiles%\PowerShell\7\pwsh.exe`, or a matching installation found through absolute `PATH` entries outside the current folder and its descendants, including links or alternate spellings back into them). Where no such PowerShell can read the script, the command asks for approval whenever a prompt or forbidden rule could match it; with no such rule, nothing changes.

On Windows, `allow` command rules (including remembered "always allow" rules) now approve only the program name exactly as the rule spells it, as on macOS and Linux (D696, #2553). Before, Windows read a command's program by its lower-cased name without `.exe`, `.cmd`, `.bat` or `.com`, so an `allow` rule for `rm` or `git push` also approved `RM x`, `GIT.EXE push` or `C:\Program Files\Git\cmd\git.exe push`. Those spellings no longer receive approval from that allow rule; the remaining rules and safety checks decide whether to ask. When no `host_executable` lists the program, a path whose file name is spelled as in the rule (`C:\tools\git push` for `git`) is still allowed. When one does, only the listed paths are allowed, spelled exactly as listed, drive letter case included. Listing a path is a way to approve `git.exe` by path without a prompt, but note that a listing also stops `forbidden` and `prompt` rules for that name from applying to paths outside the list. `forbidden` and `prompt` rules still match the same Windows spellings as before. Existing rules files load unchanged, including their `match` and `not_match` examples; only which commands `allow` rules approve without asking has narrowed.

On Windows, command rules read a PowerShell command line the way PowerShell does (#2568, D553). PowerShell runs every argument after `-Command` as one script, so `powershell.exe -Command "git status" "; git push"` runs `git push`; rules used to see only `git status`, and now see both. `-EncodedCommand` scripts are decoded and read, and abbreviated parameters (`-c`, `-nop`, `-ec`, `/Command`, `--command`) are read as PowerShell reads them. A command line rules cannot read that way (an unknown parameter such as `-NoExit`, a script from stdin, `-CommandWithArgs` with arguments, a positional script) now asks for approval whenever a prompt or forbidden rule could match it, as an unread script already did; `-File` scripts are checked as their file and arguments and ask the same way. With no prompt or forbidden rule, this extra check adds no prompt. A separate known limit remains: when the parser is available but cannot reduce a complex PowerShell script to plain commands, only the outer command is checked, so an inner prompt or forbidden rule may not apply (#2556/#2583). The commands treated as safe to run without asking read the same script, so `powershell.exe Get-Content "x; Remove-Item y"` is no longer treated as a read-only command. The PowerShell that reads scripts is no longer taken from the current folder under another spelling (letter case, 8.3 short name, junction), or from a `%SystemRoot%`/`%ProgramFiles%` that is a network or device path.

## Models and providers

Grok 4.7 remains available for explicit selection, but is excluded from automatic allocation until its long-prompt pricing is represented correctly in allocation guidance.

- ChatGPT sign-in now negotiates the current Codex compatibility version for model discovery and inference, allowing GPT-6.2 Astra and Sol without a custom provider or API key. Older cached catalogs refresh automatically.

## Resume and recovery

An ordinary interactive resume automatically saves the resumed conversation's
model and provider as defaults in `config.toml`, along with its reasoning and
service tier. If the conversation has no reasoning effort, the saved reasoning
effort is cleared; an absent service tier is saved as `service_tier = "default"`.
Explicit invocation overrides remain scoped to that invocation. Profile or
explicit session service-tier overrides, non-embedded app servers and providers
absent from local configuration do not trigger this default update (#138).

When `resume --last` finds no resumable conversation, the TUI and `exec` now explain the lookup scope, including the model provider when filtered, and say that a new conversation is starting. `exec` also explains missing conversation names and other fresh-start fallbacks on stderr, keeping JSON output unchanged.

If onboarding switches providers after `resume --last` finds no conversation, the notice names the provider that was searched.

## Windows sandbox: elevated commands can't edit the planted-folder records (D453, #1896)

- **A command in the default (elevated) Windows sandbox can no longer change
  `%LOCALAPPDATA%\CorbanuTerminalSandbox`** when one of its writable folders
  holds it (for example, when your whole user folder or `AppData` is
  writable). That folder keeps the records of which `.git`, `.codex` and
  `.agents` folders a command may have planted, and which of them you
  confirmed. Before, such a command could edit those records, for example to
  mark a folder it planted as confirmed. The non-admin sandbox already
  denied this.
- The elevated setup now denies those commands writing, deleting or renaming
  anything there, making the folder first if it doesn't exist yet. Your own
  programs, and Corbanu Terminal itself, are not affected. Writable folders
  that don't hold it are unchanged.
- Files an older sandbox account created in this folder can retain that account's owner rights until the host replaces them. Recovery of those existing entries remains tracked in #1953; the deny alone does not remove owner rights.

## Windows sandbox: code caches in `%TEMP%` are renamed aside at the upgrade (D456, #1871)

- **At the one-time upgrade step that takes back older versions' sandbox
  grant on your `%TEMP%` (D377), Corbanu Terminal now also renames aside the
  code caches there**, so the programs that use them build new ones. Older
  versions let sandboxed commands write anywhere in `%TEMP%`, including caches
  that your own programs later run code from, whoever made them. Measured on
  a typical older session, 557 of the 571 entries its commands left in
  `%TEMP%` were in Node's `node-compile-cache`.
- What counts as a code cache, by name: at the top of `%TEMP%`,
  `node-compile-cache`, `v8-compile-cache` (and `v8-compile-cache-*`),
  `node-jiti`, `jiti`, `tsx-*` and `.net` (.NET single-file apps' extracted
  libraries); anywhere in `%TEMP%`, `__pycache__`. Each is renamed in place
  to `<name>.corbanu-sandbox-cache-<random>`. A link with one of these names
  is renamed itself, and what it points to is left alone. Nothing is
  deleted. The log of the first command after the upgrade counts them
  (`code caches N (renamed aside M, capped …)`). A cache in use at that
  moment may fail to rename. It is then left in place and counted.

## MCP OAuth requests no longer reach internal addresses (#1947)

- **A remote MCP server can no longer make Corbanu send OAuth requests to
  your local machine, private network or cloud metadata service.** OAuth
  discovery, client registration, token and refresh requests (and each
  redirect) now go only to public addresses, unless the MCP server's own URL
  is `localhost`, a `*.localhost` name, or a loopback or private IP; then
  those addresses are allowed too. Link-local and cloud metadata addresses (`169.254.169.254`,
  `fd00:ec2::254`) and metadata host names are refused unless the server URL
  is that very IP address. Host names are checked after DNS resolution, and
  the request connects only to the address that was checked. Before, an MCP
  server could point these requests at a host name that resolves to an
  internal address (blind SSRF). rmcp's own fix for this
  (RUSTSEC-2026-0457, rmcp 3.7+) does not cover the OAuth HTTP client
  Corbanu uses, whatever the rmcp version.
- Not changed: public MCP servers, and local ones named by `localhost` or an
  IP address, sign in as before.
- **Changed:** an MCP server named by a host name that resolves to a loopback
  or private address (an intranet name, `host.docker.internal`, a `.local`
  name, an `/etc/hosts` alias) can no longer use OAuth; sign-in fails with
  `MCP OAuth request blocked: ...`, and a server there that doesn't use OAuth
  shows auth status unknown with a startup warning. Using the server's IP
  address in `url` works where its certificate allows that. With an HTTP
  proxy, only IP literals and host names themselves are checked. Corbanu does
  not send OAuth requests through a remote executor from before this change
  (sign-in fails until it is updated). Details: `docs/config.md#mcp-oauth-addresses`.

## MCP servers that opt in to 2026-07-28: `server/discover` wait (#1950)

- **A stdio MCP server that opted in to MCP 2026-07-28
  (`CODEX_MCP_PROTOCOL_VERSION=2026-07-28`) and falls back to legacy is now
  started twice.** When it rejects `server/discover` as legacy, or doesn't
  answer it in time, Corbanu stops that process and starts the server again for
  the legacy `initialize`, so a late discover reply can no longer fail startup.
  The second start gets the startup timeout of its own, so a server that never
  answers `server/discover` can take up to about twice `startup_timeout_sec`.
  That timeout also covers stopping the first process and starting the second,
  so a remote executor that never finishes either can no longer hang startup
  (#2019). On a remote executor Corbanu doesn't wait for the first process to
  exit, so the old and new server processes can briefly overlap.
- **New `discover_timeout_sec` setting** for these servers: how long Corbanu
  waits for `server/discover` before it falls back. It defaults to, and is
  capped at, `startup_timeout_sec`, and is ignored for servers that haven't
  opted in.

## Project config can't move Corbanu's databases or logs (D483, #1945)

- **A project's `.codex/config.toml` no longer sets `sqlite_home` or
  `log_dir`.** Corbanu ignores them there and shows a startup warning naming
  them, as it already did for provider URLs, `notify` and `otel`. Before, a
  trusted repository could choose where thread history, memories, state and
  logs were written (into the repository itself, for example), and have
  Corbanu open state databases it shipped.
- **A project's `.codex/config.toml` no longer sets `[debug.config_lockfile]`
  either.** Replaying a config lock replaces the whole config, so a project
  could use one to set `sqlite_home`, `log_dir` or any other setting a project
  can't set (provider URLs, `notify`); it could also choose where lock files
  are written.
- Not changed: `sqlite_home` and `log_dir` in your user config, with `-c`, in
  managed config or requirements, and `CODEX_SQLITE_HOME`, work as before, as
  does `[debug.config_lockfile]` in your user config. If a project of yours
  set any of them, move them to your user config.

## Project agent roles follow the project-config denylist (D512, #1997)

- **An agent role a project declares can no longer set the settings a
  project's own config can't set (the project-config denylist).** This
  covers role files in a project's `.codex/agents/` folder and files named
  by `[agents.<role>] config_file` in a project's config. Before, a child agent spawned with such a role could get a different model
  provider or provider URL (where its requests and credentials go), `notify`,
  `otel`, `sqlite_home`, `log_dir` or `[debug.config_lockfile]`, even though a
  project's `.codex/config.toml` can't set them. Corbanu now ignores those
  settings in project roles, the child keeps its parent's values, and a startup
  warning names the ignored settings. A project role file also can't declare
  roles of its own for the child to spawn (`[agents.<role>]` entries), which
  would otherwise have reached a grandchild agent with those settings.
- Not changed: roles you declare yourself (in `CODEX_HOME/agents/` or your user
  config), roles from managed config and built-in roles. If one of your
  projects' roles chose a provider, declare that role in your user config.
- The other rules that apply to project config now apply to project roles
  too; see the D525 (#2052) entry.

## Project agent roles follow every project-config rule (D525, #2052)

- **Settings in an agent role file a project declares now follow the same
  rules as the same settings in the project's `.codex/config.toml`.** Before,
  a child agent spawned with such a role treated them as your own launch
  settings. Now:
  - hooks in a project role are listed as project hooks, keyed by the role
    file, and need your review before they run; a `[hooks.state]` table in the
    role file is ignored, as in project config;
  - an MCP server a project role defines or changes counts as a project's
    server: no Aggressive "always allow" record, and an approval isn't saved
    to your config;
  - a project role can't change a security level's feature defaults (for
    example `features.broker_model_auth`), skill settings (`[[skills.config]]`)
    or `project_root_markers`.

  A project role's `[security]` table already could only raise the security
  level, as in project config; that is unchanged.
- **`[agents] provider_allowlist` can be narrowed but not widened by a
  project.** A list in a project's config or in a project role only removes
  providers from the list your own, managed or `-c` config sets; it can't add
  any.
- Not changed: roles you declare yourself (in `CODEX_HOME/agents/` or your user
  config), roles from managed config and built-in roles.

- An empty provider list after project narrowing now also refuses crew records,
  matching the existing spawn refusal.
- The empty-list startup warning now describes project restrictions without
  assuming you configured your own provider list.
- Preflight identifies project role files by their source and path. App-server
  config layers and metadata report them as `projectRole` with the file path,
  keeping their existing precedence; actual launch overrides stay `sessionFlags`.

## Providers and models

- Refresh an open session model picker when provider recovery or activation finishes. Restored providers, including OpenAI, appear without reopening the picker; the current session and selected provider tab are preserved. Late results do not reopen a dismissed menu or replace a reasoning dialog (#2102).
- Provider rate-limit cooldowns now block locally for at most 60 minutes, including saved cooldowns from older versions. A request after expiry probes the provider: success clears the cooldown and backoff; another 429 starts a new bounded wait. TUI and exec errors show the remaining local wait and any longer provider reset hint. Use `/model clear-cooldown` or `corbanu -m MODEL -c 'model_provider="PROVIDER"' model clear-cooldown` to clear only the selected provider/model/credential's cooldown. CLI recovery honors `--account` for named credentials (#2567, D698).

## Dependencies and security

- Update the Rust OpenSSL dependency to 0.10.80 to address RUSTSEC-2026-0474, RUSTSEC-2026-0476, and RUSTSEC-2026-0477. No advisory exception was added (#2104).

The TypeScript SDK's development dependencies now use Zod 4.6.7, clearing
GHSA-mrhx-c7xc-m3w5. The structured-output sample uses Zod 4's built-in
JSON Schema converter, preserving required fields, enum values, and the
restriction on additional properties. The seven-day dependency release-age
policy is unchanged.

## Providers

Preserve compatible Anthropic providers’ optional thinking/reasoning-token breakdown in existing usage totals, including later stream updates that omit it. It remains part of output, never an extra billed bucket. The TPS display no longer adds the reasoning subset to total output a second time. Adapted from @goodalexander’s #28 contribution; synthetic fixture compatibility does not establish live provider support.

- Adapted @goodalexander's #88 recovery work: Anthropic signed-thinking rejection can be repaired once when it arrives after stream metadata but before any model output. Requests are not replayed after output begins, and plan-entitlement failures do not trigger credential refresh. Selected-account credential-file preservation is a separate follow-up, not included in this stage.

## Reliability

Remote MCP process handles remain eligible for cleanup when a startup timeout cancels an outstanding termination request. Successful termination still suppresses duplicate cleanup. Thanks to @bradsbots for the fix and cancellation regression test (#2132).

- Telegram polling now resets its consecutive-failure counter after every successful poll, including polls with no messages. Reaching the failure cap reports that cause instead of incorrectly claiming another bot poller is running.
- Completed Telegram handler failures now free their inbox slots and remain distinct from successful updates. A chat notice identifies the failed update and explains how to resend it after checking for actions that may already have completed. Crash-pending updates and unresolved queued messages remain available for restart recovery; mutating sends are never automatically retried.
- Telegram keeps its message and approval-button subscriptions when the first
  poll fails, then omits the subscription hint after a successful poll. Thanks
  to @bradsbots for the fix and regression test (#2576).

Concurrent turns coordinate omitted-tool notices so the same tools are not reported twice. A cancelled or undelivered notice remains eligible for retry; successful delivery is acknowledged before further event processing.

## Fixes

- The app-server daemon no longer downloads or runs the upstream OpenAI Codex
  installer. Managed self-install/update is disabled pending Corbanu installer
  support. Bootstrap keeps starting the daemon, reports automatic updates as
  disabled, and directs users to the [Corbanu installation instructions](https://github.com/CorbanuCore/CorbanuTerminal#install).
  Install/update manually and restart the daemon. Mutating lifecycle commands
  stop an updater left by an earlier build, including when the managed binary
  is missing; an old updater remains active until one of those commands runs
  or the machine reboots.

Multiline pastes in terminals without bracketed paste now stay together instead of sending each long line separately. Fast-typed single-line prompts still send on Enter after a brief 32 ms wait.

- Keep credential filtering for shell snapshots off async workers. A timed-out or cancelled scan refuses the snapshot, and ARM macOS/Linux use runtime-detected SHA acceleration while preserving matching and scan limits.

## Command execution

- Unix commands running without a terminal can be interrupted even when Corbanu started with interrupt signals ignored or blocked. Shell interrupt traps work in these commands again (#2364).

## Linux sandbox

- Accept the AppArmor launcher for system bubblewrap only after its namespace probe exits successfully. A timeout, startup error, or unrelated failure remains inconclusive rather than being treated as confirmed namespace access. If a host cannot complete the probe, sandbox startup may fail; check bubblewrap and the AppArmor `rootlesskit` profile, then retry after resolving the failure or host load.

## Agent orchestration

After a lasting turn error, pending input and agent mail are saved for a later explicit turn instead of repeatedly retrying the failed request. Mail a turn took but did not answer also waits for the next turn. Kill-switch and sign-in refusal recovery is unchanged (D659, D660; #2443).

## CLI and installation

- Installed Unix launchers preserve an explicitly selected `CORBANU_HOME`,
  `PFTERMINAL_HOME` or `CODEX_HOME` instead of inserting their default home.
  Conflicting home variables produce a warning. The debug launcher also
  preserves caller-selected homes, with explicit debug-home settings taking
  precedence (#408, #422).
- `corbanu exec` can wait for access to a temporarily unreadable home to
  return, with bounded backoff and diagnostics. `--home-wait-secs` controls
  the wait window (default 3600 seconds); `0` fails immediately (#861).

## Provider request state

- Provider request throttling uses a per-home salt for default API-key
  fingerprints. On upgrade, active legacy cooldowns and leases are carried
  over when that credential is next used; expired idle legacy rows can be
  removed. Named-account throttling follows the credential rather than only
  the account's name (#485, #569).

## Retired Claude model pins

- A saved `claude-opus-4-8` or `claude-opus-4-8-plan` selection on the direct
  Anthropic or Claude Plan connection now uses that connection's current
  default and displays a retirement notice. Choose a replacement with
  `/model` or update the pin in `config.toml` to stop the notice. Other
  providers keep their own model catalogs (#698).

## Models

- Add GPT-6.2 Astra and GPT-6.2 Sol on OpenAI, with ChatGPT sign-in and API-key catalog support, and Claude Fable 5.2 on direct Anthropic. Fable 5.2 defaults to High effort and always uses adaptive thinking. Existing model entries, defaults, aliases, and crew presets are unchanged.
- The new OpenAI entries include Fast and Ultrafast options and published API pricing. Access depends on the connected account. Fable 5.2 is not offered on Corbanu Plan because the available gateway did not advertise it and rejected the live probe; Bedrock support awaits a compatible transport (#2570).
