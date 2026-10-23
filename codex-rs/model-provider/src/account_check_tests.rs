use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use pretty_assertions::assert_eq;

use super::*;

/// PF-84-S06: an unconfigured Bedrock account is refused before the session
/// starts, with the `aws-profile` recovery command.
#[test]
fn unconfigured_bedrock_account_names_the_profile_kind() {
    // A home with no vault: the account cannot be configured there.
    let home = std::env::temp_dir().join("pf84-s06-home-without-a-vault");
    let mut provider = ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None);
    provider.account = Some(NamedProviderAccount {
        provider_id: "amazon-bedrock".to_string(),
        name: "work".to_string(),
    });
    assert_eq!(
        selected_account_error(&home, &provider),
        Some(
            "account `work` of provider `amazon-bedrock` is not configured; add it with \
             `corbanu account add amazon-bedrock work --kind aws-profile --value <profile>` or \
             pick another with `--account` (see `corbanu account list`)"
                .to_string()
        )
    );
}

/// PF-84 #555: inside the TUI the recovery is `/providers`, not a CLI flag.
#[test]
fn tui_recovery_points_to_providers() {
    let home = std::env::temp_dir().join("pf84-555-home-without-a-vault");
    let mut provider = ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None);
    provider.account = Some(NamedProviderAccount {
        provider_id: "amazon-bedrock".to_string(),
        name: "work".to_string(),
    });
    assert_eq!(
        selected_account_error_with_recovery(&home, &provider, AccountRecovery::Providers),
        Some(
            "account `work` of provider `amazon-bedrock` is not configured; add it or pick \
             another account in `/providers`"
                .to_string()
        )
    );
}
