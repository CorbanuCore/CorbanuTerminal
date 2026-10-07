//! PF-29-S01 inventory and preflight regressions. Synthetic secrets only.

use std::path::Path;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::preflight::ConfigLayerInput;
use super::preflight::Disposition;
use super::preflight::EntryStatus;
use super::preflight::Finding;
use super::preflight::FindingKind;
use super::preflight::InventorySources;
use super::preflight::Preflight;
use super::preflight::ReadinessFlags;
use super::preflight::ReadinessState;
use super::preflight::SecretClass;

const FAKE_KEY: &str = "sk-pf29fakeAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const FAKE_CORE_VALUE: &str = "pf29-core-managed-value-0001";
const KEY: [u8; 32] = [7; 32];
const ALL_ON: ReadinessFlags = ReadinessFlags {
    secretless_launch: true,
    credential_broker: true,
    output_gate: true,
    untrusted_content: true,
};

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().unwrap_or_else(|err| panic!("tempdir: {err}"));
        for dir in ["home", "corbanu", "work"] {
            std::fs::create_dir_all(root.path().join(dir))
                .unwrap_or_else(|err| panic!("mkdir: {err}"));
        }
        Self { root }
    }

    fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    fn corbanu(&self) -> PathBuf {
        self.root.path().join("corbanu")
    }

    fn write(&self, path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|err| panic!("mkdir: {err}"));
        }
        std::fs::write(path, contents).unwrap_or_else(|err| panic!("write: {err}"));
    }

    fn sources(&self) -> InventorySources {
        InventorySources {
            codex_home: self.corbanu(),
            home: Some(self.home()),
            cwd: self.root.path().join("work"),
            env: Vec::new(),
            config_layers: Vec::new(),
        }
    }

    fn run(&self, sources: &InventorySources) -> Preflight {
        Preflight::run_with_key(sources, ALL_ON, KEY)
    }
}

fn find(preflight: &Preflight, kind: FindingKind) -> Vec<&Finding> {
    preflight
        .inventory
        .findings
        .iter()
        .filter(|finding| finding.kind == kind)
        .collect()
}

fn layer(label: &str, path: Option<PathBuf>, toml: &str) -> ConfigLayerInput {
    ConfigLayerInput {
        label: label.to_string(),
        path,
        toml: toml::from_str(toml).unwrap_or_else(|err| panic!("toml: {err}")),
    }
}

#[test]
fn pf_29_s01_empty_machine_is_clean_only_when_controls_are_ready() {
    let fixture = Fixture::new();
    let sources = fixture.sources();
    assert!(fixture.run(&sources).is_clean());

    let off = Preflight::run_with_key(&sources, ReadinessFlags::default(), KEY);
    assert!(!off.is_clean());
    let blockers = off.blockers();
    for key in [
        "secretless_agent_launch",
        "isolated_credential_broker",
        "secret_output_gate",
    ] {
        assert!(
            blockers.iter().any(|line| line.contains(key)),
            "{key} missing from {blockers:?}"
        );
    }
}

#[test]
fn pf_29_s01_credential_files_and_ssh_keys_are_isolated_not_blocking() {
    let fixture = Fixture::new();
    let home = fixture.home();
    fixture.write(
        &home.join(".ssh/id_ed25519"),
        b"-----BEGIN OPENSSH PRIVATE KEY-----\nfake\n",
    );
    fixture.write(&home.join(".ssh/id_ed25519.pub"), b"ssh-ed25519 AAAA fake");
    fixture.write(
        &home.join(".ssh/deploy"),
        b"-----BEGIN RSA PRIVATE KEY-----\nfake\n",
    );
    fixture.write(&home.join(".ssh/config"), b"Host *\n");
    fixture.write(&home.join(".kube/config"), b"token: fake");
    fixture.write(&home.join(".netrc"), b"machine x login y password fake");
    let preflight = fixture.run(&fixture.sources());

    let isolated = preflight.inventory.isolation_paths();
    // Whole credential folders, so later or oddly named keys are covered.
    for path in [".ssh", ".ssh/id_ed25519", ".ssh/deploy", ".kube", ".netrc"] {
        assert!(isolated.contains(&home.join(path)), "{path} not isolated");
    }
    for path in [".ssh/id_ed25519.pub", ".ssh/config", ".kube/config"] {
        assert!(
            !isolated.contains(&home.join(path)),
            "{path} listed separately"
        );
    }
    assert_eq!(find(&preflight, FindingKind::SshPrivateKey).len(), 2);
    assert!(preflight.is_clean(), "{:?}", preflight.blockers());
}

