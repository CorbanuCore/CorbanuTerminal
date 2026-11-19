#![cfg(unix)]

use super::*;
use codex_protocol::models::NetworkPermissions;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

struct Fixture {
    _dir: tempfile::TempDir,
    user_home: PathBuf,
    codex_home: PathBuf,
    project: PathBuf,
    denials: ReadDenials,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let user_home = dir.path().canonicalize().unwrap().join("user");
    let codex_home = user_home.join(".corbanu");
    let project = user_home.join("project");
    for folder in [
        &codex_home.join("wallet"),
        &codex_home.join("tmp"),
        &user_home.join(".ssh"),
        &project,
    ] {
        std::fs::create_dir_all(folder).unwrap();
    }
    std::fs::write(user_home.join(".ssh/id_ed25519"), "x").unwrap();
    std::fs::write(codex_home.join("wallet/dummy.txt"), "x").unwrap();
    std::fs::write(codex_home.join("state_5.sqlite"), "x").unwrap();
    std::fs::write(project.join("notes.txt"), "x").unwrap();
    let denials = ReadDenials::collect(
        &codex_home,
        Some(&user_home),
        /*claude_config_dir*/ None,
        &[AbsolutePathBuf::from_absolute_path(&project).unwrap()],
    );
    Fixture {
        _dir: dir,
        user_home,
        codex_home,
        project,
        denials,
    }
}

fn entry(path: &std::path::Path, access: FileSystemAccessMode) -> FileSystemSandboxEntry {
    FileSystemSandboxEntry::new(
        FileSystemPath::Path {
            path: codex_utils_path_uri::PathUri::from_abs_path(
                &AbsolutePathBuf::from_absolute_path(path).unwrap(),
            ),
        },
        access,
    )
}

fn additional(entries: Vec<FileSystemSandboxEntry>) -> AdditionalPermissionProfile {
    AdditionalPermissionProfile {
        network: None,
        file_system: Some(FileSystemPermissions {
            entries,
            glob_scan_max_depth: None,
        }),
    }
}

#[test]
fn issue_245_an_entry_at_or_under_a_protected_path_opens_it() {
    let f = fixture();
    let ssh_key = f.user_home.join(".ssh/id_ed25519");
    let wallet_file = f.codex_home.join("wallet/dummy.txt");
    let database = f.codex_home.join("state_5.sqlite");
    let link = f.project.join("keys");
    std::os::unix::fs::symlink(f.user_home.join(".ssh"), &link).unwrap();
    for (path, access) in [
        (ssh_key.clone(), FileSystemAccessMode::Read),
        (ssh_key.clone(), FileSystemAccessMode::Write),
        (f.user_home.join(".ssh"), FileSystemAccessMode::Read),
        (wallet_file, FileSystemAccessMode::Read),
        (database, FileSystemAccessMode::Read),
        // A link from the project into a protected folder.
        (link.join("id_ed25519"), FileSystemAccessMode::Read),
    ] {
        assert!(
            opened_protected_path(
                &f.denials,
                &additional(vec![entry(&path, access)]),
                &[f.project.as_path()]
            )
            .is_some(),
            "{} {access:?}",
            path.display()
        );
    }
    // An entry for `/` does not hide a narrower one.
    let with_root = additional(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Read,
        ),
        entry(&ssh_key, FileSystemAccessMode::Read),
    ]);
    assert_eq!(
        opened_protected_path(&f.denials, &with_root, &[f.project.as_path()]),
        Some(ssh_key.display().to_string())
    );
}

#[test]
fn issue_245_entries_that_open_no_protected_path_pass() {
    let f = fixture();
    for (path, access) in [
        (f.project.join("notes.txt"), FileSystemAccessMode::Read),
        (f.project.clone(), FileSystemAccessMode::Write),
        // A folder above a protected path: the denial is more specific.
        (f.user_home.clone(), FileSystemAccessMode::Read),
        // Deny entries only narrow.
        (f.user_home.join(".ssh"), FileSystemAccessMode::Deny),
    ] {
        assert_eq!(
            opened_protected_path(
                &f.denials,
                &additional(vec![entry(&path, access)]),
                &[f.project.as_path()]
            ),
            None,
            "{} {access:?}",
            path.display()
        );
    }
    let network_only = AdditionalPermissionProfile {
        network: Some(NetworkPermissions {
            enabled: Some(true),
        }),
        file_system: None,
    };
    assert_eq!(
        opened_protected_path(&f.denials, &network_only, &[f.project.as_path()]),
        None
    );
}

