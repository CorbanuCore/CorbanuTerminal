## Reliability

- Completed Telegram handler failures now free their inbox slots and remain distinct from successful updates. A chat notice identifies the failed update and explains how to resend it after checking for actions that may already have completed. Crash-pending updates and unresolved queued messages remain available for restart recovery; mutating sends are never automatically retried.
