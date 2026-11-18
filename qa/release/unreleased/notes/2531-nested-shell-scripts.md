## Security

Command rules now see into a shell script nested in another shell script (#2531, D553): `bash -lc "bash -c 'git push'"`, `sh -c "zsh -c '…'"` and chains mixing launchers and shells such as `sudo bash -c "env bash -c '…'"` are checked against the rules for the commands inside, like `bash -lc 'git push'`. Up to eight nested scripts are read below each command; a command nested deeper asks for approval. No "always allow" is offered for such an outer script, since it could not approve the commands inside.

On Windows, rules now also see into PowerShell scripts whatever the spelling of the shell's name (#2524, D688): `PWSH -c`, `PowerShell.EXE -Command` and `C:\…\POWERSHELL.EXE -Command` are read like `pwsh -c` and `powershell.exe -Command`.
