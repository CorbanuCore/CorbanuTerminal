## Security

On macOS and Windows, `forbidden` and `prompt` command rules now match the program name in any letter case (D692, #2510). These systems find programs without regard to case, so `RM -rf x`, `Rm x` or `/bin/RM x` run `rm`, and a rule that refuses or asks for `rm` now applies to them, including behind launchers and inside shell scripts. `allow` rules still need the exact name, so they never approve more than before. Arguments still compare exactly. On Linux, where `RM` is a different program, nothing changes.
