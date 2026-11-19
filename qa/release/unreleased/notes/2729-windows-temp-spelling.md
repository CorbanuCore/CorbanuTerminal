## Windows sandbox

- When an elevated command's environment spells `TEMP` (or `TMP`) more than once in different letter case, its private session temp folder is now always made in the folder the command itself sees (`TEMP` before `Temp`), not in whichever spelling happened to be read first.
