You are the TUI lane worker (round 5, continuation). The previous worker was stopped by the coordinator because its
demo/probe runs (Claude panes, a synthetic Keychain probe) coincided with repeated macOS login-keychain password
popups for Travis. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; note the new
keychain rule), the brief sec-tui5.md, and skim the tail of sec-tui5.log (very long; use rg/tail) to recover state.
Open PRs: #226 (nested check in standalone binaries), #230 (#218 part 1), #231 (registry read-only), #232 (#218 part 2).

Rules for the rest of this run:
- Do not create, read or probe any item in the user's real login keychain, and don't run the real `claude` CLI with
  access to the user's Claude login. Keychain-protection evidence must come from tests with a mocked keychain or a
  separate throwaway keychain file created with `security create-keychain` under the disposable home and never added
  to the search list; if that isn't possible, record the limit instead of probing.
- List in your report any keychain items the earlier probes created (service/account), so Travis can delete them
  in Keychain Access. Don't delete them yourself.
Then see the four PRs through CI and merge when green. Report as in sec-common.md.
