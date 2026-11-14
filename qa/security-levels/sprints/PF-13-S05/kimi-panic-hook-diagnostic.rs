use std::panic::{catch_unwind, set_hook};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    const MARKER: &str = "synthetic-pf13-panic-marker-not-a-credential";
    let observed = Arc::new(AtomicBool::new(false));
    let hook_observed = Arc::clone(&observed);
    set_hook(Box::new(move |info| {
        let contains_marker = info
            .payload()
            .downcast_ref::<String>()
            .is_some_and(|payload| payload.contains(MARKER));
        hook_observed.store(contains_marker, Ordering::SeqCst);
    }));
    let outcome = catch_unwind(|| panic!("synthetic callback: {}", MARKER));
    assert!(outcome.is_err());
    assert!(observed.load(Ordering::SeqCst));
    println!(
        "PASS diagnostic: panic was caught, but its hook already observed the synthetic marker"
    );
}
