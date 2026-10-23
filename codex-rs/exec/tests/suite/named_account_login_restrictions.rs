//! PF-84-S06 (#570): with `forced_login_method` or
//! `forced_chatgpt_workspace_id` ruling out the default OpenAI login, a session
//! on a named OpenAI account checks the restriction against that account only
//! and never logs out (deletes) the default login. Default sessions keep
//! today's behaviour. Fake tokens and a mock server only.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use codex_config::types::AuthCredentialsStoreMode;
use codex_login::AuthDotJson;
use codex_login::AuthKeyringBackendKind;
use codex_login::CODEX_API_KEY_ENV_VAR;
use codex_login::NamedChatgptAccount;
use codex_login::OPENAI_API_KEY_ENV_VAR;
use core_test_support::responses::ResponseMock;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_repeating;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex_exec::TestCodexExecBuilder;
use core_test_support::test_codex_exec::test_codex_exec;
use serde_json::json;

const DEFAULT_KEY: &str = "sk-default-login-570";
const NAMED_KEY: &str = "sk-named-key-account-570";
const NAMED_ACCESS: &str = "named-chat-access-570";
const DEFAULT_ACCESS: &str = "default-chat-access-570";

#[derive(Clone, Copy)]
enum DefaultLogin {
    ApiKey,
    /// A ChatGPT sign-in in workspace `ws-main`.
    Chatgpt,
}

fn b64url(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, byte)| n | (u32::from(*byte) << (16 - 8 * i)));
        for i in 0..=chunk.len() {
            out.push(char::from(TABLE[((n >> (18 - 6 * i)) & 63) as usize]));
        }
    }
    out
}

fn jwt(claims: &serde_json::Value) -> String {
    format!(
        "{}.{}.{}",
        b64url(br#"{"alg":"none","typ":"JWT"}"#),
        b64url(claims.to_string().as_bytes()),
        b64url(b"sig")
    )
}

/// A fake access token: a JWT that expires in 2100, so no refresh is tried.
fn access_token(label: &str) -> String {
    jwt(&json!({ "label": label, "exp": 4_102_444_800_u64 }))
}

fn chatgpt_sign_in(user: &str, workspace: &str, access: &str) -> AuthDotJson {
    let id_token = jwt(&json!({
        "email": format!("{user}@example.com"),
        "https://api.openai.com/auth": {
            "chatgpt_plan_type": "plus",
            "chatgpt_user_id": user,
            "chatgpt_account_id": workspace,
        },
    }));
    serde_json::from_value(json!({
        "auth_mode": "chatgpt",
        "OPENAI_API_KEY": null,
        "tokens": {
            "id_token": id_token,
            "access_token": access_token(access),
            "refresh_token": format!("{access}-refresh"),
            "account_id": workspace,
        },
        // The access token's own expiry (2100) decides refreshes.
        "last_refresh": "2026-10-01T00:00:00Z",
    }))
    .unwrap()
}

fn account(name: &str) -> NamedChatgptAccount {
    NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: name.to_string(),
    }
}

/// A home with the given default login and two named OpenAI accounts:
/// `chat` (ChatGPT sign-in in workspace `ws-named`) and `key` (API key).
fn seed_home(home: &Path, default_login: DefaultLogin) -> Vec<u8> {
    match default_login {
        DefaultLogin::ApiKey => codex_login::login_with_api_key(
            home,
            DEFAULT_KEY,
            AuthCredentialsStoreMode::File,
            AuthKeyringBackendKind::default(),
        )
        .unwrap(),
        DefaultLogin::Chatgpt => codex_login::save_auth(
            home,
            &chatgpt_sign_in("deflt", "ws-main", DEFAULT_ACCESS),
            AuthCredentialsStoreMode::File,
            AuthKeyringBackendKind::default(),
        )
        .unwrap(),
    }
    codex_login::save_named_account_auth(
        home,
        &account("chat"),
        &chatgpt_sign_in("named", "ws-named", NAMED_ACCESS),
        /*must_be_new*/ true,
    )
    .unwrap();
    codex_login::login_named_account_with_api_key(home, &account("key"), NAMED_KEY).unwrap();
    std::fs::read(home.join("auth.json")).unwrap()
}

struct Run {
    /// The default login before the run.
    before: Vec<u8>,
    success: bool,
    stderr: String,
    /// `Authorization` of each model request.
    authorizations: Vec<Option<String>>,
    /// The default login afterwards (`None`: deleted).
    default_login: Option<Vec<u8>>,
}

