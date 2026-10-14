// Real folders, `/bin/bash` and the Unix sandboxes' protected paths.
#![cfg(unix)]

use super::*;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::NetworkSandboxPolicy;
use pretty_assertions::assert_eq;

/// A Corbanu home holding `auth.json`, a project beside it, and the inputs
/// a Moderate session captures for them.
struct Fixture {
    _dirs: (tempfile::TempDir, tempfile::TempDir),
    home: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().expect("home");
        let project = tempfile::tempdir().expect("project");
        let (home_path, project_path) = (
            home.path().canonicalize().expect("home path"),
            project.path().canonicalize().expect("project path"),
        );
        std::fs::write(home_path.join("auth.json"), "{}").expect("auth.json");
        // Before any explanation: on Linux, building the rules for a profile
        // that can write the home creates its missing runtime folders.
        std::fs::create_dir(home_path.join("skills")).expect("skills");
        std::fs::create_dir(project_path.join(".codex")).expect(".codex");
        Self {
            _dirs: (home, project),
            home: home_path,
            project: project_path,
        }
    }

    /// The workspace-write profile without the temporary folders, so a
    /// write beside the project is the person's own sandbox's denial.
    fn profile() -> PermissionProfile {
        PermissionProfile::workspace_write_with(
            &[],
            NetworkSandboxPolicy::Restricted,
            /*exclude_tmpdir_env_var*/ true,
            /*exclude_slash_tmp*/ true,
        )
    }

    fn inputs(&self, profile: PermissionProfile) -> ReadPolicyInputs {
        ReadPolicyInputs {
            codex_home: self.home.clone(),
            sqlite_home: self.home.clone(),
            turn_cwd: AbsolutePathBuf::from_absolute_path(&self.project).expect("cwd"),
            workspace_roots: Vec::new(),
            profile,
            user_home: dirs::home_dir(),
            keychain_closed: false,
        }
    }

    fn contract(&self) -> LaunchContract {
        LaunchContract::capture(
            &AbsolutePathBuf::from_absolute_path(&self.home).expect("home"),
            std::iter::empty(),
            /*hardened*/ false,
        )
    }

    fn explain(
        &self,
        profile: PermissionProfile,
        contract: Option<&LaunchContract>,
        script: &str,
        line: &str,
    ) -> Option<String> {
        explain(
            &self.inputs(profile),
            contract,
            &[
                "/bin/bash".to_string(),
                "-c".to_string(),
                script.to_string(),
            ],
            &self.project,
            &[line.to_string()],
        )
    }
}

/// #748 (c6, c3): a read of Corbanu's sign-in (or vault, or config) that
/// Moderate's rules denied names Moderate and /security; when secretless
/// launch keeps it at every level too, that is named instead.
#[test]
fn issue_748_moderate_read_denial_names_the_level() {
    let fixture = Fixture::new();
    let auth = format!("{}/auth.json", fixture.home.display());
    let script = format!("cat '{auth}'");
    let line = format!("cat: {auth}: Operation not permitted");

    let explained = fixture.explain(Fixture::profile(), /*contract*/ None, &script, &line);
    assert_eq!(
        explained,
        Some(read_explanation(&auth, WayForward::Permissive))
    );
    let explained = explained.expect("explained");
    assert!(
        explained.starts_with("Security level Moderate blocked access to"),
        "{explained}"
    );
    assert!(explained.contains("Permissive in /security"), "{explained}");

    // Not Moderate's alone: a person's approval may have lifted its rules.
    let contract = fixture.contract();
    let explained = fixture.explain(Fixture::profile(), Some(&contract), &script, &line);
    assert_eq!(explained, Some(kept_read_explanation(&auth)));
    let explained = explained.expect("explained");
    assert!(
        explained.starts_with("Secretless agent launch blocked access to"),
        "{explained}"
    );
    assert!(explained.contains("Moderate included"), "{explained}");
    assert!(!explained.contains("Permissive"), "{explained}");

    // A failure that names no denied path is not the level's.
    assert_eq!(
        fixture.explain(
            Fixture::profile(),
            /*contract*/ None,
            &script,
            "cat: /etc/other.txt: Operation not permitted"
        ),
        None
    );
}

/// #748: with the person's own full access, a write to the project's
/// `.codex` that Moderate's rules denied names Moderate. Under secretless
/// launch no level runs the command outside a sandbox, so then switching to
/// Permissive is not offered. A write to Corbanu's home that secretless
/// launch keeps read-only is attributed to it.
#[test]
fn issue_748_moderate_write_denial_names_the_level() {
    let fixture = Fixture::new();
    let script = "printf x > .codex/p.toml";
    let line = "bash: .codex/p.toml: Operation not permitted";
    assert_eq!(
        fixture.explain(
            PermissionProfile::Disabled,
            /*contract*/ None,
            script,
            line
        ),
        Some(write_explanation(".codex/p.toml", WayForward::Permissive))
    );
    let contract = fixture.contract();
    let explained = fixture.explain(PermissionProfile::Disabled, Some(&contract), script, line);
    assert_eq!(
        explained,
        Some(write_explanation(".codex/p.toml", WayForward::NoLevel))
    );
    let explained = explained.expect("explained");
    assert!(explained.contains("only inside a sandbox"), "{explained}");
    assert!(!explained.contains("Permissive"), "{explained}");

    // A runtime entry of Corbanu's home: Moderate's, and secretless
    // launch's when it is on.
    let skill = format!("{}/skills/x.md", fixture.home.display());
    let skill_script = format!("printf x > '{skill}'");
    let skill_line = format!("bash: {skill}: Operation not permitted");
    assert_eq!(
        fixture.explain(
            PermissionProfile::Disabled,
            /*contract*/ None,
            &skill_script,
            &skill_line
        ),
        Some(write_explanation(&skill, WayForward::Permissive))
    );
    assert_eq!(
        fixture.explain(
            PermissionProfile::Disabled,
            Some(&contract),
            &skill_script,
            &skill_line
        ),
        Some(kept_write_explanation(&skill))
    );
    // Workspace-write keeps `.codex` read-only itself: not Moderate's.
    assert_eq!(
        fixture.explain(Fixture::profile(), /*contract*/ None, script, line),
        None
    );
}

/// #748: what Moderate's rules did not add is not attributed to the level:
/// a write outside the project (the person's own sandbox), and a read the
/// person's own profile already denies.
#[test]
fn issue_748_denials_moderate_did_not_add_are_not_attributed() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.explain(
            Fixture::profile(),
            /*contract*/ None,
            "printf x > ../outside-notes.txt",
            "bash: ../outside-notes.txt: Operation not permitted"
        ),
        None
    );

    let auth = fixture.home.join("auth.json");
    let (mut file_system, network) = Fixture::profile().to_runtime_permissions();
    file_system.entries.push(FileSystemSandboxEntry::new(
        FileSystemPath::Path {
            path: AbsolutePathBuf::from_absolute_path(&auth).expect("auth"),
        },
        FileSystemAccessMode::Deny,
    ));
    let own_denial = PermissionProfile::from_runtime_permissions(&file_system, network);
    let auth = auth.display().to_string();
    assert_eq!(
        fixture.explain(
            own_denial,
            /*contract*/ None,
            &format!("cat '{auth}'"),
            &format!("cat: {auth}: Operation not permitted")
        ),
        None
    );
}
