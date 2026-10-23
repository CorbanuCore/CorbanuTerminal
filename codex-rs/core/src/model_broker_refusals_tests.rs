use super::*;
use codex_http_client::ModelBrokerOutcome;
use pretty_assertions::assert_eq;

fn frame(name: &'static str) -> HeaderValue {
    HeaderValue::from_static(name)
}

/// #878: a refusal is its owner's; another session's request the broker
/// serves does not clear it, and its own does.
#[test]
fn issue_878_only_the_owners_served_request_clears_its_refusal() {
    let refusals = Refusals::default();
    let (a, b) = (ThreadId::new(), ThreadId::new());
    refusals.refused(Some(a), Refusal::NoKey);
    refusals.signed(Some(b), frame("b-1"));
    refusals.answered(&frame("b-1"), ModelBrokerOutcome::Served);
    // The wallet or the model list: outside any session.
    refusals.signed(/*owner*/ None, frame("wallet"));
    refusals.answered(&frame("wallet"), ModelBrokerOutcome::Served);
    assert_eq!(
        refusals.reason(&[a]),
        Some(Refusal::NoKey.text(Scope::Session))
    );
    assert_eq!(refusals.reason(&[b]), None);

    refusals.signed(Some(a), frame("a-1"));
    refusals.answered(&frame("a-1"), ModelBrokerOutcome::Served);
    assert_eq!(refusals.reason(&[a]), None);
}

/// #878: the broker denying a signed request, or no broker to send it to,
/// is a refusal; a provider URL the broker could not reach is not.
#[test]
fn issue_878_the_brokers_own_denials_are_refusals() {
    let refusals = Refusals::default();
    let session = ThreadId::new();
    refusals.signed(Some(session), frame("denied"));
    refusals.answered(&frame("denied"), ModelBrokerOutcome::Denied("replay"));
    let denied = refusals.reason(&[session]).expect("denied");
    assert!(denied.contains("credential broker denied it"), "{denied}");

    refusals.signed(/*owner*/ None, frame("gone"));
    refusals.answered(&frame("gone"), ModelBrokerOutcome::NotRunning);
    assert_eq!(
        refusals.reason(&[ThreadId::new()]),
        Some(Refusal::BrokerUnavailable.text(Scope::Process))
    );
    // The session's own refusal comes first.
    assert_eq!(refusals.reason(&[session]), Some(denied));

    // A frame this broker did not sign changes nothing.
    refusals.answered(&frame("unknown"), ModelBrokerOutcome::Served);
    assert_eq!(refusals.reason(&[session]), Some(denied));
}

/// #878: answers apply in the order the requests were signed: a request
/// signed before a refusal does not clear it, and a request denied before a
/// later one was served does not bring a refusal back.
#[test]
fn issue_878_answers_follow_the_order_requests_were_signed() {
    let refusals = Refusals::default();
    let session = ThreadId::new();
    refusals.signed(Some(session), frame("early"));
    refusals.refused(Some(session), Refusal::Unbindable);
    refusals.answered(&frame("early"), ModelBrokerOutcome::Served);
    assert_eq!(
        refusals.reason(&[session]),
        Some(Refusal::Unbindable.text(Scope::Session))
    );

    refusals.signed(Some(session), frame("older"));
    refusals.signed(Some(session), frame("newer"));
    refusals.answered(&frame("newer"), ModelBrokerOutcome::Served);
    assert_eq!(refusals.reason(&[session]), None);
    refusals.answered(&frame("older"), ModelBrokerOutcome::Denied("replay"));
    assert_eq!(refusals.reason(&[session]), None);
}

/// #878: requests whose answer never comes are forgotten after a bound; a
/// late answer to one of them changes nothing.
#[test]
fn issue_878_unanswered_frames_are_bounded() {
    let refusals = Refusals::default();
    let session = ThreadId::new();
    refusals.signed(Some(session), frame("first"));
    for index in 0..MAX_PENDING {
        let value = HeaderValue::from_str(&format!("frame-{index}")).expect("frame");
        refusals.signed(Some(session), value);
    }
    assert_eq!(refusals.state().pending.len(), MAX_PENDING);
    refusals.answered(&frame("first"), ModelBrokerOutcome::Denied("replay"));
    assert_eq!(refusals.reason(&[session]), None);
}

/// #878: the inspector's texts are fixed and say whose request it was.
#[test]
fn issue_878_texts_name_the_owner_and_the_kind() {
    for refusal in [
        Refusal::Unbindable,
        Refusal::NoKey,
        Refusal::BrokerUnavailable,
        Refusal::Rejected,
        Refusal::Unsigned,
        Refusal::Denied,
    ] {
        assert!(
            refusal.text(Scope::Session).starts_with(
                "the last model request of this session or one of its agents was refused: "
            ),
            "{refusal:?}"
        );
        assert!(
            refusal.text(Scope::Process).starts_with(
                "the last brokered request outside a session (the model list or the wallet) was refused: "
            ),
            "{refusal:?}"
        );
    }
}

/// #878: refusals of sessions that ended do not pile up: past the bound the
/// oldest goes first, and the newest is kept.
#[test]
fn issue_878_refused_owners_are_bounded() {
    let refusals = Refusals::default();
    let first = ThreadId::new();
    refusals.refused(Some(first), Refusal::NoKey);
    let mut last = first;
    for _ in 0..MAX_REFUSED {
        last = ThreadId::new();
        refusals.refused(Some(last), Refusal::NoKey);
    }
    assert_eq!(refusals.state().refused.len(), MAX_REFUSED);
    assert_eq!(refusals.reason(&[first]), None);
    assert_eq!(
        refusals.reason(&[last]),
        Some(Refusal::NoKey.text(Scope::Session))
    );
}