#[test]
fn issue_245_relative_and_project_root_entries_resolve_against_every_folder() {
    let f = fixture();
    let project_root_key = additional(vec![FileSystemSandboxEntry::new(
        FileSystemPath::Special {
            value: FileSystemSpecialPath::project_roots(Some(".ssh/id_ed25519".into())),
        },
        FileSystemAccessMode::Read,
    )]);
    // Checked against the project only it opens nothing; the sandbox may
    // resolve it against the home folder, where it opens the key.
    assert_eq!(
        opened_protected_path(&f.denials, &project_root_key, &[f.project.as_path()]),
        None
    );
    assert_eq!(
        opened_protected_path(
            &f.denials,
            &project_root_key,
            &[f.project.as_path(), f.user_home.as_path()]
        ),
        Some(f.user_home.join(".ssh/id_ed25519").display().to_string())
    );
    // A write above a protected folder stays below its denial.
    assert_eq!(
        opened_protected_path(
            &f.denials,
            &additional(vec![entry(&f.user_home, FileSystemAccessMode::Write)]),
            &[f.project.as_path()]
        ),
        None
    );
}

#[test]
fn issue_245_grants_drop_only_protected_entries() {
    let f = fixture();
    let notes = entry(&f.project.join("notes.txt"), FileSystemAccessMode::Read);
    let key = entry(
        &f.user_home.join(".ssh/id_ed25519"),
        FileSystemAccessMode::Read,
    );
    assert_eq!(
        without_protected_paths(
            &f.denials,
            Some(additional(vec![notes.clone(), key.clone()])),
            &[f.project.as_path()]
        ),
        Some(additional(vec![notes]))
    );
    assert_eq!(
        without_protected_paths(
            &f.denials,
            Some(additional(vec![key.clone()])),
            &[f.project.as_path()]
        ),
        None
    );
    let mut with_network = additional(vec![key]);
    with_network.network = Some(NetworkPermissions {
        enabled: Some(true),
    });
    assert_eq!(
        without_protected_paths(&f.denials, Some(with_network), &[f.project.as_path()]),
        Some(AdditionalPermissionProfile {
            network: Some(NetworkPermissions {
                enabled: Some(true)
            }),
            file_system: None,
        })
    );
}

#[test]
fn issue_245_refusals_name_the_level_and_the_way_forward() {
    assert_eq!(
        refusal(SecurityLevel::Aggressive, "/home/u/.ssh/id"),
        crate::security::protected_reads::explanation("/home/u/.ssh/id")
    );
    let moderate = refusal(SecurityLevel::Moderate, "/home/u/.ssh/id\u{1b}[2J");
    assert!(
        moderate.starts_with("Security level Moderate blocked access to `/home/u/.ssh/id"),
        "{moderate}"
    );
    assert!(!moderate.contains('\u{1b}'), "{moderate}");
    assert!(moderate.contains("protected files"), "{moderate}");
    assert!(moderate.contains("Permissive in /security"), "{moderate}");
}

/// The run's policy with the rules applied, for a profile that reads
/// everything and writes `writable`.
fn protected_policy(
    denials: &ReadDenials,
    writable: &[&std::path::Path],
) -> FileSystemSandboxPolicy {
    let mut entries = vec![FileSystemSandboxEntry::new(
        FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        FileSystemAccessMode::Read,
    )];
    entries.extend(
        writable
            .iter()
            .map(|path| entry(path, FileSystemAccessMode::Write)),
    );
    let profile = codex_protocol::models::PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(entries),
        codex_protocol::permissions::NetworkSandboxPolicy::Restricted,
    );
    denials
        .apply(&profile)
        .expect("restricted profile")
        .file_system_sandbox_policy()
}