#[test]
fn pf_29_s01_shell_profile_exports_block_and_never_expose_values() {
    let fixture = Fixture::new();
    let profile = format!(
        "# comment {FAKE_KEY}\nexport OPENAI_API_KEY=\"{FAKE_KEY}\"\nexport SAFE_URL=https://example.com\nexport GH_TOKEN=\"$(corbanu vault auth-helper gh)\"\nPLAIN={FAKE_KEY}\n"
    );
    fixture.write(&fixture.home().join(".zshrc"), profile.as_bytes());
    fixture.write(
        &fixture.home().join(".config/fish/config.fish"),
        format!("set -gx ANTHROPIC_API_KEY {FAKE_KEY}\n").as_bytes(),
    );
    let preflight = fixture.run(&fixture.sources());

    let exports = find(&preflight, FindingKind::ShellProfileExport);
    let locations = exports
        .iter()
        .map(|finding| finding.location.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        locations,
        vec![
            "~/.zshrc:2 OPENAI_API_KEY".to_string(),
            "~/.zshrc:5 PLAIN".to_string(),
            "~/.config/fish/config.fish:1 ANTHROPIC_API_KEY".to_string(),
        ]
    );
    assert!(
        exports
            .iter()
            .all(|finding| finding.disposition == Disposition::Migrate)
    );
    let references = find(&preflight, FindingKind::Reference);
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].class, SecretClass::PermittedDerived);
    assert!(!preflight.is_clean());

    let rendered = format!("{preflight:?}{:?}", preflight.blockers());
    assert!(!rendered.contains(FAKE_KEY), "value leaked");
    assert!(!rendered.contains("pf29fake"), "partial value leaked");
}

