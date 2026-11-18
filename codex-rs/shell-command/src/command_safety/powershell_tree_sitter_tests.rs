use pretty_assertions::assert_eq;
use serde::Deserialize;

use super::powershell_restrictive_prefixes;
use super::try_parse_powershell_commands;

#[derive(Debug, Deserialize)]
struct FixtureCase {
    name: String,
    script: String,
    expected: Option<Vec<Vec<String>>>,
}

#[test]
fn lowers_compact_literal_fixture() {
    // Supported outputs were captured from the PowerShell 7 AST subprocess parser. Rare forms
    // that need additional PowerShell-specific lowering are intentionally recorded as unsupported.
    let cases: Vec<FixtureCase> =
        serde_json::from_str(include_str!("fixtures/powershell_lowering.json"))
            .expect("valid PowerShell lowering fixture");

    for case in cases {
        assert_eq!(
            try_parse_powershell_commands(&case.script),
            case.expected,
            "fixture case: {}",
            case.name
        );
    }
}

#[test]
fn rejects_requires_directives() {
    assert_eq!(
        try_parse_powershell_commands("#requires -Modules Evil\nGet-Location"),
        None
    );
}

/// Restrictive prefixes include commands nested in another command's
/// arguments, see through `&` and `. `, and stop at `--%` (#2633 review).
#[test]
fn restrictive_prefixes_include_nested_commands() {
    let git_push = vec!["git".to_string(), "push".to_string()];
    for script in [
        "1..3 | ForEach-Object { git push }",
        "Invoke-Command { git push }",
        "& { git push }",
        "Write-Output (git push)",
        "Write-Output $(git push)",
        ". git push",
        "& git push $x",
        "git push --% %REMOTE% x",
    ] {
        let prefixes = powershell_restrictive_prefixes(script);
        assert!(prefixes.contains(&git_push), "{script}: {prefixes:?}");
    }
    assert_eq!(
        powershell_restrictive_prefixes("git push --% x"),
        vec![git_push]
    );
    assert_eq!(
        powershell_restrictive_prefixes(".\\tool.ps1 run $x"),
        vec![vec![".\\tool.ps1".to_string(), "run".to_string()]]
    );
}
