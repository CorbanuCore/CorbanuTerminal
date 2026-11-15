## Windows sandbox

- Ctrl-C now reaches legacy Windows sandbox terminal commands even when Corbanu inherited Ctrl-C suppression from its launcher, including elevated sessions. A sandboxed console helper resets only the command's inherited state; tokens, access controls and Corbanu's own signal handling are unchanged.