#[test]
fn pf_29_s01_environment_is_classified_without_values() {
    let fixture = Fixture::new();
    let mut sources = fixture.sources();
    sources.env = vec![
        ("OPENAI_API_KEY".to_string(), FAKE_KEY.to_string()),
        ("MY_SERVICE_TOKEN".to_string(), FAKE_CORE_VALUE.to_string()),
        (
            "DATABASE_URL".to_string(),
            "postgres://app:hunter22@db/x".to_string(),
        ),
        ("INNOCENT".to_string(), FAKE_KEY.to_string()),
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("EMPTY_TOKEN".to_string(), String::new()),
    ];
    let preflight = fixture.run(&sources);
    let env = find(&preflight, FindingKind::EnvironmentVariable)
        .into_iter()
        .map(|finding| (finding.location.clone(), finding.class, finding.disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        env,
        vec![
            (
                "environment DATABASE_URL".to_string(),
                SecretClass::UnmanagedSecret,
                Disposition::Migrate
            ),
            (
                "environment INNOCENT".to_string(),
                SecretClass::UnmanagedSecret,
                Disposition::Migrate
            ),
            (
                "environment MY_SERVICE_TOKEN".to_string(),
                SecretClass::UnmanagedSecret,
                Disposition::Denied
            ),
            (
                "environment OPENAI_API_KEY".to_string(),
                SecretClass::ManagedSecret,
                Disposition::Denied
            ),
        ]
    );
    assert_eq!(preflight.inventory.ordinary_env_count, 1);
    assert!(!format!("{preflight:?}").contains("hunter22"));
}

#[test]
fn pf_29_s01_config_layers_including_shadowed_values() {
    let fixture = Fixture::new();
    let user_path = fixture.corbanu().join("config.toml");
    let user = layer(
        "user config",
        Some(user_path.clone()),
        &format!(
            r#"
notify = ["say"]
[shell_environment_policy.set]
DEPLOY_TOKEN = "{FAKE_CORE_VALUE}"
[mcp_servers.docs]
command = "docs-mcp"
env = {{ DOCS_API_KEY = "{FAKE_KEY}", LOG_LEVEL = "debug" }}
[mcp_servers.remote]
url = "https://mcp.example"
http_headers = {{ Authorization = "Bearer abc", Accept = "json" }}
bearer_token_env_var = "REMOTE_TOKEN"
[model_providers.corp]
experimental_bearer_token = "corp-literal-token"
env_key = "CORP_KEY"
[model_providers.exec]
auth = {{ command = "print-token" }}
[hooks]
"#
        ),
    );
    // A higher layer replaces the server; the user file still holds the value.
    let project = layer(
        "project config",
        Some(fixture.root.path().join("work/.codex/config.toml")),
        r#"
[mcp_servers.docs]
command = "docs-mcp"
"#,
    );
    let flags = layer(
        "launch flags",
        None,
        r#"permissions.p.filesystem."/x/secrets" = "deny""#,
    );
    let mut sources = fixture.sources();
    sources.config_layers = vec![flags, project, user];
    let preflight = fixture.run(&sources);

    let locations = |kind| {
        find(&preflight, kind)
            .into_iter()
            .map(|finding| finding.location.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        locations(FindingKind::McpSecretLiteral),
        vec![
            "user config mcp_servers.docs.env.DOCS_API_KEY".to_string(),
            "user config mcp_servers.remote.http_headers.Authorization".to_string(),
        ]
    );
    assert_eq!(
        locations(FindingKind::ProviderSecretLiteral),
        vec!["user config model_providers.corp.experimental_bearer_token".to_string()]
    );
    let set = find(&preflight, FindingKind::ConfigLiteral);
    assert_eq!(set.len(), 1);
    assert_eq!(
        set[0].location,
        "user config shell_environment_policy.set.DEPLOY_TOKEN"
    );
    assert_eq!(set[0].disposition, Disposition::Isolate);
    assert_eq!(set[0].paths, vec![user_path]);
    let routes = locations(FindingKind::Integration);
    for route in [
        "user config notify",
        "user config hooks",
        "user config mcp_servers.docs",
        "project config mcp_servers.docs",
        "user config mcp_servers.remote",
    ] {
        assert!(
            routes.contains(&route.to_string()),
            "{route} not in {routes:?}"
        );
    }
    assert!(
        find(&preflight, FindingKind::Integration)
            .iter()
            .all(|finding| finding.disposition == Disposition::NotContained)
    );
    assert!(preflight.readiness.iter().any(|item| matches!(
        item.state,
        ReadinessState::ConsentRequired(_)
    ) && item.label.contains("exec")));
    assert!(!format!("{preflight:?}").contains("corp-literal-token"));
}

#[test]
fn pf_29_s01_old_memory_content_blocks_until_removed() {
    let fixture = Fixture::new();
    let memories = fixture.corbanu().join("memories");
    fixture.write(&memories.join("clean.md"), b"prefers short answers");
    fixture.write(
        &memories.join("rollout_summaries/old.md"),
        format!("user pasted {FAKE_CORE_VALUE} once").as_bytes(),
    );
    fixture.write(&memories.join("raw.bin"), &[0xff, 0xfe, 0x00, 0x80]);
    let mut sources = fixture.sources();
    sources.env = vec![("SERVICE_SECRET".to_string(), FAKE_CORE_VALUE.to_string())];
    let preflight = fixture.run(&sources);

    let memory = find(&preflight, FindingKind::Memory);
    assert_eq!(memory.len(), 1);
    assert_eq!(
        memory[0].location,
        memories
            .join("rollout_summaries/old.md")
            .display()
            .to_string()
    );
    assert_eq!(memory[0].disposition, Disposition::RemoveFromContext);
    assert!(!preflight.is_clean());
}

#[cfg(unix)]
#[test]
fn pf_29_s01_symlinks_isolate_targets_and_survive_loops() {
    let fixture = Fixture::new();
    let home = fixture.home();
    let elsewhere = fixture.root.path().join("elsewhere/creds");
    fixture.write(&elsewhere, b"machine x password fake");
    std::os::unix::fs::symlink(&elsewhere, home.join(".netrc"))
        .unwrap_or_else(|err| panic!("{err}"));
    std::os::unix::fs::symlink(home.join("missing"), home.join(".git-credentials"))
        .unwrap_or_else(|err| panic!("{err}"));
    std::os::unix::fs::symlink(home.join(".pgpass"), home.join(".pgpass"))
        .unwrap_or_else(|err| panic!("{err}"));
    let preflight = fixture.run(&fixture.sources());

    let isolated = preflight.inventory.isolation_paths();
    let target = std::fs::canonicalize(&elsewhere).unwrap_or_else(|err| panic!("{err}"));
    assert!(isolated.contains(&home.join(".netrc")));
    assert!(isolated.contains(&target));
    let entry = |name: &str| {
        preflight
            .inventory
            .manifest
            .entries
            .iter()
            .find(|entry| entry.path == home.join(name))
            .cloned()
            .unwrap_or_else(|| panic!("{name} missing from manifest"))
    };
    assert_eq!(entry(".netrc").symlink_target, Some(target));
    assert_eq!(entry(".netrc").status, EntryStatus::Supported);
    for name in [".git-credentials", ".pgpass"] {
        assert!(
            matches!(entry(name).status, EntryStatus::Unreadable(_)),
            "{name}: {:?}",
            entry(name).status
        );
    }
}

#[cfg(unix)]
#[test]
fn pf_29_s01_denied_reads_are_listed_not_hidden() {
    use std::os::unix::fs::PermissionsExt as _;
    let fixture = Fixture::new();
    let netrc = fixture.home().join(".netrc");
    let profile = fixture.home().join(".bashrc");
    fixture.write(&netrc, b"machine x password fake");
    fixture.write(&profile, format!("export API_KEY={FAKE_KEY}\n").as_bytes());
    for path in [&netrc, &profile] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o000))
            .unwrap_or_else(|err| panic!("{err}"));
    }
    let preflight = fixture.run(&fixture.sources());
    for path in [&netrc, &profile] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .unwrap_or_else(|err| panic!("{err}"));
    }
    // Root reads everything; the denial cannot be observed then.
    if nix_is_root() {
        return;
    }
    let entry = preflight
        .inventory
        .manifest
        .entries
        .iter()
        .find(|entry| entry.path == netrc)
        .unwrap_or_else(|| panic!("netrc missing"));
    assert!(matches!(entry.status, EntryStatus::Unreadable(_)));
    assert_eq!(entry.mode, Some(0o000));
    let profile_finding = find(&preflight, FindingKind::ShellProfileExport);
    assert_eq!(profile_finding.len(), 1);
    assert_eq!(profile_finding[0].location, "~/.bashrc (not readable)");
}

