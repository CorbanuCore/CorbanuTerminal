use crate::deny_read_targets::DenyReadRule;
use crate::deny_read_targets::DenyReadTargets;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::ReadDenyMatcher;
use codex_protocol::permissions::credential_glob_exempts_templates;
use codex_protocol::permissions::credential_rule_roots;
use codex_protocol::permissions::is_credential_template_name;
use codex_protocol::permissions::windows_deny_read_glob_scan;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

#[path = "deny_read_walker.rs"]
mod walker;

use walker::DirectoryScanMode;
use walker::collect_existing_glob_directory_matches;

/// The external scanner upstream uses to speed up deny-glob expansion, listed
/// for the dependency inventory. Corbanu does not run it: a scanner found
/// through `PATH` could only drop deny entries, so every glob is expanded by
/// the in-process, matcher-backed walk.
pub const GLOB_SCAN_PROGRAM: &str = "rg";

#[derive(Debug, Eq, PartialEq)]
struct GlobScanPlan {
    root: PathBuf,
    max_depth: Option<usize>,
}

/// Resolve split filesystem `None` read entries into concrete Windows ACL targets.
///
/// Windows ACLs do not understand Codex filesystem glob patterns directly. Exact
/// unreadable roots can be passed through as-is, including paths that do not
/// exist yet. Glob entries are snapshot-expanded to the files/directories that
/// already exist under their literal scan root; future exact paths are handled
/// later by materializing them before the deny ACE is applied.
pub fn resolve_windows_deny_read_paths(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    cwd: &AbsolutePathBuf,
) -> Result<Vec<AbsolutePathBuf>, String> {
    Ok(resolve_windows_deny_read_targets(file_system_sandbox_policy, cwd)?.paths())
}

/// Upstream's entry point for a launcher with an explicit command
/// environment, where its external scanner ran in that environment. The
/// in-process walk needs no environment, so this resolves exactly as
/// [`resolve_windows_deny_read_paths`].
pub fn resolve_windows_deny_read_paths_in_environment(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    cwd: &AbsolutePathBuf,
    _env: &HashMap<String, String>,
    _command_cwd: &Path,
) -> Result<Vec<AbsolutePathBuf>, String> {
    resolve_windows_deny_read_paths(file_system_sandbox_policy, cwd)
}

/// [`resolve_windows_deny_read_paths`], keeping which rule produced each
/// path. Every configured rule is listed, including a glob that matches
/// nothing now, so the persistent sync keeps its earlier entries (#304/S1).
pub fn resolve_windows_deny_read_targets(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    cwd: &AbsolutePathBuf,
) -> Result<DenyReadTargets, String> {
    let mut targets = DenyReadTargets::default();
    for path in file_system_sandbox_policy.get_unreadable_roots_with_cwd(cwd.as_path()) {
        let path = absolute_path(path.as_path())?;
        targets.add(DenyReadRule::Path(path.to_path_buf()), vec![path]);
    }

    let globs = file_system_sandbox_policy.get_unreadable_globs_with_cwd(cwd.as_path());
    // Upstream (#44327): refuse a glob that would scan a whole drive without a
    // depth limit, before any expansion.
    let scan_plans = globs
        .iter()
        .map(|pattern| {
            glob_scan_plan_checked(pattern, file_system_sandbox_policy.glob_scan_max_depth)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // #442 (Travis, 2026-10-12): under the whole credential rule the
    // templates (`.env.example`, `.env.sample`, `.env.template`) get no deny.
    let rule_roots = credential_rule_roots(&globs);
    for (pattern, scan_plan) in globs.into_iter().zip(scan_plans) {
        let exempts_templates = credential_glob_exempts_templates(&pattern, &rule_roots);
        let glob_policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                scan: None,
                pattern: pattern.clone(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }]);
        let mut paths = Vec::new();
        if let Some(matcher) =
            ReadDenyMatcher::try_new_for_local_paths(&glob_policy, cwd.as_path())?
            && scan_plan.root.exists()
        {
            collect_existing_glob_directory_matches(
                &scan_plan.root,
                &matcher,
                &mut paths,
                &mut HashSet::new(),
                scan_plan.max_depth,
                DirectoryScanMode::IncludeAllFiles,
            )?;
        }
        if exempts_templates {
            paths.retain(|path| {
                !path
                    .as_path()
                    .file_name()
                    .is_some_and(is_credential_template_name)
            });
        }
        targets.add(DenyReadRule::Glob(pattern), paths);
    }

    Ok(targets)
}

