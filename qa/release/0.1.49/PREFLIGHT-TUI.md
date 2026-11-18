# Current-main TUI preflight (not final release QA)

Date: 2026-11-17 UTC. These runs prepare and exercise the native harnesses while
the release gates remain open. Repeat on the final release-branch build. No
real-provider, elevated-Windows, live-repository or packaged-release acceptance
is claimed by these deterministic cases. Separate [live local-model fixture
checks](LIVE-TUI-PREFLIGHT.md) subsequently exercised real tool reads/writes.

Source: preparation commit `5fd571822bef255237f963c7bc18286a9110f58b`, which
contains main `d3fe32513ee65deb397fa228c786212fb5afacd3` plus release evidence and
notes only. Both binaries still report 0.1.48. Builds used Rust 1.99.1, private
targets and `-j8`, sequentially: macOS 4m19s, native Windows 3m42s.

| Platform | Executable SHA-256 | Result |
| --- | --- | --- |
| macOS arm64 | `53f77726294ee6323aa565ae1400ce4cf53a0e15ab305be9bb10ca3bb812cb57` | Reply, Escape cancellation, recovery pass; two persisted completed turns and an abort |
| Native Windows x64, DESKTOP-UJS3HC4 | `5e5173876cc2c300eaaac7f3143bdfc9be8bd88396f7d470da15d3a2d160c011` | Ready TUI, menus, reply, cancellation and recovery pass; two persisted completed turns and an abort |

## Isolation and execution

- macOS: direct debug executable, private home/workspace, all three profile
  aliases pinned, clean child environment, native-keyring denial flag, and a
  Seatbelt profile denying operator-home reads/writes and the three keychain
  services. A negative read probe passed. This is ordinary debug qualification,
  not the production/native credential-store lane.
- Windows: native MSVC `corbanu.exe` under the existing MSYS2 tmux harness,
  started over SSH. Private profile, USERPROFILE/APPDATA/LOCALAPPDATA/TEMP,
  clean environment and debug keyring denial. Fresh onboarding selected the
  non-admin sandbox. No UAC action or elevated-sandbox claim; the active desktop
  session was observed in preflight but this run was not a desktop GUI run.
- Both: 140x45 PTY, actual text and separate Enter events, trace logging to
  private paths, local mock Responses endpoint, no real credentials. `/status`,
  `/model`, `/providers` and the command list opened by `/` were captured and
  checked in the successful navigation replay.

## Preserved attempts and corrections

1. The initial macOS script assumed `/help` existed. It does not: the UI
   explicitly rejected it. The plan was corrected to typing `/` without Enter,
   and input is cleared between navigation cases. The successful prompt/cancel
   cases remain valid, but the initial `/help` attempt is not a navigation pass.
2. First macOS navigation replay sent `/status` before raw TUI startup finished;
   the capture showed the echoed input, not a status panel. That assertion failed.
   A fresh replay waited for the ready footer and passed all four navigation
   checkpoints. Both result files are retained.
3. Windows attempt 1 waited at the directory-trust prompt. Attempt 2 confirmed
   trust but waited at sandbox selection. Neither timed-out startup is counted
   as a pass. Attempt 3 confirmed trust and chose the non-admin sandbox, then
   completed every asserted checkpoint.

## Signing observations

The macOS preflight executable has an **ad-hoc, linker-created signature**, no
TeamIdentifier and no Developer ID authority. The Windows executable reports
**NotSigned** through `Get-AuthenticodeSignature`. These are debug build facts,
not validation of the future release archives or notarization tickets.
The publish workflow's lack of explicit signing/notarization is disclosed in
the QA plan. No signing certificate or login keychain was accessed for these
read-only inspections.

## Evidence

Small raw results are under [preflight](preflight/). Raw key-driven captures,
rollouts and traces remain at:

- macOS: `.codex-work/astra-rel149-macos-smoke/`, including `nav-replay/` and
  `nav-replay2/`;
- Windows: `C:\CorbanuQA\astra-rel149\smoke`, `smoke2`, `smoke3`;
- harness sources: `$W/astra-rel149-macos-smoke.py`,
  `$W/astra-rel149-macos-nav.py`, `$W/astra-rel149-windows-smoke.py`.

Linux TUI and persisted-state compatibility evidence is separately recorded in
[UPGRADE-COMPATIBILITY.md](UPGRADE-COMPATIBILITY.md).
