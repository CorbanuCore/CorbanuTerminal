# State DB migrations: Corbanu numbering

Corbanu's `migrations/` diverges from upstream Codex. Every version a released or installed Corbanu build has applied stays
as it is, with the same file, name and checksum; the runtime refuses a state DB whose applied migration has another checksum.

- **0001-0039:** shared with upstream (merge base `413492cd6c`).
- **0040-0045 and 0053:** Corbanu only (provider request state, GPU rentals, agent mailbox, provider request key salt).
- **0046-0052:** upstream's 0040-0046, renumbered on `main` before `rust-v0.1.48`.
- **0054-0069:** upstream's 0047-0062 from `rust-v0.167.0`, renumbered in the upstream sync (S1c, #1628). Upstream reused numbers that Corbanu builds had already applied.

| Upstream (`rust-v0.167.0`) | Corbanu | Corbanu file previously at that number |
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

`tests/state_db_upgrade.rs` checks three things:

- this table against the files;
- upgrades of a state DB written by the `rust-v0.1.48` release code (migrations 1-52) and one written by `main` `18fa372a` (1-53);
- that no applied migration row changes in either upgrade.

The fixtures are in `tests/fixtures/state_db_upgrade/`. Opening the upgraded DB with the previous release's binary is tested in S5.

Upstream tests that name migration versions (`src/migrations_tests.rs`) use the Corbanu numbers. Future syncs add any new upstream migrations after the highest Corbanu number in the same way.
