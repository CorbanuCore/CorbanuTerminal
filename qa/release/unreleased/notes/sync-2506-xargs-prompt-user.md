## Security

When a managed `prompt-user` rule could match once `xargs` adds words read from its input (for example `printf push | xargs git` with a managed `prompt-user` rule for `git push`), the command now needs a fresh human decision instead of an approval the configured reviewer could give. This also applies when `xargs` runs a command, script or options taken from its input, or has options Corbanu cannot read. The approval prompt now also shows a `prompt-user` rule's reason (#2506).
