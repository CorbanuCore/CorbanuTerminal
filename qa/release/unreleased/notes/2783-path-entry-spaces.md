## Windows sandbox

- A `cmd /c` command that names a program without its extension (`npm`) and has cmd metacharacters in its arguments is now also refused when a batch file of that name sits in a PATH folder whose entry has leading or trailing spaces (`C:\tools ;…`). cmd keeps those spaces when it searches PATH, but the check had trimmed them and looked in a different folder (#2783).
