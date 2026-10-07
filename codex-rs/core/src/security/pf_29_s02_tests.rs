//! PF-29-S02 migration and recovery regressions. Synthetic secrets only.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use tempfile::TempDir;
use zeroize::Zeroizing;

use super::migration::CredentialStore;
use super::migration::FailPoint;
use super::migration::MigrationError;
use super::migration::MigrationPlan;
use super::migration::journal_path;
use super::migration::recover;
use super::migration::run;
use super::preflight::ConfigLayerInput;
use super::preflight::FindingKind;
use super::preflight::InventorySources;
use super::preflight::Preflight;
use super::preflight::ReadinessFlags;
use super::preflight::ReadinessState;

const OPENAI: &str = "fake-openai-value-0001";
const GITHUB: &str = "fake-github-value-0002";
const FLAGS: ReadinessFlags = ReadinessFlags {
    secretless_launch: true,
    credential_broker: true,
    output_gate: true,
};

#[derive(Default)]
struct FakeVault {
    values: RefCell<BTreeMap<String, String>>,
    puts: Cell<usize>,
    fail_puts: Cell<bool>,
}

impl CredentialStore for FakeVault {
    fn put(&self, label: &str, _name: &str, _origin: &str, value: &str) -> Result<(), String> {
        if self.fail_puts.get() {
            return Err("vault is locked".to_string());
        }
        let mut values = self.values.borrow_mut();
        if values.contains_key(label) {
            return Err(format!("{label} exists"));
        }
        values.insert(label.to_string(), value.to_string());
        self.puts.set(self.puts.get() + 1);
        Ok(())
    }

    fn get(&self, label: &str) -> Result<Option<Zeroizing<String>>, String> {
        Ok(self.values.borrow().get(label).cloned().map(Zeroizing::new))
    }
}

struct Machine {
    root: TempDir,
}

impl Machine {
    fn new() -> Self {
        let root = TempDir::new().unwrap_or_else(|err| panic!("{err}"));
        for dir in ["home", "corbanu", "work"] {
            std::fs::create_dir_all(root.path().join(dir)).unwrap_or_else(|err| panic!("{err}"));
        }
        let machine = Self { root };
        machine.write(
            &machine.zshrc(),
            &format!(
                "# my settings\nexport PATH=\"$HOME/bin:$PATH\"\nexport OPENAI_API_KEY=\"{OPENAI}\"  # work\nalias ll='ls -l'\nexport GH_TOKEN={GITHUB}\n"
            ),
        );
        machine
    }

    fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    fn corbanu(&self) -> PathBuf {
        self.root.path().join("corbanu")
    }

    fn zshrc(&self) -> PathBuf {
        self.home().join(".zshrc")
    }

    fn write(&self, path: &Path, contents: &str) {
        std::fs::write(path, contents).unwrap_or_else(|err| panic!("{err}"));
    }

