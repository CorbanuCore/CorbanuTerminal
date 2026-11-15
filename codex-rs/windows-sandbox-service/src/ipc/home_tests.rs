use super::prepare_codex_home;
use codex_windows_sandbox::DirectoryOpenDisposition;
use codex_windows_sandbox::SetupRuntime;
use pretty_assertions::assert_eq;
use std::fs;

#[test]
fn issue_2315_home_admission_accepts_regular_files_and_its_guard() {
    for runtime in [SetupRuntime::Legacy, SetupRuntime::Registered] {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        fs::create_dir_all(home.join(".sandbox")).unwrap();
        fs::write(home.join(".sandbox/ordinary"), b"ordinary").unwrap();
        let (_, handles) =
            prepare_codex_home(&home, runtime, DirectoryOpenDisposition::OpenOrCreate).unwrap();
        assert!(!handles.is_empty());
    }
}

#[test]
fn issue_2315_home_admission_refuses_hard_links_before_setup() {
    for runtime in [SetupRuntime::Legacy, SetupRuntime::Registered] {
        let folders: &[&str] = match runtime {
            SetupRuntime::Legacy => &[".sandbox", ".sandbox-secrets", ".sandbox-bin"],
            SetupRuntime::Registered => &[".sandbox", ".sandbox-secrets"],
        };
        for folder in folders {
            let temp = tempfile::tempdir().unwrap();
            let home = temp.path().join("home");
            fs::create_dir_all(home.join(folder)).unwrap();
            let victim = temp.path().join("outside");
            fs::write(&victim, b"unchanged").unwrap();
            fs::hard_link(&victim, home.join(folder).join("planted")).unwrap();
            let error = prepare_codex_home(&home, runtime, DirectoryOpenDisposition::OpenOrCreate)
                .expect_err("hard link must be refused at admission");
            assert!(format!("{error:#}").contains("hard-linked"), "{error:#}");
            assert_eq!(fs::read(victim).unwrap(), b"unchanged");
        }
    }
}
