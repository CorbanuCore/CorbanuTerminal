## Fixes

On macOS and Linux, a shell command whose approval prompt ended at the wrong moment could lose its timeout and keep running past it, or be timed out while the prompt was still open. The timeout now always resumes and pauses correctly (#2722).
