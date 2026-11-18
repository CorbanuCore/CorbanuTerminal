## Named accounts (behind `named_accounts`)

- With `[features] named_accounts = true`, `corbanu account` can list, add,
  rename and remove accounts for a provider. Secret values for `account add`
  are read from stdin. The account list shows names and kinds, not credentials.
  `/providers` also exposes named-account management (#409, #430).
- By default, resuming a conversation preserves its recorded provider and
  account. An explicit `--account` for the same provider overrides the recorded
  account, including a removed account. Explicit model/provider overrides
  (`-m`, `-c model_provider`, or a profile model) use the session's selected
  provider. Without an override, a missing or removed named account requires
  recovery rather than silently using the default login (#566, #1299).
