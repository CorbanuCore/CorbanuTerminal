## Security

- Linux, Aggressive: the deny-read scan no longer runs a `rg` from a folder the sandboxed command could write (such as a `bin/` folder in the workspace), so a planted `rg` can no longer make the sandbox miss files that match a deny-read glob. When the only `rg` on `PATH` is in such a folder, or is owned by you while the profile allows writes everywhere, the built-in file walker does the scan instead, which can be slower on large folders (#2901).
