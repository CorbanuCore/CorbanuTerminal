//! #1766 MEASUREMENT ONLY (never merged): with the working folder reached
//! through a junction or folder symlink, is a read-only folder configured by
//! its real (resolved) path enforced by the non-admin backend? The resolvers
//! pass no extra deny-write paths for it.

use super::*;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;

pub(super) fn link(kind: &str, link: &Path, target: &Path) -> bool {
    std::process::Command::new(system32_exe("cmd.exe"))
        .args(["/d", "/c", "mklink", kind])
        .arg(link)
        .arg(target)
        .output()
        .is_ok_and(|out| out.status.success())
}

pub(super) fn profile_with_read_only(docs: &Path) -> PermissionProfile {
    PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: AbsolutePathBuf::from_absolute_path(docs).expect("absolute"),
                },
                FileSystemAccessMode::Read,
            ),
        ]),
        NetworkSandboxPolicy::Restricted,
    )
}

#[test]
#[ignore = "#1766 measurement; run on request"]
fn measure_1766_legacy_read_only_by_real_path_under_linked_cwd() {
    let _guard = legacy_process_test_guard();
    let test_root = user_owned_test_root();
    let real = test_root.path().join("real");
    fs::create_dir_all(real.join("docs")).expect("create docs");
    for (kind, flag) in [("junction", "/J"), ("dir symlink", "/D")] {
        let linked = test_root
            .path()
            .join(format!("linked-{}", flag.trim_start_matches('/')));
        if !link(flag, &linked, &real) {
            println!("| legacy | {kind} | could not make the link | | |");
            continue;
        }
        for (spelling, docs) in [
            ("real path", real.join("docs")),
            ("through the link", linked.join("docs")),
        ] {
            let probe = format!(
                "probe-{}-{}.txt",
                flag.trim_start_matches('/'),
                spelling.len()
            );
            let target = real.join("docs").join(&probe);
            let started = super::carve_out_link::run_script(
                &profile_with_read_only(&docs),
                &linked,
                &[],
                HashMap::new(),
                &format!(
                    "echo x> \"{}\"\r\necho y> \"{}\"\r\n",
                    target.display(),
                    linked.join("ok.txt").display()
                ),
            );
            println!(
                "| legacy (non-admin) | cwd via {kind} | read-only docs by {spelling} | started={started:?} | docs written={} | workspace written={} |",
                target.exists(),
                real.join("ok.txt").exists()
            );
            let _ = fs::remove_file(&target);
            let _ = fs::remove_file(real.join("ok.txt"));
        }
        let _ = fs::remove_dir(&linked);
    }
}
