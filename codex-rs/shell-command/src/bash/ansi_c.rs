/// Decode a literal Bash ANSI-C string for the forced-rm detector only.
/// This must not be used to prove that a command is safe.
pub(super) fn decode(raw: &str) -> Option<String> {
    let body = raw.strip_prefix("$'")?.strip_suffix('\'')?;
    let mut chars = body.chars().peekable();
    let mut decoded = String::new();
    while let Some(ch) = chars.next() {
        let value = if ch != '\\' {
            ch
        } else {
            match chars.next()? {
                'a' => '\u{7}',
                'b' => '\u{8}',
                'e' | 'E' => '\u{1b}',
                'f' => '\u{c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\u{b}',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                '?' => '?',
                'c' => {
                    let next = chars.next()?;
                    if !next.is_ascii() {
                        return None;
                    }
                    char::from(if next == '?' { 127 } else { (next as u8) & 31 })
                }
                first @ ('0'..='7' | 'x' | 'u' | 'U') => {
                    let (radix, remaining, mut value) = match first {
                        'x' => (16, 2, 0),
                        'u' => (16, 4, 0),
                        'U' => (16, 8, 0),
                        _ => (8, 2, first.to_digit(8)?),
                    };
                    let mut digits = 0;
                    while digits < remaining {
                        let Some(digit) = chars.peek().and_then(|ch| ch.to_digit(radix)) else {
                            break;
                        };
                        chars.next();
                        value = value.checked_mul(radix)?.checked_add(digit)?;
                        digits += 1;
                    }
                    if digits == 0 && matches!(first, 'x' | 'u' | 'U') {
                        decoded.push('\\');
                        first
                    } else if radix == 8 || first == 'x' {
                        char::from((value & 255) as u8)
                    } else {
                        char::from_u32(value)?
                    }
                }
                other => {
                    decoded.push('\\');
                    other
                }
            }
        };
        // Bash truncates this quoted segment at the first NUL.
        if value == '\0' {
            break;
        }
        decoded.push(value);
    }
    Some(decoded)
}