#[cfg(unix)]
fn nix_is_root() -> bool {
    std::env::var("USER").is_ok_and(|user| user == "root")
}

/// The vault unlocks through the OS keyring only when a credential is used,
/// so a locked vault shows up as a store that cannot be read; a damaged
/// store is the same. An empty `secrets/` (lock file only) is no vault.
#[test]
fn pf_29_s01_locked_or_damaged_vault_is_incomplete_readiness() {
    let fixture = Fixture::new();
    let store = fixture.corbanu().join("secrets/local.age");
    fixture.write(&fixture.corbanu().join("secrets/.vault.lock"), b"");
    assert!(fixture.run(&fixture.sources()).is_clean());

    let vault_incomplete = |preflight: &Preflight| {
        preflight
            .readiness
            .iter()
            .any(|item| item.id == "vault" && matches!(item.state, ReadinessState::Incomplete(_)))
    };
    fixture.write(&store, b"not an age file");
    let damaged = fixture.run(&fixture.sources());
    assert!(vault_incomplete(&damaged));
    assert!(!damaged.is_clean());

    fixture.write(&store, b"age-encryption.org/v1\n-> scrypt salt 16\n---");
    let present = fixture.run(&fixture.sources());
    assert!(present.is_clean(), "{:?}", present.blockers());
    assert_eq!(
        find(&present, FindingKind::VaultStore)[0].disposition,
        Disposition::Denied
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&store, std::fs::Permissions::from_mode(0o000))
            .unwrap_or_else(|err| panic!("{err}"));
        let locked = fixture.run(&fixture.sources());
        std::fs::set_permissions(&store, std::fs::Permissions::from_mode(0o600))
            .unwrap_or_else(|err| panic!("{err}"));
        if !nix_is_root() {
            assert!(vault_incomplete(&locked));
        }
    }
}

#[test]
fn pf_29_s01_corrupt_snapshots_and_histories_are_isolated() {
    let fixture = Fixture::new();
    let home = fixture.corbanu();
    fixture.write(
        &home.join("shell_snapshots/s.sh"),
        &[0x00, 0xff, 0x13, 0x37],
    );
    fixture.write(&home.join("sessions/2026/x.jsonl"), b"{not json");
    fixture.write(&home.join("history.jsonl"), b"\xff\xfe");
    fixture.write(&home.join("state_5.sqlite"), b"garbage");
    fixture.write(&home.join("state_5.sqlite-wal"), b"garbage");
    fixture.write(&home.join("wallet/keystore"), b"fake");
    let preflight = fixture.run(&fixture.sources());
    let isolated = preflight.inventory.isolation_paths();
    for name in [
        "shell_snapshots",
        "sessions",
        "history.jsonl",
        "state_5.sqlite",
        "wallet",
    ] {
        assert!(isolated.contains(&home.join(name)), "{name} not isolated");
    }
    assert!(isolated.contains(&home.join("state_5.sqlite-wal")));
    assert_eq!(
        find(&preflight, FindingKind::Transcript)
            .iter()
            .filter(|finding| finding.location.contains("state_5"))
            .count(),
        1
    );
    assert_eq!(
        find(&preflight, FindingKind::Wallet)[0].class,
        SecretClass::CustodyKey
    );
    assert!(preflight.is_clean(), "{:?}", preflight.blockers());
}