    fn read(&self, path: &Path) -> String {
        std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{err}"))
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

    fn preflight(&self) -> Preflight {
        Preflight::run(&self.sources(), FLAGS)
    }

    fn plan(&self) -> MigrationPlan {
        MigrationPlan::from_preflight(&self.preflight(), &BTreeSet::new())
    }
}

fn migrated_zshrc() -> String {
    "# my settings\nexport PATH=\"$HOME/bin:$PATH\"\nexport OPENAI_API_KEY=\"$(corbanu vault auth-helper migrated/openai_api_key)\"  # work\nalias ll='ls -l'\nexport GH_TOKEN=\"$(corbanu vault auth-helper migrated/gh_token)\"\n".to_string()
}

#[test]
fn pf_29_s02_plan_previews_destinations_and_unsupported_items_without_values() {
    let machine = Machine::new();
    let mut sources = machine.sources();
    sources.config_layers = vec![ConfigLayerInput {
        label: "user config".to_string(),
        path: Some(machine.corbanu().join("config.toml")),
        toml: toml::from_str(
            r#"mcp_servers.docs = { command = "x", env = { DOCS_API_KEY = "fake-mcp" } }"#,
        )
        .unwrap_or_else(|err| panic!("{err}")),
    }];
    let taken = BTreeSet::from(["migrated/openai_api_key".to_string()]);
    let plan = MigrationPlan::from_preflight(&Preflight::run(&sources, FLAGS), &taken);

    let moves = plan
        .moves
        .iter()
        .map(|planned| (planned.location.clone(), planned.label.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        moves,
        vec![
            (
                "~/.zshrc:3 OPENAI_API_KEY".to_string(),
                "migrated/openai_api_key-2".to_string()
            ),
            (
                "~/.zshrc:5 GH_TOKEN".to_string(),
                "migrated/gh_token".to_string()
            ),
        ]
    );
    assert_eq!(plan.unsupported.len(), 1);
    assert!(
        plan.unsupported[0]
            .action
            .contains("cannot rewrite this config value")
    );
    assert_eq!(plan.restricted_files(), vec![machine.zshrc()]);
    let rendered = format!("{plan:?}");
    for value in [OPENAI, GITHUB, "fake-mcp"] {
        assert!(!rendered.contains(value), "{value} leaked");
    }
}

#[test]
fn pf_29_s02_migration_moves_values_rewrites_references_and_reaudits_clean() {
    let machine = Machine::new();
    assert!(!machine.preflight().is_clean());
    let vault = FakeVault::default();
    let outcome = run(
        &machine.corbanu(),
        &machine.plan(),
        &vault,
        /*fail_at*/ None,
    )
    .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(machine.read(&machine.zshrc()), migrated_zshrc());
    assert_eq!(
        vault.values.borrow().clone(),
        BTreeMap::from([
            ("migrated/gh_token".to_string(), GITHUB.to_string()),
            ("migrated/openai_api_key".to_string(), OPENAI.to_string()),
        ])
    );
    assert_eq!(
        outcome.rotate,
        vec!["GH_TOKEN".to_string(), "OPENAI_API_KEY".to_string()]
    );
    assert!(outcome.skipped.is_empty());
    assert!(!journal_path(&machine.corbanu()).exists());
    // No backup or temporary copy is left beside the profile.
    let names = std::fs::read_dir(machine.home())
        .unwrap_or_else(|err| panic!("{err}"))
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(names, vec![".zshrc".to_string()]);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(machine.zshrc())
            .unwrap_or_else(|err| panic!("{err}"))
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    // Re-audit: the references are not secrets.
    let after = machine.preflight();
    assert!(after.is_clean(), "{:?}", after.blockers());
    assert_eq!(
        after
            .inventory
            .findings
            .iter()
            .filter(|finding| finding.kind == FindingKind::Reference)
            .count(),
        2
    );
}

#[test]
fn pf_29_s02_every_interruption_recovers_to_the_same_result_and_locks_until_then() {
    for point in [
        FailPoint::Prepared,
        FailPoint::Stored,
        FailPoint::Rewritten,
        FailPoint::Committed,
    ] {
        let machine = Machine::new();
        let level = machine.corbanu().join("security_level.toml");
        machine.write(&level, "version = 1\nlevel = \"permissive\"\n");
        let vault = FakeVault::default();
        let err = run(&machine.corbanu(), &machine.plan(), &vault, Some(point))
            .err()
            .unwrap_or_else(|| panic!("{point:?} did not stop"));
        assert!(matches!(err, MigrationError::Interrupted { .. }), "{err}");

        // Locked: protected levels stay blocked while the journal exists.
        let locked = machine.preflight();
        assert!(
            locked.readiness.iter().any(|item| item.id == "migration"
                && matches!(item.state, ReadinessState::Incomplete(_))),
            "{point:?}"
        );
        assert!(!locked.is_clean());
        // A second migration cannot start over the unfinished one.
        assert_eq!(
            run(&machine.corbanu(), &machine.plan(), &vault, None).err(),
            Some(MigrationError::InProgress)
        );

        let outcome = recover(&machine.corbanu(), &vault)
            .unwrap_or_else(|err| panic!("{point:?}: {err}"))
            .unwrap_or_else(|| panic!("{point:?}: nothing to recover"));
        assert_eq!(
            machine.read(&machine.zshrc()),
            migrated_zshrc(),
            "{point:?}"
        );
        assert_eq!(outcome.moved.len(), 2, "{point:?}");
        assert_eq!(vault.puts.get(), 2, "{point:?}: each value stored once");
        assert!(!journal_path(&machine.corbanu()).exists());
        assert_eq!(recover(&machine.corbanu(), &vault), Ok(None));
        // Recovery never changes the level.
        assert_eq!(
            machine.read(&level),
            "version = 1\nlevel = \"permissive\"\n"
        );
        assert!(machine.preflight().is_clean(), "{point:?}");
    }
}

#[test]
fn pf_29_s02_recovery_never_overwrites_a_later_edit() {
    let machine = Machine::new();
    let vault = FakeVault::default();
    let _ = run(
        &machine.corbanu(),
        &machine.plan(),
        &vault,
        Some(FailPoint::Stored),
    );
    // The person edits one line after the interruption.
    let edited = machine.read(&machine.zshrc()).replace(
        &format!("GH_TOKEN={GITHUB}"),
        "GH_TOKEN=fake-rotated-value-0003",
    );
    machine.write(&machine.zshrc(), &edited);

    let outcome = recover(&machine.corbanu(), &vault)
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("nothing to recover"));
    let text = machine.read(&machine.zshrc());
    assert!(text.contains("GH_TOKEN=fake-rotated-value-0003"));
    assert!(
        text.contains("OPENAI_API_KEY=\"$(corbanu vault auth-helper migrated/openai_api_key)\"")
    );
    assert_eq!(
        outcome
            .skipped
            .iter()
            .map(|(location, _)| location.clone())
            .collect::<Vec<_>>(),
        vec!["~/.zshrc:5 GH_TOKEN".to_string()]
    );
    assert!(outcome.rotate.contains(&"GH_TOKEN".to_string()));
}

#[test]
fn pf_29_s02_store_failure_changes_no_file_and_recovers_later() {
    let machine = Machine::new();
    let before = machine.read(&machine.zshrc());
    let vault = FakeVault::default();
    vault.fail_puts.set(true);
    let err = run(&machine.corbanu(), &machine.plan(), &vault, None)
        .err()
        .unwrap_or_else(|| panic!("store failure ignored"));
    assert!(err.to_string().contains("vault is locked"), "{err}");
    assert_eq!(machine.read(&machine.zshrc()), before);

    vault.fail_puts.set(false);
    let _ = recover(&machine.corbanu(), &vault).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(machine.read(&machine.zshrc()), migrated_zshrc());
}

#[test]
fn pf_29_s02_fish_and_linked_profiles() {
    let machine = Machine::new();
    machine.write(&machine.zshrc(), "\n");
    let fish = machine.home().join(".config/fish/config.fish");
    std::fs::create_dir_all(fish.parent().unwrap_or_else(|| panic!("parent")))
        .unwrap_or_else(|err| panic!("{err}"));
    machine.write(&fish, &format!("set -gx ANTHROPIC_API_KEY {OPENAI}\n"));
    #[cfg(unix)]
    {
        let dotfiles = machine.root.path().join("dotfiles-bashrc");
        machine.write(&dotfiles, &format!("export LINKED_TOKEN={GITHUB}\n"));
        std::os::unix::fs::symlink(&dotfiles, machine.home().join(".bashrc"))
            .unwrap_or_else(|err| panic!("{err}"));
    }
    let plan = machine.plan();
    assert_eq!(plan.moves.len(), 1);
    #[cfg(unix)]
    assert!(
        plan.unsupported
            .iter()
            .any(|item| item.action.contains("a link"))
    );

    let vault = FakeVault::default();
    run(&machine.corbanu(), &plan, &vault, None).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        machine.read(&fish),
        "set -gx ANTHROPIC_API_KEY (corbanu vault auth-helper migrated/anthropic_api_key)\n"
    );
}

