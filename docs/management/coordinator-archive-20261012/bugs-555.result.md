I've merged and closed all of #555 with every check green, and added the nightly developer-accounting job. Each PR got an Opus 5.5 High review that ended at APPROVE.

| Item | PR | Merge commit |
|---|---|---|
| #555 (a): forks follow the source account | #561 | `060ce90947` |
| #555 (b), (c), `/providers` wording, new tests | #566 | `aebbc18667` |
| Nightly job: codex-core tests with developer-accounting | #567 | `077ba3ebe2` |

**What shipped**
- **(a) Forks:** `/fork`, editing an earlier prompt, `/side` and `corbanu fork` now run on the source conversation's account, and `/status` names it. The app server takes the source thread's account, or the one it last recorded if the thread isn't loaded; an explicit `--account` still wins. `thread/fork` has a new experimental `providerAccount` field.
- **(b) Other provider:** a conversation recorded on another provider resumes there when this home has credentials for it. Otherwise it exits naming the provider and account, with the recovery, instead of showing onboarding. This only applies to a resume without `-m`/`-c model_provider`; forks and overridden resumes stay on the session's provider, matching what the server does.
- **(c) `[provider_accounts]`:** for `corbanu resume`, that check now runs once the conversation is chosen, so a missing entry no longer blocks resuming on an existing recorded account. A missing recorded account is named with `corbanu account add`.
- **Wording:** inside the TUI, the missing-account message says "add it or pick another account in `/providers`"; the CLI keeps `--account`.
- **Esc:** in the `corbanu resume` picker shown before onboarding, the footer reads `esc quit` and Esc exits with status 0. Inside a session, Esc in `/resume` still returns to the session.
- **Nightly:** codex-core tests with developer-accounting now run nightly only, not on PRs. I ran it once by hand on main after merging: 4,145 passed in about 27 minutes. One test fails only with the feature (the same kind of race as #546); it's skipped in that job and filed as #564.

**Gate evidence**
- **Regression tests:**
  - The two new app-server fork tests fail on the old code and pass after.
  - The `corbanu fork` tmux test fails on main's binary (`account personal`).
  - All five tests in the new tmux suite `resume_provider_accounts` fail on main's binary and pass after. The in-app one fails on main only at the `/providers` wording, as expected.
- **Focused tests:** pass on macOS and glitch, with and without developer-accounting:
  - `codex-tui` lib: 253
  - `codex-core` `config::`: 505
  - `codex-model-provider`: 94
  - app-server fork tests: 4/4
  - tmux suites: 8/8
- **Clippy `-D warnings`:** clean on macOS and on glitch, both ways.
- **Reviews:**
  - #561 approved in round 1.
  - #566 got REQUEST CHANGES in round 1, for applying another provider's recorded account to forks and overridden resumes, and a #426 regression for forks. After the fixes it was approved in round 2, and round 3 approved the merge with main.
  - #567 got REQUEST CHANGES for a missing bubblewrap install, then was approved in round 2.
- **tmux checks:** done on GLM 5.3 Flash in disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`. Before/after captures are in `qa/provider-auth/pf-84/fix-555-20261012/`, and I re-ran them on the final binary. The real key appears 0 times in the captures and diffs.
- **Code-blind design:** recorded before human testing, in `.../code-blind/`. A fresh designer that saw only the intent and screens proposed 33 cases; each is mapped to evidence or marked open.
- **Videos (leak scan passed):**
  - [fork](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-555-pf84-555-fork-account-ae55961c40fd-2026-10-11.mp4)
  - [cross-provider resume](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-555-resume-pf84-555-cross-provider-resume-935283eec8ec-2026-10-11.mp4)
  - [`[provider_accounts]` ghost](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-555-resume-pf84-555-provider-accounts-ghost-935283eec8ec-2026-10-11.mp4)
  - [Esc in the picker](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-555-resume-pf84-555-resume-picker-esc-935283eec8ec-2026-10-11.mp4)
  - [in-app `/resume` and `/providers`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-555-resume-pf84-555-in-app-resume-providers-935283eec8ec-2026-10-11.mp4)

  The four resume videos were recorded before #566's review round 1. That round didn't change the flows they show.

**Still open**
- **Esc behaviour (needs a decision):** I read "Esc must not lead to onboarding" as "Esc quits that picker". The alternative is to keep "start new" and show a message instead. Quitting is what's merged; say if you want the other.
- **Code-blind execution:** no separate code-blind executor has run the 33 cases, so a human-test handoff must say so. Open cases include the prompt-edit fork live run, a fork after its account is removed, feature-off live runs and a default-only recorded provider.
- **Change class:** both reviewers raised whether this counts as a bounded fix or a product initiative, since it changes which vault credential a fork or resume uses. It's done under the active PF-84 plan, like #552.
- **Non-blocking review items left as follow-ups:**
  - the fork path checks the feature flag in the server's base config rather than the fork's own config;
  - a paginated fork cut at an earlier turn uses that turn's account;
  - a fork's missing-account error still says "resumes";
  - a missing `[provider_accounts]` entry still refuses `corbanu fork` at launch, as before (now documented);
  - `corbanu resume --last` only finds conversations from the configured provider (unchanged upstream behaviour).

All my worktrees, branches, build folders, disposable homes, glitch clone, target and logs are removed.