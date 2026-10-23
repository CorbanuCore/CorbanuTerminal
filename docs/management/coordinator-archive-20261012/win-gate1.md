You are the Windows qualification worker. Read sec-common.md (binding) and sec-win1.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first.
A real Windows machine is now available (Travis tested SSH):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
  (fallback IP 100.71.146.84; RDP: Microsoft Windows App, PC desktop-ujs3hc4.tail36b615.ts.net, user DESKTOP-UJS3HC4\User).
  Windows 11 Pro, 16 logical cores, 31 GB RAM, 604 GB free, user is an admin. Present: git, python, gh, winget.
  Missing: Rust, VS Build Tools, MSYS2/tmux, ffmpeg, asciinema/agg. If a Windows password is ever required, it is in the
  Corbanu vault under the label "drone windows password"; use it only via
  PW="$(corbanu vault auth-helper "drone windows password")" piped to the consuming command over stdin; never print it.
  Put any Z.AI key on that machine the same way (stdin into a user-only file, never on a command line or in logs).
1. Provision the machine with the toolchain listed in sec-win1.result.md (non-interactively via winget/ssh; record versions).
   Keep everything under a dedicated directory; do not disable Defender or change security settings beyond what the
   elevated sandbox setup requires.
2. Complete PF-27-S06's gate: build Corbanu from origin/main on Windows, run the PF-27-S06 functional checks interactively
   per the repo tmux SOP driving GLM 5.2 (provider zai) on a disposable CODEX_HOME, and record short demo videos (secretless
   launch blocked from vault/auth/config/sqlite under the elevated sandbox; refusal with reason under unelevated; broker
   pipe refusing a foreign client). Verify the recorder works under MSYS2 tmux first; fall back to Windows Terminal +
   ffmpeg screen capture if not. Leak-scan every video. Upload to the `demos` release, index qa/demos/index/PF-27-S06.md,
   update the gate record. Another worker (sec-win2) is changing Windows code concurrently in PF-27-S07: build from
   origin/main as of your start and note the commit; don't touch its files.
3. Docs-only PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`). Do NOT archive PF-27-S06 unless
   the plan/sprint checkers allow it (another worker, plan-swap1, is activating the P1 plan).
Clean up build dirs you don't need afterwards. Report: tool versions, what passed/failed, video links, anything for Travis.