fn push_absolute_path(
    paths: &mut Vec<AbsolutePathBuf>,
    seen: &mut HashSet<PathBuf>,
    path: PathBuf,
) -> Result<(), String> {
    let absolute_path = absolute_path(&path)?;
    if seen.insert(absolute_path.to_path_buf()) {
        paths.push(absolute_path);
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<AbsolutePathBuf, String> {
    AbsolutePathBuf::from_absolute_path(dunce::simplified(path)).map_err(|err| err.to_string())
}

fn glob_scan_plan_checked(
    pattern: &str,
    configured_max_depth: Option<usize>,
) -> Result<GlobScanPlan, String> {
    let scan_plan = glob_scan_plan(pattern, configured_max_depth);
    if scan_plan.max_depth.is_none() && scan_plan.root.parent().is_none() {
        return Err(format!(
            "unreadable glob `{pattern}` cannot be safely expanded from a filesystem root without `glob_scan_max_depth`; configure `glob_scan_max_depth` or use a non-root directory prefix"
        ));
    }
    Ok(scan_plan)
}

fn glob_scan_plan(pattern: &str, configured_max_depth: Option<usize>) -> GlobScanPlan {
    let scan = windows_deny_read_glob_scan(pattern, configured_max_depth);
    GlobScanPlan {
        root: PathBuf::from(scan.root),
        max_depth: scan.max_depth,
    }
}

#[cfg(test)]
#[path = "deny_read_resolver_access_tests.rs"]
mod access_tests;

#[cfg(test)]
mod tests {
    use super::glob_scan_plan;
    use super::resolve_windows_deny_read_paths;
    use codex_protocol::permissions::FileSystemAccessMode;
    use codex_protocol::permissions::FileSystemPath;
    use codex_protocol::permissions::FileSystemSandboxEntry;
    use codex_protocol::permissions::FileSystemSandboxPolicy;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    fn unreadable_glob_entry(pattern: String) -> FileSystemSandboxEntry {
        FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern,
                scan: None,
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }
    }

    fn unreadable_path_entry(path: PathBuf) -> FileSystemSandboxEntry {
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(path)
                    .expect("absolute path")
                    .into(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }
    }

    #[test]
    fn scan_root_uses_literal_prefix_before_glob() {
        assert_eq!(
            glob_scan_plan("/tmp/work/**/*.env", /*configured_max_depth*/ None).root,
            PathBuf::from("/tmp/work")
        );
        assert_eq!(
            glob_scan_plan(
                r"C:\Users\dev\repo\**\*.env",
                /*configured_max_depth*/ None,
            )
            .root,
            PathBuf::from(r"C:\Users\dev\repo")
        );
        assert_eq!(
            glob_scan_plan(r"C:\*.env", /*configured_max_depth*/ None).root,
            PathBuf::from(r"C:\")
        );
    }

    #[test]
    fn scan_depth_is_bounded_for_non_recursive_globs() {
        assert_eq!(
            glob_scan_plan("/tmp/work/*.env", /*configured_max_depth*/ None).max_depth,
            Some(1)
        );
        assert_eq!(
            glob_scan_plan("/tmp/work/*/*.env", /*configured_max_depth*/ None).max_depth,
            Some(2)
        );
        assert_eq!(
            glob_scan_plan("/tmp/work/**/*.env", /*configured_max_depth*/ None).max_depth,
            None
        );
    }

    #[test]
    fn configured_depth_caps_recursive_glob_scans() {
        assert_eq!(
            glob_scan_plan("/tmp/work/**/*.env", Some(2)).max_depth,
            Some(2)
        );
        assert_eq!(
            glob_scan_plan("/tmp/work/*/*.env", Some(1)).max_depth,
            Some(1)
        );
    }

    #[test]
    fn project_recursive_globs_without_depth_expand_existing_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let secret = tmp.path().join("nested").join("secret.env");
        std::fs::create_dir_all(secret.parent().expect("secret parent")).expect("create parent");
        std::fs::write(&secret, "secret").expect("write secret");
        let pattern = tmp.path().join("**").join("*.env").display().to_string();
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(pattern)]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("unbounded project glob"),
            vec![AbsolutePathBuf::from_absolute_path(secret).expect("absolute secret")]
        );
    }

    #[test]
    fn relative_recursive_globs_without_depth_expand_existing_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let secret = tmp.path().join("nested").join("secret.env");
        std::fs::create_dir_all(secret.parent().expect("secret parent")).expect("create parent");
        std::fs::write(&secret, "secret").expect("write secret");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
            "**/*.env".to_string(),
        )]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd)
                .expect("unbounded relative project glob"),
            vec![AbsolutePathBuf::from_absolute_path(secret).expect("absolute secret")]
        );
    }

    #[test]
    fn root_recursive_globs_without_depth_fail_before_expansion() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let root = cwd.as_path().ancestors().last().expect("filesystem root");
        let pattern = root.join("**").join("*.env").display().to_string();
        let policy =
            FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(pattern.clone())]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect_err("unbounded root glob"),
            format!(
                "unreadable glob `{pattern}` cannot be safely expanded from a filesystem root without `glob_scan_max_depth`; configure `glob_scan_max_depth` or use a non-root directory prefix"
            )
        );
    }

    #[test]
    fn configured_depth_bounds_root_recursive_glob_scans() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let root = cwd.as_path().ancestors().last().expect("filesystem root");
        let pattern = root.join("**").join("*.env").display().to_string();
        let scan_plan = glob_scan_plan(&pattern, Some(2));

        assert_eq!(scan_plan.root, root);
        assert_eq!(scan_plan.max_depth, Some(2));
    }

    #[test]
    fn exact_missing_paths_are_preserved() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let missing = tmp.path().join("missing.env");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_path_entry(missing)]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("resolve"),
            vec![
                AbsolutePathBuf::from_absolute_path(
                    dunce::canonicalize(tmp.path())
                        .expect("canonical tempdir")
                        .join("missing.env")
                )
                .expect("absolute missing")
            ]
        );
    }

    /// #442 (Travis, 2026-10-12): under the whole credential rule the
    /// templates get no deny ACE while `.env` and `.env.local` do; a person's
    /// own `**/.env.*` alone still covers them.
    #[test]
    fn credential_rule_leaves_templates_out() {
        let tmp = TempDir::new().expect("tempdir");
        let root = dunce::canonicalize(tmp.path()).expect("canonical");
        let cwd = AbsolutePathBuf::from_absolute_path(&root).expect("absolute cwd");
        for name in [
            ".env",
            ".env.local",
            ".env.example",
            "app/.ENV.SAMPLE",
            ".env.template",
        ] {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("parent");
            std::fs::write(path, "x").expect("write");
        }
        let resolve = |globs: Vec<String>| -> HashSet<PathBuf> {
            let policy = FileSystemSandboxPolicy::restricted(
                globs.into_iter().map(unreadable_glob_entry).collect(),
            );
            resolve_windows_deny_read_paths(&policy, &cwd)
                .expect("resolve")
                .into_iter()
                .map(|path| {
                    PathBuf::from(
                        path.as_path()
                            .strip_prefix(&root)
                            .expect("below root")
                            .to_string_lossy()
                            .replace('\\', "/"),
                    )
                })
                .collect()
        };
        let rule = codex_protocol::permissions::credential_file_globs(&root.to_string_lossy());
        assert_eq!(
            resolve(rule),
            [".env", ".env.local"]
                .into_iter()
                .map(PathBuf::from)
                .collect()
        );
        assert_eq!(
            resolve(vec![format!(
                "{}{sep}**{sep}.env.*",
                root.to_string_lossy(),
                sep = std::path::MAIN_SEPARATOR
            )]),
            [
                ".env.local",
                ".env.example",
                "app/.ENV.SAMPLE",
                ".env.template"
            ]
            .into_iter()
            .map(PathBuf::from)
            .collect()
        );
    }

    #[test]
    fn glob_patterns_expand_to_existing_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let root_env = tmp.path().join(".env");
        let nested_env = tmp.path().join("app").join(".env");
        let uppercase_env = tmp.path().join("app").join("SECRET.ENV");
        let notes = tmp.path().join("app").join("notes.txt");
        std::fs::create_dir_all(notes.parent().expect("parent")).expect("create parent");
        std::fs::write(&root_env, "secret").expect("write root env");
        std::fs::write(&nested_env, "secret").expect("write nested env");
        std::fs::write(&uppercase_env, "secret").expect("write uppercase env");
        std::fs::write(&notes, "notes").expect("write notes");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(format!(
            "{}/**/*.env",
            tmp.path().display()
        ))]);

        let actual: HashSet<PathBuf> = resolve_windows_deny_read_paths(&policy, &cwd)
            .expect("resolve")
            .into_iter()
            .map(AbsolutePathBuf::into_path_buf)
            .collect();
        let mut expected = [root_env, nested_env].into_iter().collect::<HashSet<_>>();
        if cfg!(windows) {
            expected.insert(uppercase_env);
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn recursive_globs_include_hidden_and_ignored_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let hidden_secret = tmp.path().join("nested").join(".env");
        let ignored_secret = tmp.path().join("nested").join("ignored.env");
        std::fs::create_dir_all(hidden_secret.parent().expect("secret parent"))
            .expect("create parent");
        std::fs::write(tmp.path().join(".ignore"), "ignored.env\n").expect("write ignore file");
        std::fs::write(&hidden_secret, "secret").expect("write hidden secret");
        std::fs::write(&ignored_secret, "secret").expect("write ignored secret");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
            "**/*.env".to_string(),
        )]);

        let actual: HashSet<PathBuf> = resolve_windows_deny_read_paths(&policy, &cwd)
            .expect("resolve hidden and ignored matches")
            .into_iter()
            .map(AbsolutePathBuf::into_path_buf)
            .collect();

        assert_eq!(
            actual,
            [hidden_secret, ignored_secret].into_iter().collect()
        );
    }

    #[test]
    fn recursive_globs_preserve_matching_empty_directories() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let root_secret = tmp.path().join("root.env");
        let nested_secret = tmp.path().join("nested").join("empty.env");
        std::fs::create_dir_all(&root_secret).expect("create root secret directory");
        std::fs::create_dir_all(&nested_secret).expect("create nested secret directory");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
            "**/*.env".to_string(),
        )]);

        let actual: HashSet<PathBuf> = resolve_windows_deny_read_paths(&policy, &cwd)
            .expect("resolve matching directories")
            .into_iter()
            .map(AbsolutePathBuf::into_path_buf)
            .collect();

        assert_eq!(actual, [root_secret, nested_secret].into_iter().collect());
    }

    #[test]
    fn configured_depth_excludes_deeper_recursive_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let shallow_secret = tmp.path().join("shallow.env");
        let deep_secret = tmp.path().join("nested").join("deep.env");
        std::fs::create_dir_all(deep_secret.parent().expect("secret parent"))
            .expect("create parent");
        std::fs::write(&shallow_secret, "secret").expect("write shallow secret");
        std::fs::write(&deep_secret, "secret").expect("write deep secret");
        let mut policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
            "**/*.env".to_string(),
        )]);
        policy.glob_scan_max_depth = Some(1);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("resolve depth-limited glob"),
            vec![AbsolutePathBuf::from_absolute_path(shallow_secret).expect("absolute secret")]
        );
    }

    #[test]
    fn recursive_globs_rescan_for_new_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        std::fs::write(tmp.path().join("notes.txt"), "notes").expect("write notes");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
            "**/*.env".to_string(),
        )]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("resolve unmatched glob"),
            Vec::<AbsolutePathBuf>::new()
        );
        let secret = tmp.path().join("new.env");
        std::fs::write(&secret, "new secret").expect("create matching file");
        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("fresh resolution"),
            vec![AbsolutePathBuf::from_absolute_path(secret).expect("secret path")]
        );
    }

    #[test]
    fn shared_scan_roots_preserve_matches_at_each_required_depth() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let env = tmp.path().join(".env");
        let pem = tmp.path().join("nested").join("credentials.pem");
        std::fs::create_dir_all(pem.parent().expect("pem parent")).expect("create pem parent");
        std::fs::write(&env, "secret").expect("write env");
        std::fs::write(&pem, "secret").expect("write pem");
        let policy = FileSystemSandboxPolicy::restricted(vec![
            unreadable_glob_entry(format!("{}/*.env", tmp.path().display())),
            unreadable_glob_entry(format!("{}/*/*.pem", tmp.path().display())),
        ]);

        let actual: HashSet<PathBuf> = resolve_windows_deny_read_paths(&policy, &cwd)
            .expect("resolve shared root")
            .into_iter()
            .map(AbsolutePathBuf::into_path_buf)
            .collect();
        let expected = [env, pem].into_iter().collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_glob_patterns_fail_before_expansion() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(format!(
            "{}/**/[z-a]",
            tmp.path().display()
        ))]);

        let err = resolve_windows_deny_read_paths(&policy, &cwd).expect_err("invalid glob");
        assert!(
            err.contains("invalid deny-read glob pattern"),
            "unexpected error: {err}"
        );
        assert!(err.contains("invalid range"), "unexpected error: {err}");
    }

    #[test]
    fn non_recursive_globs_do_not_expand_nested_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let root_env = tmp.path().join(".env");
        let nested_env = tmp.path().join("app").join(".env");
        std::fs::create_dir_all(nested_env.parent().expect("parent")).expect("create parent");
        std::fs::write(&root_env, "secret").expect("write root env");
        std::fs::write(&nested_env, "secret").expect("write nested env");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(format!(
            "{}/*.env",
            tmp.path().display()
        ))]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("resolve"),
            vec![AbsolutePathBuf::from_absolute_path(root_env).expect("absolute root env")]
        );
    }

    #[cfg(unix)]
    #[test]
    fn aliased_glob_roots_each_preserve_their_lexical_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let target = tmp.path().join("target");
        let alias_a = tmp.path().join("alias-a");
        let alias_b = tmp.path().join("alias-b");
        let secret = target.join("secret.env");
        std::fs::create_dir_all(&target).expect("create target");
        std::fs::write(&secret, "secret").expect("write secret");
        symlink(&target, &alias_a).expect("create alias a");
        symlink(&target, &alias_b).expect("create alias b");
        let policy = FileSystemSandboxPolicy::restricted(vec![
            unreadable_glob_entry(format!("{}/**/*.env", alias_a.display())),
            unreadable_glob_entry(format!("{}/**/*.env", alias_b.display())),
        ]);

        let actual: HashSet<PathBuf> = resolve_windows_deny_read_paths(&policy, &cwd)
            .expect("resolve")
            .into_iter()
            .map(AbsolutePathBuf::into_path_buf)
            .collect();
        let expected = [alias_a.join("secret.env"), alias_b.join("secret.env")]
            .into_iter()
            .collect();

        assert_eq!(actual, expected);
    }

    #[cfg(unix)]
    #[test]
    fn recursive_globs_preserve_symlinked_file_matches() {
        let tmp = TempDir::new().expect("tempdir");
        let cwd = AbsolutePathBuf::from_absolute_path(tmp.path()).expect("absolute cwd");
        let target = tmp.path().join("target.txt");
        let alias = tmp.path().join("linked.env");
        std::fs::write(&target, "secret").expect("write symlink target");
        symlink(&target, &alias).expect("create symlink");
        let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
            "**/*.env".to_string(),
        )]);

        assert_eq!(
            resolve_windows_deny_read_paths(&policy, &cwd).expect("resolve symlinked secret"),
            vec![AbsolutePathBuf::from_absolute_path(alias).expect("absolute symlink")]
        );
    }
}
