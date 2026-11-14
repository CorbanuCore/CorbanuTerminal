## Reliability

- Telegram polling now resets its consecutive-failure counter after every successful poll, including polls with no messages. Reaching the failure cap reports that cause instead of incorrectly claiming another bot poller is running.
