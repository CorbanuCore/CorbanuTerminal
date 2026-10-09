#[cfg(any(target_os = "macos", windows))]
use rama_tls_rustls::dep::pki_types::CertificateDer;
use rustls_native_certs::CertificateResult;
#[cfg(any(target_os = "macos", windows))]
use rustls_native_certs::Error;
#[cfg(any(target_os = "macos", windows))]
use rustls_native_certs::ErrorKind;

// `rustls_native_certs::load_native_certs()` first consults SSL_CERT_FILE and
// SSL_CERT_DIR. Load platform roots directly so a startup custom CA can be
// layered onto the managed bundle without replacing the platform trust store.
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn load_platform_native_certs() -> CertificateResult {
    let mut result =
        rustls_native_certs::load_certs_from_paths(platform_cert_file().as_deref(), None);
    for cert_dir in platform_cert_dirs() {
        extend_certificate_result(
            &mut result,
            rustls_native_certs::load_certs_from_paths(None, Some(&cert_dir)),
        );
    }
    dedupe_certs(&mut result);
    result
}

#[cfg(target_os = "macos")]
pub(crate) fn load_platform_native_certs() -> CertificateResult {
    use security_framework::trust_settings::Domain;
    use security_framework::trust_settings::TrustSettings;
    use security_framework::trust_settings::TrustSettingsForCertificate;
    use std::collections::BTreeMap;

    let mut result = CertificateResult::default();
    let mut all_certs = BTreeMap::new();
    for domain in &[Domain::User, Domain::Admin, Domain::System] {
        let ts = TrustSettings::new(*domain);
        let iter = match ts.iter() {
            Ok(iter) => iter,
            Err(err) => {
                result.errors.push(Error {
                    context: match domain {
                        Domain::User => "failed to load user trust settings",
                        Domain::Admin => "failed to load admin trust settings",
                        Domain::System => "failed to load system trust settings",
                    },
                    kind: ErrorKind::Os(err.into()),
                });
                continue;
            }
        };

        for cert in iter {
            let der = cert.to_der();
            let trusted = match ts.tls_trust_settings_for_certificate(&cert) {
                Ok(trusted) => trusted.unwrap_or(TrustSettingsForCertificate::TrustRoot),
                Err(err) => {
                    result.errors.push(Error {
                        context: "certificate not trusted",
                        kind: ErrorKind::Os(err.into()),
                    });
                    continue;
                }
            };
            all_certs.entry(der).or_insert(trusted);
        }
    }

    for (der, trusted) in all_certs {
        use TrustSettingsForCertificate::*;

        if let TrustRoot | TrustAsRoot = trusted {
            result.certs.push(CertificateDer::from(der));
        }
    }
    result
}

/// The current user's root store (which includes the machine's roots), opened
/// read-only: PF-27-S09 found that the credential broker's write-restricted
/// token cannot open it for writing, which `schannel`'s `open_current_user`
/// asks for, and the broker then trusted no root at all.
#[cfg(windows)]
pub(crate) fn load_platform_native_certs() -> CertificateResult {
    use windows_sys::Win32::Security::Cryptography::CERT_STORE_PROV_SYSTEM_W;
    use windows_sys::Win32::Security::Cryptography::CERT_STORE_READONLY_FLAG;
    use windows_sys::Win32::Security::Cryptography::CERT_SYSTEM_STORE_CURRENT_USER_ID;
    use windows_sys::Win32::Security::Cryptography::CERT_SYSTEM_STORE_LOCATION_SHIFT;
    use windows_sys::Win32::Security::Cryptography::CertCloseStore;
    use windows_sys::Win32::Security::Cryptography::CertEnumCertificatesInStore;
    use windows_sys::Win32::Security::Cryptography::CertOpenStore;
    use windows_sys::Win32::Security::Cryptography::CertVerifyTimeValidity;

    let mut result = CertificateResult::default();
    let name: Vec<u16> = "ROOT".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `name` is a NUL-terminated store name; the store is closed below.
    let store = unsafe {
        CertOpenStore(
            CERT_STORE_PROV_SYSTEM_W,
            0,
            0,
            (CERT_SYSTEM_STORE_CURRENT_USER_ID << CERT_SYSTEM_STORE_LOCATION_SHIFT)
                | CERT_STORE_READONLY_FLAG,
            name.as_ptr().cast(),
        )
    };
    if store.is_null() {
        result.errors.push(Error {
            context: "failed to open current user certificate store",
            kind: ErrorKind::Os(std::io::Error::last_os_error().into()),
        });
        return result;
    }
    let mut context = std::ptr::null();
    loop {
        // SAFETY: the store is open; passing the previous context frees it.
        context = unsafe { CertEnumCertificatesInStore(store, context) };
        if context.is_null() {
            break;
        }
        // SAFETY: a valid context from the enumeration above.
        let usable = match unsafe { server_auth_allowed(context) } {
            Ok(usable) => usable,
            Err(err) => {
                result.errors.push(Error {
                    context: "failed to inspect certificate valid uses",
                    kind: ErrorKind::Os(err.into()),
                });
                continue;
            }
        };
        // SAFETY: as above; a null time means now.
        let time_valid =
            unsafe { CertVerifyTimeValidity(std::ptr::null(), (*context).pCertInfo) } == 0;
        if usable && time_valid {
            // SAFETY: the encoded certificate is `cbCertEncoded` bytes long.
            let der = unsafe {
                std::slice::from_raw_parts(
                    (*context).pbCertEncoded,
                    (*context).cbCertEncoded as usize,
                )
            };
            result.certs.push(CertificateDer::from(der.to_vec()));
        }
    }
    // SAFETY: opened above; the enumeration released its last context.
    unsafe { CertCloseStore(store, 0) };
    result
}

