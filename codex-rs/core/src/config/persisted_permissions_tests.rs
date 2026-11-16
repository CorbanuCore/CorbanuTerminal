//! #2376: a resumed thread must never get broader permissions than it persisted.
//! Concrete path entries in a saved profile (Corbanu's `.git`/`.codex` read-only
//! protections, extra writable roots) must not make the loader discard it in
//! favour of a broader configured default.

use super::ConfigBuilder;
use super::ConfigOverrides;
use codex_config::CONFIG_TOML_FILE;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::SandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::Path;
use tempfile::TempDir;

fn saved_workspace_profile(policy: &SandboxPolicy, cwd: &Path) -> PermissionProfile {
    PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(policy, cwd),
        NetworkSandboxPolicy::Restricted,
    )
}

fn has_concrete_entry(
    profile: &PermissionProfile,
    path: &Path,
    access: FileSystemAccessMode,
) -> bool {
    profile
        .file_system_sandbox_policy()
        .entries
        .iter()
        .any(|entry| {
            entry.access == access
                && matches!(&entry.path, FileSystemPath::Path { path: entry_path }
                    if entry_path.to_abs_path().is_ok_and(|entry_path| entry_path.as_path() == path))
        })
}

async fn resume_over_full_access_default(
    cwd: &Path,
    saved: &PermissionProfile,
) -> anyhow::Result<super::Config> {
    let codex_home = TempDir::new()?;
    std::fs::write(
        codex_home.path().join(CONFIG_TOML_FILE),
        "sandbox_mode = \"danger-full-access\"\n",
    )?;
    Ok(ConfigBuilder::without_managed_config_for_tests()
        .codex_home(codex_home.path().to_path_buf())
        .fallback_cwd(Some(cwd.to_path_buf()))
        .harness_overrides(ConfigOverrides {
            cwd: Some(cwd.to_path_buf()),
            persisted_permission_profile: Some(saved.clone()),
            ..Default::default()
        })
        .build()
        .await?)
}

#[tokio::test]
async fn persisted_workspace_write_with_git_protection_beats_full_access_default()
-> anyhow::Result<()> {
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?;
    std::fs::create_dir(cwd.join(".git"))?;
    let saved = saved_workspace_profile(&SandboxPolicy::new_workspace_write_policy(), &cwd);
    // The case from #2376: the `.git` protection is a concrete readable path entry.
    assert!(has_concrete_entry(
        &saved,
        &cwd.join(".git"),
        FileSystemAccessMode::Read
    ));

    let config = resume_over_full_access_default(&cwd, &saved).await?;

    assert_eq!(config.permissions.permission_profile(), &saved);
    assert!(
        matches!(
            config.legacy_sandbox_policy(),
            SandboxPolicy::WorkspaceWrite { .. }
        ),
        "expected the saved workspace-write policy, got {:?}",
        config.legacy_sandbox_policy()
    );
    Ok(())
}

#[tokio::test]
async fn materialized_workspace_write_with_extra_root_beats_full_access_default()
-> anyhow::Result<()> {
    let workspace = TempDir::new()?;
    let root = dunce::canonicalize(workspace.path())?;
    let cwd = root.join("project");
    let extra = root.join("extra");
    std::fs::create_dir_all(cwd.join(".git"))?;
    std::fs::create_dir(&extra)?;
    let policy = SandboxPolicy::WorkspaceWrite {
        writable_roots: vec![AbsolutePathBuf::from_absolute_path_checked(&extra)?],
        network_access: false,
        exclude_tmpdir_env_var: false,
        exclude_slash_tmp: false,
    };
    // Thread settings persist the profile with workspace roots materialized.
    let saved = saved_workspace_profile(&policy, &cwd)
        .materialize_project_roots_with_workspace_roots(&[
            AbsolutePathBuf::from_absolute_path_checked(&cwd)?,
        ]);
    assert!(has_concrete_entry(
        &saved,
        &cwd,
        FileSystemAccessMode::Write
    ));
    assert!(has_concrete_entry(
        &saved,
        &extra,
        FileSystemAccessMode::Write
    ));

    let config = resume_over_full_access_default(&cwd, &saved).await?;

    assert_eq!(config.permissions.permission_profile(), &saved);
    assert_ne!(
        config.legacy_sandbox_policy(),
        SandboxPolicy::DangerFullAccess
    );
    Ok(())
}

