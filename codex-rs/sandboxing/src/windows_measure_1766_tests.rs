//! #1766 MEASUREMENT ONLY (never merged): which non-canonical working
//! folders the three Windows checks refuse. Prints a table.

use super::*;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;
use std::process::Command;

fn run(program: &str, args: &[&str]) -> (bool, String) {
    match Command::new(program).args(args).output() {
        Ok(out) => (
            out.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout).trim(),
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        ),
        Err(err) => (false, err.to_string()),
    }
}

fn short_name(path: &Path) -> Option<PathBuf> {
    let (ok, out) = run(
        "cmd",
        &[
            "/d",
            "/c",
            &format!("for %I in (\"{}\") do @echo %~sI", path.display()),
        ],
    );
    let short = PathBuf::from(out.lines().next()?.trim());
    (ok && short != path).then_some(short)
}

fn summary(
    result: std::result::Result<Option<WindowsSandboxFilesystemOverrides>, String>,
) -> String {
    match result {
        Ok(None) => "ok".to_string(),
        Ok(Some(o)) => format!(
            "ok (deny-write {}{})",
            o.additional_deny_write_paths.len(),
            if o.write_roots_override.is_some() {
                ", write-roots override"
            } else {
                ""
            }
        ),
        Err(err) => format!("REFUSED: {}", err.chars().take(90).collect::<String>()),
    }
}

