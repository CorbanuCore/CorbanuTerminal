## Moderate and Aggressive: more command lines ask after untrusted content

After the agent has read untrusted content, these lines used to run without
asking and now ask first:

- `rg --hostname-bin=PROG`, which runs a program (#1600, #1573, thanks to
  @bradsbots).
- zsh's `=name`, which names the program `name` on `PATH`, as a redirection
  target (`echo x >> =mytool`; #1602, #1573, thanks to @citadelculture), and
  as a writer's operand (`tee -a =mytool`) or in `eval`, `trap` and `alias`
  text (#1651, #1620).
- Writes to a relative file such as `git log --output=.zshenv` or
  `git config -f .zshenv …` when the folder it lands in can't be known: after
  `cd "$X"` or another `cd` the check can't follow, after a wrapper's folder
  option (`sudo -D ~`, `env -C ~`, `docker exec -w ~`), or in a function, trap
  or loop that runs after a later `cd` (#1599, #1309, thanks to @bradsbots;
  #1651, #1606, when an option value spelled `git` comes before the wrapper's
  folder option).
- `git grep -O<pager>` and `--open-files-in-pager`, which run a program on the
  matching files (#1527, #1516). On Windows, also git named by its full path
  (#1527) and git's dashed helpers (`git-grep.exe`, `git-clone.exe`) (#1598,
  #1528).
- `-O`, `-u`, `-c`, `--open-files-in-pager`, `--exec`, `--template` and
  `--config` after a git subcommand that isn't one of git's own commands,
  since an alias or an autocorrected typo can be `grep`, `clone`, `init` or
  `push` (#1598, #1528, thanks to @Holic101, with parts of
  @wizbubba1's #1597), and program-running options after built-ins that older
  gits don't have, such as `refs` or `hook` (#1647, #1635).
- git commands cut short by an unquoted bracket the shell reads as text
  (`git -C { -c core.pager=sh log`; #1525, #1515), and git options that brace
  expansion produces (`git clone r d {-q,--upload-pack=./x}`; #1644, #1636).
- zsh's `>!`, `>>!`, `&>!` and `>&!` clobber redirections, read as writes
  (`echo x >! ~/.zshrc`; #1640, #1607).
- Commands in `eval`, `trap` and `alias` text, which now get every check
  the same command gets on a line of its own (D341, #1698, #1685): writes
  to shell start-up files (`trap 'tee -a ~/.zshrc' EXIT`,
  `eval 'tee -a ~/.zshrc'`), output options (`curl -o`, `time -o`), sending
  a file (`curl -d @f`), recursive reads of your home, scripts it runs
  (`./x.sh`, `source`, `python3 x.py`), a wrapper's folder option
  (`eval 'sudo -D sub ./run.sh'`), and, in an alias, git settings that run
  code. Before, only some of these were checked there. A `cd` in such text
  isn't followed: the folder it leaves counts as unknown. Ordinary text such
  as `trap 'rm -f x' EXIT` or `alias ll='ls -l'` stays quiet.
- Later uses of variables set in such text
  (`eval 'F=~/.zshrc'; tee -a "$F"`), settings there of variables that run
  code (`eval 'export GIT_PAGER=sh'`), and code written into the shells'
  tables of code run by name (zsh's `functions`, `commands` and alias
  tables, bash's `BASH_ALIASES` and `BASH_CMDS`), which is now checked as a
  line of its own (`functions[f]='tee -a ~/.zshrc'; f`; D341, #1698,
  #1685, #1704). Ordinary settings such as `eval "x=1"` or
  `aliases[ll]='ls -l'` stay quiet.
- A word the check can't see into (`$o`, `$(cat f)`) where a command that
  can write code into those tables still reads its options:
  `o=$(cat f); zstyle $o :x y functions`, `zstyle -a $(cat f)`,
  `zselect -t 0 -r 0 -A $n`, `zsystem flock -t 0 -f $n file` (#1704).
  `print`, `printf` and `getopts` are now read this way too. So an
  unquoted variable, `$(…)` or pattern where `print`, `printf`, `set`,
  `read`, `mapfile`, `vared` or `getopts` read their options or the
  name they write can now ask (`print -r $(cat f)`,
  `set ?A commands ls ./x`, `read -d $(cat f)`,
  `getopts $(cat f)`), and so can a quoted one with more words after it
  (`printf "$fmt" x`). `print -r -- $x`, `set -- $(cat f)`,
  `printf "$fmt"`, `print -l *.png` and `wait $pid` stay
  quiet.
- zsh's `stat` builtin, which can write those tables (`stat -A functions
  x`), now asks only when the same action may load it: a line of it runs
  `zmodload` or names `zsh/stat` (`zmodload zsh/stat; stat -f %m $(cat f)`
  asks, before or after the `stat`), holds a brace range, or runs bash's
  `enable` or a command named by a variable, command output or a pattern.
  zsh starts without it. Elsewhere `stat` is the system program, so macOS
  `stat -f %m $f` stays quiet. A module loaded by your zsh start-up files,
  or by an earlier command in the same long-running shell session, isn't
  seen (D410, #1704). `zstat` is always checked.
- A program named by a path, or a shell or interpreter, run after a leading
  redirection (`>log ./x`, `2>log sh x`, `2>&1 ./x`, `<<EOF ./x`,
  `sudo >log ./x`, `env >log ./x`), also in `eval` text (`eval '>log ./x'`),
  where the check used to read the redirection as the command (#1719,
  #1482, thanks to @darkmatter-sudo). `ls >log` and `2>/dev/null ls` are
  read as before.
- `sudo -A`/`--askpass`, `sudo -e`/`--edit` and `sudoedit`, which run a
  helper or editor named by the environment, also in `eval` text
  (`eval sudo -A ./x`) (#1719, #1482, thanks to @darkmatter-sudo).
- A command given as an argument list whose elements up to the program
  don't each read as one word (`["true ./x"]`, `["true;./x"]`,
  `["env", "A=1 ./y", "./x"]`) (#1719, #1482, thanks to @darkmatter-sudo).
  A shell's `-c` script is read as before. This doesn't apply under Windows
  path rules, where program paths hold spaces.

One line asks less: `git add src/{a,b}.rs && git commit -m "msg"` no longer
asks just because a quoted message follows the brace word (#1580, #1514,
thanks to @grapemarketer). A line with many `sed` commands and long output
names is now checked within a fixed size budget; past it, its output files
count as one unknown file, the cautious reading (#1587, #1132, thanks to
@0xzoz).
