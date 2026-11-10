## Windows sandbox

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
