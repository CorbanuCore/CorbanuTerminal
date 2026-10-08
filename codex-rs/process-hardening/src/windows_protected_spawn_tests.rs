use super::command_line;
use super::environment_block;
use pretty_assertions::assert_eq;
use std::ffi::OsString;
use std::path::Path;

fn text(wide: &[u16]) -> String {
    String::from_utf16_lossy(wide)
}

#[test]
fn pf_27_s07_command_line_quotes_like_the_msvc_runtime() {
    let args: Vec<OsString> = [
        "plain",
        "",
        "two words",
        r#"say "hi""#,
        r"trailing\",
        r"a\\b c",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    assert_eq!(
        text(&command_line(
            Path::new(r"C:\Program Files\corbanu.exe"),
            &args
        )),
        concat!(
            r#""C:\Program Files\corbanu.exe" plain "" "two words" "say \"hi\"" trailing\ "#,
            r#""a\\b c""#,
            "\0"
        )
    );
}

#[test]
fn pf_27_s07_environment_block_is_sorted_and_terminated() {
    let env = [("b", "2"), ("A", "1"), ("c", "")]
        .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    assert_eq!(
        text(&environment_block(&env).expect("block")),
        "A=1\0b=2\0c=\0\0"
    );
    assert_eq!(text(&environment_block(&[]).expect("empty")), "\0\0");
    let bad = [(OsString::from("A=B"), OsString::from("1"))];
    assert!(environment_block(&bad).is_err());
}
