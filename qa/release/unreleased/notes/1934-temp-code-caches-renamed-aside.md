## Windows sandbox: code caches in `%TEMP%` are renamed aside at the upgrade (D456, #1871)

- **At the one-time upgrade step that takes back older versions' sandbox
  grant on your `%TEMP%` (D377), Corbanu Terminal now also renames aside the
  code caches there**, so the programs that use them build new ones. Older
  versions let sandboxed commands write anywhere in `%TEMP%`, including caches
  that your own programs later run code from, whoever made them. Measured on
  a typical older session, 557 of the 571 entries its commands left in
  `%TEMP%` were in Node's `node-compile-cache`.
- What counts as a code cache, by name: at the top of `%TEMP%`,
  `node-compile-cache`, `v8-compile-cache` (and `v8-compile-cache-*`),
  `node-jiti`, `jiti`, `tsx-*` and `.net` (.NET single-file apps' extracted
  libraries); anywhere in `%TEMP%`, `__pycache__`. Each is renamed in place
  to `<name>.corbanu-sandbox-cache-<random>`. A link with one of these names
  is renamed itself, and what it points to is left alone. Nothing is
  deleted. The log of the first command after the upgrade counts them
  (`code caches N (renamed aside M, capped …)`). A cache in use at that
  moment may fail to rename. It is then left in place and counted.
