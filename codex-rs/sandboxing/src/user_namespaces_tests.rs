use super::*;
use pretty_assertions::assert_eq;

/// What the tools that need a user namespace print when the sandbox refuses
/// one (#876's glitch survey and the tools' sources), and failures that
/// have nothing to do with namespaces.
#[test]
fn issue_826_namespace_refusals_are_recognised_and_other_failures_are_not() {
    let refusals = [
        "unshare: unshare failed: Operation not permitted",
        "bwrap: No permissions to creating new namespace, likely because the kernel does not allow non-privileged user namespaces.",
        "bwrap: No permissions to create a new namespace, likely because the kernel does not allow non-privileged user namespaces.",
        // Ubuntu 24.04's bubblewrap 0.9 in a contained pane on glitch (#891).
        "bwrap: No permissions to create new namespace, likely because the kernel does not allow non-privileged user namespaces. See <https://deb.li/bubblewrap> or <file:///usr/share/doc/bubblewrap/README.Debian.gz>.",
        "bwrap: setting up uid map: Permission denied",
        "cannot clone: Operation not permitted\nError: cannot re-exec process",
        "error: cloning builder process: Operation not permitted",
        "[1016:1016:FATAL:zygote_host_impl_linux.cc(127)] Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted",
        "Corbanu's Linux sandbox cannot start here: ... where creating user namespaces is denied (Operation not permitted)",
        "unshare: write failed /proc/self/uid_map: Operation not permitted",
        "[rootlesskit:parent] error: failed to start the child: fork/exec /proc/self/exe: operation not permitted",
    ];
    let others = [
        "cat: /home/someone/.ssh/id_ed25519: Operation not permitted",
        "touch: cannot touch '../outside': Read-only file system",
        "git@github.com: Permission denied (publickey).",
        "Cloning into 'project'...\nfatal: could not create work tree dir 'project': Permission denied",
        "unshare: invalid option -- 'z'",
        "error: namespace 'default' not found",
        // A path that only contains the words (review round 1).
        "cat: namespaces/prod.yaml: Permission denied",
        "touch: cannot touch 'userns/unshare.txt': Permission denied",
        "",
    ];
    let recognised = |outputs: &[&str]| {
        outputs
            .iter()
            .map(|output| reports_user_namespace_denial(output))
            .collect::<Vec<_>>()
    };
    assert_eq!(recognised(&refusals), vec![true; refusals.len()]);
    assert_eq!(recognised(&others), vec![false; others.len()]);
}

#[test]
fn issue_826_namespace_programs_by_file_name() {
    for program in ["unshare", "/usr/bin/bwrap", "podman", "buildah", "flatpak"] {
        assert!(needs_user_namespace(program), "{program}");
    }
    for program in ["docker", "ls", "unshared", "/opt/podman-wrapper", ""] {
        assert!(!needs_user_namespace(program), "{program}");
    }
}

/// The rule is named in plain language, with the level, the tools that need
/// it and the way forward.
#[test]
fn issue_826_explanations_name_the_rule() {
    for text in [
        denial_explanation(Rule::ModerateOneCommand),
        denial_explanation(Rule::ModerateAfterUntrusted),
        approval_note(Rule::ModerateOneCommand),
        approval_note(Rule::ModerateAfterUntrusted),
        claude_pane_note("Moderate"),
    ] {
        for expected in [
            "Moderate",
            "user namespace",
            "Linux",
            "rootless Podman, bwrap, unshare, Flatpak, Nix's build sandbox and Chrome's sandbox",
            "Operation not permitted",
            "Permissive in /security",
        ] {
            assert!(text.contains(expected), "{expected:?} missing: {text}");
        }
    }
    assert!(
        denial_explanation(Rule::Aggressive)
            .starts_with("Security level Aggressive blocks creating user namespaces")
    );
    assert!(claude_pane_note("Aggressive").contains("Claude Code's own Bash sandbox"));
}

