## Windows sandbox

- **Non-admin sandbox git watch: fewer false alarms, bounded cost, and
  `\\?\C:\` paths in configs** (D486, #1877). A project inside `%TEMP%` no
  longer gets "links to a folder too large to check" after every command
  for links back into the project itself (pnpm's `node_modules`). A single
  huge folder behind a link into `%TEMP%` is no longer listed in full: past
  20,000 entries it counts as too large, as several folders already did.
  Git configs over 16 MB in all count as changed after every command,
  like configs past the other limits, and nothing more is read past any
  of those limits. A hooks folder, include or script a
  config names as `\\?\C:\...` is now watched as `C:\...` (`\\?\UNC\`
  network paths are still never opened).
