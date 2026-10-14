//! Throwaway (#636): proves the PR argument-comment lint covers test code.
//! Never merge.

use codex_utils_string::truncate_middle_chars;

#[test]
fn missing_param_comment_in_a_test() {
    assert_eq!(truncate_middle_chars("abc", 10), "abc");
}