fn write(path: &std::path::Path) -> AdditionalPermissionProfile {
    additional(vec![entry(path, FileSystemAccessMode::Write)])
}

#[test]
fn issue_513_a_write_never_reopens_the_runtime_entries() {
    let f = fixture();
    let runtime = f.codex_home.join("tmp");
    let launcher = runtime.join("launcher");
    let link = f.project.join("helpers");
    std::os::unix::fs::symlink(&runtime, &link).unwrap();
    let protected = protected_policy(&f.denials, &[f.project.as_path()]);
    let root = additional(vec![FileSystemSandboxEntry::new(
        FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        FileSystemAccessMode::Write,
    )]);
    for (name, request) in [
        ("equal", write(&runtime)),
        ("narrower", write(&launcher)),
        // Above it, where the profile itself could not write it.
        ("broader", write(&f.codex_home)),
        ("root", root),
        ("through a link", write(&link.join("launcher"))),
    ] {
        for scope in [ReadOnlyScope::Runtime, ReadOnlyScope::Kept] {
            assert!(
                opened_read_only_path(
                    &f.denials,
                    Some(&protected),
                    &request,
                    &[f.project.as_path()],
                    scope
                )
                .is_some(),
                "{name} {scope:?}"
            );
        }
    }
    // Reads never write; the rule holds where the profile could write.
    let read = additional(vec![entry(&runtime, FileSystemAccessMode::Read)]);
    // #583: the home itself is read-only too, so "broader" is its folder.
    let writes_home = protected_policy(&f.denials, &[f.project.as_path(), f.user_home.as_path()]);
    for (name, request, protected) in [
        ("read", read, &protected),
        (
            "broader, profile writes it",
            write(&f.user_home),
            &writes_home,
        ),
        ("project", write(&f.project.join("notes.txt")), &protected),
    ] {
        assert_eq!(
            opened_read_only_path(
                &f.denials,
                Some(protected),
                &request,
                &[f.project.as_path()],
                ReadOnlyScope::Kept
            ),
            None,
            "{name}"
        );
    }
}

/// #583: an additional write may not open the read-only Corbanu home (it
/// would let a command create `rules/`), where the never-lifted rules
/// apply, but one inside `worktrees` opens nothing protected.
#[test]
fn issue_583_a_write_opens_worktrees_but_never_the_corbanu_home() {
    let f = fixture();
    let worktree = f.codex_home.join("worktrees/w");
    std::fs::create_dir_all(&worktree).unwrap();
    let denials = ReadDenials::collect(
        &f.codex_home,
        Some(&f.user_home),
        /*claude_config_dir*/ None,
        &[AbsolutePathBuf::from_absolute_path(&f.project).unwrap()],
    );
    let protected = protected_policy(&denials, &[f.project.as_path(), f.user_home.as_path()]);
    let opened_by = |path: &Path| {
        opened_read_only_path(
            &denials,
            Some(&protected),
            &write(path),
            &[f.project.as_path()],
            ReadOnlyScope::NeverLifted,
        )
    };
    let home = AbsolutePathBuf::from_absolute_path(&f.codex_home).unwrap();
    assert_eq!(opened_by(&f.codex_home), Some(home.clone()));
    assert_eq!(opened_by(&f.codex_home.join("rules")), Some(home));
    assert_eq!(opened_by(&worktree), None);
    assert_eq!(opened_by(&worktree.join("src")), None);
}

