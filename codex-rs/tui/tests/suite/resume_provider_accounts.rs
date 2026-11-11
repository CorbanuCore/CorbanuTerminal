//! PF-84 #555: resume in homes whose configured provider or
//! `[provider_accounts]` differs from what the conversation recorded, Esc in
//! the resume pickers, and in-app `/resume` on a named account.
//!
//! The conversations are seeded rollouts recorded on Z.AI, so no model request
//! is made. Every home has no default Z.AI key.

use anyhow::Result;
use anyhow::ensure;
use tempfile::tempdir;

use super::resume_recorded_account::ONBOARDING;
use super::resume_recorded_account::PROMPT;
use super::resume_recorded_account::READY_TIMEOUT;
use super::resume_recorded_account::add_account;
use super::resume_recorded_account::corbanu_binary;
use super::resume_recorded_account::exit_tui;
use super::resume_recorded_account::launch_spec;
use super::resume_recorded_account::resume_spec;
use super::resume_recorded_account::seed_rollout;
use super::resume_recorded_account::wait_resumed_on_work;
use super::resume_recorded_account::write_config;
use super::resume_recorded_account::write_config_on;
use super::resume_recorded_account::write_config_with;
use crate::support::chat_ready::session_configured;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxPane;
use crate::support::tmux::TmuxServer;

/// Every provider's onboarding starts on this welcome screen.
const WELCOME: &str = "Welcome to Corbanu Terminal";
const PROVIDER_ACCOUNTS_PERSONAL: &str = "\n[provider_accounts]\nzai = \"personal\"\n";

/// (b) The home is configured for OpenAI, with no credentials; the
/// conversation ran on Z.AI account `work`, which is enrolled. It resumes
/// there instead of opening OpenAI's onboarding.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_resume_on_the_recorded_provider_without_its_onboarding() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #555 resume on the recorded provider")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config_on(
        home.path(),
        &repo_root,
        "model = \"gpt-5.5\"\nmodel_provider = \"openai\"",
        "",
    )?;
    add_account(&binary, home.path(), "work")?;
    let thread_id = seed_rollout(home.path(), &repo_root, "work")?;

    let tmux = TmuxServer::start("pf84_555_cross_provider")?;
    let session = tmux.new_session(resume_spec(
        "pf84-555-cross-provider",
        &binary,
        &repo_root,
        home.path(),
        &[thread_id.to_string()],
    ))?;
    let pane = session.primary_pane();
    let screen = pane.wait_stable_until(
        "the resumed conversation, or onboarding",
        READY_TIMEOUT,
        |capture| {
            capture.contains(WELCOME) || (session_configured(capture) && capture.contains(PROMPT))
        },
    )?;
    ensure!(
        !screen.contains(WELCOME),
        "resume opened onboarding:\n{screen}"
    );
    pane.send_literal("/status")?;
    pane.wait_stable_contains("/status", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    let status = pane.wait_stable_contains("account work", READY_TIMEOUT)?;
    ensure!(status.contains("Z.AI"), "not on Z.AI:\n{status}");
    exit_tui(pane)?;
    Ok(())
}

/// (b) Without credentials for the recorded provider, resume names it and
/// the recovery, instead of opening the configured provider's onboarding.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_resume_names_a_recorded_provider_without_credentials() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #555 recorded provider without credentials")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config_on(
        home.path(),
        &repo_root,
        "model = \"gpt-5.5\"\nmodel_provider = \"openai\"",
        "",
    )?;
    let thread_id = seed_rollout(home.path(), &repo_root, "default")?;

    let tmux = TmuxServer::start("pf84_555_cross_provider_missing")?;
    let session = tmux.new_session(resume_spec(
        "pf84-555-cross-provider-missing",
        &binary,
        &repo_root,
        home.path(),
        &[thread_id.to_string()],
    ))?;
    let pane = session.primary_pane();
    let screen = pane.wait_stable_until("an exit, or onboarding", READY_TIMEOUT, |capture| {
        capture.contains(WELCOME) || capture.contains("EXITED")
    })?;
    let screen = squash(&screen);
    ensure!(
        !screen.contains(WELCOME),
        "resume opened onboarding:\n{screen}"
    );
    for needle in [
        "This conversation ran on zai account `default`",
        "provider `zai` has no credentials in this home",
        "corbanu -c model_provider=\"zai\"",
        "EXITED 1",
    ] {
        ensure!(
            contains(&screen, needle),
            "missing {needle:?} in:\n{screen}"
        );
    }
    Ok(())
}

