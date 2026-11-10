## Windows sandbox

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
