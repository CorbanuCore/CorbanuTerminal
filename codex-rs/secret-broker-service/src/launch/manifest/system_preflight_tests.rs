use super::super::files;
use super::super::tests::fixture;
use super::super::tests::image_path;
use super::super::tests::inspect;
use super::*;
use codex_protected_state::PolicyRootStore;
use codex_protected_state::SyntheticRootFixture as Roots;
use codex_security_audit::IntegrityRootStore;
use pretty_assertions::assert_eq;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn manifest_path(t: &tempfile::TempDir) -> std::path::PathBuf {
    t.path().join("etc/corbanu-protected-test/launch.json")
}
fn change(t: &tempfile::TempDir, pointer: &str, value: serde_json::Value) {
    let path = manifest_path(t);
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    *document.pointer_mut(pointer).unwrap() = value;
    fs::write(path, document.to_string()).unwrap();
}
fn valid_fixture() -> tempfile::TempDir {
    let t = fixture();
    let bytes = super::super::elf::tests::fixture();
    fs::write(t.path().join(image_path()), &bytes).unwrap();
    let digest = files::image_digest(&mut bytes.as_slice(), super::super::IMAGE_LIMIT).unwrap();
    change(&t, "/probe_sha256", serde_json::json!(digest));
    t
}
// A per-nextest-process descriptor count, including the iterator's own fd.
// No threads/children are created by these cases or by the preflight.
fn descriptors() -> usize {
    fs::read_dir("/proc/self/fd").unwrap().count()
}
fn no_children() {
    assert_eq!(
        rustix::process::waitid(
            rustix::process::WaitId::All,
            rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOHANG
        )
        .err(),
        Some(rustix::io::Errno::CHILD)
    );
}
struct FixtureInputs<'a> {
    tree: &'a tempfile::TempDir,
    roots: &'a [Arc<ControllerRoot>; 2],
    step: usize,
    fail: Option<usize>,
    between: Option<fn(&tempfile::TempDir)>,
}
impl<'a> FixtureInputs<'a> {
    fn new(tree: &'a tempfile::TempDir, roots: &'a [Arc<ControllerRoot>; 2]) -> Self {
        Self {
            tree,
            roots,
            step: 0,
            fail: None,
            between: None,
        }
    }
    fn stage(&mut self) -> io::Result<()> {
        self.step += 1;
        if self.fail == Some(self.step) {
            Err(denied("injected preparation failure"))
        } else {
            Ok(())
        }
    }
}
impl Inputs for FixtureInputs<'_> {
    fn inspect(&mut self) -> io::Result<SyntheticManifestInspection> {
        self.stage()?;
        if self.step == 2
            && let Some(change) = self.between
        {
            change(self.tree);
        }
        inspect(self.tree)
    }
    fn seal(&mut self, image: SyntheticManifestInspection) -> io::Result<SyntheticSealedImage> {
        self.stage()?;
        image.seal()
    }
    fn profile(
        &mut self,
        image: SyntheticSealedImage,
    ) -> io::Result<SyntheticProfileInspectedImage> {
        self.stage()?;
        image.inspect_static_profile()
    }
    fn root(&mut self, namespace: RootNamespace) -> io::Result<Arc<ControllerRoot>> {
        self.stage()?;
        Ok(Arc::clone(
            &self.roots[match namespace {
                RootNamespace::Journal => 0,
                RootNamespace::Policy => 1,
            }],
        ))
    }
}

#[test]
fn pf27_system_preflight_retains_complete_owner_and_reservation() {
    let tree = valid_fixture();
    let fixture = Roots::fresh().unwrap();
    let roots = fixture.roots();
    let baseline = descriptors();
    let before = roots.each_ref().map(Arc::strong_count);
    let mut input = FixtureInputs::new(&tree, &roots);
    let prepared = SystemPreflight::prepare(&mut input).unwrap();
    assert_eq!(input.step, 8);
    assert_eq!(descriptors(), baseline + 2);
    assert_eq!(
        roots.each_ref().map(Arc::strong_count),
        before.map(|n| n + 1)
    );
    assert_eq!(
        (roots[0].load(), roots[1].load_policy()),
        (Ok(None), Ok(None))
    );
    let mut busy = FixtureInputs::new(&tree, &roots);
    assert_eq!(
        SystemPreflight::prepare(&mut busy).err().unwrap().kind(),
        io::ErrorKind::WouldBlock
    );
    assert_eq!(busy.step, 0); // Reservation acquired before any image preparation.
    no_children();
    drop(prepared);
    assert_eq!(descriptors(), baseline);
    assert_eq!(roots.each_ref().map(Arc::strong_count), before);
    assert!(Reservation::acquire().is_ok());
}

