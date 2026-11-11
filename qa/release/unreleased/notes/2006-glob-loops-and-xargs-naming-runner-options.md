## Moderate and Aggressive: more command lines ask after untrusted content

- The same runners (`tar`, `sed`, `rsync`, `rg`, `sort`) given a file name
  the line doesn't show, in two forms (D484, #2006, #1912): a loop variable
  taken from such a pattern (`for f in *; do tar rf a.tar "$f"; done`, also
  `select` and zsh's `for f (*)`), and `xargs` fed by `ls` or `find`
  (`ls | xargs tar cf a.tar`, `find . | xargs tar cf a.tar`). These lines
  now ask. `for f in ./*`, `find … -print0 | xargs -0` and a `--` before the
  names (`xargs tar cf a.tar --`) stay quiet. Other values the line doesn't
  show, such as `$(git ls-files)` or `while read`, are left as before.
