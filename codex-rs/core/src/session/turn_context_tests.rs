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
        (0..CALLERS)
            .map(|_| {
                let state = Arc::clone(&state);
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    state.record_grammar_failure()
                })
            })
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
