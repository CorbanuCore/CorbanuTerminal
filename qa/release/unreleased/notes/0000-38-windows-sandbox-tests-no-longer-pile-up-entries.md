## Windows sandbox tests no longer pile up entries on `%TEMP%` (#1890)

- **Running the `codex-windows-sandbox` and `codex-core` Windows tests no
  longer adds lasting permission entries to your `%TEMP%`.** Each test that
  ran a non-admin sandbox command left one there for good (about 25 per run
  of the `codex-windows-sandbox` tests), and every file later made in
  `%TEMP%` inherited all of them. The tests now give those commands a temp
  folder of their own, and a `codex-windows-sandbox` test fails if one still
  adds an entry to the real `%TEMP%` or `%TMP%`.
- Not changed: the non-admin sandbox itself keeps one lasting entry on
  `%TEMP%` per Corbanu home (measured: one after many sessions in several
  projects). On a machine that ran the tests before, `icacls "%TEMP%"` lists
  the old entries as bare `S-1-5-21-…` SIDs with `(OI)(CI)(M)`;
  `icacls "%TEMP%" /remove:g *S-1-5-21-…` (with the `*`) removes one. Remove
  only such bare-SID entries. If one was your Corbanu home's, the sandbox adds
  it back the next time it runs.
