use crate::CredentialStoreError;
use crate::prompt_budget::PromptBudget;
use std::sync::OnceLock;

// The keyring crate uses legacy file-based login keychains. SecItem's
// authentication-UI query flags do not cover that API. These Security.framework
// functions change UI eligibility for this process, not Keychain ACLs.
#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SecKeychainGetUserInteractionAllowed(state: *mut u8) -> i32;
    fn SecKeychainSetUserInteractionAllowed(state: u8) -> i32;
}

struct InteractionGuard {
    previous: u8,
}

impl InteractionGuard {
    fn suppress() -> Result<Self, CredentialStoreError> {
        let mut previous = 0;
        // SAFETY: Security.framework writes one Boolean into this valid pointer.
        let status = unsafe { SecKeychainGetUserInteractionAllowed(&mut previous) };
        check_status(status)?;
        // SAFETY: Boolean zero disables optional UI only for this process.
        check_status(unsafe { SecKeychainSetUserInteractionAllowed(0) })?;
        Ok(Self { previous })
    }
}

impl Drop for InteractionGuard {
    fn drop(&mut self) {
        // SAFETY: Restore the Boolean returned by Security.framework. The
        // caller retains the serialized operation permit through this drop,
        // including when the caller timed out or the callback panicked.
        let status = unsafe { SecKeychainSetUserInteractionAllowed(self.previous) };
        if status != 0 {
            tracing::warn!(status, "could not restore process Keychain UI state");
        }
    }
}

fn check_status(status: i32) -> Result<(), CredentialStoreError> {
    if status == 0 {
        Ok(())
    } else {
        Err(CredentialStoreError::from_message(format!(
            "could not bound Keychain permission prompts (OSStatus {status}); access was not attempted"
        )))
    }
}

/// Must run inside the default serialized OS-operation worker, never outside
/// its permit: the native UI setting is process-wide, not thread-local.
pub(super) fn run<T>(
    service: &str,
    account: &str,
    callback: impl FnOnce() -> Result<T, CredentialStoreError>,
) -> Result<T, CredentialStoreError> {
    static BUDGET: OnceLock<PromptBudget> = OnceLock::new();
    if BUDGET
        .get_or_init(PromptBudget::default)
        .take(service, account)
    {
        // Respect the existing native UI setting; never force interaction on.
        return callback();
    }
    let _guard = InteractionGuard::suppress()?;
    callback().map_err(|error| {
        CredentialStoreError::from_message(format!(
            "{error}; repeated Keychain prompts are suppressed for this credential. Unlock the login keychain, restart Corbanu, and choose Always Allow if you trust this build. No Keychain permissions were changed"
        ))
    })
}

#[cfg(test)]
#[path = "macos_interaction_tests.rs"]
mod tests;
