use std::collections::BTreeMap;

use codex_model_provider_info::ModelProviderInfo;
use pretty_assertions::assert_eq;

use super::*;

fn name(raw: &str) -> ProviderAccountName {
    ProviderAccountName::parse(raw).expect("valid name")
}

fn row(provider_id: &str, account: &str) -> NamedAccountRow {
    NamedAccountRow {
        provider_id: provider_id.to_string(),
        name: name(account),
        kinds: vec![ProviderAccountKind::ApiKey],
        fingerprint: Some("0123456789ab".to_string()),
    }
}

fn view() -> AccountsView {
    AccountsView {
        rows: vec![row("zai", "work"), row("zai", "spare"), row("kimi", "alt")],
        session: BTreeMap::from([("zai".to_string(), "work".to_string())]),
        defaults: BTreeMap::from([("kimi".to_string(), "alt".to_string())]),
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
    // OpenAI sign-in stays default-only until the pending S06 decision.
    let openai_login = ModelProviderInfo {
        env_key: Some("OPENAI_API_KEY".to_string()),
        requires_openai_auth: true,
        ..Default::default()
    };
    assert_eq!(add_methods("openai", &openai_login), Vec::new());
    assert_eq!(add_methods("Bad Id", &api_key), Vec::new());
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

    remove_account(home.path().to_path_buf(), "zai", &name("home")).expect("remove");
    assert_eq!(
        vault
            .list()
            .expect("list")
            .into_iter()
            .map(|entry| entry.label)
            .collect::<Vec<_>>(),
        vec!["provider/zai/accounts/keep/api_key".to_string()]
    );
    assert!(remove_account(home.path().to_path_buf(), "zai", &name("home")).is_err());
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