/// (c) `[provider_accounts]` names an account that does not exist; the
/// conversation resumes on the account it recorded. When that one is missing
/// too, it is named with its recovery.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_resume_ignores_a_missing_provider_accounts_entry() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #555 missing [provider_accounts] account")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config_with(
        home.path(),
        &repo_root,
        "\n[provider_accounts]\nzai = \"ghost\"\n",
    )?;
    add_account(&binary, home.path(), "work")?;
    let thread_id = seed_rollout(home.path(), &repo_root, "work")?;
    let gone_thread_id = seed_rollout(home.path(), &repo_root, "gone")?;

    let tmux = TmuxServer::start("pf84_555_provider_accounts_missing")?;
    let session = tmux.new_session(resume_spec(
        "pf84-555-provider-accounts-ghost",
        &binary,
        &repo_root,
        home.path(),
        &[thread_id.to_string()],
    ))?;
    let pane = session.primary_pane();
    wait_resumed_on_work(pane)?;
    exit_tui(pane)?;

    let session = tmux.new_session(resume_spec(
        "pf84-555-provider-accounts-gone",
        &binary,
        &repo_root,
        home.path(),
        &[gone_thread_id.to_string()],
    ))?;
    let pane = session.primary_pane();
    let screen = pane.wait_stable_contains("EXITED", READY_TIMEOUT)?;
    let screen = squash(&screen);
    for needle in [
        "This conversation ran on zai account `gone`",
        "corbanu account add zai gone",
        "EXITED 1",
    ] {
        ensure!(
            contains(&screen, needle),
            "missing {needle:?} in:\n{screen}"
        );
    }
    ensure!(
        !contains(&screen, "ghost"),
        "named the unused account:\n{screen}"
    );
    Ok(())
}

/// #426 (re-verified for the flag removal): a new session whose account is
/// missing, named by `--account` or by `[provider_accounts]`, exits with the
/// recovery text before any onboarding instead of asking for a default key.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_a_missing_launch_account_exits_before_onboarding() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #426 missing launch account")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let tmux = TmuxServer::start("pf84_426_missing_launch_account")?;
    for (name, extra, args) in [
        (
            "pf84-426-flag",
            "",
            vec!["--account".to_string(), "ghost".to_string()],
        ),
        (
            "pf84-426-config",
            "\n[provider_accounts]\nzai = \"ghost\"\n",
            Vec::new(),
        ),
    ] {
        let home = tempdir()?;
        write_config_with(home.path(), &repo_root, extra)?;
        add_account(&binary, home.path(), "work")?;
        let session = tmux.new_session(launch_spec(
            name,
            &binary,
            &repo_root,
            home.path(),
            /*subcommand*/ None,
            &args,
        ))?;
        let pane = session.primary_pane();
        let screen =
            pane.wait_stable_until("an exit, or onboarding", READY_TIMEOUT, |capture| {
                capture.contains(ONBOARDING)
                    || capture.contains(WELCOME)
                    || capture.contains("EXITED")
            })?;
        let screen = squash(&screen);
        ensure!(
            !screen.contains(ONBOARDING) && !screen.contains(WELCOME),
            "{name}: opened onboarding:\n{screen}"
        );
        for needle in [
            "account `ghost` of provider `zai` is not configured",
            "corbanu account add zai ghost",
            "EXITED 1",
        ] {
            ensure!(
                contains(&screen, needle),
                "{name}: missing {needle:?} in:\n{screen}"
            );
        }
    }
    Ok(())
}

