use super::ModelEditProtocolState;
use super::PatchFallbackTransition;
use std::sync::Arc;
use std::sync::Barrier;

#[test]
fn successful_parse_resets_one_grammar_failure() {
    let state = ModelEditProtocolState::default();

    assert_eq!(
        state.record_grammar_failure(),
        PatchFallbackTransition::Unchanged {
            consecutive_failures: 1,
        }
    );
    state.record_parse_success();
    assert_eq!(
        state.record_grammar_failure(),
        PatchFallbackTransition::Unchanged {
            consecutive_failures: 1,
        }
    );
    assert!(!state.fallback_enabled());
}

#[test]
fn fallback_activation_is_absorbing_and_reported_once() {
    let state = ModelEditProtocolState::default();

    assert_eq!(
        state.record_grammar_failure(),
        PatchFallbackTransition::Unchanged {
            consecutive_failures: 1,
        }
    );
    assert_eq!(
        state.record_grammar_failure(),
        PatchFallbackTransition::Activated {
            consecutive_failures: 2,
        }
    );
    state.record_parse_success();
    assert_eq!(
        state.record_grammar_failure(),
        PatchFallbackTransition::Unchanged {
            consecutive_failures: 2,
        }
    );
    assert!(state.fallback_enabled());
}

#[test]
fn concurrent_failures_have_one_activation_transition() {
    const CALLERS: usize = 8;
    let state = Arc::new(ModelEditProtocolState::default());
    let barrier = Arc::new(Barrier::new(CALLERS));
    let transitions = std::thread::scope(|scope| {
        // Spawn every caller before joining any of them. Joining inside a lazy iterator would
        // block the first caller at the barrier before the remaining callers exist.
        let mut handles = Vec::with_capacity(CALLERS);
        for _ in 0..CALLERS {
            let state = Arc::clone(&state);
            let barrier = Arc::clone(&barrier);
            handles.push(scope.spawn(move || {
                barrier.wait();
                state.record_grammar_failure()
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("failure recorder should not panic"))
            .collect::<Vec<_>>()
    });

    assert_eq!(
        transitions
            .iter()
            .filter(|transition| matches!(transition, PatchFallbackTransition::Activated { .. }))
            .count(),
        1
    );
    assert!(state.fallback_enabled());
}
