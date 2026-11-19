//! #2548 (D553): scripts composed at random from bash grammar constructs
//! around a `git push` that a rule refuses or prompts for. Every one is
//! refused or asks; none runs unasked. A fixed seed keeps CI reproducible.

use super::*;

const FORBID: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const PROMPT: &str = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;
const SEED: u64 = 0x2548_b00c_5eed_0001;
const CASES: usize = 300;
const MAX_LAYERS: u64 = 4;
/// How many constructs [`wrap`] chooses from.
const CONSTRUCTS: u64 = 108;

/// xorshift64*: deterministic, no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

fn single_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r#"'\''"#))
}

fn double_quoted(text: &str) -> String {
    let mut quoted = String::from("\"");
    for ch in text.chars() {
        if matches!(ch, '\\' | '$' | '`' | '"') {
            quoted.push('\\');
        }
        quoted.push(ch);
    }
    quoted.push('"');
    quoted
}

/// Bash grammar constructs that run `inner`, a command that runs the
/// payload. `delimiter` is unique to the layer.
fn wrap(rng: &mut Rng, inner: &str, delimiter: &str) -> String {
    let heredoc =
        |head: &str, tail: &str| format!("{head}<<'{delimiter}'\n{inner}\n{delimiter}{tail}");
    let arithmetic = single_quoted(&format!("a[$({inner}\n)]"));
    match rng.below(CONSTRUCTS) {
        0 => format!("time {inner}"),
        1 => format!("time -p {inner}"),
        2 => format!("! {inner}"),
        3 => format!("command {inner}"),
        4 => format!("builtin eval {}", single_quoted(inner)),
        5 => format!("coproc {inner}"),
        6 => format!("nohup {inner}"),
        7 => format!("env A=1 {inner}"),
        8 => format!("sudo {inner}"),
        9 => format!("xargs {inner}"),
        10 => format!("( {inner}\n)"),
        11 => format!("{{ {inner}\n}}"),
        12 => format!("f() {{ {inner}\n}}; f"),
        13 => format!("function f {{ {inner}\n}}; f"),
        14 => format!("true | {inner}"),
        15 => format!("{{ {inner}\n}} | cat"),
        16 => format!("true && {inner}"),
        17 => format!("false || {inner}"),
        18 => format!("if true; then {inner}\nfi"),
        19 => format!("for i in 1; do {inner}\ndone"),
        20 => format!("while true; do {inner}\nbreak; done"),
        21 => format!("echo $({inner}\n)"),
        22 if !inner.contains(['`', '\\']) => format!("x=`{inner}`"),
        23 => format!("eval {}", single_quoted(inner)),
        24 => format!("eval {}", double_quoted(inner)),
        25 => format!("exec {inner}"),
        26 => format!("bash -c {}", single_quoted(inner)),
        27 => format!("sh -c {}", double_quoted(inner)),
        28 => format!("echo {} | bash", single_quoted(inner)),
        29 => heredoc("bash ", ""),
        30 => format!("bash <<< {}", single_quoted(inner)),
        31 => heredoc("exec ", "\nbash"),
        32 => heredoc("source /dev/stdin ", ""),
        33 => format!(". <(echo {})", single_quoted(inner)),
        34 => format!("trap {} EXIT", single_quoted(inner)),
        35 => heredoc("f() { bash; }; f ", ""),
        36 => format!("alias g={}; g", single_quoted(inner)),
        37 => format!("timeout 5 {inner}"),
        38 => format!("command eval {}", single_quoted(inner)),
        39 => format!("{{ {inner}\n}} & wait"),
        40 => format!("case x in x) {inner}\n;; esac"),
        41 => heredoc("time ! bash ", ""),
        42 => format!(
            "coproc bash; echo {} >&\"${{COPROC[1]}}\"",
            single_quoted(inner)
        ),
        43 => format!("printf '%s\\n' {} | sh -s", single_quoted(inner)),
        44 => format!("x={}; eval \"$x\"", single_quoted(inner)),
        // Review round 1: backquotes the grammar leaves in tokens, fd 0
        // replaced after a literal input, arithmetic in assignments and
        // builtins, ksh, and `hash` renaming a program.
        46 if !inner.contains(['`', '\\']) => format!("cat <<{delimiter}\n`{inner}`\n{delimiter}"),
        47 if !inner.contains(['`', '\\']) => format!("[[ x =~ (`{inner}`) ]]"),
        48 if !inner.contains(['`', '\\']) => format!("case x in ?a`{inner}`) ;; esac"),
        49 => format!(
            "exec 3< <(echo {}); bash <<<'echo hi' 0>&3",
            single_quoted(inner)
        ),
        50 => format!("echo {} > s; bash <<<'echo hi' 00<s", single_quoted(inner)),
        // Grouped: `env A=1 OPTIND=…` alone only prints the environment.
        51 => format!("{{ OPTIND={arithmetic}\n}}"),
        52 => format!("ksh <<'{delimiter}'\nshift {arithmetic}\n{delimiter}"),
        53 => format!("hash -p /bin/sh x; x -c {}", single_quoted(inner)),
        54 => format!(
            "zsh -c {}",
            single_quoted(&format!("printf '%d' {arithmetic}"))
        ),
        55 => format!("for SECONDS in {arithmetic}; do :; done"),
        // Review round 2: comments the grammar sees where bash does not.
        56 => format!("x=(a)#; {inner}"),
        57 => format!("echo a\\\n#b; {inner}"),
        // Review round 2: other places where the grammar and the shells
        // split text differently, and zsh-only evaluation.
        58 => match inner.split_once(' ') {
            Some((head, tail)) if !head.contains(['\'', '"', '\n']) => {
                format!("{head} 2>&1 {tail}")
            }
            _ => format!("{inner} 2>&1"),
        },
        59 => match inner.char_indices().nth(1) {
            Some((at, _)) if inner.starts_with(|ch: char| ch.is_ascii_alphabetic()) => {
                format!("{}\\\n{}", &inner[..at], &inner[at..])
            }
            _ => format!("echo a\\\r\n{inner}"),
        },
        60 => format!("(echo == x; {inner}\n)"),
        61 => format!("echo ${{x#{{}}; {inner}\necho }}"),
        62 => format!("zsh -c {}", single_quoted(&format!("[ -t {arithmetic} ]"))),
        63 => format!(
            "zsh -c {}",
            single_quoted(&format!(
                "echo $HOME[{}]",
                single_quoted(&format!("$({inner}\n)"))
            ))
        ),
        64 => format!(
            "zsh -c {}",
            single_quoted(&format!("nocorrect X=1 {inner}"))
        ),
        65 => format!("echo a\\\r\n{inner}"),
        // Review round 3: here-document ends the grammar places elsewhere,
        // `[ … | … ]`, zsh's command tables, `coproc nocorrect`, positional
        // printf conversions.
        66 => format!("cat <<'{delimiter}' &&\n{delimiter}\n{inner}\n{delimiter}"),
        67 => format!("[ x | sh -c {} ]", single_quoted(inner)),
        68 => format!(
            "zsh -c {}",
            single_quoted(&format!(
                "commands=(x /bin/sh); x -c {}",
                single_quoted(inner)
            ))
        ),
        69 => format!(
            "zsh -c {}",
            single_quoted(&format!("coproc nocorrect X=1 {inner}"))
        ),
        70 => format!(
            "zsh -c {}",
            single_quoted(&format!("printf '%2$d %1$s' 5 {arithmetic}"))
        ),
        71 => format!("cat <<'{delimiter}' && echo $(\n{inner}\n)\n{delimiter}"),
        // `[` across a newline (review round 4 log).
        72 => format!("[\n! sh -c {} ]", single_quoted(inner)),
        // Review round 4: process substitution in pattern tokens, same-name
        // here-documents, zsh `|=` and `$#a[x]`, printf length modifiers,
        // dash, and a `case` arm inside a substitution.
        73 => format!("[ x != ab<({inner}\n) ]"),
        74 => format!("case x in ab<({inner}\n)) ;; esac"),
        75 => format!(
            "cat <<'{delimiter}' && bash <<'{delimiter}'\necho hi\n{delimiter}\n{inner}\n{delimiter}"
        ),
        76 => format!(
            "zsh -c {}",
            single_quoted(&format!("[ x |=sh -c {} ]", single_quoted(inner)))
        ),
        77 => format!(
            "zsh -c {}",
            single_quoted(&format!("x={arithmetic}; a=(1); cat <<Q\nz $#a[x]\nQ"))
        ),
        78 => format!(
            "zsh -c {}",
            single_quoted(&format!("printf '%ld' {arithmetic}"))
        ),
        79 => format!(
            "sh -c {}",
            single_quoted(&format!("echo $'\\' ; {inner} ; echo ' #'"))
        ),
        80 => format!(
            "sh -c {}",
            single_quoted(&format!("[[ -z x || sh -c {} ]]", single_quoted(inner)))
        ),
        // Review round 6: bash 3.2's `$(…)` scan, dash's `time`, multibyte
        // locales.
        82 => format!("echo \"$$(\" '\")\"\n{inner}\n'"),
        83 => format!("echo $(echo x\\ #'\n)\n{inner}\n')"),
        84 => format!("sh -c {}", single_quoted(&format!("time -l {inner}"))),
        85 => format!("LC_ALL=zh_TW.Big5\necho \"\u{4e01}\\\"\n{inner}\necho \""),
        86 => format!("echo \"$${{\" '\"}}\"\n{inner}\n'"),
        // Review round 7: dash's `time` options, zsh's `""=cmd`, builtin
        // wrapper programs.
        87 => format!("sh -c {}", single_quoted(&format!("time -p -l {inner}"))),
        88 => format!("sh -c {}", single_quoted(&format!("time '-l' {inner}"))),
        89 => format!("zsh -c {}", single_quoted(&format!("\"\"={inner}"))),
        90 => format!("/usr/bin/jobs -x sh -c {}", single_quoted(inner)),
        // Review round 8: an escaped blank before `#`.
        91 => format!("echo \\ #; {inner}"),
        92 => format!("echo x\\\t#; {inner}"),
        // D713: forms read precisely now, and attempts to hide a command
        // or run code through them.
        93 => format!("[[ -n $HOME ]] && {inner}"),
        94 => format!("[ -f x ] || {inner}"),
        95 => format!("echo \"$({inner}\n)\""),
        96 if !inner.contains(['`', '\\', '\'', '"']) => format!("echo \"`{inner}`\""),
        97 => format!("x=$HOME {inner}"),
        98 => format!("echo ${{x:-$({inner}\n)}}"),
        99 => format!("echo $((1 + $?)); {inner}"),
        100 => format!("if [[ $x == y ]]; then {inner}\nfi"),
        101 => format!("x={arithmetic}; [[ 1 -eq $x ]]"),
        102 => format!("x={arithmetic}; echo $((x))"),
        103 => format!("x='-v'; [ $x {arithmetic} ]"),
        104 if inner.starts_with(|ch: char| ch.is_ascii_alphabetic()) => {
            format!("true\n\\{inner}")
        }
        105 => format!("[[ -n x || sh -c {} ]]", single_quoted(inner)),
        106 => format!("cmd=sh; $cmd -c {}", single_quoted(inner)),
        107 => format!("for f in $(echo a); do {inner}\ndone"),
        81 => format!(
            "cat <<'{delimiter}' && echo $(case x in x) cat <<'{delimiter}'\n{delimiter}\n{inner}\nesac)\n{delimiter}"
        ),
        _ => format!("noglob {inner}"),
    }
}