/// Review 1: quoting, second commands, several words and hard links.
#[test]
fn pf_29_s02_only_plain_single_assignments_are_rewritten() {
    let machine = Machine::new();
    machine.write(
        &machine.zshrc(),
        "export QUOTED_TOKEN=\"fake-a #b-0001\"\nexport SINGLE_TOKEN='fake-c #d-0002'\nexport FIRST_TOKEN=fake-e-0003; export SECOND_TOKEN=fake-f-0004\nESCAPED_TOKEN=fake\\ #g-0005\nGLUED_TOKEN=\"fake-h-0006\"#x\n",
    );
    let fish = machine.home().join(".config/fish/config.fish");
    std::fs::create_dir_all(fish.parent().unwrap_or_else(|| panic!("parent")))
        .unwrap_or_else(|err| panic!("{err}"));
    machine.write(
        &fish,
        "set -gx x_TOKEN fake-i-0007\nset -gx WORDS_TOKEN fake-j fake-k\nset -gx ESC_TOKEN 'fake\\\\l'\n",
    );
    machine.write(
        &machine.home().join(".bashrc"),
        "export TILDE_TOKEN=~fake-m\nexport BRACE_TOKEN=fake{n,o}\n",
    );
    let plan = machine.plan();
    let mut moved = plan
        .moves
        .iter()
        .map(|planned| planned.name.clone())
        .collect::<Vec<_>>();
    moved.sort();
    assert_eq!(
        moved,
        vec![
            "QUOTED_TOKEN".to_string(),
            "SINGLE_TOKEN".to_string(),
            "x_TOKEN".to_string()
        ]
    );
    // One finding per line: the second command on line 3 is not listed separately.
    assert_eq!(plan.unsupported.len(), 7, "{:?}", plan.unsupported);

    let vault = FakeVault::default();
    run(&machine.corbanu(), &plan, &vault, None).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        vault
            .values
            .borrow()
            .get("migrated/quoted_token")
            .map(String::as_str),
        Some("fake-a #b-0001")
    );
    let text = machine.read(&machine.zshrc());
    assert!(
        !text.contains("fake-a") && !text.contains("fake-c"),
        "{text}"
    );
    assert!(text.contains("FIRST_TOKEN=fake-e-0003; export SECOND_TOKEN=fake-f-0004"));
    assert_eq!(
        machine.read(&fish),
        "set -gx x_TOKEN (corbanu vault auth-helper migrated/x_token)\nset -gx WORDS_TOKEN fake-j fake-k\nset -gx ESC_TOKEN 'fake\\\\l'\n"
    );

    #[cfg(unix)]
    {
        let linked = Machine::new();
        std::fs::hard_link(linked.zshrc(), linked.root.path().join("other-name"))
            .unwrap_or_else(|err| panic!("{err}"));
        let plan = linked.plan();
        assert!(plan.moves.is_empty());
        assert!(
            plan.unsupported
                .iter()
                .all(|item| item.action.contains("hard links"))
        );
    }
}
