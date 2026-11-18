//! #2548: here-documents the grammar reads as bash does, and their text.

use tree_sitter::Node;

/// The scanner only removes quotes reliably for simple whole-word quoting.
/// A mixed delimiter can swallow real outer commands into a fictitious body.
/// Reject the entire script, including non-shell here-documents, in that case.
pub(super) fn heredoc_boundaries_readable(root: Node<'_>, script: &str) -> bool {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        // Bash joins escaped newlines before recognizing an unquoted end
        // marker. The scanner can miss that end and swallow outer commands.
        // It also omits argv placed after a here-document start from commands.
        if (node.kind() == "heredoc_body"
            && node
                .utf8_text(script.as_bytes())
                .is_ok_and(|text| text.contains("\\\n")))
            || (node.kind() == "heredoc_redirect" && node.child_by_field_name("argument").is_some())
        {
            return false;
        }
        if node.kind() == "heredoc_start" {
            let Ok(text) = node.utf8_text(script.as_bytes()) else {
                return false;
            };
            let word = text
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .or_else(|| text.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                .or_else(|| text.strip_prefix('\\'))
                .unwrap_or(text);
            if word.is_empty()
                || !word
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
            {
                return false;
            }
            let Some(redirect) = node.parent() else {
                return false;
            };
            let mut cursor = redirect.walk();
            let Some(end) = redirect
                .named_children(&mut cursor)
                .find(|child| child.kind() == "heredoc_end")
            else {
                return false;
            };
            let before = script[..end.start_byte()].rsplit('\n').next().unwrap_or("");
            let after = script[end.end_byte()..].split('\n').next().unwrap_or("");
            let strip_tabs = script[redirect.start_byte()..node.start_byte()].contains("<<-");
            if end.utf8_text(script.as_bytes()).ok() != Some(word)
                || !after.is_empty()
                || !(before.is_empty() || (strip_tabs && before.bytes().all(|ch| ch == b'\t')))
            {
                return false;
            }
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    true
}

/// The text a here-document gives its command's input, where it is known
/// before the script runs: its delimiter is quoted, or its body has nothing
/// the shell expands.
pub(super) fn literal_heredoc(redirect: Node<'_>, script: &str) -> Option<String> {
    if redirect.kind() != "heredoc_redirect"
        || redirect.child_by_field_name("descriptor").is_some()
        || redirect.child_by_field_name("right").is_some()
        || redirect.child_by_field_name("argument").is_some()
    {
        return None;
    }
    let mut cursor = redirect.walk();
    let start = redirect
        .named_children(&mut cursor)
        .find(|n| n.kind() == "heredoc_start")?;
    let mut cursor = redirect.walk();
    let body = redirect
        .named_children(&mut cursor)
        .find(|n| n.kind() == "heredoc_body")?;
    let delimiter = start.utf8_text(script.as_bytes()).ok()?;
    let text = body.utf8_text(script.as_bytes()).ok()?;
    let quoted = delimiter.contains(['\'', '"', '\\']);
    // Unquoted here-documents perform expansions and backslash processing.
    // Do not mistake their source text for the bytes the shell will read.
    if !quoted && text.contains(['$', '`', '\\']) {
        return None;
    }
    let prefix = &script[redirect.start_byte()..start.start_byte()];
    Some(if prefix.contains("<<-") {
        text.split_inclusive('\n')
            .map(|line| line.trim_start_matches('\t'))
            .collect()
    } else {
        text.to_string()
    })
}
