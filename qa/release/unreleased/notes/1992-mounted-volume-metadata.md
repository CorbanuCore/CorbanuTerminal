## Windows sandbox

Nested `.codex` and `.agents` folders in a workspace on a volume mounted inside another folder no longer count as changed after every command solely because of that mount. The watch checks the volume and file identity before reading their contents. Changed files and replaced folders are still flagged; ordinary junctions, symbolic links and mounts that cannot be verified retain the conservative warning.
