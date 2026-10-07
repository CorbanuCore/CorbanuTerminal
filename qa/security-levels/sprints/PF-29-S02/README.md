# PF-29-S02 evidence: human-reviewed credential migration and recovery

Behind `protected_mode_preflight` (PF-29-S01, default off; only with `security_levels`). Flag off is unchanged.
Base: the PF-29-S01 branch (`72d9a5dfbf`); worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf-29-s02-20261006`.

## What it does

| Step | Where | Behaviour |
| --- | --- | --- |
| Preview | `m` in a blocked Aggressive review; `tui/src/bottom_pane/security_migration.rs` | For each shell-profile export: location, finding ID, vault label (never replacing an existing one) and the reference the line becomes. Also: owner-only access, restart, unsupported items with what to do, and rotation. No values. Esc returns to the review; nothing changes. |
| Confirm | `core/src/security/migration.rs` `run` | Rechecks the reviewed preflight (any drift moves nothing), creates the journal exclusively (0600, labels and stages only), stores each value in the vault (`ManualSecret`), then rewrites each line atomically to `"$(corbanu vault auth-helper LABEL)"` (fish `(…)`). The file is `0600`, synced, and its directory synced. No backup or plaintext copy is kept. |
| Strictness | `parse_line`, `Assignment::literal` | Only one plain value is moved: `'…'`, `"…"` without `$`, `` ` `` or `\`, or an unquoted word without shell syntax, globs, braces or `~`, followed by nothing or a comment. Anything else (a second command, several words, fish escapes) is listed as unsupported, and so are linked or hard-linked profiles. |
| Later owners | `rewrite_file`, `file_stamp` | A line is rewritten only while it still holds the stored value. An unstored entry is skipped if its file changed after confirmation. Just before the rename, the file must have the same identity, link count, size and mtime as when it was read. |
| Interruption | preflight readiness `migration` | A stop at any stage keeps the level. The review says "Migration stopped" and Aggressive cannot be saved until `r` runs recovery. |
| Recovery | `recover` | Rolls forward from the journal and converges on the uninterrupted result. It never restores plain text and never writes the level. A damaged journal says what to check and that the file can then be deleted. |
| Re-audit | picker | After a move or recovery the review reruns the preflight before Aggressive can be saved. Saving resets activation, so earlier conversations stay unresumable (PF-29-S01). |

Debug builds only: `CORBANU_TEST_MIGRATION_FAIL_AT=prepared|stored|rewritten|committed` stops the next migration
at that point. This is the hook the recovery video uses.

## Tests (final tree)

- `cargo test -p codex-core --lib pf_29_s0`: 21 passed (S02: plan preview without values; move + re-audit clean, no
  backup, 0600; every fail point recovers to the same file and vault, with Aggressive locked until then and
  concurrent runs refused; later edit never overwritten; store failure changes no file; fish and linked profiles;
  the review's quoting, multi-command, multi-word, glob, brace, tilde, fish-escape and hard-link cases).
- `cargo test -p codex-tui pf_29_s0`: 12 passed (S02: preview then Esc changes nothing; confirm, re-audit, save;
  failure locks until `r`; change after the preview moves nothing).
- `just fix -p codex-core -p codex-tui`, `just fmt`; clippy is clean on the changed code. No Linux-only code.

## Functional run (GLM 5.2, real TUI in tmux, demo SOP)

Recorded on `821c0b8f7ef3` with `glm-5.2` / `zai`, a disposable `HOME` and Corbanu home, a real disposable vault
(file key store via `CORBANU_TEST_NO_NATIVE_KEYRING`), and synthetic values. Videos:
[`qa/demos/index/PF-29-S02.md`](../../../demos/index/PF-29-S02.md).

| Demo | Result |
| --- | --- |
| `pf29s02-preview-cancel` | Preview lists `~/.zshrc:2 OPENAI_API_KEY → migrated/openai_api_key`, 0600, restart and rotate. Esc: "Migration cancelled. Nothing changed."; the shell check shows one plain-text value left and the mode unchanged. |
| `pf29s02-migrate-and-save` | "Moved 1 credential… Rotate at the provider: OPENAI_API_KEY", then "Preflight passed" and "Saved: Aggressive". The shell check shows 0 plain-text values, `-rw-------`, and the reference line. |
| `pf29s02-failure-recovery-restart` | Failure injected after the rewrite: "Migration stopped", level still Permissive, review blocked ("a credential migration did not finish"). After a real restart (no hook), `r` gives "Moved 1 credential", then "Preflight passed" and "Saved: Aggressive". |

## Independent review

Opus 5.5 High, `corbanu exec -m claude-opus-5-5-plan`, read-only. [Review 1](review-opus-1.md): **CHANGES REQUIRED**
(P1 quoted `#`; P2 second commands and fish name lookup; P3s). [Review 2](review-opus-2.md): **APPROVED**, with five
P3s. Dispositions: all of review 1 fixed as listed in review 2. From review 2: the stamp and link check now runs before
the rename and before chmod; globs, braces, `~` and fish single-quote escapes are unsupported; journal-creation errors
are reported as the "prepare" stage. The scanner's lenient ` #` split (display only) is left as is; migration
re-parses strictly.

## Not done here

- Only shell-profile exports are migrated. Config literals (MCP/provider), environment variables and memories are
  listed with actions.
- Live capabilities (broker leases) are not revoked on migration; the restart that activates Aggressive covers it.
- The reference line runs `corbanu vault auth-helper` when the profile loads, which may prompt for the OS keychain in
  the person's own shell. Agents get no value: Aggressive strips secret-named variables and skips profiles.
