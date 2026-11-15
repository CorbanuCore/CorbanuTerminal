## Security

- The existing forced-`rm` protection now recognizes abbreviated `--force` options (such as `--forc`) and ANSI-C or translated quoted flags (such as `$'-rf'`) at every security level. Ordinary non-forced removal keeps its existing behavior.
