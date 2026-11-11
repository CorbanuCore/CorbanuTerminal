//! Runs under the isolated test runner (`CORBANU_TEST_NO_NATIVE_KEYRING=1`).

use std::sync::Arc;

use pretty_assertions::assert_eq;

use super::*;
use crate::auth::named_chatgpt::login_named_account_with_api_key;

fn account(name: &str) -> NamedChatgptAccount {
    NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: name.to_string(),
    }
}

async fn key(manager: Option<Arc<AuthManager>>) -> Option<String> {
    manager?
        .auth()
        .await
        .and_then(|auth| auth.api_key().map(str::to_string))
}

/// #658: the process's backend requests use the account its user sessions
/// run on; never the default login while a named account is in use, and
/// nothing when sessions run on different accounts.
#[tokio::test]
async fn the_session_manager_follows_the_sessions_and_never_falls_back_to_the_default() {
    let home = tempfile::tempdir().expect("home");
    login_named_account_with_api_key(home.path(), &account("work"), "sk-canary-work")
        .expect("sign-in");
    login_named_account_with_api_key(home.path(), &account("play"), "sk-canary-play")
        .expect("sign-in");
    let base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-canary-default"),
        home.path().to_path_buf(),
    );
    let work_manager = base.for_named_account(account("work"));
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-default")
    );

    // A launch whose account is known only when its thread starts: nothing.
    base.register_configured_sign_in(SessionSignIn::Unknown);
    assert!(base.for_session().is_none());
    assert!(base.session_account_conflict().is_some());

    // Before any session: the configured account.
    base.register_configured_sign_in(SessionSignIn::Named(account("work")));
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-work")
    );

    // A session overrides the configured stand-in; a Z.AI pane does not count.
    let main = work_manager.register_session_sign_in(SessionSignIn::Named(account("work")));
    let zai = base.register_session_sign_in(SessionSignIn::NotOpenAi);
    for manager in [base.for_session(), work_manager.for_session()] {
        let manager = manager.expect("one account");
        assert_eq!(manager.named_account(), Some(&account("work")));
        assert_eq!(key(Some(manager)).await.as_deref(), Some("sk-canary-work"));
    }
    assert_eq!(base.session_account_conflict(), None);

    // A conversation the user left (no client attached) does not count.
    let left = base.register_session_sign_in(SessionSignIn::Default);
    assert!(base.for_session().is_none());
    left.set_attached(/*attached*/ false);
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-work")
    );
    left.update(SessionSignIn::Named(account("play")));
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-work")
    );
    left.set_attached(/*attached*/ true);
    assert!(base.for_session().is_none());
    drop(left);

    // The configured stand-in, set again later, does not count over sessions.
    base.register_configured_sign_in(SessionSignIn::Unknown);
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-work")
    );

    // A pane on the default login, or another named account: nothing is sent.
    let default_pane = base.register_session_sign_in(SessionSignIn::Default);
    assert!(base.for_session().is_none());
    assert_eq!(base.session_auth().await.map(|_| ()), None);
    assert!(base.session_account_conflict().is_some_and(
        |reason| reason.contains("openai:work") && reason.contains("the default login")
    ));
    default_pane.update(SessionSignIn::Named(account("play")));
    assert!(base.for_session().is_none());
    drop(default_pane);
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-work")
    );

    // A named account without a sign-in yields no auth, not the default.
    main.update(SessionSignIn::Named(account("ghost")));
    assert_eq!(key(base.for_session()).await, None);

    // Back on the default login; after every session has ended the
    // configured stand-in counts again.
    main.update(SessionSignIn::Default);
    assert!(Arc::ptr_eq(&base.for_session().expect("default"), &base));
    drop((main, zai));
    assert!(base.session_account_unknown());
    base.register_configured_sign_in(SessionSignIn::Default);
    assert!(Arc::ptr_eq(&base.for_session().expect("default"), &base));
}

/// #1085: a server that starts on the default login because its home had no
/// named account holds requests outside a conversation for good once a
/// session on a named account (one enrolled while it runs) registers, so
/// leaving that session never falls back to the default login.
#[tokio::test]
async fn a_server_held_once_named_never_falls_back_after_a_named_session() {
    let home = tempfile::tempdir().expect("home");
    login_named_account_with_api_key(home.path(), &account("work"), "sk-canary-work")
        .expect("sign-in");
    let base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-canary-default"),
        home.path().to_path_buf(),
    );
    base.register_configured_sign_in(SessionSignIn::Default);
    base.hold_configured_sign_in_once_named();
    // No named session yet: the default login, as with named accounts off.
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-default")
    );
    // A session on the default login does not hold.
    let default_pane = base.register_session_sign_in(SessionSignIn::Default);
    drop(default_pane);
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-default")
    );

    let work = base.register_session_sign_in(SessionSignIn::Named(account("work")));
    assert_eq!(
        key(base.for_session()).await.as_deref(),
        Some("sk-canary-work")
    );
    // Left: nothing, never the default login.
    work.set_attached(/*attached*/ false);
    assert!(base.for_session().is_none());
    drop(work);
    assert!(base.session_account_unknown());

    // A mid-session switch to a named account holds too.
    let other = tempfile::tempdir().expect("home");
    let other_base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-canary-default"),
        other.path().to_path_buf(),
    );
    other_base.register_configured_sign_in(SessionSignIn::Default);
    other_base.hold_configured_sign_in_once_named();
    let pane = other_base.register_session_sign_in(SessionSignIn::Default);
    pane.update(SessionSignIn::Named(account("work")));
    drop(pane);
    assert!(other_base.session_account_unknown());

    // Held after a named session already registered: it holds at once.
    let late = tempfile::tempdir().expect("home");
    let late_base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-canary-default"),
        late.path().to_path_buf(),
    );
    late_base.register_configured_sign_in(SessionSignIn::Default);
    let named = late_base.register_session_sign_in(SessionSignIn::Named(account("work")));
    late_base.hold_configured_sign_in_once_named();
    drop(named);
    assert!(late_base.session_account_unknown());
}
