## Fixes

PowerShell scripts that cannot be fully read now require approval whenever a prompt or forbidden command rule could apply, without offering an always-allow rule. Recognizable forbidden commands remain refused, including inside encoded scripts. Rules on PowerShell's own options remain effective, and dangerous-command checks read joined command arguments.