#[test]
fn issue_513_other_write_protections_count_under_aggressive_until_a_grant_lifts_them() {
    let f = fixture();
    let protected = protected_policy(&f.denials, &[f.project.as_path()]);
    let check = |denials: &ReadDenials, writes, target: &Path| {
        opened(
            Check {
                denials,
                protected: Some(&protected),
                reads: true,
                writes,
            },
            &write(target),
            &[f.project.as_path()],
        )
    };
    let mut granted = ReadDenials::collect(
        &f.codex_home,
        Some(&f.user_home),
        /*claude_config_dir*/ None,
        &[AbsolutePathBuf::from_absolute_path(&f.project).unwrap()],
    );
    granted.lift_write_rules();
    // A git hook counts until an Aggressive grant lifts it. The project's
    // `.agents` (#442, Travis 2026-10-11) and the runtime entries never
    // lift with a grant. Under Moderate a person's fresh approval (scope
    // Runtime) lifts all but the runtime entries (#513, Travis 2026-10-11).
    for (protected_path, target, rule) in [
        (
            f.project.join(".git/hooks"),
            f.project.join(".git/hooks/pre-commit"),
            WriteRule::Liftable,
        ),
        (
            f.project.join(".agents"),
            f.project.join(".agents/skill.md"),
            WriteRule::NeverGranted,
        ),
        (
            f.codex_home.join("tmp"),
            f.codex_home.join("tmp"),
            WriteRule::Runtime,
        ),
    ] {
        let refused = Some(Opened::Write {
            path: protected_path.display().to_string(),
            rule,
        });
        assert_eq!(check(&f.denials, ReadOnlyScope::Kept, &target), refused);
        let after_grant = if rule == WriteRule::Liftable {
            None
        } else {
            refused.clone()
        };
        // #565: before the approval only what no grant lifts counts, so it
        // is refused before anyone is asked; a git hook waits for the grant.
        assert_eq!(
            check(&f.denials, ReadOnlyScope::NeverLifted, &target),
            after_grant,
            "{}",
            target.display()
        );
        assert_eq!(
            check(&granted, ReadOnlyScope::Kept, &target),
            after_grant,
            "{}",
            target.display()
        );
    }
    assert_eq!(
        check(
            &f.denials,
            ReadOnlyScope::Runtime,
            &f.project.join(".agents/skill.md")
        ),
        None
    );
}

#[test]
fn issue_513_grants_drop_entries_that_reopen_a_write_protection() {
    let f = fixture();
    let notes = entry(&f.project.join("notes.txt"), FileSystemAccessMode::Write);
    let launcher = entry(
        &f.codex_home.join("tmp/launcher"),
        FileSystemAccessMode::Write,
    );
    let protected = protected_policy(&f.denials, &[f.project.as_path()]);
    let check = Check {
        denials: &f.denials,
        protected: Some(&protected),
        reads: true,
        writes: ReadOnlyScope::Runtime,
    };
    assert_eq!(
        without_opened(
            check,
            Some(additional(vec![notes.clone(), launcher])),
            &[f.project.as_path()]
        ),
        Some(additional(vec![notes]))
    );
}

#[test]
fn issue_513_write_refusals_say_whether_a_grant_can_lift_them() {
    let runtime = write_refusal(
        SecurityLevel::Moderate,
        "/home/u/.corbanu/tmp\u{1b}[2J",
        WriteRule::Runtime,
    );
    assert!(
        runtime.starts_with("Security level Moderate blocked writing to `/home/u/.corbanu/tmp"),
        "{runtime}"
    );
    assert!(!runtime.contains('\u{1b}'), "{runtime}");
    assert!(runtime.contains("cannot be granted"), "{runtime}");
    assert!(runtime.contains("Permissive in /security"), "{runtime}");
    let hook = write_refusal(
        SecurityLevel::Aggressive,
        "/p/.git/hooks",
        WriteRule::Liftable,
    );
    assert!(hook.starts_with("Security level Aggressive"), "{hook}");
    assert!(hook.contains("\"Grant once\""), "{hook}");
    // #565: "Grant once" never lifts .codex or .agents under Aggressive.
    assert!(!hook.contains(".agents"), "{hook}");
    assert!(!hook.contains("cannot be granted"), "{hook}");
    // #520: kept even with a grant under Aggressive.
    let kept = write_refusal(
        SecurityLevel::Aggressive,
        "/p/.agents",
        WriteRule::NeverGranted,
    );
    assert!(
        kept.contains("neither an additional permission nor"),
        "{kept}"
    );
    assert!(kept.contains("Permissive in /security"), "{kept}");
    // Moderate: only a person's fresh approval lifts it, whatever the rule
    // under Aggressive.
    for rule in [WriteRule::Liftable, WriteRule::NeverGranted] {
        let hook = write_refusal(SecurityLevel::Moderate, "/p/.git/hooks", rule);
        assert!(
            hook.starts_with("Security level Moderate blocked writing to `/p/.git/hooks`"),
            "{hook}"
        );
        assert!(hook.contains("Only a person's own approval"), "{hook}");
        for approver in [
            "automatic reviewer",
            "permission hook",
            "\"don't ask again\"",
            "permission grant",
        ] {
            assert!(hook.contains(approver), "{approver}: {hook}");
        }
        assert!(!hook.contains("Grant once"), "{hook}");
        assert!(hook.contains("Permissive in /security"), "{hook}");
    }
}

