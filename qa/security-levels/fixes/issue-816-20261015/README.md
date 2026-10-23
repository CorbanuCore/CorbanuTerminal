# #816: what Aggressive enforces before the restart (AGG-04)

Replay of the AGG-04 screens on this PR's head `a842d5a4f8`, in tmux with a disposable home and
GLM 5.3 Flash on zai, with the milestone's feature flags
(`qa/demos/specs/issue816-aggressive-before-restart-says-what-holds.toml`): under Permissive,
`/security`, Aggressive, scroll to the bottom of the review, Enter (screen 07), Esc, `/security`
(the level list), `i` (the inspector). No restart.

- `before-*.txt`: copied from the targeted2 executor packet (V05 and V06 of candidate `df3d8c5d1c`;
  U06, the level list, from the packet's earlier screens, whose wording main still has). The review, screen 07 and the list said the sandbox
  for approved commands and network-off start at the restart, while the inspector's rows and a host
  check showed them in force (#816).
- `after-*.txt`: this fix.
  - Review: "From then until the restart, also: approved and allow-listed commands stay inside this
    session's sandbox (commands can write to the current folder, /private/tmp), network is off for
    agent commands, and secret-like environment variables are removed and login profiles and shell
    snapshots are not used."
  - Screen 07: the same under "Also in force now in this session"; "Starts when you restart" keeps
    only the Aggressive sandbox (writes only in the current folder), web search off, model keys in
    the credential broker and the child-agent rules.
  - Level list: "also in force now: …"; only Aggressive's sandbox, network (web search),
    model-key and child-agent rows wait for the restart.
  - Inspector: its Sandbox, Network and Vault rows say the same; its status line says "the rest of
    Aggressive's session controls apply after restart".

The environment item appears because this profile has `secretless_agent_launch` on and uses no
login profile or shell snapshot; the screens take each item from the facts behind the inspector's
rows (`security::current::before_restart`), so a session without them is not told they hold.

Known limit (review r2, conservative): the facts are read when `/security` opens and assume the
confirmation sets approval for every command (#505). If a managed requirement keeps a session on
`never` while its permission-request tools are on, the inspector then says "Network: off for agent
commands", but screen 07 and the list leave network-off under the restart. That understates; it
never claims a protection that does not hold.

Video: [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-816-issue816-aggressive-before-restart-says-what-holds-a842d5a4f8c5-2026-10-14.mp4).
