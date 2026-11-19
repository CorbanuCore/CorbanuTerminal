## Fixes

- Windows elevated commands with a symbolic temporary-folder write grant resolve their writable roots after selecting the private session folder, preventing an earlier setup override from granting the parent temporary directory. Explicit path grants and deny rules remain in effect.
