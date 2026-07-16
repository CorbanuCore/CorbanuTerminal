# PFTerminal 0.1.13

## Fixed

- Fixed native Windows startup by deriving the Claude Plan authentication working directory from
  the active filesystem root instead of validating the Unix-only path `/` on every platform.
- Added native Windows provider-initialization tests to the release workflow so this startup class
  is exercised before Windows packages are published.

## Qualification status

- The provider-information test suite passes locally on Linux.
- The release workflow runs the same suite natively on Windows before compiling and packaging the
  release executable.

Previous release: 0.1.12.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
