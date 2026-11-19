//! Upstream's automatic thread title: after the first user message of a thread, the TUI asks
//! the session's model for a short title (`tui/src/app/thread_title.rs`). That is one more
//! Responses request per new thread, which fixtures that count or sequence requests must expect.

use core_test_support::responses;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::matchers::body_string_contains;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

/// The opening of upstream's title prompt (`thread_title_prompt`).
pub(crate) const THREAD_TITLE_PROMPT: &str = "Generate a concise, single-line task title";

/// Whether a Responses request body is the automatic title request.
pub(crate) fn is_thread_title_request(body: &[u8]) -> bool {
    String::from_utf8_lossy(body).contains(THREAD_TITLE_PROMPT)
}

/// Answers every title request ahead of the fixture's own sequenced responses.
pub(crate) async fn mount_thread_title_response(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path_regex(".*/responses$"))
        .and(body_string_contains(THREAD_TITLE_PROMPT))
        .respond_with(responses::sse_response(responses::sse(vec![
            responses::ev_response_created("thread-title-response"),
            responses::ev_assistant_message("thread-title-message", r#"{"title":"Fixture"}"#),
            responses::ev_completed("thread-title-response"),
        ])))
        .with_priority(1)
        .mount(server)
        .await;
}
