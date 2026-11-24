## Fixes

Linux, Aggressive: a command no longer fails to start when a file or folder under a deny-read scan root disappears while the sandbox scans it (for example, a background plugin sync cleaning up its temp clone). Any other scan error still blocks the command (#2893).
