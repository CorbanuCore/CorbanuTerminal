## Security

- Linux, Aggressive: a ripgrep config file named by `RIPGREP_CONFIG_PATH` in your environment can no longer make the sandbox miss files that match a deny-read glob. The scan now ignores ripgrep config files (#2897).