/// Esc in the `corbanu resume` picker shown before onboarding leaves; it does
/// not start a new session that would open onboarding.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_esc_in_the_resume_picker_does_not_open_onboarding() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #555 Esc in the resume picker")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config(home.path(), &repo_root)?;
    add_account(&binary, home.path(), "work")?;
    seed_rollout(home.path(), &repo_root, "work")?;

    let tmux = TmuxServer::start("pf84_555_picker_esc")?;
    let session = tmux.new_session(resume_spec(
        "pf84-555-picker-esc",
        &binary,
        &repo_root,
        home.path(),
        &[],
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains(PROMPT, READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    let screen = pane.wait_stable_until("an exit, or onboarding", READY_TIMEOUT, |capture| {
        capture.contains(ONBOARDING) || capture.contains(WELCOME) || capture.contains("EXITED")
    })?;
    ensure!(
        !screen.contains(ONBOARDING) && !screen.contains(WELCOME),
        "Esc opened onboarding:\n{screen}"
    );
    ensure!(screen.contains("EXITED 0"), "Esc did not exit:\n{screen}");
    Ok(())
}

/// In a session on `personal` (`[provider_accounts]`): `/resume` of a
/// conversation recorded on `work` shows `account work`; Esc in the `/resume`
/// picker returns to the session; a conversation on a missing account is
/// named with the `/providers` recovery.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_in_app_resume_on_a_named_account() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #555 in-app /resume on a named account")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config_with(home.path(), &repo_root, PROVIDER_ACCOUNTS_PERSONAL)?;
    add_account(&binary, home.path(), "work")?;
    add_account(&binary, home.path(), "personal")?;
    seed_rollout(home.path(), &repo_root, "work")?;

    let tmux = TmuxServer::start("pf84_555_in_app_resume")?;
    let session = tmux.new_session(launch_spec(
        "pf84-555-in-app-resume",
        &binary,
        &repo_root,
        home.path(),
        /*subcommand*/ None,
        &[],
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_until("the session", READY_TIMEOUT, session_configured)?;

    // Esc in the picker returns to this session.
    open_resume_picker(pane)?;
    pane.send_key(TmuxKey::Escape)?;
    let screen = pane.wait_stable_until("the session again", READY_TIMEOUT, |capture| {
        !capture.contains("Resume a previous session") && capture.contains("via zai")
    })?;
    ensure!(
        !screen.contains(WELCOME) && !screen.contains(ONBOARDING),
        "Esc opened onboarding:\n{screen}"
    );

    open_resume_picker(pane)?;
    pane.send_key(TmuxKey::Enter)?;
    wait_resumed_on_work(pane)?;
    exit_tui(pane)?;

    // A conversation on an account that no longer exists.
    let home = tempdir()?;
    write_config_with(home.path(), &repo_root, PROVIDER_ACCOUNTS_PERSONAL)?;
    add_account(&binary, home.path(), "personal")?;
    seed_rollout(home.path(), &repo_root, "gone")?;
    let session = tmux.new_session(launch_spec(
        "pf84-555-in-app-resume-missing",
        &binary,
        &repo_root,
        home.path(),
        /*subcommand*/ None,
        &[],
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_until("the session", READY_TIMEOUT, session_configured)?;
    open_resume_picker(pane)?;
    pane.send_key(TmuxKey::Enter)?;
    let screen = pane.wait_stable_contains("/providers", READY_TIMEOUT)?;
    let screen = squash(&screen);
    for needle in [
        "This conversation ran on zai account `gone`",
        "account `gone` of provider `zai` is not configured; add it or pick another account in",
    ] {
        ensure!(
            contains(&screen, needle),
            "missing {needle:?} in:\n{screen}"
        );
    }
    ensure!(
        !contains(&screen, "--account"),
        "pointed to a CLI flag inside the TUI:\n{screen}"
    );
    exit_tui(pane)?;
    Ok(())
}

fn open_resume_picker(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/resume")?;
    pane.wait_stable_contains("/resume", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(PROMPT, READY_TIMEOUT)?;
    Ok(())
}

/// The screen as one line with single spaces, for messages.
fn squash(screen: &str) -> String {
    screen.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `screen` shows `needle`, wherever the terminal wrapped it.
fn contains(screen: &str, needle: &str) -> bool {
    let strip = |text: &str| text.split_whitespace().collect::<String>();
    strip(screen).contains(&strip(needle))
}
