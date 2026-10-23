use std::collections::BTreeMap;

use codex_model_provider_info::ModelProviderInfo;
use pretty_assertions::assert_eq;

use super::*;

fn route() -> codex_login::AuthRouteConfig {
    codex_login::test_support::transport_default_auth_route_config()
}

fn name(raw: &str) -> ProviderAccountName {
    ProviderAccountName::parse(raw).expect("valid name")
}

fn row(provider_id: &str, account: &str) -> NamedAccountRow {
    NamedAccountRow {
        provider_id: provider_id.to_string(),
        name: name(account),
        kinds: vec![ProviderAccountKind::ApiKey],
        fingerprint: Some("0123456789ab".to_string()),
        aws_profile: None,
        sign_in: None,
    }
}

fn view() -> AccountsView {
    AccountsView {
        rows: vec![row("zai", "work"), row("zai", "spare"), row("kimi", "alt")],
        session: BTreeMap::from([("zai".to_string(), "work".to_string())]),
        defaults: BTreeMap::from([("kimi".to_string(), "alt".to_string())]),
        status_accounts: BTreeMap::new(),
        status_account_details: BTreeMap::new(),
    }
}

#[test]
fn markers_name_the_session_and_default_accounts() {
    let view = view();
    assert_eq!(view.session_account("zai"), "work");
    assert_eq!(view.session_account("kimi"), "default");
    assert_eq!(view.default_account("zai"), "default");
    assert_eq!(view.markers("zai", "work"), " · this session");
    assert_eq!(
        view.markers("zai", "default"),
        " · default for new sessions"
    );
    assert_eq!(
        view.markers("kimi", "default"),
        " · this session".to_string()
    );
    assert_eq!(view.markers("kimi", "alt"), " · default for new sessions");
    assert!(view.in_use("zai", "work"));
    assert!(view.in_use("kimi", "alt"));
    assert!(!view.in_use("zai", "spare"));
    let zai = vec!["zai".to_string()];
    assert_eq!(
        view.rows_for(&zai)
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        vec!["work", "spare"]
    );
}

#[test]
fn only_providers_that_can_hold_accounts_offer_add() {
    let api_key = ModelProviderInfo {
        env_key: Some("ZAI_API_KEY".to_string()),
        ..Default::default()
    };
    assert_eq!(add_methods("zai", &api_key), vec![AddAccountMethod::ApiKey]);
    // PF-84-S06: the OpenAI sign-in provider takes ChatGPT and API-key sign-ins.
    assert_eq!(
        add_methods(
            "openai",
            &ModelProviderInfo::create_openai_provider(/*base_url*/ None)
        ),
        vec![
            AddAccountMethod::ChatgptDeviceCode,
            AddAccountMethod::OpenaiApiKey
        ]
    );
    // An OpenAI-auth provider that also takes an env key stays default-only.
    let openai_login = ModelProviderInfo {
        env_key: Some("OPENAI_API_KEY".to_string()),
        requires_openai_auth: true,
        ..Default::default()
    };
    assert_eq!(add_methods("openai", &openai_login), Vec::new());
    assert_eq!(add_methods("Bad Id", &api_key), Vec::new());
    // PF-84-S06: Bedrock takes AWS profile accounts, unless it uses command auth.
    let bedrock = ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None);
    assert_eq!(
        add_methods("amazon-bedrock", &bedrock),
        vec![AddAccountMethod::AwsProfile]
    );
    assert_eq!(
        add_methods("plain", &ModelProviderInfo::default()),
        Vec::new()
    );
}

#[test]
fn rows_describe_kinds_and_fingerprint_only() {
    let mut row = row("claude-plan", "work");
    row.kinds = vec![
        ProviderAccountKind::ClaudeOauthToken,
        ProviderAccountKind::ClaudeConfigDir,
    ];
    assert_eq!(
        row_detail(&row),
        "subscription token + Claude Code login · fp 0123456789ab"
    );
    row.fingerprint = None;
    assert_eq!(row_detail(&row), "subscription token + Claude Code login");
    assert_eq!(
        format!("{:?}", AccountValue::new("sk-canary-value".to_string())),
        "<redacted account value>"
    );
}

