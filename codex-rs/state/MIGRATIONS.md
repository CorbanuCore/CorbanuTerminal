# State DB migrations: Corbanu numbering

Corbanu's `migrations/` diverges from upstream Codex. Every version a released or installed Corbanu build has applied stays
as it is, with the same file, name and checksum; the runtime refuses a state DB whose applied migration has another checksum.

- **0001-0039:** shared with upstream (merge base `413492cd6c`).
- **0040-0045 and 0053:** Corbanu only (provider request state, GPU rentals, agent mailbox, provider request key salt).
- **0046-0052:** upstream's 0040-0046, renumbered on `main` before `rust-v0.1.48`.
- **0054-0069:** upstream's 0047-0062 from `rust-v0.167.0`, renumbered in the upstream sync (S1c, #1628). Upstream reused numbers that Corbanu builds had already applied.
- **0070:** Corbanu only (provider reset hint, main #2573). `rust-v0.1.49` shipped it before 0054-0069 existed, so a 0.1.49 state DB
  has 1-53 and 70, and upgrading it applies 0054-0069 *below* an applied migration. sqlx applies every missing version in order,
  and no migration in 0054-0069 touches `provider_request_state`, the only table 0070 changes, so either order gives the same schema.

| Upstream (`rust-v0.167.0`) | Corbanu | Corbanu file at the upstream number (unchanged) |
|---|---|---|
| `0047_rollout_migration_state.sql` | `0054_rollout_migration_state.sql` | `0047_threads_name` |
| `0048_thread_section_appearance.sql` | `0055_thread_section_appearance.sql` | `0048_drop_agent_jobs` |
| `0049_projects.sql` | `0056_projects.sql` | `0049_threads_is_pinned` |
| `0050_threads_section_empty_preview_indexes.sql` | `0057_threads_section_empty_preview_indexes.sql` | `0050_external_agent_config_imports_provider_id` |
| `0051_thread_artifacts.sql` | `0058_thread_artifacts.sql` | `0051_threads_section` |
| `0052_projects_recency.sql` | `0059_projects_recency.sql` | `0052_threads_section_order` |
| `0053_threads_originator.sql` | `0060_threads_originator.sql` | `0053_provider_request_key_salt` |
| `0054_threads_daybreak_enabled.sql` | `0061_threads_daybreak_enabled.sql` | `(none)` |
| `0055_thread_attachments.sql` | `0062_thread_attachments.sql` | `(none)` |
| `0056_threads_creator_identity.sql` | `0063_threads_creator_identity.sql` | `(none)` |
| `0057_cleanup_guardian_thread_metadata.sql` | `0064_cleanup_guardian_thread_metadata.sql` | `(none)` |
| `0058_threads_archive_sort_indexes.sql` | `0065_threads_archive_sort_indexes.sql` | `(none)` |
| `0059_thread_attachment_reverse_lookup.sql` | `0066_thread_attachment_reverse_lookup.sql` | `(none)` |
| `0060_guardian_review_feedback.sql` | `0067_guardian_review_feedback.sql` | `(none)` |
| `0061_thread_read_state.sql` | `0068_thread_read_state.sql` | `(none)` |
| `0062_threads_attached_parent.sql` | `0069_threads_attached_parent.sql` | `(none)` |

Checks:

- `src/migrations_tests.rs` (`upstream_migrations_are_renumbered_after_0053`): versions 1-70, one migration each, with this
  table's names.
- `tests/state_db_upgrade.rs`: upgrades a state DB written by the `rust-v0.1.48` release code (migrations 1-52), one
  written by `main` `18fa372a` (1-53) and one written by the `rust-v0.1.49` release code (1-53 and 70, #2812); no applied
  migration row changes, exactly 0053 (0.1.48 only), 0054-0069 and 0070 (unless already applied) are added, the threads
  survive, the schema equals a new database's, and a second start has nothing to do. For 0.1.49 it also checks that the
  0070 provider reset hint and the memories DB's stage-1 memory survive and that the memories DB gains only its 0002.

The fixtures in `tests/fixtures/state_db_upgrade/` (WAL databases, checkpointed) were made once by a throwaway test at each
ref: `StateRuntime::init(SqliteConfig::from_sqlite_home(dir), "openai")`, then `upsert_thread` of two
`test_thread_metadata` threads (one named, one archived), `close()`, `PRAGMA wal_checkpoint(TRUNCATE)`, and a copy of
`pfterminal_state_5.sqlite`. The `release-0.1.49` one (tag `rust-v0.1.49`, `776b4364e7`; same state code as the release
assembly `c46a9d10ed`) also recorded a 429 with a 308,374 s retry hint through `record_provider_request_throttle_outcome`
and one stage-1 memory through `try_claim_stage1_job` and `mark_stage1_job_succeeded`, and keeps
`pfterminal_memories_1.sqlite` as well. Don't regenerate them.

**Rollback:** an older Corbanu build cannot open an upgraded state DB. Since `main`'s 0053,
`validate_applied_migrations` refuses any applied version it doesn't know ("appears to belong to a different ...
distribution"); `rust-v0.1.48` does the same for 0053-0069, and `rust-v0.1.49` for 0054-0069. Going back means restoring
the whole Corbanu home backed up before the upgrade (release note "Upgrading from 0.1.49"; exercised in
`qa/release/unreleased/upgrade-from-0.1.49/`). Whether to let builds tolerate versions newer than their own is a
decision for Travis (D745; see the S1c report).

Upstream tests that name migration versions (`src/migrations_tests.rs`) use the Corbanu numbers. Future syncs add any new
upstream migrations after the highest Corbanu number in the same way.
