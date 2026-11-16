## Security

- After untrusted content, additional shell-generated output targets and git options require confirmation: zsh `=name` writes through launchers and output-file options, targets made by braces or empty variables, and quoted anchored parameter replacements. Existing prompts remain, including the stricter checks for bash.