/// Issue #513 (Travis, 2026-10-11): which read-only paths count, by level
/// and approver. Under Moderate only a person's fresh approval of the
/// command leaves the persistence files to it; grants never carry them.
#[test]
fn issue_513_moderate_scope_follows_the_approver() {
    use SecurityLevel::Aggressive;
    use SecurityLevel::Moderate;
    use SecurityLevel::Permissive;
    let (runtime, never_lifted, kept) = (
        ReadOnlyScope::Runtime,
        ReadOnlyScope::NeverLifted,
        ReadOnlyScope::Kept,
    );
    for (level, person, before, after) in [
        (Moderate, true, runtime, runtime),
        (Moderate, false, kept, kept),
        // Aggressive: a grant for the run is known only after the approval;
        // what no grant lifts is refused before anyone is asked (#565).
        (Aggressive, true, never_lifted, kept),
        (Aggressive, false, never_lifted, kept),
        (Permissive, false, runtime, runtime),
    ] {
        assert_eq!(
            ReadOnlyScope::before_approval(level, person),
            before,
            "{level:?} {person}"
        );
        assert_eq!(
            ReadOnlyScope::after_approval(level, person),
            after,
            "{level:?} {person}"
        );
    }
    assert_eq!(ReadOnlyScope::for_grant(Moderate), kept);
    assert_eq!(ReadOnlyScope::for_grant(Aggressive), kept);
    assert_eq!(ReadOnlyScope::for_grant(Permissive), runtime);
}

/// Review r2 (#524): a grant is checked with the workspace roots, so a write
/// to a second root's `.agents` is dropped under Aggressive (it would make
/// every later command fail), and under Moderate too (#513, Travis
/// 2026-10-11: a grant never lifts it).
#[test]
fn issue_524_grants_are_checked_with_every_workspace_root() {
    let f = fixture();
    let second = f.user_home.join("second");
    std::fs::create_dir_all(&second).unwrap();
    let roots: Vec<AbsolutePathBuf> = [&f.project, &second]
        .iter()
        .map(|root| AbsolutePathBuf::from_absolute_path(root.as_path()).unwrap())
        .collect();
    let profile = codex_protocol::models::PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            entry(&f.project, FileSystemAccessMode::Write),
            entry(&second, FileSystemAccessMode::Write),
        ]),
        codex_protocol::permissions::NetworkSandboxPolicy::Restricted,
    );
    let configs = [GrantCheckConfig {
        cwd: roots[0].clone(),
        workspace_roots: roots.clone(),
        profile,
    }];
    let grant = write(&second.join(".agents/skill.md"));
    let (kept, refusal) = strip_grant(
        SecurityLevel::Aggressive,
        &f.codex_home,
        &f.codex_home,
        &configs,
        grant.clone(),
    );
    assert_eq!(kept, None);
    let refusal = refusal.expect("a refusal");
    assert!(refusal.contains("blocked writing to"), "{refusal}");
    let (kept, refusal) = strip_grant(
        SecurityLevel::Moderate,
        &f.codex_home,
        &f.codex_home,
        &configs,
        grant.clone(),
    );
    assert_eq!(kept, None);
    let refusal = refusal.expect("a refusal");
    assert!(
        refusal.starts_with("Security level Moderate blocked writing to"),
        "{refusal}"
    );
    assert!(refusal.contains("permission grant cannot"), "{refusal}");
    // Permissive keeps it.
    assert_eq!(
        strip_grant(
            SecurityLevel::Permissive,
            &f.codex_home,
            &f.codex_home,
            &configs,
            grant.clone()
        ),
        (Some(grant), None)
    );
}

