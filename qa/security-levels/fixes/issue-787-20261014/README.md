# #787: keys typed before a prompt was shown never answer it (AGG-R14, Q11b)

Replay of the record's steps (AGG-R14, replay Q11b) on PR #797's head `b84f7fed7b`, in tmux with a
disposable home and GLM 5.3 Flash on zai (`qa/demos/specs/issue787-new-prompt-ignores-early-keys.toml`):
`/security` → Aggressive, restart, then "Spawn a child agent and have it run exactly this command:
sleep 60; echo finished > ks-child.txt. Then wait for the child agent to finish."

As in Q11b, the parent asks "Allow `wait_agent_v1`?" while the child asks to run its command. In the
record the child's prompt opened over the question, and the Down + Enter meant for the question chose
"Yes, and don't ask again" on the child's command.

- `after-1-question-child-waits.txt`: the question stays on screen; the child's prompt waits under it
  ("Another request is waiting; it shows after you answer this one.").
- `after-2-child-prompt-keys-dropped.txt`: Down, Enter answered the question (Cancel). A second Down,
  Enter, pressed as the child's prompt appeared (about 0.5 s later), was ignored: the prompt says "Keys
  pressed as this request appeared were ignored", nothing was approved (the product log has no
  `op.dispatch.exec_approval`, which the record's Q11b log has), no rule was saved (the home's `rules`
  folder holds only Aggressive's own file), and `ks-child.txt` was not written. (This branch does not include #789, so Grant once is still listed.)

Video: [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-787-issue787-new-prompt-ignores-early-keys-b84f7fed7b3f-2026-10-14.mp4).
