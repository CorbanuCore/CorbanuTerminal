## Moderate and Aggressive: more command lines ask after untrusted content

- `git fetch` and `git pull` now ask before using an unresolved remote after
  an option containing literal, quoted or escaped braces. Those option values
  could previously hide the remote from the approval check (#1975, #2005).
