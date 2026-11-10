## Windows sandbox

- **"Set up default sandbox" now applies to the session you are in** (#2000).
  After setting the Windows sandbox up from `/permissions` in the TUI, the
  running session used to keep running agent commands without it until you
  started a new one with `/new`. Now it uses the sandbox from your next
  message, and under Moderate or Aggressive the "needs the default Windows
  sandbox" refusal stops at once. The server reads the sandbox mode from its
  own config, never from the client, and a running session's sandbox can only
  get stronger: switching to a weaker one (for example the non-admin sandbox
  after the default one) still applies from the next session.
