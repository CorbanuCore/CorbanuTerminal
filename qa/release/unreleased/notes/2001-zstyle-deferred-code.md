## Security

After untrusted content, inspect the shell code stored by `zstyle -e` before it can run during a style lookup. Deferred styles also count as possible loaders of zsh's `stat` builtin when guarding writes to shell code tables.
