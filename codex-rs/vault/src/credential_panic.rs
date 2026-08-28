use std::cell::Cell;
use std::sync::Once;

thread_local! {
    static SCOPED_CALLBACK_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

/// Whether the current thread is inside a secret-bearing synchronous callback.
/// Host panic hooks must check this before formatting a payload or chaining to
/// another hook. This does not suppress ordinary panics or other threads.
pub fn scoped_credential_callback_active() -> bool {
    SCOPED_CALLBACK_ACTIVE.try_with(Cell::get).unwrap_or(false)
}

pub(crate) struct ScopedCredentialPanicGuard {
    previous: bool,
    // A guard must leave the scope on the same thread that entered it.
    _not_send: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl ScopedCredentialPanicGuard {
    pub(crate) fn enter() -> Self {
        // Install once before any guarded callback starts, never swap/restore
        // a process-global hook around individual (possibly concurrent) uses.
        static INSTALL: Once = Once::new();
        INSTALL.call_once(|| {
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                if !scoped_credential_callback_active() {
                    previous(info);
                }
            }));
        });
        Self {
            previous: SCOPED_CALLBACK_ACTIVE.replace(true),
            _not_send: std::marker::PhantomData,
        }
    }
}

impl Drop for ScopedCredentialPanicGuard {
    fn drop(&mut self) {
        SCOPED_CALLBACK_ACTIVE.set(self.previous);
    }
}

#[cfg(test)]
#[path = "credential_panic_tests.rs"]
mod tests;