#[test]
fn save_rename_and_remove_touch_only_that_account() {
    let home = tempfile::tempdir().expect("home");
    let canary = "sk-pf84-s04-canary-0000";
    for account in ["work", "keep"] {
        save_account(
            home.path().to_path_buf(),
            "zai",
            &name(account),
            AddAccountMethod::ApiKey,
            AccountValue::new(format!("  {canary}-{account}\n")),
        )
        .expect("save");
    }
    let rows = load_rows(home.path()).expect("rows");
    assert_eq!(
        rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
        vec!["keep", "work"]
    );
    assert!(rows.iter().all(|row| {
        row.fingerprint
            .as_ref()
            .is_some_and(|fp| fp.len() == 12 && !fp.contains(canary))
    }));
    let vault = Vault::new(home.path().to_path_buf());
    assert_eq!(
        vault
            .read_provider_account("zai", &name("work"), ProviderAccountKind::ApiKey)
            .expect("read")
            .map(|value| value.to_string()),
        Some(format!("{canary}-work"))
    );

    rename_account(
        home.path().to_path_buf(),
        "zai",
        &name("work"),
        &name("home"),
    )
    .expect("rename");
    let labels_after_rename = vault
        .list()
        .expect("list")
        .into_iter()
        .map(|entry| entry.label)
        .collect::<Vec<_>>();
    assert_eq!(
        labels_after_rename,
        vec![
            "provider/zai/accounts/home/api_key".to_string(),
            "provider/zai/accounts/keep/api_key".to_string(),
        ]
    );

    remove_account(home.path().to_path_buf(), "zai", &name("home"), &route()).expect("remove");
    assert_eq!(
        vault
            .list()
            .expect("list")
            .into_iter()
            .map(|entry| entry.label)
            .collect::<Vec<_>>(),
        vec!["provider/zai/accounts/keep/api_key".to_string()]
    );
    assert!(remove_account(home.path().to_path_buf(), "zai", &name("home"), &route()).is_err());
    assert!(
        save_account(
            home.path().to_path_buf(),
            "zai",
            &name("empty"),
            AddAccountMethod::ApiKey,
            AccountValue::new("   ".to_string()),
        )
        .is_err()
    );
}

/// PF-84-S06: an AWS profile account stores a validated, non-secret profile
/// name that the row shows.
#[test]
fn aws_profile_accounts_store_and_show_the_profile_name() {
    let home = tempfile::tempdir().expect("home");
    assert_eq!(
        save_account(
            home.path().to_path_buf(),
            "amazon-bedrock",
            &name("work"),
            AddAccountMethod::AwsProfile,
            AccountValue::new("not a profile".to_string()),
        ),
        Err(codex_vault::ProviderAccountError::InvalidAwsProfile.to_string())
    );
    save_account(
        home.path().to_path_buf(),
        "amazon-bedrock",
        &name("work"),
        AddAccountMethod::AwsProfile,
        AccountValue::new(" corp-dev \n".to_string()),
    )
    .expect("save");
    let rows = load_rows(home.path()).expect("rows");
    assert_eq!(
        rows.iter()
            .map(|row| (row.aws_profile.clone(), row.kinds.clone()))
            .collect::<Vec<_>>(),
        vec![(
            Some("corp-dev".to_string()),
            vec![ProviderAccountKind::AwsProfile]
        )]
    );
    let mut row = rows[0].clone();
    row.fingerprint = Some("0123456789ab".to_string());
    assert_eq!(row_detail(&row), "AWS profile `corp-dev` · fp 0123456789ab");
}

