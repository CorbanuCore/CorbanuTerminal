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
