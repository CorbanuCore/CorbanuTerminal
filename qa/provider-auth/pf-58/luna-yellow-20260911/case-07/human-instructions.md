# 7. Restart and confirm provider persistence

Quit this Corbanu window normally, relaunch from the app shortcut, and run `/status`. Send `Reply with exactly: RESTART_OK`.

Pass: startup goes directly to chat, the selected provider/model and active providers are preserved, and the response is `RESTART_OK`.

Fail: `/providers` appears automatically, authentication is lost, a different provider is silently selected, or the request fails.

