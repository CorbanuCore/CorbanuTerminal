//! PF-84: `corbanu account` manages named accounts per provider. It shows
//! names and kinds only and reads secret values from stdin, never argv.

use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;

use anyhow::Context;
use anyhow::bail;
use codex_core::config::Config;
use codex_core::config::ConfigBuilder;
use codex_features::Feature;
use codex_utils_cli::CliConfigOverrides;
use codex_vault::ProviderAccountKind;
use codex_vault::ProviderAccountName;
use codex_vault::Vault;
use zeroize::Zeroizing;

/// Named Claude accounts are served by the built-in Claude Plan provider only.
const CLAUDE_PLAN_PROVIDER_ID: &str = "claude-plan";

#[derive(Debug, clap::Parser)]
pub struct AccountCommand {
    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    action: AccountSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum AccountSubcommand {
    /// List named accounts: provider, name and kinds (never values).
    List {
        /// Only this provider id.
        provider: Option<String>,
    },
    /// Add or replace a named account. Secret values are read from stdin.
    Add {
        /// Provider id, for example `zai` or `claude-plan`.
        provider: String,
        /// Account name: 1-32 lowercase letters, digits or '-'.
        name: String,
        /// What the account holds.
        #[arg(long, value_enum, default_value_t = AccountKindArg::ApiKey)]
        kind: AccountKindArg,
        /// The value for the non-secret `claude-config-dir` kind.
        #[arg(long)]
        value: Option<String>,
    },
    /// Remove a named account and everything it holds.
    Remove { provider: String, name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum AccountKindArg {
    /// A provider API key (stdin).
    ApiKey,
    /// A Claude subscription token from `claude setup-token` (stdin).
    ClaudeToken,
    /// The `CLAUDE_CONFIG_DIR` of a Claude Code login (`--value`).
    ClaudeConfigDir,
    /// An account of an `auth.command` provider; the command receives its name
    /// in `CORBANU_PROVIDER_ACCOUNT`.
    Command,
}

impl AccountKindArg {
    fn kind(self) -> ProviderAccountKind {
        match self {
            Self::ApiKey => ProviderAccountKind::ApiKey,
            Self::ClaudeToken => ProviderAccountKind::ClaudeOauthToken,
            Self::ClaudeConfigDir => ProviderAccountKind::ClaudeConfigDir,
            Self::Command => ProviderAccountKind::Command,
        }
    }

    fn reads_stdin(self) -> bool {
        matches!(self, Self::ApiKey | Self::ClaudeToken)
    }
}

pub async fn run_account_command(command: AccountCommand) -> anyhow::Result<()> {
    let cli_kv_overrides = command
        .config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    // As for `vault auth-helper`, the persisted posture cannot be lowered with `-c`.
    let persisted_config = ConfigBuilder::default().build().await?;
    let config = ConfigBuilder::default()
        .cli_overrides(cli_kv_overrides)
        .build()
        .await?;
    if !config.features.enabled(Feature::NamedAccounts) {
        bail!(
            "named accounts are off; enable them with `[features] named_accounts = true` \
             or `--enable named_accounts`"
        );
    }
    let vault = Vault::new(config.codex_home.to_path_buf());
    let mut stdout = std::io::stdout();
    match command.action {
        AccountSubcommand::List { provider } => {
            let accounts = vault.list_provider_accounts()?;
            let accounts = accounts
                .iter()
                .filter(|account| {
                    provider
                        .as_deref()
                        .is_none_or(|id| account.provider_id == id)
                })
                .collect::<Vec<_>>();
            if accounts.is_empty() {
                writeln!(
                    stdout,
                    "No named accounts. Every provider uses its default account."
                )?;
            }
            for account in accounts {
                let kinds = account
                    .kinds
                    .iter()
                    .map(|kind| kind.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(stdout, "{}\t{}\t{kinds}", account.provider_id, account.name)?;
            }
        }
        AccountSubcommand::Add {
            provider,
            name,
            kind,
            value,
        } => {
            ensure_changes_allowed(&persisted_config, &config)?;
            let name = ProviderAccountName::parse(&name)?;
            ensure_kind_fits_provider(&config, &provider, kind)?;
            let value = account_value(kind, value)?;
            vault.write_provider_account(&provider, &name, kind.kind(), &value)?;
            writeln!(
                stdout,
                "Saved {provider} account `{name}` ({}).",
                kind.kind().as_str()
            )?;
        }
        AccountSubcommand::Remove { provider, name } => {
            ensure_changes_allowed(&persisted_config, &config)?;
            let name = ProviderAccountName::parse(&name)?;
            if vault.remove_provider_account(&provider, &name)? {
                writeln!(stdout, "Removed {provider} account `{name}`.")?;
            } else {
                bail!("{provider} has no account `{name}`");
            }
        }
    }
    Ok(())
}

/// Aggressive allows no vault changes from a command an agent can run.
fn ensure_changes_allowed(persisted: &Config, config: &Config) -> anyhow::Result<()> {
    let level = persisted.security_level.max(config.security_level);
    if level == codex_security_policy::SecurityLevel::Aggressive {
        bail!("adding or removing accounts is unavailable under Aggressive; use Permissive");
    }
    Ok(())
}

fn ensure_kind_fits_provider(
    config: &Config,
    provider_id: &str,
    kind: AccountKindArg,
) -> anyhow::Result<()> {
    let provider = config
        .model_providers
        .get(provider_id)
        .with_context(|| format!("unknown provider `{provider_id}`"))?;
    let fits = match kind {
        AccountKindArg::ApiKey => provider.env_key.is_some() && provider.aws.is_none(),
        AccountKindArg::ClaudeToken | AccountKindArg::ClaudeConfigDir => {
            provider_id == CLAUDE_PLAN_PROVIDER_ID
        }
        AccountKindArg::Command => provider.auth.is_some() && !provider.is_claude_plan(),
    };
    if !fits {
        bail!(
            "provider `{provider_id}` does not take a {} account",
            kind.kind().as_str()
        );
    }
    Ok(())
}

fn account_value(kind: AccountKindArg, value: Option<String>) -> anyhow::Result<Zeroizing<String>> {
    if kind.reads_stdin() {
        if value.is_some() {
            bail!("secret values are read from stdin, never from --value");
        }
        // Typing a secret into a terminal would echo it on screen.
        if std::io::stdin().is_terminal() {
            bail!(
                "pipe the secret in instead of typing it, for example \
                 `cat key.txt | corbanu account add <provider> <name>`"
            );
        }
        let mut secret = Zeroizing::new(String::new());
        std::io::stdin()
            .read_to_string(&mut secret)
            .context("failed to read the secret from stdin")?;
        let trimmed = Zeroizing::new(secret.trim().to_string());
        if trimmed.is_empty() {
            bail!("no secret on stdin");
        }
        return Ok(trimmed);
    }
    match (kind, value) {
        (AccountKindArg::Command, None) => Ok(Zeroizing::new("1".to_string())),
        (AccountKindArg::Command, Some(_)) => bail!("a command account takes no --value"),
        (_, Some(value)) if !value.trim().is_empty() => {
            Ok(Zeroizing::new(value.trim().to_string()))
        }
        _ => bail!("this kind needs --value"),
    }
}