/// PF-84-S06: an OpenAI API-key sign-in is stored as that account's whole
/// sign-in and the row shows only its kind; the key never appears.
#[test]
fn openai_api_key_accounts_store_a_sign_in_and_show_no_key() {
    let home = tempfile::tempdir().expect("home");
    let canary = "sk-pf84-s06-canary-0001";
    save_account(
        home.path().to_path_buf(),
        "openai",
        &name("work"),
        AddAccountMethod::OpenaiApiKey,
        AccountValue::new(format!(" {canary}\n")),
    )
    .expect("save");
    let rows = load_rows(home.path()).expect("rows");
    assert_eq!(
        rows.iter()
            .map(|row| (row.kinds.clone(), row.sign_in.clone()))
            .collect::<Vec<_>>(),
        vec![(
            vec![ProviderAccountKind::ChatgptAuth],
            Some("OpenAI API key".to_string())
        )]
    );
    assert!(!row_detail(&rows[0]).contains(canary));
    let stored: codex_login::AuthDotJson = serde_json::from_str(
        &Vault::new(home.path().to_path_buf())
            .read_provider_account("openai", &name("work"), ProviderAccountKind::ChatgptAuth)
            .expect("read")
            .expect("stored"),
    )
    .expect("sign-in json");
    assert_eq!(stored.openai_api_key.as_deref(), Some(canary));
    assert_eq!(
        save_account(
            home.path().to_path_buf(),
            "openai",
            &name("other"),
            AddAccountMethod::ChatgptDeviceCode,
            AccountValue::new(String::new()),
        ),
        Err("The value cannot be empty.".to_string())
    );
}

/// PF-84-S06: a ChatGPT sign-in row shows its kind, email hint and plan, and
/// never a token.
#[test]
fn chatgpt_sign_in_rows_show_the_email_hint_and_plan_only() {
    let home = tempfile::tempdir().expect("home");
    let jwt = "e30.eyJlbWFpbCI6ImFubkBleGFtcGxlLmNvbSIsImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJjaGF0Z3B0X3BsYW5fdHlwZSI6InBsdXMiLCJjaGF0Z3B0X3VzZXJfaWQiOiJ1LTEifX0.c2ln";
    let sign_in = codex_login::AuthDotJson {
        auth_mode: Some(codex_protocol::auth::AuthMode::Chatgpt),
        openai_api_key: None,
        tokens: Some(codex_login::token_data::TokenData {
            id_token: codex_login::token_data::parse_chatgpt_jwt_claims(jwt).expect("claims"),
            access_token: "access-canary-0002".to_string(),
            refresh_token: "refresh-canary-0002".to_string(),
            account_id: Some("ws-1".to_string()),
        }),
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    };
    codex_login::save_named_account_auth(
        home.path(),
        &codex_login::NamedChatgptAccount {
            provider_id: "openai".to_string(),
            name: "work".to_string(),
        },
        &sign_in,
        /*must_be_new*/ true,
    )
    .expect("save");
    let rows = load_rows(home.path()).expect("rows");
    let detail = row_detail(&rows[0]);
    assert!(
        detail.starts_with("ChatGPT sign-in · a…@example.com · Plus · fp ")
            && !detail.contains("canary"),
        "{detail}"
    );
}

/// #609: an OpenAI API key added in `/providers` records that it is one, so
/// status tells it from a ChatGPT sign-in without reading it.
#[test]
fn an_openai_api_key_added_here_records_what_it_holds() {
    let home = tempfile::tempdir().expect("home");
    save_account(
        home.path().to_path_buf(),
        "openai",
        &name("work"),
        AddAccountMethod::OpenaiApiKey,
        AccountValue::new("sk-canary-609-add".to_string()),
    )
    .expect("save");
    assert_eq!(
        codex_login::provider_account_sign_in(home.path(), "openai", "work").expect("read"),
        Some(codex_vault::ProviderAccountSignIn::ApiKey)
    );
}
