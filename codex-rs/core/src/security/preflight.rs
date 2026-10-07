//! PF-29-S01 protected-mode activation preflight (feature
//! `protected_mode_preflight`).
//!
//! A protected level may be chosen only when the boundary it claims holds:
//! the secretless launch contract, isolated credential broker and output
//! gate are on, the OS sandbox is supported, the vault is usable, no provider
//! signs in through an unchecked command, and the [inventory](super::inventory)
//! finds no raw secret that would still reach an agent or the model. Paths the
//! level can deny are listed for isolation instead of blocking.
//!
//! The preflight only reads; it never runs provider commands, opens the vault
//! or changes a file. [`Preflight::recheck`] detects drift between the review
//! and the confirmation by comparing keyed manifests.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use codex_config::ConfigLayerSource;
use codex_features::Feature;

pub use super::inventory::ConfigLayerInput;
pub use super::inventory::Disposition;
pub use super::inventory::EntryStatus;
pub use super::inventory::Finding;
pub use super::inventory::FindingKind;
pub use super::inventory::Inventory;
pub use super::inventory::InventorySources;
pub use super::inventory::LIMITS;
pub use super::inventory::Manifest;
pub use super::inventory::ManifestEntry;
pub use super::inventory::Scope;
pub use super::inventory::SecretClass;
pub use super::inventory::VaultState;
use super::inventory::take_with_key;
/// PF-29-S02 migration of blocking findings.
pub use super::migration;
use crate::config::Config;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadinessFlags {
    pub secretless_launch: bool,
    pub credential_broker: bool,
    pub output_gate: bool,
    /// PF-24-S02: `source_envelopes`. Core's protected levels refuse model
    /// requests that carry unlabelled external content, so without it a
    /// session cannot work under them.
    pub untrusted_content: bool,
}

impl ReadinessFlags {
    pub fn from_config(config: &Config) -> Self {
        Self {
            secretless_launch: config.features.enabled(Feature::SecretlessAgentLaunch),
            credential_broker: config.features.enabled(Feature::IsolatedCredentialBroker),
            output_gate: config.features.enabled(Feature::SecretOutputGate),
            untrusted_content: config.features.enabled(Feature::SourceEnvelopes),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadinessState {
    Ready,
    /// A required control is off; the text says how to turn it on.
    Missing(String),
    /// Present but not usable as checked.
    Incomplete(String),
    /// A check that would run a command was skipped; it needs the human.
    ConsentRequired(String),
    Unsupported(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadinessItem {
    pub id: String,
    pub label: String,
    pub state: ReadinessState,
}

impl ReadinessItem {
    pub fn is_ready(&self) -> bool {
        self.state == ReadinessState::Ready
    }
}

#[derive(Clone)]
pub struct Preflight {
    pub readiness: Vec<ReadinessItem>,
    pub inventory: Inventory,
    key: [u8; 32],
}

impl std::fmt::Debug for Preflight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Preflight")
            .field("readiness", &self.readiness)
            .field("inventory", &self.inventory)
            .finish_non_exhaustive()
    }
}

/// What changed between two preflights, by finding ID.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Drift {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    pub readiness_changed: bool,
}

impl Drift {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && !self.readiness_changed
    }
}

impl Preflight {
    pub fn run(sources: &InventorySources, flags: ReadinessFlags) -> Self {
        Self::run_with_key(sources, flags, rand::random())
    }

    pub(crate) fn run_with_key(
        sources: &InventorySources,
        flags: ReadinessFlags,
        key: [u8; 32],
    ) -> Self {
        let inventory = take_with_key(sources, key);
        let readiness = readiness(&inventory, flags);
        Self {
            readiness,
            inventory,
            key,
        }
    }

    /// Clean only when every control is ready and no finding blocks.
    pub fn is_clean(&self) -> bool {
        self.readiness.iter().all(ReadinessItem::is_ready)
            && self.inventory.blocking().next().is_none()
    }

