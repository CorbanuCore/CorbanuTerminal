## A timed-out network prompt for remote work closes

- **The approval prompt for a remote network request now closes when the
  request times out** (#722, D279). An unanswered request is still refused
  after 10 minutes (plus the hook timeout) and the command still ends as
  failed; before, its prompt stayed on screen and answering it did nothing.
- **App-server clients:** the server resolves such a pending
  `item/commandExecution/requestApproval` request itself and sends
  `serverRequest/resolved`, as it already does when a turn ends. Any command
  approval whose waiter in the core goes away unanswered is resolved the same
  way.
