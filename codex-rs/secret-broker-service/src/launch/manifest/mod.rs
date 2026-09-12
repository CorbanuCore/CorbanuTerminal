//! Fixed-path synthetic inspection only; no executable launch authority.
mod elf;
mod files;
mod schema;
mod sealed;
pub use sealed::SyntheticSealedImage;
pub use sealed::SyntheticProfileInspectedImage;

use std::fs::File;
use std::io;

use super::SyntheticLaunchRecipe;
use files::Kind;
use schema::Manifest;

const MANIFEST_LIMIT: u64 = 8192;
const IMAGE_LIMIT: u64 = 512 * 1024 * 1024;

fn denied(category: &'static str) -> io::Error {
    io::Error::other(category)
}

/// An inspection snapshot, deliberately without Clone, descriptor/path getters
/// or a command factory. The later supervisor must establish exec binding and
/// loader trust; a held descriptor does not make bytes immutable.
pub struct SyntheticManifestInspection {
    _image: File,
    _stamp: files::Stamp,
    _recipe: SyntheticLaunchRecipe,
    _digest: String,
}

impl SyntheticManifestInspection {
    /// Inspect the fixed system manifest as actual root, without creating,
    /// modifying, executing, enrolling or activating anything.
    pub fn inspect_system() -> io::Result<Self> {
        use nix::unistd::getresgid;
        use nix::unistd::getresuid;
        let u = getresuid().map_err(|_| denied("identity unavailable"))?;
        let g = getresgid().map_err(|_| denied("identity unavailable"))?;
        if [
            u.real.as_raw(),
            u.effective.as_raw(),
            u.saved.as_raw(),
            g.real.as_raw(),
            g.effective.as_raw(),
            g.saved.as_raw(),
        ] != [0; 6]
        {
            return Err(denied("root identity required"));
        }
        let root = File::open("/").map_err(|_| denied("root directory unavailable"))?;
        inspect_at(&root, (0, 0))
    }
}

// Private fixture seam; the only public factory fixes both path and principal.
fn inspect_at(root: &File, owner: (u32, u32)) -> io::Result<SyntheticManifestInspection> {
    files::check(root, Kind::Ancestor, owner)?;
    let etc = files::open(root, "etc", Kind::Ancestor, owner)?;
    let config = files::open(
        &etc,
        "corbanu-protected-test",
        Kind::Directory(0o700),
        owner,
    )?;
    let mut manifest = files::open(&config, "launch.json", Kind::Regular(0o600), owner)?;
    let before = files::check(&manifest, Kind::Regular(0o600), owner)?;
    let mut data = Vec::new();
    std::io::Read::take(&mut manifest, MANIFEST_LIMIT + 1)
        .read_to_end(&mut data)
        .map_err(|_| denied("manifest read failed"))?;
    files::unchanged(before, files::stamp(&manifest)?)?;
    let parsed = Manifest::parse(&data)?;
    let recipe = parsed.recipe()?;
    let opt = files::open(root, "opt", Kind::Ancestor, owner)?;
    let deployment = files::open(
        &opt,
        "corbanu-protected-test",
        Kind::Directory(0o755),
        owner,
    )?;
    let commit = files::open(
        &deployment,
        &parsed.source_commit,
        Kind::Directory(0o755),
        owner,
    )?;
    let mut image = files::open(
        &commit,
        "codex-protected-root-probe",
        Kind::Regular(0o755),
        owner,
    )?;
    let before = files::check(&image, Kind::Regular(0o755), owner)?;
    let size = image
        .metadata()
        .map_err(|_| denied("image metadata failed"))?
        .len();
    if size == 0 || size > IMAGE_LIMIT {
        return Err(denied("image size invalid"));
    }
    let digest = files::image_digest(&mut image, IMAGE_LIMIT)?;
    files::unchanged(before, files::stamp(&image)?)?;
    if digest != parsed.probe_sha256 {
        return Err(denied("image digest mismatch"));
    }
    Ok(SyntheticManifestInspection {
        _image: image,
        _stamp: before,
        _recipe: recipe,
        _digest: digest,
    })
}

use std::io::Read;

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
