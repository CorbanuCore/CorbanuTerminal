# codex-utils-cargo-bin runfiles strategy

We disable the directory-based runfiles strategy and rely on the manifest
strategy across all platforms. This avoids Windows path length issues and keeps
behavior consistent in local and remote builds on all platforms. Bazel sets
`RUNFILES_MANIFEST_FILE`, and the `codex-utils-cargo-bin` helpers use the
`runfiles` crate to resolve runfiles via that manifest.

Function behavior:

- `cargo_bin`: reads `CARGO_BIN_EXE_*` environment variables (set by Cargo or
  Bazel) and resolves them via the runfiles manifest when `RUNFILES_MANIFEST_FILE`
  is present. When not under runfiles, it only accepts absolute paths from
  `CARGO_BIN_EXE_*` and returns an error otherwise. If neither spelling is set,
  it checks the legacy binary beside the test executable (above `deps/` for
  Cargo tests). Missing optional helpers return `Err`, never a lookup panic,
  so callers may try another binary or skip an unavailable optional helper.
- `find_resource!`: used by tests to locate fixtures. It chooses the Bazel
  runfiles resolution path when `RUNFILES_MANIFEST_FILE` is set, otherwise it
  falls back to a `CARGO_MANIFEST_DIR`-relative path for Cargo runs.

Background:

- https://bazel.build/docs/runfiles
- https://bazel.build/docs/runfiles#runfiles-manifest