fn scripts() -> Vec<String> {
    let mut rng = Rng(SEED);
    let payloads = ["git push", "git push --force origin main", "'git' 'push'"];
    (0..CASES)
        .map(|case| {
            let mut script = payloads[rng.below(payloads.len() as u64) as usize].to_string();
            for layer in 0..=rng.below(MAX_LAYERS) {
                script = wrap(&mut rng, &script, &format!("E{case}_{layer}"));
            }
            script
        })
        .collect()
}

async fn check(script: &str, rules: &str, mode: usize) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: vec!["bash".into(), "-lc".into(), script.into()],
        approval_policy: AskForApproval::OnRequest,
        permission_profile: if mode == 0 {
            PermissionProfile::Disabled
        } else {
            PermissionProfile::workspace_write()
        },
        sandbox_permissions: if mode == 2 {
            SandboxPermissions::RequireEscalated
        } else {
            SandboxPermissions::UseDefault
        },
        prefix_rule: None,
    })
    .await
}

#[tokio::test]
async fn composed_scripts_never_run_unasked() {
    let scripts = scripts();
    // The generator covers every construct (no layer kind is unreachable).
    assert!(scripts.iter().any(|script| script.contains("coproc")));
    assert!(scripts.iter().any(|script| script.contains("<<'E")));
    let mut failures = Vec::new();
    for script in &scripts {
        for rules in [FORBID, PROMPT] {
            for mode in 0..3 {
                let result = check(script, rules, mode).await;
                let held = match &result {
                    ExecApprovalRequirement::Forbidden { .. } => true,
                    ExecApprovalRequirement::NeedsApproval {
                        proposed_execpolicy_amendment,
                        ..
                    } => proposed_execpolicy_amendment.is_none(),
                    ExecApprovalRequirement::Skip { .. } => false,
                };
                if !held {
                    failures.push(format!("{rules} mode {mode} {script:?}: {result:?}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} checks ran unasked:\n{}",
        failures.len(),
        scripts.len() * 6,
        failures.join("\n")
    );
}

/// The share of a corpus of real agent scripts (NUL-separated, path in
/// `CORBANU_SCRIPT_CORPUS`) that asks in full access because some unrelated
/// forbidden rule exists. Run by hand to measure what fail-closed reading
/// costs: `cargo test -p codex-core --lib script_corpus_ask_rate -- --ignored --nocapture`.
#[tokio::test]
#[ignore = "needs a corpus file"]
#[allow(clippy::print_stderr, reason = "a measurement run by hand")]
async fn script_corpus_ask_rate() {
    let Ok(path) = std::env::var("CORBANU_SCRIPT_CORPUS") else {
        return;
    };
    let corpus = std::fs::read_to_string(path).expect("corpus");
    let rules = r#"prefix_rule(pattern=["zz-unrelated"], decision="forbidden")"#;
    // `CORBANU_SCRIPT_CORPUS_VERBOSE=1` lists the scripts that ask;
    // `CORBANU_SCRIPT_CORPUS_SHELL` runs them with another shell than bash.
    let verbose = std::env::var_os("CORBANU_SCRIPT_CORPUS_VERBOSE").is_some();
    let shell = std::env::var("CORBANU_SCRIPT_CORPUS_SHELL").unwrap_or_else(|_| "bash".to_string());
    let (mut total, mut asks) = (0, 0);
    for script in corpus.split('\0').filter(|script| !script.is_empty()) {
        total += 1;
        let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(rules.to_string()),
            command: vec![shell.clone(), "-lc".into(), script.into()],
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        if !matches!(result, ExecApprovalRequirement::Skip { .. }) {
            asks += 1;
            if verbose {
                eprintln!("ASK {script:?}");
            }
        }
    }
    eprintln!("CORPUS {asks} of {total} scripts ask ({shell})");
}
