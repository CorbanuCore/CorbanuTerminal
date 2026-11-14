## Windows sandbox

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
