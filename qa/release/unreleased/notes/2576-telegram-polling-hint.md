## Reliability

- Telegram keeps its message and approval-button subscriptions when the first
  poll fails, then omits the subscription hint after a successful poll. Thanks
  to @bradsbots for the fix and regression test (#2576).
