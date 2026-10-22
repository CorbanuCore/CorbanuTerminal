use super::search_pattern_indices;
use pretty_assertions::assert_eq;

/// The pattern words of `script` run by `bash -c`, split as
/// `command_words` splits it.
fn patterns(script: &str) -> Vec<String> {
    let command = [
        "/bin/bash".to_string(),
        "-c".to_string(),
        script.to_string(),
    ];
    let words = super::super::command_words(&command, /*windows*/ false);
    let mut indices: Vec<usize> = search_pattern_indices(&command, &words, /*windows*/ false)
        .into_iter()
        .collect();
    indices.sort_unstable();
    indices
        .into_iter()
        .map(|index| words[index].clone())
        .collect()
}

#[test]
fn leading_assignments_before_searches_are_program_prefixes() {
    for script in [
        "LC_ALL=C grep .env logs",
        "LC_ALL=C LANG=en_US rg .env logs",
        "EMPTY= _MODE2=dev egrep .env logs",
        "MODE=a=b fgrep .env logs",
        "LC_ALL=C /usr/bin/grep .env logs",
        "LC_ALL=C git grep .env -- logs",
        "env LC_ALL=C grep .env logs",
        "LC_ALL=C command grep .env logs",
    ] {
        assert_eq!(patterns(script), [".env"], "{script}");
    }
}

#[test]
fn assignment_like_arguments_do_not_start_searches() {
    for script in [
        "cat LC_ALL=C grep .env",
        "echo LC_ALL=C LANG=C rg .env",
        "cat env LC_ALL=C grep .env",
        "=C grep .env logs",
        "1MODE=C grep .env logs",
        "MODE-NAME=C grep .env logs",
        "./MODE=C grep .env logs",
        "MODE[0]=C grep .env logs",
        "MODE+=C grep .env logs",
    ] {
        assert_eq!(patterns(script), Vec::<String>::new(), "{script}");
    }
}

#[test]
fn assigned_searches_keep_actual_paths_out_of_patterns() {
    for script in [
        "LC_ALL=C grep token .env",
        "LC_ALL=C rg -e token .env",
        "MODE=.npmrc grep token .env",
    ] {
        assert_eq!(patterns(script), ["token"], "{script}");
    }
    assert_eq!(
        patterns("LC_ALL=C grep -f patterns.txt .env"),
        Vec::<String>::new()
    );
    assert_eq!(patterns("LC_ALL=C rg --files .env"), Vec::<String>::new());
}

#[test]
fn assignment_prefixes_respect_command_boundaries() {
    for script in [
        "cat logs; LC_ALL=C grep .env logs",
        "cat logs | LC_ALL=C rg .env",
        "echo done && LC_ALL=C grep .env logs",
        "cat MODE=C; grep .env logs",
        "echo done\nLC_ALL=C grep .env logs",
    ] {
        assert_eq!(patterns(script), [".env"], "{script}");
    }
    assert_eq!(patterns("LC_ALL=C\ncat grep .env"), Vec::<String>::new());
    assert_eq!(patterns("LC_ALL=C grep -e token; cat .env"), ["token;"]);
}

