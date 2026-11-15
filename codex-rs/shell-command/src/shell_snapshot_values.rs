//! Corbanu #2314: capture every shell variable's raw value after startup.
//! Snapshot writers compare these values, not shell-quoted declarations, so
//! quoting, concatenation and embedded newlines cannot hide a credential.

use crate::shell_detect::ShellType;
use std::borrow::Cow;

/// Record that opens the value section. It is not a shell identifier, so it
/// cannot be mistaken for the first export record of a capture without values.
pub(super) const VALUES_MARKER: &[u8] = b"#values";

/// `#values NUL (name NUL value NUL)* NUL`. The subshell keeps the loop
/// variables out of later sections; a failed subshell fails the whole capture.
pub(super) fn script(shell_type: ShellType) -> String {
    let body = match shell_type {
        ShellType::Bash => {
            r#"(
  set +u
  while IFS= read -r __codex_snapshot_value_name; do
    case "$__codex_snapshot_value_name" in
      ""|[0-9]*|*[!A-Za-z0-9_]*|__codex_snapshot_*) continue ;;
    esac
    RECORD "${!__codex_snapshot_value_name}"
  done < <(compgen -v)
  printf '\0'
) || exit 1
"#
        }
        ShellType::Zsh => {
            r#"(
  unsetopt nounset 2>/dev/null
  for __codex_snapshot_value_name in "${(@f)$(builtin typeset +)}"; do
    __codex_snapshot_value_name=${__codex_snapshot_value_name##* }
    case "$__codex_snapshot_value_name" in
      ""|[0-9]*|*[!A-Za-z0-9_]*|__codex_snapshot_*) continue ;;
    esac
    # Shell-managed specials (commands, functions, history...) are large and
    # hold no startup credentials; exported ones such as PATH are still read.
    case "${(tP)__codex_snapshot_value_name}" in
      *special*) case "${(tP)__codex_snapshot_value_name}" in *export*) ;; *) continue ;; esac ;;
    esac
    # A zsh scalar can hold NULs, which would forge records. Fail the capture.
    [[ "${(P)__codex_snapshot_value_name}" == *$'\0'* ]] && exit 1
    RECORD "${(P)__codex_snapshot_value_name}"
  done
  printf '\0'
) || exit 1
"#
        }
        ShellType::Sh => {
            r#"(
  set +u
  set | __codex_snapshot_command sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' | while IFS= read -r __codex_snapshot_value_name; do
    case "$__codex_snapshot_value_name" in
      ""|[0-9]*|*[!A-Za-z0-9_]*|__codex_snapshot_*) continue ;;
    esac
    # Only the validated identifier enters eval.
    eval '__codex_snapshot_value=${'"$__codex_snapshot_value_name"'-}'
    RECORD "$__codex_snapshot_value"
  done
  printf '\0'
) || exit 1
"#
        }
        ShellType::PowerShell | ShellType::Cmd => return String::new(),
    };
    format!(
        "printf '%s\\0' '{}'\n{}",
        String::from_utf8_lossy(VALUES_MARKER),
        body.replace(
            "RECORD",
            "printf '%s\\0%s\\0' \"$__codex_snapshot_value_name\""
        )
    )
}

/// Decode the value section after its marker. Any malformed record rejects the capture.
pub(super) fn parse<'a>(remaining: &mut &'a [u8]) -> Option<Vec<(&'a str, Cow<'a, str>)>> {
    let mut values = Vec::new();
    loop {
        let name = super::capture::take_record(remaining)?;
        if name.is_empty() {
            return Some(values);
        }
        let name = std::str::from_utf8(name).ok()?;
        if !is_identifier(name) {
            return None;
        }
        let value = super::capture::take_record(remaining)?;
        values.push((name, String::from_utf8_lossy(value)));
    }
}

fn is_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().enumerate().all(|(index, byte)| {
            byte == b'_' || byte.is_ascii_alphabetic() || index > 0 && byte.is_ascii_digit()
        })
}

#[cfg(all(test, unix))]
#[path = "shell_snapshot_values_tests.rs"]
mod tests;
