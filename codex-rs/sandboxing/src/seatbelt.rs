use codex_network_proxy::ManagedNetworkSandboxContext;
use codex_network_proxy::NetworkProxy;
use codex_network_proxy::PROXY_URL_ENV_KEYS;
use codex_network_proxy::has_proxy_url_env_vars;
use codex_network_proxy::proxy_url_env_value;
use codex_protocol::permissions::CREDENTIAL_FILE_TEMPLATE_NAMES;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::permissions::PROTECTED_METADATA_PATH_NAMES;
use codex_protocol::permissions::credential_file_glob_root;
use codex_protocol::permissions::credential_glob_exempts_templates;
use codex_protocol::permissions::credential_rule_roots;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::WritableRoot;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use tracing::warn;
use url::Url;

const MACOS_SEATBELT_BASE_POLICY: &str = include_str!("seatbelt_base_policy.sbpl");
const MACOS_SEATBELT_NETWORK_POLICY: &str = include_str!("seatbelt_network_policy.sbpl");
const MACOS_RESTRICTED_READ_ONLY_PLATFORM_DEFAULTS: &str =
    include_str!("restricted_read_only_platform_defaults.sbpl");

/// The platform defaults' scratch folders, readable and writable for tools'
/// temporary files (`:minimal` reads).
const PLATFORM_TMP_FOLDERS: [&str; 4] = ["/tmp", "/private/tmp", "/var/tmp", "/private/var/tmp"];

/// Their reads and writes as the platform defaults always granted them, kept
/// for a policy without denied paths (Permissive's shape).
const PLATFORM_TMP_ACCESS: &str = r#"(allow file-read* (subpath "/tmp"))
(allow file-read* (subpath "/private/tmp"))
(allow file-read* (subpath "/var/tmp"))
(allow file-read* (subpath "/private/var/tmp"))
(allow file-write* (subpath "/tmp"))
(allow file-write* (subpath "/private/tmp"))
(allow file-write* (subpath "/var/tmp"))
(allow file-write* (subpath "/private/var/tmp"))"#;

/// #758: the macOS keychain services agent commands may not look up, denied
/// after every other section so neither the network rules (which allow
/// `com.apple.SecurityServer` for TLS, including the restricted-network
/// branch a managed proxy or an allowed Unix socket selects) nor the platform
/// defaults reopen them: the file-based keychains' `securityd`
/// (`com.apple.SecurityServer`), the data-protection keychain's `secd`
/// (`com.apple.securityd.xpc`) and the system data-protection keychain
/// (`com.apple.securityd.systemkeychain`). Certificate trust stays with
/// `trustd`, which is not denied.
pub(crate) const MACOS_KEYCHAIN_SERVICES_DENY_POLICY: &str = r#"; deny the keychain services (#758)
(deny mach-lookup
  (global-name "com.apple.SecurityServer")
  (global-name "com.apple.securityd.xpc")
  (global-name "com.apple.securityd.systemkeychain"))"#;

/// When working with `sandbox-exec`, only consider `sandbox-exec` in `/usr/bin`
/// to defend against an attacker trying to inject a malicious version on the
/// PATH. If /usr/bin/sandbox-exec has been tampered with, then the attacker
/// already has root access.
pub const MACOS_PATH_TO_SEATBELT_EXECUTABLE: &str = "/usr/bin/sandbox-exec";

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1"
}

fn proxy_scheme_default_port(scheme: &str) -> u16 {
    match scheme {
        "https" => 443,
        "socks5" | "socks5h" | "socks4" | "socks4a" => 1080,
        _ => 80,
    }
}

fn proxy_loopback_ports_from_env(env: &HashMap<String, String>) -> Vec<u16> {
    let mut ports = BTreeSet::new();
    for key in PROXY_URL_ENV_KEYS {
        let Some(proxy_url) = proxy_url_env_value(env, key) else {
            continue;
        };
        let trimmed = proxy_url.trim();
        if trimmed.is_empty() {
            continue;
        }

        let candidate = if trimmed.contains("://") {
            trimmed.to_string()
        } else {
            format!("http://{trimmed}")
        };
        let Ok(parsed) = Url::parse(&candidate) else {
            continue;
        };
        let Some(host) = parsed.host_str() else {
            continue;
        };
        if !is_loopback_host(host) {
            continue;
        }

        let scheme = parsed.scheme().to_ascii_lowercase();
        let port = parsed
            .port()
            .unwrap_or_else(|| proxy_scheme_default_port(scheme.as_str()));
        ports.insert(port);
    }
    ports.into_iter().collect()
}

#[derive(Debug, Default)]
struct ProxyPolicyInputs {
    ports: Vec<u16>,
    has_proxy_config: bool,
    allow_local_binding: bool,
    unix_domain_socket_policy: UnixDomainSocketPolicy,
}

#[derive(Debug, Clone)]
// Keep allow-all and allowlist modes disjoint so we don't carry ignored state.
enum UnixDomainSocketPolicy {
    AllowAll,
    Restricted { allowed: Vec<AbsolutePathBuf> },
}

impl Default for UnixDomainSocketPolicy {
    fn default() -> Self {
        Self::Restricted { allowed: vec![] }
    }
}

#[derive(Debug, Clone)]
struct UnixSocketPathParam {
    index: usize,
    path: AbsolutePathBuf,
}