#[test]
fn the_first_operand_of_a_search_is_its_pattern() {
    assert_eq!(patterns(r#"rg ".env" logs/"#), [".env"]);
    assert_eq!(patterns("rg -n -i .env logs/"), [".env"]);
    assert_eq!(patterns("grep -rn .env logs/ 2>/dev/null"), [".env"]);
    assert_eq!(patterns("grep -A 3 -m 1 .env logs/x.log"), [".env"]);
    assert_eq!(patterns("rg -g '*.log' --max-depth 2 .env logs/"), [".env"]);
    assert_eq!(patterns("rg -n -- .env logs/"), [".env"]);
    assert_eq!(patterns("cat logs/x.log | grep .env"), [".env"]);
    assert_eq!(patterns("cd logs && egrep .env x.log; echo done"), [".env"]);
    assert_eq!(patterns("git grep -n .env -- logs/"), [".env"]);
    assert_eq!(patterns("/usr/bin/grep .env x.log"), [".env"]);
}

#[test]
fn a_pattern_option_gives_the_patterns() {
    assert_eq!(
        patterns("grep -e .env -e .npmrc logs/x.log"),
        [".env", ".npmrc"]
    );
    assert_eq!(patterns("grep -ie .env logs/x.log"), [".env"]);
    assert_eq!(patterns("rg --regexp .env logs/"), [".env"]);
    assert_eq!(patterns("rg --regexp=.env logs/"), ["--regexp=.env"]);
    // The patterns come from a file: every operand is a path.
    assert!(patterns("grep -f patterns.txt .env").is_empty());
    assert!(patterns("rg --file=patterns.txt .env").is_empty());
    // `-e` after the operands (GNU grep permutes).
    assert_eq!(patterns("grep secret .env -e token"), ["token"]);
}

/// The paths a search reads stay paths: no false negative.
#[test]
fn searched_paths_are_not_patterns() {
    assert_eq!(patterns("grep secret .env; echo \"exit=$?\""), ["secret"]);
    assert_eq!(patterns("rg -n token ~/.npmrc"), ["token"]);
    assert!(patterns("rg --files .env").is_empty());
    assert!(patterns("cat .env; echo \"exit=$?\"").is_empty());
    assert!(patterns("echo grep .env").is_empty());
    // An option this does not know is taken as a flag: at worst its value
    // is taken for the pattern, never a path after the pattern.
    assert_eq!(patterns("grep --unknown 3 secret .env"), ["3"]);
    // A count is taken only when it is one (BSD `grep -C` has an optional
    // count).
    assert_eq!(patterns("grep -C token .env"), ["token"]);
    // BSD grep's `--context` takes an optional count.
    assert_eq!(patterns("grep --context 2 .env"), ["2"]);
    assert_eq!(patterns("grep &>> log token .env"), ["token"]);
    assert_eq!(patterns("grep 2>&1 token .env"), ["token"]);
    // GNU grep's abbreviations of `--regexp`.
    assert_eq!(patterns("grep --reg=token .env"), ["--reg=token"]);
    assert_eq!(patterns("grep --rege token .env"), ["token"]);
    // A pattern that runs a command names what that command opens.
    assert!(patterns("grep \"$(cat ~/.ssh/id_rsa)\" notes.md").is_empty());
    assert!(patterns("rg -e \"`cat .env`\" logs/").is_empty());
    // A keyword or wrapper only starts a program where it is one itself.
    assert!(patterns("cat do grep .env").is_empty());
    assert_eq!(patterns("sudo grep .env x.log"), [".env"]);
}

/// A search ends at a separator, also one after an option's value, and at a
/// new line of the script: a later command's words are not its patterns.
#[test]
fn a_search_ends_with_its_command() {
    assert_eq!(
        patterns("grep -e token; cat -e .env; echo done"),
        ["token;"]
    );
    assert_eq!(patterns("rg -g '*.md'; cat .env"), Vec::<String>::new());
    assert_eq!(
        patterns("rg -n DATABASE_URL src/\nnode -e \"require('fs').readFileSync('.env')\" || true"),
        ["DATABASE_URL"]
    );
    assert_eq!(
        patterns("rg token logs/\ncat -e .env\necho done"),
        ["token"]
    );
    assert_eq!(patterns("rg -e\ncat .env"), Vec::<String>::new());
    // A new line in quotes, or after `\\`, does not end it.
    assert_eq!(patterns("rg 'a\nb' logs/"), ["a\nb"]);
    assert_eq!(patterns("grep \\\n  -e .env logs/"), [".env"]);
    // A comment's quote does not hide the next line.
    assert_eq!(patterns("# don't\ngrep .env x.log"), [".env"]);
}

/// Where the operands are unclear, nothing is marked.
#[test]
fn unclear_operands_are_not_patterns() {
    assert!(patterns("grep <(cat .env) logs").is_empty());
    assert_eq!(patterns("rg --pre cat token .env"), ["token"]);
}