    /// One line per blocker. Locations and remedies only, never values.
    pub fn blockers(&self) -> Vec<String> {
        let mut lines = self
            .readiness
            .iter()
            .filter_map(|item| {
                let reason = match &item.state {
                    ReadinessState::Ready => return None,
                    ReadinessState::Missing(reason)
                    | ReadinessState::Incomplete(reason)
                    | ReadinessState::ConsentRequired(reason)
                    | ReadinessState::Unsupported(reason) => reason,
                };
                Some(format!("{}: {reason}", item.label))
            })
            .collect::<Vec<_>>();
        lines.extend(self.inventory.blocking().map(|finding| {
            let remedy = match finding.disposition {
                Disposition::RemoveFromContext => {
                    "the model reads memories; remove the secret from this file"
                }
                _ => "move it into the vault and use a reference, or remove it",
            };
            format!(
                "{}: {} ({}); {remedy} [{}]",
                finding.location,
                finding.class.as_str(),
                finding.kind.as_str().replace('-', " "),
                finding.id
            )
        }));
        lines
    }

    /// Takes the inventory again with the same key and reports any change.
    pub fn recheck(&self, sources: &InventorySources, flags: ReadinessFlags) -> (Self, Drift) {
        let next = Self::run_with_key(sources, flags, self.key);
        let drift = drift(self, &next);
        (next, drift)
    }
}

fn readiness(inventory: &Inventory, flags: ReadinessFlags) -> Vec<ReadinessItem> {
    let item = |id: &str, label: &str, state: ReadinessState| ReadinessItem {
        id: id.to_string(),
        label: label.to_string(),
        state,
    };
    let required = |on: bool, key: &str| {
        if on {
            ReadinessState::Ready
        } else {
            ReadinessState::Missing(format!("off; turn on the `{key}` feature"))
        }
    };
    let sandbox = if cfg!(any(target_os = "macos", target_os = "linux")) {
        ReadinessState::Ready
    } else {
        ReadinessState::Unsupported(
            "protected launch is not available on this platform yet (PF-27-S06)".to_string(),
        )
    };
    let vault = match inventory.vault {
        VaultState::Absent | VaultState::Present => ReadinessState::Ready,
        VaultState::Unusable => ReadinessState::Incomplete(
            "a vault store file cannot be read or is damaged".to_string(),
        ),
    };
    let mut items = vec![
        item("os-sandbox", "OS sandbox", sandbox),
        item(
            "secretless-launch",
            "Secretless agent launch",
            required(flags.secretless_launch, "secretless_agent_launch"),
        ),
        item(
            "credential-broker",
            "Isolated credential broker",
            required(flags.credential_broker, "isolated_credential_broker"),
        ),
        item(
            "output-gate",
            "Secret output gate",
            required(flags.output_gate, "secret_output_gate"),
        ),
        item(
            "untrusted-content",
            "Untrusted-content labels",
            required(flags.untrusted_content, "source_envelopes"),
        ),
        item("vault", "Vault", vault),
    ];
    if inventory.migration_unfinished {
        items.push(item(
            "migration",
            "Credential migration",
            ReadinessState::Incomplete(
                "a credential migration did not finish; press r in this review to finish it"
                    .to_string(),
            ),
        ));
    }
    items.extend(inventory.exec_provider_auth.iter().map(|provider| {
        item(
            &format!("exec-provider-auth-{provider}"),
            &format!("Provider `{provider}` sign-in"),
            ReadinessState::ConsentRequired(format!(
                "signs in by running a command outside the protected boundary; preflight did not run it. Remove `model_providers.{provider}.auth` or use an environment key"
            )),
        )
    }));
    items
}

fn drift(before: &Preflight, after: &Preflight) -> Drift {
    let ids = |preflight: &Preflight| preflight.inventory.finding_ids();
    let (old, new) = (ids(before), ids(after));
    let entries = |preflight: &Preflight| {
        preflight
            .inventory
            .manifest
            .entries
            .iter()
            .map(|entry| (entry.finding_id.clone(), entry.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    let (old_entries, new_entries) = (entries(before), entries(after));
    Drift {
        added: new.difference(&old).cloned().collect(),
        removed: old.difference(&new).cloned().collect(),
        changed: old
            .intersection(&new)
            .filter(|id| old_entries.get(*id) != new_entries.get(*id))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        readiness_changed: before.readiness != after.readiness,
    }
}

/// Corbanu's own stores that a protected launch denies even before they
/// exist: Corbanu creates them during startup, after launch takes its
/// inventory.
pub const CORBANU_HOME_STORES: [&str; 9] = [
    ".credentials.json",
    "provider_auth.json",
    ".env",
    "wallet",
    "sessions",
    "archived_sessions",
    "history.jsonl",
    "shell_snapshots",
    "log",
];

/// Deny-read glob for Corbanu's state databases and their `-wal`/`-shm`
/// files, which come and go while Corbanu runs.
pub fn database_glob(codex_home: &std::path::Path) -> std::path::PathBuf {
    codex_home.join("*.sqlite*")
}

/// Whether `path` contains glob syntax, so it cannot prefix a glob pattern.
pub fn has_glob_chars(path: &std::path::Path) -> bool {
    path.to_string_lossy()
        .contains(['*', '?', '[', ']', '{', '}'])
}

/// The `-wal`, `-shm` and `-journal` files that come and go beside a
/// database, whether or not they exist now.
pub fn database_siblings(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let name = path.to_string_lossy();
    let base = name
        .split_once(".sqlite")
        .map_or(name.as_ref(), |(base, _)| base);
    ["", "-wal", "-shm", "-journal"]
        .iter()
        .map(|suffix| std::path::PathBuf::from(format!("{base}.sqlite{suffix}")))
        .collect()
}

/// Whether `path` is one of the files [`database_glob`] denies.
pub fn is_database_file(codex_home: &std::path::Path, path: &std::path::Path) -> bool {
    path.parent() == Some(codex_home)
        && path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains(".sqlite"))
}

/// The home directory the inventory checks (`$HOME` on Unix).
pub fn home_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir()
}

/// Sources for this machine and the loaded config: the process environment,
/// the home directory and every enabled config layer (shadowed values too).
pub fn sources_from_config(config: &Config) -> InventorySources {
    let config_layers = config
        .config_layer_stack
        .layers_high_to_low()
        .into_iter()
        .filter(|layer| !layer.is_disabled())
        .map(|layer| {
            let (label, path) = match &layer.name {
                ConfigLayerSource::User { file, profile } => (
                    match profile {
                        Some(profile) => format!("user config (profile {profile})"),
                        None => "user config".to_string(),
                    },
                    Some(file.to_path_buf()),
                ),
                ConfigLayerSource::Project { dot_codex_folder } => (
                    "project config".to_string(),
                    Some(dot_codex_folder.join("config.toml").to_path_buf()),
                ),
                ConfigLayerSource::System { file }
                | ConfigLayerSource::LegacyManagedConfigTomlFromFile { file } => {
                    ("managed config".to_string(), Some(file.to_path_buf()))
                }
                ConfigLayerSource::SessionFlags => ("launch flags".to_string(), None),
                ConfigLayerSource::Mdm { .. }
                | ConfigLayerSource::EnterpriseManaged { .. }
                | ConfigLayerSource::LegacyManagedConfigTomlFromMdm => {
                    ("managed config".to_string(), None)
                }
            };
            ConfigLayerInput {
                label,
                path,
                toml: layer.config.clone(),
            }
        })
        .collect();
    InventorySources {
        codex_home: config.codex_home.to_path_buf(),
        home: dirs::home_dir(),
        cwd: config.cwd.to_path_buf(),
        env: std::env::vars().collect(),
        config_layers,
    }
}

/// Sources available before any config is loaded (launch-time isolation):
/// files, plus the user and project `config.toml` read directly; no
/// environment.
pub fn file_sources(
    codex_home: &std::path::Path,
    home: Option<&std::path::Path>,
    cwd: Option<&std::path::Path>,
) -> InventorySources {
    let mut files = vec![("user config", codex_home.join("config.toml"))];
    if let Some(cwd) = cwd {
        files.push(("project config", cwd.join(".codex").join("config.toml")));
    }
    let config_layers = files
        .into_iter()
        .filter_map(|(label, path)| {
            let toml = toml::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
            Some(ConfigLayerInput {
                label: label.to_string(),
                path: Some(path),
                toml,
            })
        })
        .collect();
    InventorySources {
        codex_home: codex_home.to_path_buf(),
        home: home.map(std::path::Path::to_path_buf),
        cwd: cwd.map(std::path::Path::to_path_buf).unwrap_or_default(),
        env: Vec::new(),
        config_layers,
    }
}
