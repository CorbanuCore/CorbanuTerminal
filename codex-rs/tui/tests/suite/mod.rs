// Aggregates all former standalone integration tests as modules.
#[cfg(unix)]
<<<<<<< HEAD
mod claude_auth;
#[cfg(all(unix, feature = "developer-accounting"))]
mod cost_scope;
||||||| 413492cd6c
=======
mod daemon_compatibility;
#[cfg(unix)]
mod directory_trust;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
#[cfg(unix)]
mod focus_palette;
#[cfg(unix)]
<<<<<<< HEAD
mod memory_human_fixture;
#[cfg(unix)]
mod memory_stage_one_policy;
#[cfg(unix)]
mod multi_provider_onboarding;
mod nested_launch;
#[cfg(unix)]
mod output_text_stream;
#[cfg(unix)]
mod provider_convergence;
#[cfg(unix)]
mod provider_management;
||||||| 413492cd6c
=======
mod provider_defaults;
#[cfg(unix)]
mod reconnect;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod resize_reflow;
#[cfg(unix)]
<<<<<<< HEAD
mod resume_provider_accounts;
#[cfg(unix)]
mod resume_recorded_account;
#[cfg(unix)]
mod security_profiles;
#[cfg(unix)]
mod side_refusal;
#[cfg(unix)]
mod slash_dispatch;
||||||| 413492cd6c
=======
mod screen_reader;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod status_indicator;
mod vt100_history;
mod vt100_live_commit;
#[cfg(unix)]
mod worktree_stack;
