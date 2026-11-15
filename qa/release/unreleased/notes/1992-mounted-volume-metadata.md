## Windows sandbox

Nested `.codex` and `.agents` folders in a workspace on a volume mounted inside another folder no longer count as changed after every command solely because of that mount. The watch recognizes mounts and junctions to an entire local volume by its GUID, then checks the recorded volume and file identity before reading metadata contents. Changed files and replaced folders are still flagged. Junctions to other paths, symbolic links and anything that cannot be verified retain the conservative warning.
