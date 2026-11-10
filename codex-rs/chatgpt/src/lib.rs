pub mod apply_command;
mod chatgpt_client;
pub mod connectors;
pub mod get_task;
pub mod workspace_settings;

// ci-1110c throwaway: a deliberate argument-comment-lint violation.
#[allow(dead_code)]
fn ci_1110c_lint_probe(retry_count: usize) -> usize {
    retry_count
}

#[allow(dead_code)]
fn ci_1110c_lint_probe_caller() -> usize {
    ci_1110c_lint_probe(/*wrong_name*/ 3)
}
