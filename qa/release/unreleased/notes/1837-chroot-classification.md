## Security

Protected-action classification now also inspects commands launched directly through
`chroot /`, and requests approval when a root, command or option cannot be judged.
Existing finder/sudo and brace-reading protections are retained. This does not
claim complete coverage of commands launched indirectly by other programs or
replace operating-system sandboxing. Thanks to @citadelculture for the contribution.