/// `codex-exec` on the mock with named accounts on, no key in the
/// environment and `restriction` (a `-c` override), if any.
fn exec_cmd(
    test: &TestCodexExecBuilder,
    server: &wiremock::MockServer,
    restriction: Option<&str>,
) -> assert_cmd::Command {
    let mut cmd = test.cmd_with_server(server);
    cmd.env_remove(CODEX_API_KEY_ENV_VAR)
        .env_remove(OPENAI_API_KEY_ENV_VAR)
        .env_remove("CODEX_ACCESS_TOKEN")
        .env_remove("CORBANU_PROVIDER_ACCOUNT")
        .env_remove("CORBANU_HOME")
        .env_remove("PFTERMINAL_HOME")
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .arg("-c")
        .arg(format!(
            "chatgpt_base_url=\"{}/backend-api/\"",
            server.uri()
        ))
        .arg("-c")
        .arg("features.named_accounts=true");
    if let Some(restriction) = restriction {
        cmd.arg("-c").arg(restriction);
    }
    cmd
}

async fn run(default_login: DefaultLogin, restriction: &str, account: Option<&str>) -> Run {
    let test: TestCodexExecBuilder = test_codex_exec();
    let server = start_mock_server().await;
    let responses: ResponseMock = mount_sse_repeating(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_assistant_message("msg-1", "pong"),
            ev_completed("resp-1"),
        ]),
    )
    .await;
    let before = seed_home(test.home_path(), default_login);
    let mut cmd = exec_cmd(&test, &server, Some(restriction));
    if let Some(account) = account {
        cmd.arg("--account").arg(account);
    }
    let output = cmd
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("Reply with exactly: pong")
        .output()
        .unwrap();
    Run {
        before,
        success: output.status.success(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        authorizations: responses
            .requests()
            .iter()
            .map(|request| request.header("authorization"))
            .collect(),
        default_login: std::fs::read(test.home_path().join("auth.json")).ok(),
    }
}

/// The default login's bytes are exactly what they were before the run.
fn assert_default_unchanged(run: &Run) {
    let after = run.default_login.as_ref().expect("default login deleted");
    assert!(*after == run.before, "default login changed");
}

fn bearer(token: &str) -> Option<String> {
    Some(format!("Bearer {token}"))
}

/// Case 1: default API key, ChatGPT required, named ChatGPT account: answered
/// as `chat`, default login untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn case1_named_chatgpt_session_keeps_the_default_api_key() {
    let run = run(
        DefaultLogin::ApiKey,
        "forced_login_method=\"chatgpt\"",
        Some("chat"),
    )
    .await;
    assert!(run.success, "{}", run.stderr);
    assert!(!run.authorizations.is_empty());
    assert!(
        run.authorizations
            .iter()
            .all(|auth| *auth == bearer(&access_token(NAMED_ACCESS))),
        "{:?} {}",
        run.authorizations,
        run.stderr
    );
    assert_default_unchanged(&run);
}

/// Case 2: default API key, ChatGPT required, named API-key account: the
/// named account is refused at thread start; the default login is untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn case2_refused_named_api_key_session_keeps_the_default_login() {
    let run = run(
        DefaultLogin::ApiKey,
        "forced_login_method=\"chatgpt\"",
        Some("key"),
    )
    .await;
    assert!(!run.success);
    assert!(
        run.stderr
            .contains("account `key` holds an OpenAI API key, but this configuration requires a ChatGPT sign-in"),
        "{}",
        run.stderr
    );
    assert!(run.authorizations.is_empty());
    assert_default_unchanged(&run);
}

/// Cases 3/4: a default session keeps today's behaviour: the ruled-out
/// default login is logged out and the session continues without it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn case3_default_session_still_logs_out_a_ruled_out_default_login() {
    let run = run(
        DefaultLogin::ApiKey,
        "forced_login_method=\"chatgpt\"",
        /*account*/ None,
    )
    .await;
    assert!(run.success, "{}", run.stderr);
    assert_eq!(run.default_login, None);
    assert!(
        run.authorizations
            .iter()
            .all(|auth| auth.as_deref() != bearer(DEFAULT_KEY).as_deref()),
        "{:?}",
        run.authorizations
    );
}

/// Case 5: default ChatGPT, API key required, named API-key account:
/// answered as `key`, default login untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn case5_named_api_key_session_keeps_the_default_chatgpt_login() {
    let run = run(
        DefaultLogin::Chatgpt,
        "forced_login_method=\"api\"",
        Some("key"),
    )
    .await;
    assert!(run.success, "{}", run.stderr);
    assert!(!run.authorizations.is_empty());
    assert!(
        run.authorizations
            .iter()
            .all(|auth| *auth == bearer(NAMED_KEY)),
        "{:?}",
        run.authorizations
    );
    assert_default_unchanged(&run);
}

