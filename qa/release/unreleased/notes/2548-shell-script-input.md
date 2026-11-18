## Security

Command rules now inspect static `eval` text and literal here-documents supplied to shells, including through launchers. A shell receiving piped or non-literal input, an `eval` with unknown words, or `. file` / `source file` asks for approval whenever a prompt or forbidden rule could apply. Sourced files are deliberately not opened: their contents cannot be determined safely before execution. These scripts never offer an “always allow” amendment (#2548, D553).
