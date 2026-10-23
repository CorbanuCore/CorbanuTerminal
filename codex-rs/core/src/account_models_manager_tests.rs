//! Runs under the isolated test runner (`CORBANU_TEST_NO_NATIVE_KEYRING=1`).

use codex_login::CodexAuth;
use codex_login::login_named_account_with_api_key;
use codex_models_manager::model_info::model_info_from_slug;
use pretty_assertions::assert_eq;

use super::*;

const API_MODEL: &str = "pf84-api-model";
const CHATGPT_ONLY_MODEL: &str = "pf84-chatgpt-only-model";

fn work() -> NamedChatgptAccount {
    NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: "work".to_string(),
    }
}

/// One model any sign-in may use and one only a ChatGPT sign-in may.
fn catalog() -> ModelsResponse {
    let model = |slug: &str, supported_in_api: bool| ModelInfo {
        supported_in_api,
        ..model_info_from_slug(slug)
    };
    ModelsResponse {
        models: vec![model(API_MODEL, true), model(CHATGPT_ONLY_MODEL, false)],
    }
}

/// The default login is a ChatGPT sign-in; account `work` an API key.
fn setup() -> (tempfile::TempDir, Arc<AuthManager>) {
    let home = tempfile::tempdir().expect("home");
    login_named_account_with_api_key(home.path(), &work(), "sk-fake-work-models-0003")
        .expect("sign-in");
    let default_auth = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::create_dummy_chatgpt_auth_for_testing(),
        home.path().to_path_buf(),
    );
    (home, default_auth)
}

fn manager_for(
    provider: &ModelProviderInfo,
    home: &tempfile::TempDir,
    default_auth: &Arc<AuthManager>,
) -> AccountModelsManager {
    AccountModelsManager::for_provider(
        provider,
        Arc::clone(default_auth),
        home.path().to_path_buf(),
        Some(catalog()),
    )
    .expect("an OpenAI sign-in provider")
}

fn listed(manager: &AccountModelsManager) -> Vec<String> {
    let mut slugs: Vec<String> = manager
        .try_list_models()
        .expect("models")
        .into_iter()
        .map(|preset| preset.model)
        .collect();
    slugs.sort();
    slugs
}

fn both() -> Vec<String> {
    let mut slugs = vec![API_MODEL.to_string(), CHATGPT_ONLY_MODEL.to_string()];
    slugs.sort();
    slugs
}

/// The list is offered for the account the active session uses: a switch to
/// a named API-key account hides the ChatGPT-only model, and switching back
/// to the default ChatGPT login offers it again.
#[test]
fn the_model_list_follows_the_active_account() {
    let (home, default_auth) = setup();
    let manager = manager_for(
        &ModelProviderInfo::create_openai_provider(/*base_url*/ None),
        &home,
        &default_auth,
    );
    assert_eq!((manager.account(), listed(&manager)), (None, both()));

    assert!(manager.follow_sign_in_account(Some(&work())));
    assert!(!manager.follow_sign_in_account(Some(&work())));
    assert_eq!(
        (manager.account(), listed(&manager)),
        (Some(work()), vec![API_MODEL.to_string()])
    );

    assert!(manager.follow_sign_in_account(/*account*/ None));
    assert_eq!((manager.account(), listed(&manager)), (None, both()));
}

/// A process started on a named account lists with that account until a
/// session moves it to another.
#[test]
fn a_process_started_on_a_named_account_lists_with_it() {
    let (home, default_auth) = setup();
    let mut provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
    provider.account = Some(NamedProviderAccount {
        provider_id: "openai".to_string(),
        name: "work".to_string(),
    });
    let manager = manager_for(&provider, &home, &default_auth);
    assert_eq!(
        (manager.account(), listed(&manager)),
        (Some(work()), vec![API_MODEL.to_string()])
    );

    assert!(manager.follow_sign_in_account(/*account*/ None));
    assert_eq!(listed(&manager), both());
}

/// Providers without an OpenAI sign-in have no account to follow.
#[test]
fn providers_without_an_openai_sign_in_are_not_wrapped() {
    let (home, default_auth) = setup();
    let mut provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
    provider.requires_openai_auth = false;
    assert!(
        AccountModelsManager::for_provider(
            &provider,
            default_auth,
            home.path().to_path_buf(),
            /*model_catalog*/ None,
        )
        .is_none()
    );
}

/// Only a response from this provider on the account the list follows may
/// refresh it (its models etag says nothing about another account's list).
#[test]
fn only_the_followed_account_refreshes_the_list() {
    let (home, default_auth) = setup();
    let openai = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
    let manager = manager_for(&openai, &home, &default_auth);
    let mut work_provider = openai.clone();
    work_provider.account = Some(NamedProviderAccount {
        provider_id: "openai".to_string(),
        name: "work".to_string(),
    });
    let zai = codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None)["zai"]
        .clone();

    assert!(manager.lists_for_provider(&openai));
    assert!(!manager.lists_for_provider(&work_provider));
    assert!(!manager.lists_for_provider(&zai));
    manager.follow_sign_in_account(Some(&work()));
    assert!(manager.lists_for_provider(&work_provider));
    assert!(!manager.lists_for_provider(&openai));
}

/// A named account's list lives under its own directory; a name that is not
/// path-safe is never cached, never outside it.
#[test]
fn named_account_lists_have_their_own_cache_directory() {
    let home = tempfile::tempdir().expect("home");
    assert_eq!(
        named_account_cache_dir(home.path(), &work()),
        Some(home.path().join("models-cache").join("openai").join("work"))
    );
    let odd = NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: "../escape".to_string(),
    };
    assert_eq!(named_account_cache_dir(home.path(), &odd), None);
}