#[test]
fn pf_29_s01_drift_between_preflight_and_activation() {
    let fixture = Fixture::new();
    let netrc = fixture.home().join(".netrc");
    fixture.write(&netrc, b"one");
    let sources = fixture.sources();
    let first = fixture.run(&sources);

    let (_, unchanged) = first.recheck(&sources, ALL_ON);
    assert!(unchanged.is_empty(), "{unchanged:?}");

    fixture.write(&netrc, b"two");
    fixture.write(
        &fixture.home().join(".zshenv"),
        format!("export X_TOKEN={FAKE_KEY}\n").as_bytes(),
    );
    let (second, drift) = first.recheck(&sources, ALL_ON);
    let netrc_id = find(&first, FindingKind::CredentialFile)[0].id.clone();
    assert_eq!(drift.changed, vec![netrc_id]);
    assert_eq!(drift.added.len(), 1);
    assert!(!second.is_clean());

    let (_, flags) = first.recheck(&sources, ReadinessFlags::default());
    assert!(flags.readiness_changed);
}

#[test]
fn pf_29_s01_ids_are_stable_and_digests_are_keyed() {
    let fixture = Fixture::new();
    fixture.write(&fixture.home().join(".netrc"), b"same");
    let sources = fixture.sources();
    let one = Preflight::run_with_key(&sources, ALL_ON, [1; 32]);
    let two = Preflight::run_with_key(&sources, ALL_ON, [2; 32]);
    assert_eq!(one.inventory.finding_ids(), two.inventory.finding_ids());
    assert_ne!(
        one.inventory.manifest.digest(),
        two.inventory.manifest.digest()
    );
}

#[test]
fn pf_29_s01_workspace_env_files_found_without_recursion() {
    let fixture = Fixture::new();
    let work = fixture.root.path().join("work");
    fixture.write(&work.join(".env"), b"API_KEY=fake");
    fixture.write(&work.join("nested/.env"), b"API_KEY=fake");
    fixture.write(&fixture.home().join("projects/app/.env"), b"API_KEY=fake");
    let preflight = fixture.run(&fixture.sources());
    let env_files = find(&preflight, FindingKind::EnvFile);
    assert_eq!(env_files.len(), 1);
    assert_eq!(env_files[0].paths, vec![work.join(".env")]);
}

/// A project can ship `.env` as a link to a device or as a named pipe; the
/// preflight must neither read forever nor block.
#[cfg(unix)]
#[test]
fn pf_29_s01_devices_and_pipes_neither_hang_nor_exhaust_memory() {
    let fixture = Fixture::new();
    let work = fixture.root.path().join("work");
    std::os::unix::fs::symlink("/dev/zero", work.join(".env"))
        .unwrap_or_else(|err| panic!("{err}"));
    let fifo = work.join(".envrc");
    let status = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap_or_else(|err| panic!("mkfifo: {err}"));
    assert!(status.success());
    std::os::unix::fs::symlink(&fifo, fixture.home().join(".zshrc"))
        .unwrap_or_else(|err| panic!("{err}"));
    let mut big = vec![b'a'; 2 * 1024 * 1024];
    big.extend_from_slice(b"\nexport API_KEY=fake\n");
    fixture.write(&fixture.home().join(".bashrc"), &big);

    let started = std::time::Instant::now();
    let preflight = fixture.run(&fixture.sources());
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
    let status = |path: PathBuf| {
        preflight
            .inventory
            .manifest
            .entries
            .iter()
            .find(|entry| entry.path == path)
            .map(|entry| entry.status.clone())
    };
    for name in [".env", ".envrc"] {
        assert!(
            matches!(status(work.join(name)), Some(EntryStatus::Unreadable(_))),
            "{name}"
        );
    }
    // Too large to read: listed for the human, not parsed.
    assert!(
        find(&preflight, FindingKind::ShellProfileExport)
            .iter()
            .any(|finding| finding.location == "~/.bashrc (not readable)")
    );
}