// Temporary directories are excluded so the fixture paths, which live under
// them, are writable only through workspace roots.
const WORKSPACE_WRITE_WITHOUT_TEMP_DIRS: &str = "sandbox_mode = \"workspace-write\"\n\n\
    [sandbox_workspace_write]\nexclude_tmpdir_env_var = true\nexclude_slash_tmp = true\n";

fn workspace_write_without_temp_dirs(roots: &[AbsolutePathBuf]) -> PermissionProfile {
    PermissionProfile::workspace_write_with(
        &[],
        NetworkSandboxPolicy::Restricted,
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    )
    .materialize_project_roots_with_workspace_roots(roots)
}

async fn resume_with_workspace_roots(
    cwd: &Path,
    saved: PermissionProfile,
    current_roots: Vec<AbsolutePathBuf>,
) -> anyhow::Result<FileSystemSandboxPolicy> {
    let codex_home = TempDir::new()?;
    std::fs::write(
        codex_home.path().join(CONFIG_TOML_FILE),
        WORKSPACE_WRITE_WITHOUT_TEMP_DIRS,
    )?;
    let config = ConfigBuilder::without_managed_config_for_tests()
        .codex_home(codex_home.path().to_path_buf())
        .fallback_cwd(Some(cwd.to_path_buf()))
        .harness_overrides(ConfigOverrides {
            cwd: Some(cwd.to_path_buf()),
            workspace_roots: Some(current_roots),
            persisted_permission_profile: Some(saved),
            ..Default::default()
        })
        .build()
        .await?;
    // Resolve workspace roots the way a session does before enforcement.
    Ok(config
        .permissions
        .permission_profile()
        .clone()
        .materialize_project_roots_with_workspace_roots(&config.workspace_roots)
        .file_system_sandbox_policy())
}

#[tokio::test]
async fn materialized_profile_follows_narrower_current_workspace_roots() -> anyhow::Result<()> {
    let workspace = TempDir::new()?;
    let root = dunce::canonicalize(workspace.path())?;
    let cwd = root.join("project");
    let dropped = root.join("dropped");
    std::fs::create_dir_all(&cwd)?;
    std::fs::create_dir(&dropped)?;
    let cwd_abs = AbsolutePathBuf::from_absolute_path_checked(&cwd)?;
    let dropped_abs = AbsolutePathBuf::from_absolute_path_checked(&dropped)?;

    // The caller narrows the workspace roots: the saved grant for `dropped` must
    // not survive, and the remaining root stays writable.
    let file_system = resume_with_workspace_roots(
        &cwd,
        workspace_write_without_temp_dirs(&[cwd_abs.clone(), dropped_abs]),
        vec![cwd_abs],
    )
    .await?;

    assert_eq!(
        [&cwd, &dropped].map(|path| file_system.can_write_local_path_with_cwd(path, &cwd)),
        [true, false]
    );
    Ok(())
}

#[tokio::test]
async fn materialized_profile_follows_cleared_workspace_roots() -> anyhow::Result<()> {
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?;
    let cwd_abs = AbsolutePathBuf::from_absolute_path_checked(&cwd)?;

    // Clearing the workspace roots leaves the current default without writes; the
    // saved grant for `cwd` must not come back.
    let file_system = resume_with_workspace_roots(
        &cwd,
        workspace_write_without_temp_dirs(&[cwd_abs]),
        Vec::new(),
    )
    .await?;

    assert!(!file_system.can_write_local_path_with_cwd(&cwd, &cwd));
    Ok(())
}

#[tokio::test]
async fn unintersectable_saved_profile_does_not_regain_cleared_roots() -> anyhow::Result<()> {
    let workspace = TempDir::new()?;
    let root = dunce::canonicalize(workspace.path())?;
    let cwd = root.join("project");
    let earlier = root.join("earlier");
    std::fs::create_dir_all(&cwd)?;
    std::fs::create_dir(&earlier)?;
    // A snapshot saved after an earlier resume dropped `earlier` from the roots
    // keeps that root's optional metadata protection, which the intersection
    // refuses; the current default (no writes) is still provably narrower.
    let saved = PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                AbsolutePathBuf::from_absolute_path_checked(&cwd)?.into(),
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::skip_missing_path(
                AbsolutePathBuf::from_absolute_path_checked(earlier.join(".codex"))?.into(),
                FileSystemAccessMode::Read,
            ),
        ]),
        NetworkSandboxPolicy::Restricted,
    );

    let file_system = resume_with_workspace_roots(&cwd, saved, Vec::new()).await?;

    assert!(!file_system.can_write_local_path_with_cwd(&cwd, &cwd));
    Ok(())
}
