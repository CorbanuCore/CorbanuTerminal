## Windows sandbox

- When an elevated command's environment spells `TEMP` (or `TMP`) more than once in different letter case, its private session temp folder (and the cleanup of folders an earlier session left behind) now always uses the folder the command itself sees (`TEMP` before `Temp`), not whichever spelling happened to be read first. No extra folder became writable before; only the choice of folder was random (#2729, #2780).