fn proxy_policy_inputs(
    managed_network: Option<&ManagedNetworkSandboxContext>,
    network: Option<&NetworkProxy>,
    environment_id: Option<&str>,
    extra_allow_unix_sockets: &[AbsolutePathBuf],
) -> Result<ProxyPolicyInputs, String> {
    let extra_allowed = extra_allow_unix_sockets
        .iter()
        .filter_map(|socket_path| normalize_path_for_sandbox(socket_path.as_path()))
        .collect::<Vec<_>>();

    let unix_domain_socket_policy = match network {
        Some(network) if network.dangerously_allow_all_unix_sockets() => {
            UnixDomainSocketPolicy::AllowAll
        }
        Some(network) => {
            let mut allowed = network
                .allow_unix_sockets()
                .iter()
                .filter_map(|socket_path| {
                    match normalize_path_for_sandbox(Path::new(socket_path)) {
                        Some(path) => Some(path),
                        None => {
                            warn!(
                                "ignoring network.allow_unix_sockets entry because it could not be normalized: {socket_path}"
                            );
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();
            allowed.extend(extra_allowed);
            UnixDomainSocketPolicy::Restricted { allowed }
        }
        None => UnixDomainSocketPolicy::Restricted {
            allowed: extra_allowed,
        },
    };
    if let Some(managed_network) = managed_network {
        return Ok(ProxyPolicyInputs {
            ports: managed_network.loopback_ports.clone(),
            has_proxy_config: true,
            allow_local_binding: managed_network.allow_local_binding,
            unix_domain_socket_policy,
        });
    }
    match network {
        Some(network) => {
            let mut env = HashMap::new();
            network
                .apply_to_env_for_optional_environment(&mut env, environment_id)
                .map_err(|err| err.to_string())?;
            Ok(ProxyPolicyInputs {
                ports: proxy_loopback_ports_from_env(&env),
                has_proxy_config: has_proxy_url_env_vars(&env),
                allow_local_binding: network.allow_local_binding(),
                unix_domain_socket_policy,
            })
        }
        None => Ok(ProxyPolicyInputs {
            unix_domain_socket_policy,
            ..Default::default()
        }),
    }
}

fn normalize_path_for_sandbox(path: &Path) -> Option<AbsolutePathBuf> {
    // `AbsolutePathBuf::from_absolute_path()` normalizes relative paths against the current
    // working directory, so keep the explicit check to avoid silently accepting relative entries.
    if !path.is_absolute() {
        return None;
    }

    let absolute_path = AbsolutePathBuf::from_absolute_path(path).ok()?;
    let normalized_path = absolute_path
        .as_path()
        .canonicalize()
        .ok()
        .and_then(|canonical_path| AbsolutePathBuf::from_absolute_path(canonical_path).ok());
    normalized_path.or(Some(absolute_path))
}

fn unix_socket_path_params(proxy: &ProxyPolicyInputs) -> Vec<UnixSocketPathParam> {
    let mut deduped_paths: BTreeMap<String, AbsolutePathBuf> = BTreeMap::new();
    let UnixDomainSocketPolicy::Restricted { allowed } = &proxy.unix_domain_socket_policy else {
        return vec![];
    };
    for path in allowed {
        deduped_paths
            .entry(path.to_string_lossy().to_string())
            .or_insert_with(|| path.clone());
    }

    deduped_paths
        .into_values()
        .enumerate()
        .map(|(index, path)| UnixSocketPathParam { index, path })
        .collect()
}

fn unix_socket_path_param_key(index: usize) -> String {
    format!("UNIX_SOCKET_PATH_{index}")
}

fn unix_socket_dir_params(proxy: &ProxyPolicyInputs) -> Vec<(String, PathBuf)> {
    unix_socket_path_params(proxy)
        .into_iter()
        .map(|param| {
            (
                unix_socket_path_param_key(param.index),
                param.path.into_path_buf(),
            )
        })
        .collect()
}

/// Returns zero or more complete Seatbelt policy lines for unix socket rules.
/// When non-empty, the returned string is newline-terminated so callers can
/// append it directly to larger policy blocks.
fn unix_socket_policy(proxy: &ProxyPolicyInputs) -> String {
    let socket_params = unix_socket_path_params(proxy);
    let has_unix_socket_access = matches!(
        proxy.unix_domain_socket_policy,
        UnixDomainSocketPolicy::AllowAll
    ) || !socket_params.is_empty();
    if !has_unix_socket_access {
        return String::new();
    }

    let mut policy = String::new();
    policy.push_str("(allow system-socket (socket-domain AF_UNIX))\n");
    if matches!(
        proxy.unix_domain_socket_policy,
        UnixDomainSocketPolicy::AllowAll
    ) {
        // Keep AllowAll genuinely broad here; path qualifiers look narrower
        // without a clear macOS behavioral benefit.
        policy.push_str("(allow network-bind (local unix-socket))\n");
        policy.push_str("(allow network-outbound (remote unix-socket))\n");
        return policy;
    }

    for param in socket_params {
        let key = unix_socket_path_param_key(param.index);
        // Use subpath so allowlists cover sockets created beneath approved directories.
        policy.push_str(&format!(
            "(allow network-bind (local unix-socket (subpath (param \"{key}\"))))\n"
        ));
        policy.push_str(&format!(
            "(allow network-outbound (remote unix-socket (subpath (param \"{key}\"))))\n"
        ));
    }
    policy
}

#[cfg_attr(not(test), allow(dead_code))]
fn dynamic_network_policy(
    sandbox_policy: &SandboxPolicy,
    enforce_managed_network: bool,
    proxy: &ProxyPolicyInputs,
) -> String {
    dynamic_network_policy_for_network(
        NetworkSandboxPolicy::from(sandbox_policy),
        enforce_managed_network,
        proxy,
    )
}

fn dynamic_network_policy_for_network(
    network_policy: NetworkSandboxPolicy,
    enforce_managed_network: bool,
    proxy: &ProxyPolicyInputs,
) -> String {
    let has_some_unix_socket_access = match &proxy.unix_domain_socket_policy {
        UnixDomainSocketPolicy::AllowAll => true,
        UnixDomainSocketPolicy::Restricted { allowed } => !allowed.is_empty(),
    };
    let should_use_restricted_network_policy = !proxy.ports.is_empty()
        || proxy.has_proxy_config
        || enforce_managed_network
        || (!network_policy.is_enabled() && has_some_unix_socket_access);
    if should_use_restricted_network_policy {
        let mut policy = String::new();
        if proxy.allow_local_binding {
            policy.push_str("; allow local binding and loopback traffic\n");
            policy.push_str("(allow network-bind (local ip \"*:*\"))\n");
            policy.push_str("(allow network-inbound (local ip \"localhost:*\"))\n");
            policy.push_str("(allow network-outbound (remote ip \"localhost:*\"))\n");
        }
        if proxy.allow_local_binding && !proxy.ports.is_empty() {
            policy.push_str("; allow DNS lookups while application traffic remains proxy-routed\n");
            policy.push_str("(allow network-outbound (remote ip \"*:53\"))\n");
        }
        for port in &proxy.ports {
            policy.push_str(&format!(
                "(allow network-outbound (remote ip \"localhost:{port}\"))\n"
            ));
        }
        let unix_socket_policy = unix_socket_policy(proxy);
        if !unix_socket_policy.is_empty() {
            policy.push_str("; allow unix domain sockets for local IPC\n");
            policy.push_str(&unix_socket_policy);
        }
        return format!("{policy}{MACOS_SEATBELT_NETWORK_POLICY}");
    }

    if proxy.has_proxy_config {
        // Proxy configuration is present but we could not infer any valid loopback endpoints.
        // Fail closed to avoid silently widening network access in proxy-enforced sessions.
        return String::new();
    }

    if enforce_managed_network {
        // Managed network requirements are active but no usable proxy endpoints
        // are available. Fail closed for network access.
        return String::new();
    }

    if network_policy.is_enabled() {
        // No proxy env is configured: retain the existing full-network behavior.
        let mut policy = String::from("(allow network-outbound)\n(allow network-inbound)\n");
        let unix_socket_policy = unix_socket_policy(proxy);
        if !unix_socket_policy.is_empty() {
            policy.push_str("; allow unix domain sockets for local IPC\n");
            policy.push_str(&unix_socket_policy);
        }
        format!("{policy}{MACOS_SEATBELT_NETWORK_POLICY}")
    } else {
        String::new()
    }
}

fn root_absolute_path() -> AbsolutePathBuf {
    match AbsolutePathBuf::from_absolute_path(Path::new("/")) {
        Ok(path) => path,
        Err(err) => panic!("root path must be absolute: {err}"),
    }
}

#[derive(Debug, Clone)]
struct SeatbeltAccessRoot {
    root: AbsolutePathBuf,
    excluded_subpaths: Vec<AbsolutePathBuf>,
    protected_metadata_names: Vec<String>,
    /// #536: `root` is the real location pinned when the session first saw
    /// the writable root; it is emitted as is, never resolved again, so a
    /// link swapped in since cannot move the grant.
    pinned: bool,
}

/// A [`SeatbeltAccessRoot`] in the spellings Seatbelt matches, which is how
/// it is emitted: the root normalized, and each excluded path in every
/// spelling from [`exclusion_spellings`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct SeatbeltClause {
    root: AbsolutePathBuf,
    excluded: Vec<AbsolutePathBuf>,
    protected_metadata_names: Vec<String>,
}

impl SeatbeltClause {
    fn new(access_root: SeatbeltAccessRoot) -> Self {
        let root = if access_root.pinned {
            access_root.root
        } else {
            normalize_path_for_sandbox(access_root.root.as_path()).unwrap_or(access_root.root)
        };
        let mut excluded = Vec::new();
        for path in &access_root.excluded_subpaths {
            for spelling in exclusion_spellings(path) {
                if !excluded.contains(&spelling) {
                    excluded.push(spelling);
                }
            }
        }
        Self {
            root,
            excluded,
            protected_metadata_names: access_root.protected_metadata_names,
        }
    }

    /// Whether the clause allows `path`, spelled as Seatbelt sees it.
    /// Protected metadata names are not evaluated (read clauses have none).
    fn allows(&self, path: &Path) -> bool {
        path.starts_with(self.root.as_path())
            && !self
                .excluded
                .iter()
                .any(|excluded| path.starts_with(excluded.as_path()))
    }
}

/// The spellings Seatbelt may match for an excluded path: canonical (as
/// written when it cannot be resolved, such as a path not created yet), and
/// with its nearest existing folder resolved, which is how Seatbelt sees a
/// path not created yet once it is (`link/later.key` is checked as
/// `real/later.key`).
fn exclusion_spellings(path: &AbsolutePathBuf) -> Vec<AbsolutePathBuf> {
    let mut spellings =
        vec![normalize_path_for_sandbox(path.as_path()).unwrap_or_else(|| path.clone())];
    if let Some(resolved) = resolve_nearest_existing(path.as_path())
        .and_then(|resolved| AbsolutePathBuf::from_absolute_path(resolved).ok())
        && !spellings.contains(&resolved)
    {
        spellings.push(resolved);
    }
    spellings
}

/// Whether Seatbelt reaches `path` through the clause rooted at `root`: by
/// the spellings as written, or by the spellings Seatbelt matches. A path
/// reached through a link (`link/.env` for a root `real`, or `real/.env`
/// for a root `link`) needs its exclusion on that root too (#488).
fn reached_through_root(path: &AbsolutePathBuf, root: &AbsolutePathBuf) -> bool {
    if path.as_path().starts_with(root.as_path()) {
        return true;
    }
    let root = normalize_path_for_sandbox(root.as_path()).unwrap_or_else(|| root.clone());
    exclusion_spellings(path)
        .iter()
        .any(|spelling| spelling.as_path().starts_with(root.as_path()))
}

/// Whether `path` (another root's protected path) is the explicit writable
/// root `root` itself, which stays writable (#529): spelled alike once links
/// resolve (also not created yet), with no link in `root` below its
/// top-level folder. A root that
/// is (or passes through) such a link, like a writable link to a
/// workspace's `.git`, reopens nothing.
fn is_explicit_root(path: &AbsolutePathBuf, root: &AbsolutePathBuf) -> bool {
    if has_nested_link(root.as_path()) {
        return false;
    }
    let root_spellings = exclusion_spellings(root);
    path == root
        || exclusion_spellings(path)
            .iter()
            .any(|spelling| root_spellings.contains(spelling))
}

/// Whether a folder of `path` below its top-level one is a link (`/tmp` and
/// `/var` on macOS are top-level aliases).
fn has_nested_link(path: &Path) -> bool {
    path.ancestors()
        .filter(|ancestor| ancestor.parent().and_then(Path::parent).is_some())
        .any(|ancestor| {
            std::fs::symlink_metadata(ancestor)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        })
}

/// File-read access as the Seatbelt profile grants it.
enum SeatbeltReadAccess {
    /// `(allow file-read*)`.
    Full,
    Clauses(Vec<SeatbeltClause>),
}

/// The file-read clauses for `policy`. [`create_seatbelt_command_args`]
/// emits exactly these, and [`seatbelt_allows_file_read`] evaluates them.
fn seatbelt_read_access(policy: &FileSystemSandboxPolicy, cwd: &Path) -> SeatbeltReadAccess {
    let unreadable_roots = policy.get_unreadable_roots_with_cwd(cwd);
    let roots = if policy.has_full_disk_read_access() {
        if unreadable_roots.is_empty() {
            return SeatbeltReadAccess::Full;
        }
        vec![SeatbeltAccessRoot {
            pinned: false,
            root: root_absolute_path(),
            excluded_subpaths: unreadable_roots,
            protected_metadata_names: Vec::new(),
        }]
    } else {
        // Each denied path is excluded from every readable root that
        // reaches it, however either is spelled.
        policy
            .get_readable_roots_with_cwd(cwd)
            .into_iter()
            .map(|root| SeatbeltAccessRoot {
                pinned: false,
                excluded_subpaths: unreadable_roots
                    .iter()
                    .filter(|path| reached_through_root(path, &root))
                    .cloned()
                    .collect(),
                protected_metadata_names: Vec::new(),
                root,
            })
            .collect()
    };
    SeatbeltReadAccess::Clauses(roots.into_iter().map(SeatbeltClause::new).collect())
}

/// Whether the Seatbelt profile built for `policy` lets agent commands read
/// `path` (relative paths resolve against `cwd`). It evaluates the same
/// file-read clauses [`create_seatbelt_command_args`] emits, against the
/// spelling Seatbelt checks (links resolved), so a launch check built on it
/// cannot drift from the sandbox. It errs towards `true`: glob denials are
/// not evaluated, and with platform defaults every path counts as readable.
/// The base profile's terminal devices (`/dev/ptmx`, `/dev/ttys*`) are not
/// modeled.
pub fn seatbelt_allows_file_read(
    policy: &FileSystemSandboxPolicy,
    cwd: &Path,
    path: &Path,
) -> bool {
    if policy.include_platform_defaults() {
        return true;
    }
    let clauses = match seatbelt_read_access(policy, cwd) {
        SeatbeltReadAccess::Full => return true,
        SeatbeltReadAccess::Clauses(clauses) => clauses,
    };
    let path = cwd.join(path);
    let path = resolve_nearest_existing(&path).unwrap_or(path);
    clauses.iter().any(|clause| clause.allows(&path))
}

fn build_seatbelt_access_policy(
    action: &str,
    param_prefix: &str,
    roots: Vec<SeatbeltAccessRoot>,
) -> (String, Vec<(String, PathBuf)>) {
    build_seatbelt_clause_policy(
        action,
        param_prefix,
        roots.into_iter().map(SeatbeltClause::new).collect(),
    )
}

fn build_seatbelt_clause_policy(
    action: &str,
    param_prefix: &str,
    clauses: Vec<SeatbeltClause>,
) -> (String, Vec<(String, PathBuf)>) {
    let mut policy_components = Vec::new();
    let mut params = Vec::new();

    for (index, clause) in clauses.into_iter().enumerate() {
        let SeatbeltClause {
            root,
            excluded,
            protected_metadata_names,
        } = clause;
        let root_param = format!("{param_prefix}_{index}");
        params.push((root_param.clone(), root.clone().into_path_buf()));

        if excluded.is_empty() && protected_metadata_names.is_empty() {
            policy_components.push(format!("(subpath (param \"{root_param}\"))"));
            continue;
        }

        let mut require_parts = vec![format!("(subpath (param \"{root_param}\"))")];
        for (excluded_index, excluded_subpath) in excluded.into_iter().enumerate() {
            let excluded_param = format!("{param_prefix}_{index}_EXCLUDED_{excluded_index}");
            params.push((excluded_param.clone(), excluded_subpath.into_path_buf()));
            // Exclude both the exact protected path and anything beneath it.
            // `subpath` alone leaves a gap for first-time creation of the
            // protected directory itself, such as `mkdir .codex`.
            require_parts.push(format!(
                "(require-not (literal (param \"{excluded_param}\")))"
            ));
            require_parts.push(format!(
                "(require-not (subpath (param \"{excluded_param}\")))"
            ));
        }
        for metadata_name in protected_metadata_names {
            let regex =
                seatbelt_protected_metadata_name_regex(&root, &metadata_name).replace('"', "\\\"");
            require_parts.push(format!(r#"(require-not (regex #"{regex}"))"#));
        }
        policy_components.push(format!("(require-all {} )", require_parts.join(" ")));
    }

    if policy_components.is_empty() {
        (String::new(), Vec::new())
    } else {
        (
            format!("(allow {action}\n{}\n)", policy_components.join(" ")),
            params,
        )
    }
}

fn seatbelt_protected_metadata_name_regex(root: &AbsolutePathBuf, name: &str) -> String {
    let mut root = root.to_string_lossy().to_string();
    while root.len() > 1 && root.ends_with('/') {
        root.pop();
    }
    let root = regex_lite::escape(&root);
    let name = regex_lite::escape(name);
    if root == "/" {
        format!(r#"^/{name}(/.*)?$"#)
    } else {
        format!(r#"^{root}/{name}(/.*)?$"#)
    }
}

fn protected_metadata_names_for_writable_root(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    writable_root: &WritableRoot,
    cwd: &Path,
) -> Vec<String> {
    let mut names = writable_root.protected_metadata_names.clone();
    for name in PROTECTED_METADATA_PATH_NAMES {
        if names.iter().any(|existing| existing == name) {
            continue;
        }
        let path = writable_root.root.join(*name);
        if !file_system_sandbox_policy.can_write_path_with_cwd(path.as_path(), cwd) {
            names.push((*name).to_string());
        }
    }
    names
}

/// Spellings of `path` Seatbelt may see: as written, fully resolved, and
/// with its nearest existing folder resolved (a path that does not exist yet
/// under a symlinked folder such as `/tmp`).
fn sandbox_spellings(path: &Path) -> BTreeSet<PathBuf> {
    let mut spellings = BTreeSet::from([path.to_path_buf()]);
    if let Some(normalized) = normalize_path_for_sandbox(path) {
        spellings.insert(normalized.into_path_buf());
    }
    if let Some(resolved) = resolve_nearest_existing(path) {
        spellings.insert(resolved);
    }
    spellings
}

/// `path` with its nearest existing ancestor (itself, when it exists)
/// resolved: the spelling Seatbelt checks, including for a path not created
/// yet under a linked folder.
fn resolve_nearest_existing(path: &Path) -> Option<PathBuf> {
    let mut existing = path;
    let mut tail = Vec::new();
    while !existing.exists() {
        let (Some(parent), Some(name)) = (existing.parent(), existing.file_name()) else {
            break;
        };
        tail.push(name);
        existing = parent;
    }
    let mut resolved = existing.canonicalize().ok()?;
    for name in tail.iter().rev() {
        resolved.push(name);
    }
    Some(resolved)
}

/// Seatbelt rules match paths, not files: renaming a protected path, or any
/// folder above it, would move it out from under its rule (`mv ~/.config x`,
/// then write `x/fish/config.fish`, then move it back), and removing a
/// protected symlink would let a new file take its name. Deny unlinking
/// (which covers rename and remove) of every protected path and every folder
/// above it, in each spelling. Where part of the path does not exist yet,
/// deny creating its first missing component too, so a folder prepared
/// elsewhere cannot be renamed into place (`mv x ~/.config`); Linux mounts
/// the same component read-only. Creating and writing other files inside
/// those folders stays allowed.
fn build_seatbelt_rename_guard_policy(
    protected: Vec<AbsolutePathBuf>,
) -> (String, Vec<(String, PathBuf)>) {
    let mut guarded = BTreeSet::new();
    let mut uncreatable = BTreeSet::new();
    for path in protected {
        for spelling in sandbox_spellings(path.as_path()) {
            if let Some(missing) = spelling
                .ancestors()
                .filter(|ancestor| std::fs::symlink_metadata(ancestor).is_err())
                .last()
            {
                uncreatable.insert(missing.to_path_buf());
            }
            for ancestor in spelling.ancestors() {
                if ancestor.parent().is_none() {
                    break;
                }
                guarded.insert(ancestor.to_path_buf());
            }
        }
    }
    let mut params = Vec::new();
    let mut literals = |prefix: &str, paths: BTreeSet<PathBuf>| {
        paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                let key = format!("{prefix}_{index}");
                let literal = format!("(literal (param \"{key}\"))");
                params.push((key, path));
                literal
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let unlink = literals("RENAME_GUARD", guarded);
    let create = literals("CREATE_GUARD", uncreatable);
    let mut policy = Vec::new();
    if !unlink.is_empty() {
        policy.push(format!("(deny file-write-unlink {unlink})"));
    }
    if !create.is_empty() {
        policy.push(format!("(deny file-write* {create})"));
    }
    (policy.join("\n"), params)
}

/// #605/#649: the platform defaults' scratch-folder `action` (reads or
/// writes) once the policy denies reads: each folder stays usable except the
/// `protected` paths it holds (in every spelling). For writes, a workspace
/// below `/tmp` keeps its `.git`, `.codex` and `.agents` (and every other
/// read-only path) read-only, as its own writable root does; for reads, a
/// denied path below `/tmp` stays unreadable, as below a readable root.
/// Emitted with the policy's own clauses, ahead of the deny sections (glob
/// denials, unlistable folders, rename guard), which then hold there too.
fn platform_tmp_policy(
    action: &str,
    param_prefix: &str,
    protected: &[AbsolutePathBuf],
) -> (String, Vec<(String, PathBuf)>) {
    let mut clauses: Vec<SeatbeltClause> = Vec::new();
    for folder in PLATFORM_TMP_FOLDERS {
        let Ok(root) = AbsolutePathBuf::from_absolute_path(folder) else {
            continue;
        };
        let clause = SeatbeltClause::new(SeatbeltAccessRoot {
            excluded_subpaths: protected
                .iter()
                .filter(|path| reached_through_root(path, &root))
                .cloned()
                .collect(),
            protected_metadata_names: Vec::new(),
            pinned: false,
            root,
        });
        // `/tmp` and `/private/tmp` are one folder once normalized.
        if !clauses.iter().any(|existing| existing.root == clause.root) {
            clauses.push(clause);
        }
    }
    build_seatbelt_clause_policy(action, param_prefix, clauses)
}

/// Issue #506: deny listing each directory (reading its entries), in every
/// spelling Seatbelt may see. `file-read-data` on the directory itself only:
/// its metadata and every path below it keep the access the rest of the
/// policy gives them, so commands still run helpers kept below it.
fn build_seatbelt_unlistable_policy(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
) -> (String, Vec<(String, PathBuf)>) {
    let spellings = file_system_sandbox_policy
        .get_unlistable_directories()
        .into_iter()
        .flat_map(|directory| sandbox_spellings(directory.as_path()))
        .collect::<BTreeSet<_>>();
    let mut params = Vec::new();
    let literals = spellings
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let key = format!("UNLISTABLE_DIR_{index}");
            let literal = format!("(literal (param \"{key}\"))");
            params.push((key, path));
            literal
        })
        .collect::<Vec<_>>();
    if literals.is_empty() {
        return (String::new(), params);
    }
    (
        format!("(deny file-read-data {})", literals.join(" ")),
        params,
    )
}

fn build_seatbelt_unreadable_glob_policy(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    cwd: &Path,
) -> String {
    // Seatbelt does not understand the filesystem policy's glob syntax directly.
    // Convert each unreadable pattern into an anchored regex deny rule and apply
    // it to both reads and unlink-style writes so a denied path cannot be probed
    // through destructive filesystem operations.
    let unreadable_globs = file_system_sandbox_policy.get_unreadable_globs_with_cwd(cwd);
    if unreadable_globs.is_empty() {
        return String::new();
    }

    let rule_roots = credential_rule_roots(&unreadable_globs);
    let mut policy_components = Vec::new();
    for pattern in unreadable_globs {
        let exempts_templates = credential_glob_exempts_templates(&pattern, &rule_roots);
        let mut regexes = BTreeSet::new();
        if let Some(regex) = seatbelt_regex_for_unreadable_glob(&pattern) {
            regexes.insert(regex);
        }
        if let Some(pattern) = canonicalize_glob_static_prefix_for_sandbox(&pattern)
            && let Some(regex) = seatbelt_regex_for_unreadable_glob(&pattern)
        {
            regexes.insert(regex);
        }
        // #442: a credential file name is also never created, linked, renamed
        // to or written by an agent command (folders aside, such as a `.env`
        // virtualenv). Otherwise a command could plant a link or pipe under the
        // name and read what the person later writes through it.
        let credential_name = credential_file_glob_root(&pattern).is_some();
        if credential_name {
            regexes = regexes
                .into_iter()
                .map(|regex| credential_name_in_any_case(&regex))
                .collect();
        }
        for regex in regexes {
            let regex = regex.replace('"', "\\\"");
            if credential_name {
                // #442 (Travis, 2026-10-12): under the whole credential rule
                // the template names stay usable, except as a link or pipe
                // that a person's later `.env` could be renamed from.
                let matches = if exempts_templates {
                    let template = credential_template_regex();
                    policy_components.push(format!(
                        r#"(deny file-write-create (require-all (regex #"{regex}") (regex #"{template}") (require-any (vnode-type SYMLINK) (vnode-type FIFO))))"#
                    ));
                    format!(
                        r#"(require-all (regex #"{regex}") (require-not (regex #"{template}")))"#
                    )
                } else {
                    format!(r#"(regex #"{regex}")"#)
                };
                policy_components.push(format!("(deny file-read* {matches})"));
                policy_components.push(format!("(deny file-write-unlink {matches})"));
                policy_components.push(format!(
                    "(deny file-write* (require-all {matches} (require-not (vnode-type DIRECTORY))))"
                ));
                continue;
            }
            policy_components.push(format!(r#"(deny file-read* (regex #"{regex}"))"#));
            policy_components.push(format!(r#"(deny file-write-unlink (regex #"{regex}"))"#));
        }
    }

    policy_components.join("\n")
}

/// #442 (Travis, 2026-10-12): a path ending in a template name
/// ([`CREDENTIAL_FILE_TEMPLATE_NAMES`]), in any case.
fn credential_template_regex() -> String {
    let any_case = |text: &str| {
        text.chars()
            .map(|ch| {
                if ch.is_ascii_alphabetic() {
                    format!("[{}{}]", ch.to_ascii_lowercase(), ch.to_ascii_uppercase())
                } else {
                    regex_lite::escape(&ch.to_string())
                }
            })
            .collect::<String>()
    };
    let names = CREDENTIAL_FILE_TEMPLATE_NAMES
        .iter()
        .map(|name| any_case(name))
        .collect::<Vec<_>>()
        .join("|");
    format!("/({names})$")
}

/// #442: the regex of a credential-name glob with the name matching in any
/// case. Seatbelt compares the name as stored, and on a case-insensitive
/// volume a stored `.ENV` is where the person's `.env` writes go.
fn credential_name_in_any_case(regex: &str) -> String {
    const NAME: &str = r"\.env";
    let Some(start) = regex.rfind(NAME) else {
        return regex.to_string();
    };
    let (head, tail) = regex.split_at(start);
    let tail = &tail[NAME.len()..];
    let (rc, rest) = match tail.strip_prefix("rc") {
        Some(rest) => ("[rR][cC]", rest),
        None => ("", tail),
    };
    format!(r"{head}\.[eE][nN][vV]{rc}{rest}")
}

/// #651: `text` as a glob matching only itself: each metacharacter in a
/// one-character class (a lone `]` is literal already).
fn escape_glob_literal(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '*' | '?' | '[' | '{' | '}' => format!("[{ch}]"),
            other => other.to_string(),
        })
        .collect()
}

fn canonicalize_glob_static_prefix_for_sandbox(pattern: &str) -> Option<String> {
    let first_glob_index = pattern
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '*' | '?' | '[' | ']').then_some(index));
    let Some(first_glob_index) = first_glob_index else {
        return normalize_path_for_sandbox(Path::new(pattern))
            .map(|path| path.to_string_lossy().to_string());
    };

    let static_prefix = &pattern[..first_glob_index];
    let prefix_end = if static_prefix.ends_with('/') {
        static_prefix.len() - 1
    } else {
        static_prefix.rfind('/').unwrap_or(0)
    };
    if prefix_end == 0 {
        return None;
    }

    let root = normalize_path_for_sandbox(Path::new(&pattern[..prefix_end]))?;
    // #651: the resolved folder escaped, should it hold glob syntax (a
    // database folder reached through a link to `db[1]`).
    let root = escape_glob_literal(&root.to_string_lossy());
    let suffix = &pattern[prefix_end..];
    let normalized_pattern = format!("{root}{suffix}");
    (normalized_pattern != pattern).then_some(normalized_pattern)
}

fn seatbelt_regex_for_unreadable_glob(pattern: &str) -> Option<String> {
    if pattern.is_empty() {
        return None;
    }

    // Translate the supported git-style glob subset into a Seatbelt regex:
    // `*` and `?` stay within one path component, `**/` can consume zero or
    // more components, and closed character classes remain character classes.
    // A pattern with no glob metacharacters is treated as exact path plus subtree.
    let mut regex = String::from("^");
    let mut chars = pattern.chars().collect::<VecDeque<_>>();
    let mut saw_glob = false;

    while let Some(ch) = chars.pop_front() {
        match ch {
            '*' => {
                saw_glob = true;
                if chars.front() == Some(&'*') {
                    chars.pop_front();
                    if chars.front() == Some(&'/') {
                        chars.pop_front();
                        regex.push_str("(.*/)?");
                    } else {
                        regex.push_str(".*");
                    }
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => {
                saw_glob = true;
                regex.push_str("[^/]");
            }
            '[' => {
                saw_glob = true;
                let mut class = Vec::new();
                let mut closed = false;
                while let Some(class_ch) = chars.pop_front() {
                    if class_ch == ']' {
                        closed = true;
                        break;
                    }
                    class.push(class_ch);
                }
                if !closed {
                    regex.push_str("\\[");
                    for class_ch in class.into_iter().rev() {
                        chars.push_front(class_ch);
                    }
                    continue;
                }

                regex.push('[');
                let mut class_chars = class.into_iter();
                if let Some(first) = class_chars.next() {
                    match first {
                        '!' => regex.push('^'),
                        '^' => regex.push_str("\\^"),
                        _ => regex.push(first),
                    }
                }
                for class_ch in class_chars {
                    match class_ch {
                        '\\' => regex.push_str("\\\\"),
                        _ => regex.push(class_ch),
                    }
                }
                regex.push(']');
            }
            ']' => {
                saw_glob = true;
                regex.push_str("\\]");
            }
            _ => regex.push_str(&regex_lite::escape(&ch.to_string())),
        }
    }

    if !saw_glob {
        regex.push_str("(/.*)?");
    }
    regex.push('$');
    Some(regex)
}

#[cfg_attr(not(test), allow(dead_code))]
fn create_seatbelt_command_args_for_legacy_policy(
    command: Vec<String>,
    sandbox_policy: &SandboxPolicy,
    sandbox_policy_cwd: &Path,
    enforce_managed_network: bool,
    network: Option<&NetworkProxy>,
) -> Result<Vec<String>, String> {
    let file_system_sandbox_policy = FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(
        sandbox_policy,
        sandbox_policy_cwd,
    );
    create_seatbelt_command_args(CreateSeatbeltCommandArgsParams {
        command,
        file_system_sandbox_policy: &file_system_sandbox_policy,
        network_sandbox_policy: NetworkSandboxPolicy::from(sandbox_policy),
        sandbox_policy_cwd,
        enforce_managed_network,
        managed_network: None,
        environment_id: None,
        network,
        extra_allow_unix_sockets: &[],
    })
}

#[derive(Debug)]
pub struct CreateSeatbeltCommandArgsParams<'a> {
    pub command: Vec<String>,
    pub file_system_sandbox_policy: &'a FileSystemSandboxPolicy,
    pub network_sandbox_policy: NetworkSandboxPolicy,
    pub sandbox_policy_cwd: &'a Path,
    pub enforce_managed_network: bool,
    pub managed_network: Option<&'a ManagedNetworkSandboxContext>,
    pub environment_id: Option<&'a str>,
    pub network: Option<&'a NetworkProxy>,
    pub extra_allow_unix_sockets: &'a [AbsolutePathBuf],
}

pub fn create_seatbelt_command_args(
    args: CreateSeatbeltCommandArgsParams<'_>,
) -> Result<Vec<String>, String> {
    let CreateSeatbeltCommandArgsParams {
        command,
        file_system_sandbox_policy,
        network_sandbox_policy,
        sandbox_policy_cwd,
        enforce_managed_network,
        managed_network,
        environment_id,
        network,
        extra_allow_unix_sockets,
    } = args;

    let unreadable_roots =
        file_system_sandbox_policy.get_unreadable_roots_with_cwd(sandbox_policy_cwd);
    // Protected paths whose folders may not be renamed, once a policy denies
    // reads (a policy without denied paths keeps today's rules): every denied
    // path and every read-only path inside a writable root except the default
    // `.git`/`.codex`/`.agents` carve-outs at the root itself.
    let mut rename_guarded = unreadable_roots.clone();
    // #506: an unlistable directory renamed elsewhere would leave its rule.
    rename_guarded.extend(file_system_sandbox_policy.get_unlistable_directories());
    // #594/#598: a path the policy says may not be moved (the Corbanu home,
    // in each spelling: a link to it may not be removed either), also for a
    // command whose other rules a person's approval lifted.
    for path in file_system_sandbox_policy.get_unmovable_paths() {
        if !rename_guarded.contains(&path) {
            rename_guarded.push(path);
        }
    }
    let writable_roots = if file_system_sandbox_policy.has_full_disk_write_access() {
        Vec::new()
    } else {
        file_system_sandbox_policy.get_writable_roots_with_cwd(sandbox_policy_cwd)
    };
    // #529: each root's protected paths (read-only paths and protected
    // metadata names, also not created yet), once the policy denies reads.
    // A root not created yet protects nothing yet (Linux skips it too), so
    // it can still be created inside another root.
    let protected_by_root: Vec<Vec<AbsolutePathBuf>> = if unreadable_roots.is_empty() {
        Vec::new()
    } else {
        writable_roots
            .iter()
            .map(|root| {
                if !root.root.as_path().exists() {
                    return Vec::new();
                }
                let mut protected = root.read_only_subpaths.clone();
                for name in protected_metadata_names_for_writable_root(
                    file_system_sandbox_policy,
                    root,
                    sandbox_policy_cwd,
                ) {
                    let path = root.root.join(name);
                    if !protected.contains(&path) {
                        protected.push(path);
                    }
                }
                protected
            })
            .collect()
    };
    // The paths another writable root reaches: `(index of that root, path)`.
    let reached_from_other_roots: Vec<(usize, AbsolutePathBuf)> = writable_roots
        .iter()
        .enumerate()
        .flat_map(|(index, root)| {
            protected_by_root
                .iter()
                .enumerate()
                .filter(move |(owner, _)| *owner != index)
                .flat_map(|(_, protected)| protected.iter())
                .filter(|path| {
                    reached_through_root(path, &root.root) && !is_explicit_root(path, &root.root)
                })
                .map(move |path| (index, path.clone()))
        })
        .collect();
    if !unreadable_roots.is_empty() && !file_system_sandbox_policy.has_full_disk_write_access() {
        for root in &writable_roots {
            rename_guarded.extend(
                root.read_only_subpaths
                    .iter()
                    .filter(|subpath| {
                        !(subpath.as_path().parent() == Some(root.root.as_path())
                            && subpath
                                .as_path()
                                .file_name()
                                .and_then(|name| name.to_str())
                                .is_some_and(|name| PROTECTED_METADATA_PATH_NAMES.contains(&name)))
                    })
                    .cloned(),
            );
        }
        // #583: a writable root inside another root's read-only path (the
        // Corbanu home's `worktrees`) may not be renamed or removed either:
        // a link put in its place would make the next sandbox's root where
        // the link points. (Its protected names below already guard it; this
        // does not depend on them.)
        for root in &writable_roots {
            let inside_read_only = writable_roots.iter().any(|other| {
                other.root != root.root
                    && other.read_only_subpaths.iter().any(|read_only| {
                        root.root.as_path() != read_only.as_path()
                            && root.root.as_path().starts_with(read_only.as_path())
                    })
            });
            if inside_read_only && !rename_guarded.contains(&root.root) {
                rename_guarded.push(root.root.clone());
            }
        }
        // #529: a root's own `.git`/`.codex`/`.agents` may not be renamed
        // with it (nor created) through another root that holds it.
        for (_, path) in &reached_from_other_roots {
            if !rename_guarded.contains(path) {
                rename_guarded.push(path.clone());
            }
        }
    }
    let (rename_guard_policy, rename_guard_params) =
        build_seatbelt_rename_guard_policy(rename_guarded);
    // #605/#649: with `:minimal` reads, the scratch folders' reads and
    // writes; once the policy denies reads, without the paths protected
    // elsewhere (writes) or denied (reads).
    let include_platform_defaults = file_system_sandbox_policy.include_platform_defaults();
    let platform_tmp_unchanged = unreadable_roots.is_empty();
    let (platform_tmp_read_policy, platform_tmp_read_params) =
        if !include_platform_defaults || platform_tmp_unchanged {
            (String::new(), Vec::new())
        } else {
            platform_tmp_policy("file-read*", "PLATFORM_TMP_READ", &unreadable_roots)
        };
    let (platform_tmp_write_policy, platform_tmp_write_params) =
        if !include_platform_defaults || platform_tmp_unchanged {
            (String::new(), Vec::new())
        } else {
            let protected: Vec<AbsolutePathBuf> = protected_by_root
                .iter()
                .flatten()
                .cloned()
                .chain(file_system_sandbox_policy.get_read_only_paths_with_cwd(sandbox_policy_cwd))
                .chain(unreadable_roots.iter().cloned())
                .chain(file_system_sandbox_policy.get_unmovable_paths())
                .collect();
            platform_tmp_policy("file-write*", "PLATFORM_TMP", &protected)
        };
    let (file_write_policy, file_write_dir_params) =
        if file_system_sandbox_policy.has_full_disk_write_access() {
            if unreadable_roots.is_empty() {
                // Allegedly, this is more permissive than `(allow file-write*)`.
                (
                    r#"(allow file-write* (regex #"^/"))"#.to_string(),
                    Vec::new(),
                )
            } else {
                build_seatbelt_access_policy(
                    "file-write*",
                    "WRITABLE_ROOT",
                    vec![SeatbeltAccessRoot {
                        pinned: false,
                        root: root_absolute_path(),
                        excluded_subpaths: unreadable_roots,
                        protected_metadata_names: Vec::new(),
                    }],
                )
            }
        } else {
            // #529: Seatbelt allows a write any writable root allows, so a
            // protected path of one root stayed writable through another root
            // reaching it (a writable link to a workspace's `.git`, or `/tmp`
            // holding the workspace). Each root also excludes the other
            // roots' protected paths it reaches, as Linux does since #511,
            // except a path that is itself an explicit writable root. Like the
            // rename guard, only once the policy denies reads: a policy
            // without denied paths keeps today's rules.
            build_seatbelt_access_policy(
                "file-write*",
                "WRITABLE_ROOT",
                writable_roots
                    .into_iter()
                    .enumerate()
                    .map(|(index, root)| {
                        // Denied paths the root reaches only through a link
                        // are excluded too (#488); the rest already are.
                        let mut excluded_subpaths = root.read_only_subpaths.clone();
                        for (_, path) in reached_from_other_roots
                            .iter()
                            .filter(|(reaching, _)| *reaching == index)
                        {
                            if !excluded_subpaths.contains(path) {
                                excluded_subpaths.push(path.clone());
                            }
                        }
                        excluded_subpaths.extend(
                            unreadable_roots
                                .iter()
                                .filter(|path| reached_through_root(path, &root.root))
                                .cloned(),
                        );
                        // #536: grant at the real location the session
                        // pinned for this root, when it has seen it.
                        let pinned =
                            crate::writable_root_ledger::pinned_real_location(root.root.as_path())
                                .and_then(|real| AbsolutePathBuf::from_absolute_path(real).ok());
                        SeatbeltAccessRoot {
                            protected_metadata_names: protected_metadata_names_for_writable_root(
                                file_system_sandbox_policy,
                                &root,
                                sandbox_policy_cwd,
                            ),
                            pinned: pinned.is_some(),
                            root: pinned.unwrap_or(root.root),
                            excluded_subpaths,
                        }
                    })
                    .collect(),
            )
        };

    let (file_read_policy, file_read_dir_params) =
        match seatbelt_read_access(file_system_sandbox_policy, sandbox_policy_cwd) {
            SeatbeltReadAccess::Full => (
                "; allow read-only file operations\n(allow file-read*)".to_string(),
                Vec::new(),
            ),
            SeatbeltReadAccess::Clauses(clauses) => {
                let (policy, params) =
                    build_seatbelt_clause_policy("file-read*", "READABLE_ROOT", clauses);
                if policy.is_empty() {
                    (String::new(), params)
                } else {
                    (
                        format!("; allow read-only file operations\n{policy}"),
                        params,
                    )
                }
            }
        };

    let proxy = proxy_policy_inputs(
        managed_network,
        network,
        environment_id,
        extra_allow_unix_sockets,
    )?;
    let network_policy =
        dynamic_network_policy_for_network(network_sandbox_policy, enforce_managed_network, &proxy);

    let deny_read_policy =
        build_seatbelt_unreadable_glob_policy(file_system_sandbox_policy, sandbox_policy_cwd);
    let (unlistable_policy, unlistable_params) =
        build_seatbelt_unlistable_policy(file_system_sandbox_policy);
    let mut policy_sections = vec![
        MACOS_SEATBELT_BASE_POLICY.to_string(),
        file_read_policy,
        platform_tmp_read_policy,
        file_write_policy,
        platform_tmp_write_policy,
        deny_read_policy,
        unlistable_policy,
        rename_guard_policy,
        network_policy,
    ];
    if include_platform_defaults {
        policy_sections.push(MACOS_RESTRICTED_READ_ONLY_PLATFORM_DEFAULTS.to_string());
        // Only allows: placed here as before (#605/#649 emit them above when
        // the policy denies reads).
        if platform_tmp_unchanged {
            policy_sections.push(PLATFORM_TMP_ACCESS.to_string());
        }
    }
    // #758: last, so no allow above can reopen them.
    if file_system_sandbox_policy.denies_keychain_services() {
        policy_sections.push(MACOS_KEYCHAIN_SERVICES_DENY_POLICY.to_string());
    }

    let full_policy = policy_sections.join("\n");

    let dir_params = [
        file_read_dir_params,
        platform_tmp_read_params,
        file_write_dir_params,
        platform_tmp_write_params,
        rename_guard_params,
        unlistable_params,
        unix_socket_dir_params(&proxy),
    ]
    .concat();

    let mut seatbelt_args: Vec<String> = vec!["-p".to_string(), full_policy];
    let definition_args = dir_params
        .into_iter()
        .map(|(key, value): (String, PathBuf)| {
            format!("-D{key}={value}", value = value.to_string_lossy())
        });
    seatbelt_args.extend(definition_args);
    seatbelt_args.push("--".to_string());
    seatbelt_args.extend(command);
    Ok(seatbelt_args)
}

#[cfg(test)]
#[path = "seatbelt_tests.rs"]
mod tests;
