# PF-25-S01: per-sprint gate (2026-10-07)

- **Branch:** `feat/pf-25-s01-grant-tui` off main at `e4d17dbdc6`. Behind `security_levels`; grants are offered only
  under a live Aggressive policy (approval `untrusted`), so with the flag off or any other level nothing changes.

## Flow

```text
command approval under Aggressive (human reviewer)
  Core: offer {command, folder, digest, actor chain, session, epoch}   <- data only, ends with the approval
  TUI:  g -> review (opens on "Back") -> ↓ "Grant and run" -> Enter
        confirm(shown, uses): offer still open, equal, same epoch and chain -> choice recorded on the offer
        then Accept is sent for this approval
  Core: approval approved -> take the choice -> apply under the policy now
        "1 run": rules lifted for this run, never held
        "until it expires" (u): held 10 minutes for any run of the exact command, listed in /security
  Declined, cancelled, abandoned, or state changed: nothing applies; the rules stay.
```

## Results

- **Focused tests (final tree):**
  - `just test -p codex-core` (`grant_offer`, `aggressive`, `pf_25_s01`, `pf_23_s02`, `transition`): 62/62.
  - `just test -p codex-tui` (`pf_25_s01`, `approval_overlay`, `security`): 164/164.
  - Core integration (`core/tests/suite/pf_25_s01.rs`, macOS seatbelt): the offer key is the approval id the TUI
    answers and Core's command equals the approval's; the approved "1 run" reads the protected file and the next
    identical run, only approved, gets "Operation not permitted"; a declined approval leaves nothing.
- **Wider run** (round-1 tree): `just test -p codex-tui -p codex-app-server-client` 4414/4416. The two failures are
  the known command-menu snapshot and kitty pet image tests.
- **Linux clippy** (`-D warnings`; core, tui, app-server-client, `--tests`) on the RTX box: clean at `535350b9cf`,
  `c5fa36b31f` and `d76c577e8c` (at `aee150e46a` it caught an `eprintln!` in a test, removed).
- **tmux (GLM 5.2, `-c model_provider="zai"`, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`;
  `.codex-work/workers-20261002/sec-tui8.log`):** under Aggressive (confirmed, restarted) GLM's hidden-home read asks
  first; `g` opens the review; ↓ Enter grants one run and the file is read; the same read again, approved with `y`,
  gets "Operation not permitted"; Esc in the review goes back and `y` runs it under the rules; `u` holds the grant and
  `/security` lists it with its expiry. The first take found that Esc in the review reached the pane's cancel path
  and declined the command; fixed (the overlay takes Esc while the review is open) with a pane-level test.
- **Review (Opus 5.5 High, installed `corbanu exec`, read-only):** round 1 APPROVE WITH FIXES (9 findings), round 2
  APPROVE WITH FIXES (8 low), round 3 APPROVE WITH FIXES (1 medium, 5 low/info), round 4 **APPROVE**. Each finding
  is fixed or recorded in `review/disposition.md`.
- **Videos (GLM 5.2, commit `aee150e46a`; later commits change one test and docs only):** grant one run, Esc grants
  nothing, a grant until expiry listed in `/security` (`qa/demos/index/PF-25-S01.md`).

## Known limits

- Grants live in memory: a restart ends them (PF-23-S02's design), so the sprint text's "persist the grant record"
  is not done, on purpose.
- Only commands get an offer: patches have no grant option, and typing into an unconfined process stays refused
  under Aggressive. A command answered by a cache, hook or automatic reviewer never shows the option.
- `/security` lists grants of every session of the process, including ended ones, until they expire; `admit` still
  refuses any that a commit made stale.
- Only the grant review may call `security_grant::confirm`; a workspace scan test pins that (no type-level token).
- The end-to-end Core test runs on macOS only (seatbelt); the Linux CI covers the unit tests.