#[test]
fn pf27_system_preflight_every_stage_failure_releases_inputs() {
    let tree = valid_fixture();
    let fixture = Roots::fresh().unwrap();
    let roots = fixture.roots();
    let baseline = descriptors();
    let before = roots.each_ref().map(Arc::strong_count);
    for stage in 1..=8 {
        let mut input = FixtureInputs::new(&tree, &roots);
        input.fail = Some(stage);
        assert!(SystemPreflight::prepare(&mut input).is_err());
        assert_eq!(input.step, stage);
        assert_eq!(descriptors(), baseline);
        assert_eq!(roots.each_ref().map(Arc::strong_count), before);
        assert_eq!(
            (roots[0].load(), roots[1].load_policy()),
            (Ok(None), Ok(None))
        );
        assert!(Reservation::acquire().is_ok());
    }
    no_children();
}

#[test]
fn pf27_system_preflight_same_bytes_different_recipe_and_source_deny() {
    let fixture = Roots::fresh().unwrap();
    let roots = fixture.roots();
    let mutations: [fn(&tempfile::TempDir); 5] = [
        |t| change(t, "/principals/journal/uid", serde_json::json!(2004)),
        |t| change(t, "/principals/policy/gid", serde_json::json!(3004)),
        |t| change(t, "/anchor_gid", serde_json::json!(4001)),
        |t| {
            let old = t.path().join(image_path());
            let new = t
                .path()
                .join(image_path().replace(&"a".repeat(40), &"b".repeat(40)));
            fs::create_dir(new.parent().unwrap()).unwrap();
            fs::set_permissions(new.parent().unwrap(), fs::Permissions::from_mode(0o755)).unwrap();
            fs::rename(old, new).unwrap();
            change(t, "/source_commit", serde_json::json!("b".repeat(40)));
        },
        |t| {
            let path = t.path().join(image_path());
            let replacement = t.path().join("replacement");
            fs::write(&replacement, fs::read(&path).unwrap()).unwrap();
            fs::set_permissions(&replacement, fs::Permissions::from_mode(0o755)).unwrap();
            fs::rename(replacement, path).unwrap();
        },
    ];
    for mutation in mutations {
        let tree = valid_fixture();
        let baseline = descriptors();
        let mut input = FixtureInputs::new(&tree, &roots);
        input.between = Some(mutation);
        assert_eq!(
            SystemPreflight::prepare(&mut input)
                .err()
                .unwrap()
                .to_string(),
            "inconsistent system preparation"
        );
        assert_eq!(input.step, 2); // Neither image has been consumed/sealed.
        assert_eq!(descriptors(), baseline);
        assert!(Reservation::acquire().is_ok());
    }
}

#[test]
fn pf27_system_preflight_actual_invalid_manifest_hash_profile_deny() {
    let fixture = Roots::fresh().unwrap();
    let roots = fixture.roots();
    for scenario in 0..3 {
        let tree = valid_fixture();
        match scenario {
            0 => fs::write(manifest_path(&tree), b"not json").unwrap(),
            1 => change(&tree, "/probe_sha256", serde_json::json!("0".repeat(64))),
            _ => {
                let mut bytes = super::super::elf::tests::fixture();
                bytes[68] = 7; // Hash-valid image with forbidden writable executable load.
                fs::write(tree.path().join(image_path()), &bytes).unwrap();
                let digest =
                    files::image_digest(&mut bytes.as_slice(), super::super::IMAGE_LIMIT).unwrap();
                change(&tree, "/probe_sha256", serde_json::json!(digest));
            }
        }
        let baseline = descriptors();
        let mut input = FixtureInputs::new(&tree, &roots);
        assert!(SystemPreflight::prepare(&mut input).is_err());
        assert_eq!(input.step, if scenario == 2 { 4 } else { 1 });
        assert_eq!(descriptors(), baseline);
        assert!(Reservation::acquire().is_ok());
    }
    no_children();
}

#[test]
fn pf27_system_preflight_real_non_root_entry_denies_without_side_effects() {
    assert!(!nix::unistd::getuid().is_root());
    assert!(!nix::unistd::geteuid().is_root());
    let baseline = descriptors();
    assert_eq!(
        SystemPreflight::inspect_system().err().unwrap().to_string(),
        "root identity required"
    );
    assert_eq!(descriptors(), baseline);
    assert!(Reservation::acquire().is_ok());
    no_children();
}