/// D27 (a): under Moderate the explanations say how a person allows it for
/// one command (after a failure, whose output is untrusted content, only in
/// a new session); under Aggressive that nothing does. The prompt's warning
/// says what approving allows, that it can get past the file protections,
/// and what "one run" covers.
#[test]
fn d27_moderate_names_the_one_command_way_and_aggressive_does_not() {
    for text in [
        denial_explanation(Rule::ModerateOneCommand),
        denial_explanation(Rule::ModerateAfterUntrusted),
        approval_note(Rule::ModerateOneCommand),
        approval_note(Rule::ModerateAfterUntrusted),
    ] {
        for expected in ["outside the sandbox", "untrusted content", "Moderate"] {
            assert!(text.contains(expected), "{expected:?} missing: {text}");
        }
    }
    // After a failure the session already has untrusted content: a new one.
    for rule in [Rule::ModerateOneCommand, Rule::ModerateAfterUntrusted] {
        let text = denial_explanation(rule);
        for expected in [
            "a person can allow it for one command",
            "this one's too",
            "in a new session",
        ] {
            assert!(text.contains(expected), "{expected:?} missing: {text}");
        }
    }
    let note = approval_note(Rule::ModerateOneCommand);
    assert!(
        note.contains("approving this prompt doesn't lift it"),
        "{note}"
    );
    let note = approval_note(Rule::ModerateAfterUntrusted);
    assert!(
        note.contains("approving this prompt doesn't lift it"),
        "{note}"
    );
    // Kill switch on, or Moderate not fully in force: no new-session advice.
    for text in [
        denial_explanation(Rule::ModerateNotInForce),
        approval_note(Rule::ModerateNotInForce),
    ] {
        for expected in [
            "Security level Moderate",
            "kill switch",
            "Permissive in /security",
        ] {
            assert!(
                text.to_lowercase().contains(&expected.to_lowercase()),
                "{expected:?} missing: {text}"
            );
        }
        assert!(!text.to_lowercase().contains("new session"), "{text}");
    }
    for text in [
        denial_explanation(Rule::Aggressive),
        approval_note(Rule::Aggressive),
    ] {
        assert!(
            text.to_lowercase()
                .contains("no approval or grant lifts this"),
            "{text}"
        );
        assert!(!text.contains("outside the sandbox"), "{text}");
    }
    let warning = moderate_one_run_warning();
    for expected in [
        "Warning (Linux): approving also lets this one run create user namespaces",
        "rootless Podman, bwrap, unshare, Flatpak, Nix's build sandbox and Chrome's sandbox",
        "this command can get past the file protections",
        "may not run at Moderate",
        "kernel",
        "what it starts or is given while it runs",
        "never remembered",
    ] {
        assert!(
            warning.contains(expected),
            "{expected:?} missing: {warning}"
        );
    }
}

/// D48 (c): after untrusted content under Moderate, the explanations name
/// the known tools as the only commands it may still be allowed for (and a
/// new session's first command); the list is the owner's.
#[test]
fn d48_moderate_names_the_known_tools() {
    assert_eq!(
        KNOWN_TOOLS,
        ["unshare", "bwrap", "podman", "buildah", "flatpak", "nix"]
    );
    for text in [
        denial_explanation(Rule::ModerateOneCommand),
        denial_explanation(Rule::ModerateAfterUntrusted),
        approval_note(Rule::ModerateOneCommand),
        approval_note(Rule::ModerateAfterUntrusted),
    ] {
        assert!(
            text.contains(
                "one plain command that runs unshare, bwrap, podman, buildah, flatpak or nix"
            ),
            "{text}"
        );
    }
    for text in [
        denial_explanation(Rule::ModerateNotInForce),
        approval_note(Rule::ModerateNotInForce),
        denial_explanation(Rule::Aggressive),
        approval_note(Rule::Aggressive),
    ] {
        assert!(!text.contains("plain command"), "{text}");
    }
}
