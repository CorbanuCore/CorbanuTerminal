## Security

Wrapped shell commands in a top-level heredoc no longer offer a reusable approval rule. The command still receives its normal approval check; wrappers such as `sudo`, `env`, and `chroot` cannot become remembered shortcuts around the inner command's policy.