/// Whether the certificate may be used for TLS server authentication: no
/// enhanced key usage restriction at all, or one that includes it (as
/// `schannel`'s `valid_uses` reported it).
#[cfg(windows)]
unsafe fn server_auth_allowed(
    context: *const windows_sys::Win32::Security::Cryptography::CERT_CONTEXT,
) -> std::io::Result<bool> {
    use windows_sys::Win32::Foundation::CRYPT_E_NOT_FOUND;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Security::Cryptography::CTL_USAGE;
    use windows_sys::Win32::Security::Cryptography::CertGetEnhancedKeyUsage;
    let mut len = 0_u32;
    // SAFETY: a size query on a valid context.
    if unsafe { CertGetEnhancedKeyUsage(context, 0, std::ptr::null_mut(), &mut len) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    // `u64` elements keep the buffer aligned for `CTL_USAGE`.
    let mut buffer = vec![0_u64; (len as usize).div_ceil(8)];
    let usage = buffer.as_mut_ptr().cast::<CTL_USAGE>();
    // SAFETY: `buffer` holds `len` bytes.
    if unsafe { CertGetEnhancedKeyUsage(context, 0, usage, &mut len) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: filled in above.
    let usage = unsafe { &*usage };
    if usage.cUsageIdentifier == 0 {
        // No usages: allowed for everything, unless the call reported that
        // the certificate is valid for no usage at all.
        // SAFETY: reads the thread's last error.
        return match unsafe { GetLastError() } as i32 {
            CRYPT_E_NOT_FOUND => Ok(true),
            0 => Ok(false),
            code => Err(std::io::Error::from_raw_os_error(code)),
        };
    }
    for index in 0..usage.cUsageIdentifier as usize {
        // SAFETY: `cUsageIdentifier` NUL-terminated OID strings.
        let oid =
            unsafe { std::ffi::CStr::from_ptr((*usage.rgpszUsageIdentifier.add(index)).cast()) };
        if oid.to_bytes() == PKIX_SERVER_AUTH.as_bytes() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(not(any(all(unix, not(target_os = "macos")), target_os = "macos", windows)))]
pub(crate) fn load_platform_native_certs() -> CertificateResult {
    rustls_native_certs::load_native_certs()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn extend_certificate_result(result: &mut CertificateResult, extra: CertificateResult) {
    result.certs.extend(extra.certs);
    result.errors.extend(extra.errors);
}

#[cfg(all(unix, not(target_os = "macos")))]
fn dedupe_certs(result: &mut CertificateResult) {
    result.certs.sort_unstable_by(|a, b| a.cmp(b));
    result.certs.dedup();
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_cert_file() -> Option<std::path::PathBuf> {
    PLATFORM_CERTIFICATE_FILE_NAMES
        .iter()
        .map(std::path::Path::new)
        .find(|path| path.exists())
        .map(std::path::Path::to_path_buf)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_cert_dirs() -> impl Iterator<Item = std::path::PathBuf> {
    PLATFORM_CERTIFICATE_DIRS
        .iter()
        .map(std::path::Path::new)
        .filter(|path| path.exists())
        .map(std::path::Path::to_path_buf)
}

#[cfg(all(unix, not(target_os = "macos"), target_os = "linux"))]
const PLATFORM_CERTIFICATE_DIRS: &[&str] = &[
    "/etc/ssl/certs",
    "/etc/pki/tls/certs",
    "/etc/security/certificates",
];

#[cfg(all(unix, not(target_os = "macos"), target_os = "freebsd"))]
const PLATFORM_CERTIFICATE_DIRS: &[&str] = &["/etc/ssl/certs", "/usr/local/share/certs"];

#[cfg(all(
    unix,
    not(target_os = "macos"),
    any(target_os = "illumos", target_os = "solaris")
))]
const PLATFORM_CERTIFICATE_DIRS: &[&str] = &["/etc/certs/CA"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "netbsd"))]
const PLATFORM_CERTIFICATE_DIRS: &[&str] = &["/etc/openssl/certs"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "aix"))]
const PLATFORM_CERTIFICATE_DIRS: &[&str] = &["/var/ssl/certs"];

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "solaris",
        target_os = "netbsd",
        target_os = "aix"
    ))
))]
const PLATFORM_CERTIFICATE_DIRS: &[&str] = &["/etc/ssl/certs"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "linux"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &[
    "/etc/ssl/certs/ca-certificates.crt",
    "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem",
    "/etc/pki/tls/certs/ca-bundle.crt",
    "/etc/ssl/ca-bundle.pem",
    "/etc/pki/tls/cacert.pem",
    "/etc/ssl/cert.pem",
    "/opt/etc/ssl/certs/ca-certificates.crt",
    "/etc/ssl/certs/cacert.pem",
];

#[cfg(all(unix, not(target_os = "macos"), target_os = "freebsd"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/usr/local/etc/ssl/cert.pem"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "dragonfly"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/usr/local/share/certs/ca-root-nss.crt"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "netbsd"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/etc/openssl/certs/ca-certificates.crt"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "openbsd"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/etc/ssl/cert.pem"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "solaris"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/etc/certs/ca-certificates.crt"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "illumos"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] =
    &["/etc/ssl/cacert.pem", "/etc/certs/ca-certificates.crt"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "android"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] =
    &["/data/data/com.termux/files/usr/etc/tls/cert.pem"];

#[cfg(all(unix, not(target_os = "macos"), target_os = "haiku"))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/boot/system/data/ssl/CARootCertificates.pem"];

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
        target_os = "illumos",
        target_os = "android",
        target_os = "haiku",
    ))
))]
const PLATFORM_CERTIFICATE_FILE_NAMES: &[&str] = &["/etc/ssl/certs/ca-certificates.crt"];

