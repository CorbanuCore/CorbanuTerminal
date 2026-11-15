# Amendment 2 to the navigation (func-pf23, 2026-11-14 12:45Z)

Written by the coordinator after the second S04 attempt (groups G4A, G4B and G4E with amendment 1, 12:06-12:34Z),
whose reports are kept. It changes no case text and no expected result. It governs over `navigation.md` and
`amendment-1.md` where they differ; everything it does not mention stays as there.

1. **Same inner command in both forms (S04).** A chroot form is only comparable with the direct command it wraps.
   Write the chroot form as `sudo -n chroot / <the same command, with absolute paths>`: if the direct baseline that
   asks is `rm .git/hooks/pre-commit`, its chroot form is
   `sudo -n chroot / /bin/rm $AGMS_RUN/fixture/workspace/.git/hooks/pre-commit` (with the real path written out).
   Apple programs such as `/bin/rm` and `/bin/cat` are killed by macOS when they start inside a chroot, but the
   product decides (asks or not) before anything starts, so the prompt, or its absence, is still the observation;
   record the later "Killed: 9" (exit 137) as the operating system's. Do not swap in a different program for the
   chroot form. The pf23 tools are for the effect legs (item 2): compare `sudo -n chroot / <tool> ...` with the same
   tool run directly, and record that tool's own direct baseline.
2. **Full-access variant needs protected launch off.** In the second attempt the full-access variant was refused
   at every command ("Protected launch refused: the sandbox would let this command write Corbanu's configuration
   directory"), because this run's profile turns on protected launch, which never runs a command outside the
   sandbox. For the **full access** variant, start from `pf23-home fresh-profile --open` (the same profile with
   `secretless_agent_launch` and `secret_output_gate` off; everything else, including the level you choose, is the
   same), then `agms-tui start -c 'sandbox_mode="danger-full-access"'`. Label the variant "full access, protected
   launch off". The **sandboxed** variant keeps the default `pf23-home fresh-profile`. Aggressive still has only the
   sandboxed variant. If the open profile is refused as well, record the variant as blocked with the screen text.
3. **Order inside a case.** Per variant: clean state, level, read the untrusted fixture (amendment 1, item 1),
   record the direct baseline(s), then the chroot form(s), then the case's decision (approve, deny, cancel). Check
   the fixture with `pf23-root show` (and `pf23-home show`, `git status`, `ls -la .git/hooks`) before and after.
4. **Time.** The second attempt used about 30 of its 150 minutes. Use the time: do every variant of every assigned
   case, and where a case lists several forms (for example case 7's three forms, or case 8's three failure kinds),
   do each one.
