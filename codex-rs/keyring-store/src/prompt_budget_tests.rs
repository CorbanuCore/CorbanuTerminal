use super::PromptBudget;
use std::sync::Arc;
use std::thread;

#[test]
fn repeated_access_consumes_one_prompt_regardless_of_operation_result() {
    let budget = PromptBudget::default();
    assert!(budget.take("service", "account"));
    // Allow, deny, cancel, timeout and missing-item results do not reset it.
    for _ in 0..100 {
        assert!(!budget.take("service", "account"));
    }
}

#[test]
fn credentials_are_isolated_and_new_process_state_can_retry() {
    let budget = PromptBudget::default();
    assert!(budget.take("first", "account"));
    assert!(budget.take("second", "account"));
    assert!(budget.take("first", "other-account"));
    assert!(!budget.take("first", "account"));
    assert!(PromptBudget::default().take("first", "account"));
}

#[test]
fn concurrent_requests_receive_only_one_interactive_permit() {
    let budget = Arc::new(PromptBudget::default());
    let mut workers = Vec::new();
    // Start every request before joining; a lazy spawn/join iterator would
    // accidentally turn this into a sequential test.
    for _ in 0..32 {
        let budget = Arc::clone(&budget);
        workers.push(thread::spawn(move || {
            usize::from(budget.take("service", "account"))
        }));
    }
    let interactive: usize = workers.into_iter().map(|w| w.join().unwrap()).sum();
    assert_eq!(interactive, 1);
}
