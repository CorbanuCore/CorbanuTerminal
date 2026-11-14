## Project agent roles follow every project-config rule (D525, #2052)

- An empty provider list after project narrowing now also refuses crew records,
  matching the existing spawn refusal.
- The empty-list startup warning now describes project restrictions without
  assuming you configured your own provider list.
- Preflight identifies project role files by their source and path. App-server
  config layers and metadata report them as `projectRole` with the file path,
  keeping their existing precedence; actual launch overrides stay `sessionFlags`.
