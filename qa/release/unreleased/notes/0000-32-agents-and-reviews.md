## Agents and reviews

- **A message handed to a running agent is delivered once** (#1579, #1545,
  thanks to @bradsbots). If recovery ran just after a live sender handed a
  message over, it could send the same message again.
- **A review cancelled before it starts stays cancelled** (#1574). A `/review`
  or automatic approval review whose cancellation came before its session
  started could still start it.