#[test]
fn issue_524_a_failed_check_is_refused_and_explained() {
    let text = Opened::Unchecked.refusal(SecurityLevel::Aggressive);
    assert_eq!(text, unchecked_refusal(SecurityLevel::Aggressive));
    assert!(
        text.starts_with("Security level Aggressive could not check"),
        "{text}"
    );
    assert!(text.contains("Permissive in /security"), "{text}");
}

/// Issue #513 (Travis, 2026-10-11), the orchestrator with a probe tool:
/// under Moderate an additional write to the project's `.agents` runs only
/// when a person was asked about it now. The person's prompt says which file
/// approving opens; answered without asking the same write is refused and
/// explained, both where nobody may decide it (a run preapproved as an allow
/// rule makes it) and where the prompt is answered by something other than a
/// person asked now (here the approval cache). This prompt offers no "for
/// this session" answer, so a client that sends one anyway is refused (#2213)
/// and primes nothing; the test primes the cache directly with what an
/// offered "for this session" answer would store. A stored session grant of
/// that write does not even preapprove it: the grant is read through the
/// protected-path strip (#245). The person picks the most lasting answer the
/// prompt offers (#2187 review).
#[tokio::test]
async fn issue_513_orchestrator_lets_only_a_person_asked_now_open_a_persistence_file() {
    use crate::session::turn_context::TurnEnvironment;
    use crate::tools::sandboxing::ApprovalAction;
    use crate::tools::sandboxing::ExecApprovalRequirement;
    use crate::tools::sandboxing::ToolError;
    use codex_protocol::protocol::AskForApproval;
    use codex_protocol::protocol::EventMsg;
    use codex_protocol::protocol::ReviewDecision;
    use codex_utils_path_uri::PathUri;
    use std::sync::Arc;
    use std::sync::Mutex;

    struct Probe {
        additional: AdditionalPermissionProfile,
        requirement: ExecApprovalRequirement,
        cwd: PathUri,
        ran: bool,
    }
    impl crate::tools::sandboxing::Approvable<TurnEnvironment> for Probe {
        fn additional_permissions<'a>(
            &self,
            _req: &'a TurnEnvironment,
        ) -> Option<&'a AdditionalPermissionProfile> {
            // The probe owns the profile; the trait wants it borrowed from
            // the request, so leak one per call (a test only).
            Some(Box::leak(Box::new(self.additional.clone())))
        }
        fn sandbox_permissions(
            &self,
            _req: &TurnEnvironment,
        ) -> crate::sandboxing::SandboxPermissions {
            crate::sandboxing::SandboxPermissions::WithAdditionalPermissions
        }
        fn exec_approval_requirement(
            &self,
            _req: &TurnEnvironment,
        ) -> Option<ExecApprovalRequirement> {
            Some(self.requirement.clone())
        }
        fn approval_action(
            &self,
            _req: &TurnEnvironment,
            call_id: &str,
        ) -> std::io::Result<ApprovalAction> {
            Ok(ApprovalAction::ExecCommand {
                id: call_id.to_string(),
                environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
                command: vec!["true".into()],
                hook_command: "true".to_string(),
                cwd: self.cwd.clone(),
                sandbox_permissions:
                    crate::sandboxing::SandboxPermissions::WithAdditionalPermissions,
                additional_permissions: Some(self.additional.clone()),
                justification: None,
                tty: false,
                proposed_execpolicy_amendment: None,
            })
        }
    }
    impl crate::tools::sandboxing::Sandboxable for Probe {
        fn sandbox_preference(&self) -> codex_sandboxing::SandboxablePreference {
            codex_sandboxing::SandboxablePreference::Auto
        }
    }
    impl crate::tools::sandboxing::ToolRuntime<TurnEnvironment, ()> for Probe {
        fn turn_environment<'a>(&self, req: &'a TurnEnvironment) -> &'a TurnEnvironment {
            req
        }
        async fn run(
            &mut self,
            _req: &TurnEnvironment,
            _attempt: &crate::tools::sandboxing::SandboxAttempt<'_>,
            _ctx: &crate::tools::sandboxing::ToolCtx,
        ) -> Result<(), ToolError> {
            self.ran = true;
            Ok(())
        }
    }

    let (session, events) = crate::session::tests::make_session_with_config_and_rx(|config| {
        config.security_level = SecurityLevel::Moderate;
        config
            .features
            .enable(codex_features::Feature::SourceEnvelopes)
            .expect("source envelopes");
        config
            .permissions
            .approval_policy
            .set(AskForApproval::OnRequest)
            .expect("approval policy");
    })
    .await
    .expect("session");
    let _control = session
        .services
        .agent_control
        .local()
        .expect("the local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Moderate,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("policy");
    *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
    // The person records the prompt's reason and gives the most lasting
    // answer it offers.
    let reasons = Arc::new(Mutex::new(Vec::new()));
    // A non-conforming client answers "for this session" whatever is offered.
    let non_conforming = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let answerer = tokio::spawn({
        let session = Arc::clone(&session);
        let reasons = Arc::clone(&reasons);
        let non_conforming = Arc::clone(&non_conforming);
        async move {
            while let Ok(event) = events.recv().await {
                let EventMsg::ExecApprovalRequest(request) = event.msg else {
                    continue;
                };
                reasons.lock().expect("lock").push(request.reason.clone());
                let offered = request.available_decisions.clone().unwrap_or_default();
                let decision = if offered.contains(&ReviewDecision::ApprovedForSession)
                    || non_conforming.load(std::sync::atomic::Ordering::SeqCst)
                {
                    ReviewDecision::ApprovedForSession
                } else {
                    ReviewDecision::Approved
                };
                let id = request.approval_id.unwrap_or(request.call_id);
                session.notify_approval(&id, decision).await;
            }
        }
    });
    let turn = session.new_default_turn().await;
    let environment = turn
        .initial_environments
        .primary()
        .expect("a primary environment")
        .clone();
    #[allow(deprecated)]
    let agents = turn.cwd.join(".agents");
    let tool_ctx = crate::tools::sandboxing::ToolCtx {
        cancellation_token: tokio_util::sync::CancellationToken::new(),
        session: Arc::clone(&session),
        step_context: crate::session::step_context::StepContext::for_test(Arc::clone(&turn)),
        call_id: "probe-call".to_string(),
        tool_name: codex_tools::ToolName::plain("probe"),
    };
    let run = |additional: AdditionalPermissionProfile, requirement: ExecApprovalRequirement| {
        let tool_ctx = &tool_ctx;
        let environment = &environment;
        async move {
            let mut probe = Probe {
                additional,
                requirement,
                cwd: environment.cwd().clone(),
                ran: false,
            };
            // A prompt that is never sent (or never answered) fails the test
            // instead of hanging it (#2187 review).
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(60),
                crate::tools::orchestrator::ToolOrchestrator::new().run(
                    &mut probe,
                    environment,
                    tool_ctx,
                ),
            )
            .await
            .expect("the probe run finished");
            (result.map(|_| ()), probe.ran)
        }
    };
    let mut results = Vec::new();
    // Asked now.
    results.push(
        run(
            write(agents.as_path()),
            ExecApprovalRequirement::NeedsApproval {
                reason: None,
                proposed_execpolicy_amendment: None,
            },
        )
        .await,
    );
    // A stored session grant of the same write, read as a handler does,
    // does not cover it.
    assert!(
        session.record_granted_request_permissions_for_turn(
            &codex_protocol::request_permissions::RequestPermissionsResponse {
                permissions: write(agents.as_path()).into(),
                scope: codex_protocol::request_permissions::PermissionGrantScope::Session,
                strict_auto_review: false,
            },
            &environment.selection.environment_id,
            turn.as_ref(),
            session.services.authority_marker(),
        ),
        "the grant is recorded"
    );
    let granted = crate::tools::handlers::apply_granted_turn_permissions(
        &session,
        &turn,
        &environment,
        environment.cwd(),
        crate::sandboxing::SandboxPermissions::WithAdditionalPermissions,
        Some(write(agents.as_path())),
    )
    .await;
    assert!(
        !granted.permissions_preapproved,
        "a stored grant does not cover a protected write"
    );
    // Not asked: the same write preapproved (as an allow rule makes it).
    results.push(
        run(
            write(agents.as_path()),
            ExecApprovalRequirement::Skip {
                bypass_sandbox: false,
                proposed_execpolicy_amendment: None,
            },
        )
        .await,
    );
    // Asked now, answered "for this session" by a non-conforming client: the
    // prompt did not offer it, so #2213 refuses the answer, nothing runs and
    // nothing is remembered.
    non_conforming.store(true, std::sync::atomic::Ordering::SeqCst);
    let asked_now = ExecApprovalRequirement::NeedsApproval {
        reason: None,
        proposed_execpolicy_amendment: None,
    };
    results.push(run(write(agents.as_path()), asked_now.clone()).await);
    non_conforming.store(false, std::sync::atomic::Ordering::SeqCst);
    // Prime the approval cache as an offered "for this session" answer to
    // this command would.
    let primed = crate::tools::sandboxing::Approvable::approval_action(
        &Probe {
            additional: write(agents.as_path()),
            requirement: asked_now.clone(),
            cwd: environment.cwd().clone(),
            ran: false,
        },
        &environment,
        "probe-call",
    )
    .expect("the probe's approval action");
    let decided_under = session.services.approval_cache().await.policy();
    session
        .services
        .remember_approvals(
            crate::tools::exec_command_cache_keys(&primed, &turn.initial_environments),
            decided_under,
        )
        .await;
    // The same write answered from that cache, nobody asked now.
    results.push(run(write(agents.as_path()), asked_now).await);
    answerer.abort();

    // Two prompts (the first run and the refused non-conforming answer), each
    // saying which file approving opens; the preapproved and cached runs ask
    // nobody.
    let reasons = reasons.lock().expect("lock").clone();
    assert_eq!(reasons.len(), 2, "{reasons:?}");
    for reason in &reasons {
        let reason = reason.clone().unwrap_or_default();
        assert!(
            reason.contains("Approving lets this command write"),
            "{reason}"
        );
        assert!(reason.contains(".agents"), "{reason}");
    }
    let assert_ran = |index: usize| {
        let (result, ran) = &results[index];
        assert!(result.is_ok(), "run {index}: {result:?}");
        assert!(*ran, "run {index}");
    };
    // Nobody asked now: refused and explained, nothing ran.
    let assert_refused = |index: usize| {
        let (result, ran) = &results[index];
        let Err(ToolError::Rejected(message)) = result else {
            panic!("run {index}: expected a refusal: {result:?}");
        };
        assert!(
            message.starts_with("Security level Moderate blocked writing to"),
            "run {index}: {message}"
        );
        assert!(
            message.contains("Only a person's own approval"),
            "run {index}: {message}"
        );
        assert!(!*ran, "run {index}");
    };
    assert_ran(0);
    assert_refused(1);
    // #2213: the unoffered "for this session" answer is refused, not run.
    let (result, ran) = &results[2];
    let Err(ToolError::Declined(message)) = result else {
        panic!("run 2: expected a refusal: {result:?}");
    };
    assert_eq!(message, "approval answer was not offered by this prompt");
    assert!(!*ran, "run 2: {message}");
    assert_refused(3);
}
