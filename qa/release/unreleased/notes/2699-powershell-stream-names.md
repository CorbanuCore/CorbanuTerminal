## Security

On Windows, a PowerShell program spelled as an NTFS stream or with an 8.3 short-name tail (`pwsh.exe:x.exe -Command "git status"`, `C:\PS\pwsh.exe:s -c …`) is no longer read as PowerShell: Windows runs the stream's content, not PowerShell, so command rules no longer judge it by its `-Command` script, and a remembered approval for a PowerShell script no longer covers it. Such a command now asks for approval as one unknown command, at the top level and inside bash, cmd.exe or PowerShell scripts, and the prompt never offers an always-allow rule (#2699).