#[cfg(windows)]
const PKIX_SERVER_AUTH: &str = "1.3.6.1.5.5.7.3.1";

#[cfg(all(test, windows))]
mod pf_27_s09_tests {
    const CHILD_ENV: &str = "CODEX_PF27_S09_ROOTS_CHILD";
    const CHILD_TEST: &str = "native_certs::pf_27_s09_tests::pf_27_s09_roots_child_entry";

    /// Prints how many platform roots this process loads.
    #[test]
    #[expect(clippy::print_stdout, reason = "the parent test reads the count")]
    fn pf_27_s09_roots_child_entry() {
        if std::env::var_os(CHILD_ENV).is_none() {
            return;
        }
        let result = super::load_platform_native_certs();
        println!(
            "pf27s09-roots:{} errors:{}",
            result.certs.len(),
            result.errors.len()
        );
    }

    /// PF-27-S09: under the credential broker's token (low integrity,
    /// write-restricted) the platform roots still load, so the broker can
    /// verify the model providers it calls. Before, it opened the store for
    /// writing, was refused, and trusted no root. This process is the control.
    #[test]
    fn pf_27_s09_broker_token_loads_the_platform_roots() {
        use std::io::Read as _;
        let here = super::load_platform_native_certs();
        assert!(!here.certs.is_empty(), "control: {:?}", here.errors);
        let mut env: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
        env.push((CHILD_ENV.into(), "1".into()));
        let args: Vec<std::ffi::OsString> = [CHILD_TEST, "--exact", "--nocapture"]
            .into_iter()
            .map(Into::into)
            .collect();
        let (mut child, mut stdout) = codex_process_hardening::spawn_protected(
            &std::env::current_exe().expect("test binary"),
            &args,
            &env,
        )
        .expect("start under the broker token");
        let mut output = String::new();
        stdout.read_to_string(&mut output).expect("child output");
        assert!(child.wait().expect("child exit").success(), "{output}");
        let roots: usize = output
            .split("pf27s09-roots:")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|count| count.parse().ok())
            .unwrap_or_else(|| panic!("no count from the child: {output}"));
        assert_eq!(roots, here.certs.len(), "{output}");
    }
}
