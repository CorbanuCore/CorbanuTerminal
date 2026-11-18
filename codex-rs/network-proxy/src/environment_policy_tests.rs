use pretty_assertions::assert_eq;

use super::EnvironmentNetworkPolicy;
use crate::NetworkProxyConfig;
use crate::NetworkUnixSocketPermission::Allow;
use crate::NetworkUnixSocketPermission::Deny;
use crate::NetworkUnixSocketPermissions;

fn sockets(
    entries: &[(&str, crate::NetworkUnixSocketPermission)],
) -> Option<NetworkUnixSocketPermissions> {
    Some(NetworkUnixSocketPermissions {
        entries: entries
            .iter()
            .map(|(path, permission)| ((*path).to_string(), *permission))
            .collect(),
    })
}

fn owner(
    allow_all: bool,
    unix_sockets: Option<NetworkUnixSocketPermissions>,
) -> EnvironmentNetworkPolicy {
    EnvironmentNetworkPolicy::from_config(
        &NetworkProxyConfig {
            dangerously_allow_all_unix_sockets: Some(allow_all),
            unix_sockets,
            ..Default::default()
        },
        /*managed_allowed_domains_only*/ false,
    )
}

/// D714 (owner decision 2026-11-18, accepted as built): a controller that sets
/// neither `dangerously_allow_all_unix_sockets` nor `unix_sockets` inherits the
/// environment owner's explicit allow-all grant. Any explicit controller
/// restriction (false, a finite or empty socket map) and any deny entry on
/// either side still withhold it, and an owner without the grant gets none.
#[test]
fn d714_unset_controller_socket_setting_inherits_owner_allow_all() {
    let allow_all_owner = owner(/*allow_all*/ true, None);
    for (name, controller, owner, expected) in [
        (
            "controller unset",
            NetworkProxyConfig::default(),
            &allow_all_owner,
            true,
        ),
        (
            "controller explicit allow",
            NetworkProxyConfig {
                dangerously_allow_all_unix_sockets: Some(true),
                ..Default::default()
            },
            &allow_all_owner,
            true,
        ),
        (
            "controller explicit deny",
            NetworkProxyConfig {
                dangerously_allow_all_unix_sockets: Some(false),
                ..Default::default()
            },
            &allow_all_owner,
            false,
        ),
        (
            "controller empty socket map",
            NetworkProxyConfig {
                unix_sockets: sockets(&[]),
                ..Default::default()
            },
            &allow_all_owner,
            false,
        ),
        (
            "controller finite socket map",
            NetworkProxyConfig {
                unix_sockets: sockets(&[("/tmp/allowed.sock", Allow)]),
                ..Default::default()
            },
            &allow_all_owner,
            false,
        ),
        (
            "controller allow-all with a denied socket",
            NetworkProxyConfig {
                dangerously_allow_all_unix_sockets: Some(true),
                unix_sockets: sockets(&[("/tmp/denied.sock", Deny)]),
                ..Default::default()
            },
            &allow_all_owner,
            false,
        ),
        (
            "owner allow-all with a denied socket",
            NetworkProxyConfig::default(),
            &owner(
                /*allow_all*/ true,
                sockets(&[("/tmp/denied.sock", Deny)]),
            ),
            false,
        ),
        (
            "owner without allow-all",
            NetworkProxyConfig::default(),
            &owner(/*allow_all*/ false, None),
            false,
        ),
    ] {
        let mut config = controller;
        owner.apply_to(&mut config);
        assert_eq!(
            config.dangerously_allow_all_unix_sockets,
            Some(expected),
            "{name}"
        );
    }
}

/// D714: with the controller's socket settings unset, the owner's finite
/// socket grants and denials apply as given.
#[test]
fn d714_unset_controller_socket_setting_keeps_owner_socket_entries() {
    let mut config = NetworkProxyConfig::default();
    owner(
        /*allow_all*/ false,
        sockets(&[("/tmp/allowed.sock", Allow), ("/tmp/denied.sock", Deny)]),
    )
    .apply_to(&mut config);
    assert_eq!(
        (
            config.dangerously_allow_all_unix_sockets,
            config.unix_sockets
        ),
        (
            Some(false),
            sockets(&[("/tmp/allowed.sock", Allow), ("/tmp/denied.sock", Deny)])
        )
    );
}
