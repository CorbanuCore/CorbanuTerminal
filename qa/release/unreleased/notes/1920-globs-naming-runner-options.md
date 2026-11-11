## Moderate and Aggressive: more command lines ask after untrusted content

These lines also ask now:

- A pattern the shell expands to file names, where `tar`, `sed`, `rsync`,
  `rg` or `sort` still reads options (#1920, #1912). A file named like an
  option, such as `--checkpoint-action=exec=sh x`, turns `tar cf a.tar *`
  into a command that runs a program, so these lines now ask:
  `tar cf a.tar *`, `sed -i 's/a/b/' *.md`, `rsync -a * dest/`,
  `rg x *`. For `rg` and `sort` only a pattern that ends in a wildcard
  counts (`rg x *.rs` stays quiet). Patterns that can't start with an
  option name stay quiet (`./*`, `src/*`, `--exclude=*.log`), and so does
  a pattern after a `--` that ends the options (`tar cf a.tar -- *`; in
  `tar cf -- *` the `--` is the archive's name, so that line asks).
