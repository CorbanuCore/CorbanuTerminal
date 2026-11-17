## Named accounts (behind `named_accounts`)

- With `[features] named_accounts = true`, `corbanu account` can list, add,
  rename and remove accounts for a provider. Secret values for `account add`
  are read from stdin. The account list shows names and kinds, not credentials.
  `/providers` also exposes named-account management (#409, #430).
- Resuming a conversation preserves its recorded provider and account. A
  missing or removed named account requires recovery rather than silently
  using the default login (#566, #1299).
