# Review disposition (Opus 5.5 High, approve with fixes)

1. **Registry creation failure:** fixed. Verification fails ("registry … is missing") when the workspace contains the registry and it does not exist, so bubblewrap never places a placeholder there.
2. **Glob characters in the path:** fixed. Such a path (or a non-UTF-8 one) is left out of the profile, so the config still loads.
3. **Unclear message:** fixed. An inexpressible path is reported as such when the workspace contains it.
4. **Test quality:** fixed. `base_overrides_with_registry` / `verify_with_registry` take the registry explicitly; the tests use a temporary account home as the working folder, check the read-only subpath handed to the platform sandboxes, and cover a missing registry, a glob path and the nested-exec overrides. The tests no longer depend on `CORBANU_TEST_ACCOUNT_HOME`.
5. **Real home touched by plain `cargo test`:** recorded in the README (launch tests under plain `cargo test` create the registry folder, as an Aggressive launch does; `just test` uses a disposable account home).
6. **Docs:** the Sandbox row says the Corbanu home and the registry stay read-only; the README marks the ancestor-rename finding as macOS-verified only. Linux runs the same tests (RTX, see README).