/// Case 6: a default session with a ruled-out ChatGPT default login keeps
/// today's behaviour (logged out, continues).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn case6_default_session_still_logs_out_a_ruled_out_chatgpt_login() {
    let run = run(
        DefaultLogin::Chatgpt,
        "forced_login_method=\"api\"",
        /*account*/ None,
    )
    .await;
    assert!(run.success, "{}", run.stderr);
    assert_eq!(run.default_login, None);
    assert!(
        run.authorizations
            .iter()
            .all(|auth| auth.as_deref() != bearer(&access_token(DEFAULT_ACCESS)).as_deref()),
        "{:?}",
        run.authorizations
    );
}

/// Case 7: default ChatGPT in `ws-main`, workspace `ws-other` required, named
/// API-key account: the named session runs (an API key has no workspace) and
/// is not refused because of the default login's workspace.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn case7_named_session_is_not_refused_for_the_default_logins_workspace() {
    let run = run(
        DefaultLogin::Chatgpt,
        "forced_chatgpt_workspace_id=\"ws-other\"",
        Some("key"),
    )
    .await;
    assert!(run.success, "{}", run.stderr);
    assert!(!run.stderr.contains("Logging out"), "{}", run.stderr);
    assert!(
        run.authorizations
            .iter()
            .all(|auth| *auth == bearer(NAMED_KEY)),
        "{:?}",
        run.authorizations
    );
    assert_default_unchanged(&run);

    // A default session still ends on the ruled-out workspace, as before.
    let default_session = self::run(
        DefaultLogin::Chatgpt,
        "forced_chatgpt_workspace_id=\"ws-other\"",
        /*account*/ None,
    )
    .await;
    assert!(!default_session.success);
    assert!(
        default_session
            .stderr
            .contains("Login is restricted to workspace(s) ws-other, but current credentials belong to ws-main. Logging out."),
        "{}",
        default_session.stderr
    );
    assert_eq!(default_session.default_login, None);
}

/// A named ChatGPT account in a workspace the restriction rules out is
/// refused for itself (not for the default login's workspace), and the
/// default login is left in place.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn named_session_restriction_names_only_its_own_account() {
    let run = run(
        DefaultLogin::Chatgpt,
        "forced_chatgpt_workspace_id=\"ws-other\"",
        Some("chat"),
    )
    .await;
    assert!(!run.success);
    assert!(
        run.stderr.contains(
            "account `chat` is signed in to a ChatGPT workspace this configuration does not allow"
        ),
        "{}",
        run.stderr
    );
    assert!(!run.stderr.contains("ws-main"), "{}", run.stderr);
    assert!(run.authorizations.is_empty());
    assert_default_unchanged(&run);
}

/// Review of #570: `exec resume --last` of a thread recorded on named account
/// `chat`, without `--account`, while `forced_login_method = "chatgpt"` rules
/// out the default API key. The thread's account is known only after
/// startup, so the default login must not be logged out first; the thread
/// resumes on `chat`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resumed_named_thread_keeps_the_default_login() {
    let test = test_codex_exec();
    let server = start_mock_server().await;
    let responses: ResponseMock = mount_sse_repeating(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_assistant_message("msg-1", "pong"),
            ev_completed("resp-1"),
        ]),
    )
    .await;
    let before = seed_home(test.home_path(), DefaultLogin::ApiKey);
    let first = exec_cmd(&test, &server, /*restriction*/ None)
        .arg("--account")
        .arg("chat")
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("Reply with exactly: pong")
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let resumed = exec_cmd(&test, &server, Some("forced_login_method=\"chatgpt\""))
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("Reply with exactly: pong again")
        .arg("resume")
        .arg("--last")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&resumed.stderr);
    assert!(resumed.status.success(), "{stderr}");
    assert!(!stderr.contains("Logging out"), "{stderr}");
    let authorizations = responses
        .requests()
        .iter()
        .map(|request| request.header("authorization"))
        .collect::<Vec<_>>();
    assert!(authorizations.len() >= 2, "{authorizations:?}");
    assert!(
        authorizations
            .iter()
            .all(|auth| *auth == bearer(&access_token(NAMED_ACCESS))),
        "{authorizations:?} {stderr}"
    );
    assert!(
        std::fs::read(test.home_path().join("auth.json")).ok() == Some(before),
        "default login changed"
    );
}
