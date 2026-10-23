# #817: the child's prompt takes a deliberate Enter (AGG-R14)

Replay of AGG-R14 with the frozen keystroke driver of the second targeted run, unchanged
(`../../milestones/aggressive-20261014-targeted2/executor-packet/v6/drivers/AGG-R14.json`,
SHA-256 `0310f8cb…89701`, the same digest as in the run), on this PR's head `e679bfd468`
(debug build, `corbanu 0.1.48`), on the host Mac in tmux: disposable HOME and CORBANU_HOME, no
native keyring, the run's fixture and product config, GLM 5.3 Flash on zai, 150x45. Not code-blind
and not a milestone result.

Harness: the run's `harness/agms-drive` and `harness/agms-tui` with only their fixed paths changed
(driver folder, `agms-tui`, tmux binary and config, and the product launcher taken from the
environment). The launcher starts the candidate the way `agms-product` does, without the guest's
proxy; the real Z.AI key is resolved at launch through the installed corbanu's vault helper.

What happened (`driver-AGG-R14.steps.log`; screens in this folder):

- S0/S1: trust, `/security`, Aggressive, `r`; after the restart `/security` showed
  "Active in this session: Aggressive".
- The parent asked about `spawn_agent_v1` and then `wait_agent_v1`; the driver answered both with
  Enter (Allow once). The child's command prompt (`sleep 60; echo finished > ks-child.txt`) then
  showed, with "1. Yes, proceed (y)" highlighted and no notice
  (`A1-child-prompt-before-enter.txt`).
- 06:03:52.954Z: the driver's Enter on that prompt, about 9 s after it was first seen
  (06:03:44Z), approved it. The product log has the `exec_approval` at 06:03:52.79Z and the child's
  `exec_command` running until the kill switch ended it. In the second targeted run this Enter was
  refused ("This is a new request. Choose an option with ↑/↓, then enter") and the driver stopped.
- 06:04:05Z: `/security`, `g`, "Turn the kill switch on", Enter: "Stopped: the security kill switch
  was turned on, so Corbanu ended this reply and the commands it was running"
  (`A2-kill-switch-on.txt`).
- `ks-child.txt` was absent right after the kill switch and 90 s later
  (`fixture-ks-child.txt`); the `sleep 60` would have finished by 06:04:53Z.
- The driver completed all steps ("DRIVER-END completed all steps"). Its observation step found no
  Managed Crew row to highlight (the child was spawned by the model, not with `/spawn`).

Second replay, on the final head `8d72690909` (`r2-final-head/`, same driver and harness): this
time the model asked no agent-tool question, so the child's prompt showed directly. The driver's
Enter 13 s after it appeared (06:46:39.389Z) approved it (`exec_approval` at 06:46:39.23Z, the
child's `exec_command` then running); the kill switch went on at 06:46:52Z and `ks-child.txt` was
absent then and 90 s later. The driver completed all steps.

SOP video (`qa/demos/specs/issue817-child-prompt-takes-a-later-enter.toml`, final head
`8d72690909`): the parent's `wait_agent_v1` question showed with the child's prompt waiting under it
(`demo-first-prompt.txt`); Enter answered it; the child's prompt showed with no notice
(`demo-second-prompt.txt`); 11 s later Enter approved it (`demo-answered.txt`); `ks-child.txt` was
written with `finished` (`demo-ks-child-check.txt`). In the last frame the child asks to read the
file back (`demo-child-ran.txt`).
[mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-817-issue817-child-prompt-takes-a-later-enter-8d7269090963-2026-10-14.mp4)
(an earlier recording on `e679bfd468`, before review r1's change to queued requests, is also in
the index).