#[test]
#[ignore = "#1766 measurement; run on request"]
fn measure_1766_non_canonical_cwd() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let base = dunce::canonicalize(temp_dir.path()).expect("canonical temp");
    let parent = base.join("Long Parent Folder");
    let real = parent.join("Workspace Folder");
    let other = base.join("Other Writable");
    for dir in [real.join("docs"), other.clone()] {
        std::fs::create_dir_all(&dir).expect("create dirs");
    }
    // 8.3 names for the temp folder's drive.
    let drive = format!("{}", &base.display().to_string()[..2]);
    println!(
        "8dot3 query {drive}: {:?}",
        run("fsutil", &["8dot3name", "query", &drive])
    );

    let mut forms: Vec<(&str, Option<PathBuf>, String)> =
        vec![("canonical", Some(real.clone()), String::new())];
    let junction = base.join("junction");
    let (ok, out) = run(
        "cmd",
        &[
            "/d",
            "/c",
            "mklink",
            "/J",
            &junction.display().to_string(),
            &real.display().to_string(),
        ],
    );
    forms.push(("junction", ok.then(|| junction.clone()), out));
    let symlink = base.join("symlink");
    let (ok, out) = run(
        "cmd",
        &[
            "/d",
            "/c",
            "mklink",
            "/D",
            &symlink.display().to_string(),
            &real.display().to_string(),
        ],
    );
    forms.push(("dir symlink", ok.then(|| symlink.clone()), out));
    let (ok, out) = run("subst", &["W:", &parent.display().to_string()]);
    forms.push((
        "subst drive",
        ok.then(|| PathBuf::from("W:\\Workspace Folder")),
        out,
    ));
    let short = short_name(&real);
    forms.push(("8.3 short name", short.clone(), format!("{short:?}")));
    let share_path = format!(
        "\\\\localhost\\{}$\\{}",
        &drive[..1],
        &parent.display().to_string()[3..]
    );
    let (ok, out) = run("net", &["use", "V:", &share_path]);
    forms.push((
        "mapped drive (net use)",
        ok.then(|| PathBuf::from("V:\\Workspace Folder")),
        out,
    ));
    let unc = PathBuf::from(format!("{share_path}\\Workspace Folder"));
    forms.push((
        "UNC admin share",
        unc.is_dir().then(|| unc.clone()),
        String::new(),
    ));
    forms.push((
        "lower-case spelling",
        Some(PathBuf::from(real.display().to_string().to_lowercase())),
        String::new(),
    ));
    forms.push((
        "verbatim \\\\?\\ prefix",
        Some(PathBuf::from(format!("\\\\?\\{}", real.display()))),
        String::new(),
    ));

    let root_read = FileSystemSandboxEntry::new(
        FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        FileSystemAccessMode::Read,
    );
    let project_roots = |subpath: Option<&str>, access| {
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(subpath.map(str::to_string)),
            },
            access,
        )
    };
    let entry = |path: &Path, access| {
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
            },
            access,
        )
    };
    let profile = |entries: Vec<FileSystemSandboxEntry>| {
        PermissionProfile::from_runtime_permissions(
            &FileSystemSandboxPolicy::restricted(entries),
            NetworkSandboxPolicy::Restricted,
        )
    };

    println!(
        "\n| cwd form | profile | command/exec (non-admin, full checks) | agent path (non-admin) | default (elevated) resolver |"
    );
    println!("|---|---|---|---|---|");
    for (name, cwd, note) in &forms {
        let Some(cwd) = cwd else {
            println!("| {name} | (could not make: {note}) | | | |");
            continue;
        };
        let exists = cwd.is_dir();
        let cwd_abs = match AbsolutePathBuf::from_absolute_path(cwd) {
            Ok(path) => path,
            Err(err) => {
                println!("| {name} | (not absolute: {err}) | | | |");
                continue;
            }
        };
        let profiles: Vec<(&str, PermissionProfile)> = vec![
            (
                "workspace-write (`:project_roots`)",
                profile(vec![
                    root_read.clone(),
                    project_roots(None, FileSystemAccessMode::Write),
                ]),
            ),
            (
                "+ read-only `:project_roots/docs`",
                profile(vec![
                    root_read.clone(),
                    project_roots(None, FileSystemAccessMode::Write),
                    project_roots(Some("docs"), FileSystemAccessMode::Read),
                ]),
            ),
            (
                "cwd path write + read-only cwd\\docs (same spelling)",
                profile(vec![
                    root_read.clone(),
                    entry(cwd, FileSystemAccessMode::Write),
                    entry(&cwd.join("docs"), FileSystemAccessMode::Read),
                ]),
            ),
            (
                "`:project_roots` write + read-only canonical docs",
                profile(vec![
                    root_read.clone(),
                    project_roots(None, FileSystemAccessMode::Write),
                    entry(&real.join("docs"), FileSystemAccessMode::Read),
                ]),
            ),
            (
                "+ extra writable root + read-only `:project_roots/docs`",
                profile(vec![
                    root_read.clone(),
                    project_roots(None, FileSystemAccessMode::Write),
                    entry(&other, FileSystemAccessMode::Write),
                    project_roots(Some("docs"), FileSystemAccessMode::Read),
                ]),
            ),
        ];
        for (profile_name, profile) in &profiles {
            let full = summary(resolve_windows_restricted_token_filesystem_overrides(
                SandboxType::WindowsRestrictedToken,
                profile,
                &cwd_abs,
                WindowsSandboxLevel::RestrictedToken,
            ));
            let agent = match refuse_unenforceable_windows_restricted_token_profile(
                SandboxType::WindowsRestrictedToken,
                profile,
                &cwd_abs,
                WindowsSandboxLevel::RestrictedToken,
            ) {
                Ok(()) => "ok".to_string(),
                Err(err) => format!("REFUSED: {}", err.chars().take(90).collect::<String>()),
            };
            let elevated = summary(resolve_windows_elevated_filesystem_overrides(
                SandboxType::WindowsRestrictedToken,
                profile,
                &cwd_abs,
                /*use_windows_elevated_backend*/ true,
            ));
            println!(
                "| {name} `{}`{} | {profile_name} | {full} | {agent} | {elevated} |",
                cwd.display(),
                if exists { "" } else { " (missing)" }
            );
        }
    }
    let _ = run("subst", &["W:", "/D"]);
    let _ = run("net", &["use", "V:", "/delete", "/y"]);
}
