## Fixes

Partial command-rule lookup now applies the same exact program spelling requirement as complete command matching (D696). On Windows, an allow rule for `git push` no longer treats `GIT` or `git.exe` as an allowed partial command; stricter rules retain their existing lookup.
