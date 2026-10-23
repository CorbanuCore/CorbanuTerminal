Found in the independent review of #479 (the #467 fix). This predates that PR and exists on main. It affects macOS (Seatbelt) only.

**Gap.** The Aggressive launch check `verify_isolation` (via `FileSystemSandboxPolicy::can_read_effective_path_with_cwd`, previously `can_read_path_with_cwd`) can count a credential path as unreadable while Seatbelt lets agent commands read it. This happens when the workspace is reached through a **nested** link:
- The workspace is opened as `…/link`, where `link -> real`. `-C` keeps the logical spelling (`tui/src/lib.rs`).
- `…/real/.env` is denied, for example because launch computed it from `current_dir()`.
- The readable roots are `/` and `…/link`. `sandboxing/src/seatbelt.rs` (~806-818) attaches a denial to a readable root only when the normalized spellings match literally (`path.starts_with(root)`). The `…/link` root gets no exclusion. It is then canonicalized to `(subpath …/real)`, which allows reading `.env`.
- The check counts `…/real/.env` as denied, because the deny entry is the most specific match.

Linux is not affected: bubblewrap applies its masks after the binds, and mounts follow links. Top-level aliases such as `/tmp` -> `/private/tmp` are normalized on both sides and are fine (#467).

**Options.**
1. Make the check rebuild Seatbelt's clauses: the readable roots, with exclusions attached by the same literal match, then the canonicalized roots and exclusions compared against the canonicalized path. Ideally this is a helper shared with `seatbelt.rs`, so the two can't drift apart. It would also need a test that cross-checks the generated Seatbelt arguments.
2. Fix Seatbelt to attach exclusions by comparing canonicalized roots and denials. This changes the sandbox boundary, so it is a product initiative.
