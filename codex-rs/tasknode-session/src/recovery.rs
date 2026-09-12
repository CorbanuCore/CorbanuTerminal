//! Complete a profile's pending relink before reusing its previous credential.
use crate::*;
use reqwest::Method;
use std::fs::OpenOptions;
use std::path::Path;

/// Resolve authority for a TUI or CLI worker. Call outside an async runtime.
/// The per-profile file lock serializes one-time exchanges across processes.
pub fn resolve_scoped(
    codex_home: &Path,
    scope: &SessionScope,
    requested_origin: Option<&str>,
) -> Result<ActiveSession, String> {
    let locks = codex_home.join("tasknode").join("session-locks");
    std::fs::create_dir_all(&locks).map_err(|error| error.to_string())?;
    let lock_name = scope.label("default", "resolve").replace('/', "-");
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let lock = options
        .open(locks.join(lock_name))
        .map_err(|error| error.to_string())?;
    lock.lock().map_err(|error| error.to_string())?;
    resolve_from_store(
        &Vault::new(codex_home.to_path_buf()),
        scope,
        requested_origin,
        |client, path| {
            client
                .request_blocking(Method::GET, path, None)
                .map_err(|error| error.to_string())
        },
    )
}

fn resolve_from_store<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
    requested_origin: Option<&str>,
    request: impl Fn(&Client, &str) -> Result<Response, String>,
) -> Result<ActiveSession, String> {
    let state = load_scoped_from_store(store, scope).map_err(|error| error.to_string())?;
    let active = state.active.filter(|session| !session.is_expired());
    let Some(pending) = state.pending else {
        return active.ok_or_else(|| {
            "Task Node is not linked in this Corbanu profile. Run /tasknode link.".to_string()
        });
    };
    let origin = normalize_origin(&pending.origin).map_err(|error| error.to_string())?;
    if let Some(requested) = requested_origin {
        let requested = normalize_origin(requested).map_err(|error| error.to_string())?;
        if requested != origin {
            return Err(ClientError::OriginMismatch {
                saved: origin,
                requested,
            }
            .to_string());
        }
    }
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("requestId", &pending.request_id)
        .append_pair("pollToken", &pending.poll_token)
        .finish();
    let anonymous = Client::anonymous(&origin).map_err(|error| error.to_string())?;
    let response = request(&anonymous, &format!("/api/auth/terminal/session?{query}"))?;
    match response.status {
        200 if response.is_ok() => {
            let issued =
                serde_json::from_value::<TerminalSessionIssued>(response.body).map_err(|_| {
                    "Task Node returned an invalid linked-session response.".to_string()
                })?;
            let candidate = ActiveSession::from_issued(origin.clone(), issued);
            let client =
                Client::for_session(&candidate, &origin).map_err(|error| error.to_string())?;
            let status = request(&client, "/api/terminal/tasknode/status")?;
            if !status.is_ok() {
                return Err(format!(
                    "The new Task Node session failed validation; the previous session is preserved. {}",
                    status.message()
                ));
            }
            let confirmed_account = status
                .body
                .get("accountId")
                .and_then(serde_json::Value::as_str);
            if candidate.account_id.as_deref().filter(|id| !id.is_empty()) != confirmed_account
                || confirmed_account.is_none()
            {
                return Err("Task Node returned a different account during link validation; the previous session is preserved.".to_string());
            }
            // A newer user-started link must not be overwritten by a slow response.
            let current =
                load_scoped_from_store(store, scope).map_err(|error| error.to_string())?;
            if current.pending.as_ref() != Some(&pending) {
                return Err("The Task Node link changed while it was being checked. Run /tasknode status again.".to_string());
            }
            promote_active_scoped_to_store(store, scope, &candidate)
                .map_err(|error| error.to_string())?;
            Ok(candidate)
        }
        202 => {
            if let Some(active) = active {
                let client =
                    Client::for_session(&active, &origin).map_err(|error| error.to_string())?;
                if request(&client, "/api/terminal/tasknode/status")?.is_ok() {
                    return Ok(active);
                }
            }
            Err(format!(
                "Task Node linking is waiting for GitHub. Choose your account at {} and then run /tasknode status.",
                pending.verification_url
            ))
        }
        404 | 409 => {
            clear_pending_scoped_from_store(store, scope).map_err(|error| error.to_string())?;
            active.ok_or_else(|| {
                "The Task Node link expired. Run /tasknode link to start again.".to_string()
            })
        }
        _ => Err(response.message()),
    }
}

#[cfg(test)]
pub(crate) fn resolve_fixture<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
    requested_origin: Option<&str>,
    request: impl Fn(&Client, &str) -> Result<Response, String>,
) -> Result<ActiveSession, String> {
    resolve_from_store(store, scope, requested_origin, request)
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
