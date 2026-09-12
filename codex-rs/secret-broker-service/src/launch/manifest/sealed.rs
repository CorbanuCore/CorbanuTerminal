use super::IMAGE_LIMIT;
use super::SyntheticLaunchRecipe;
use super::SyntheticManifestInspection;
use super::denied;
use super::files;
use rustix::fs::MemfdFlags;
use rustix::fs::SealFlags;
use rustix::fs::fcntl_add_seals;
use rustix::fs::fcntl_get_seals;
use rustix::fs::memfd_create;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::io::{self};

/// Immutable main-image bytes only, not loader trust or permission to execute.
/// No descriptor/command export; the future supervisor must bind actual exec.
pub struct SyntheticSealedImage {
    _image: File,
    _recipe: SyntheticLaunchRecipe,
}

impl SyntheticManifestInspection {
    /// Consume this inspection into bounded, kernel-sealed image storage.
    /// Never executes the image or changes installation or host policy.
    pub fn seal(self) -> io::Result<SyntheticSealedImage> {
        prepare(self, &mut KernelOps)
    }
}

fn required_seals() -> SealFlags {
    SealFlags::WRITE | SealFlags::SHRINK | SealFlags::GROW | SealFlags::SEAL
}

/// Private fault-injection seam. Only KernelOps is reachable from the public
/// inspection method; fixtures cannot supply an authority-bearing implementation.
trait Operations {
    fn source_stamp(&mut self, file: &File) -> io::Result<files::Stamp>;
    fn create(&mut self) -> io::Result<File>;
    fn rewind(&mut self, file: &mut File) -> io::Result<()>;
    fn copy(&mut self, source: &mut File, image: &mut File) -> io::Result<()>;
    fn seal(&mut self, file: &File) -> io::Result<SealFlags>;
    fn digest(&mut self, file: &mut File) -> io::Result<String>;
}

struct KernelOps;
impl Operations for KernelOps {
    fn source_stamp(&mut self, file: &File) -> io::Result<files::Stamp> {
        files::stamp(file)
    }
    fn create(&mut self) -> io::Result<File> {
        memfd_create(
            "corbanu-synthetic-image",
            MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING | MemfdFlags::EXEC,
        )
        .map(File::from)
        .map_err(|_| denied("sealed image creation failed"))
    }
    fn rewind(&mut self, file: &mut File) -> io::Result<()> {
        file.rewind().map_err(|_| denied("image rewind failed"))
    }
    fn copy(&mut self, source: &mut File, image: &mut File) -> io::Result<()> {
        copy_bounded(source, image, IMAGE_LIMIT)
    }
    fn seal(&mut self, file: &File) -> io::Result<SealFlags> {
        fcntl_add_seals(file, required_seals()).map_err(|_| denied("image sealing failed"))?;
        fcntl_get_seals(file).map_err(|_| denied("image seals unavailable"))
    }
    fn digest(&mut self, file: &mut File) -> io::Result<String> {
        files::image_digest(file, IMAGE_LIMIT)
    }
}

fn prepare(
    mut inspected: SyntheticManifestInspection,
    ops: &mut impl Operations,
) -> io::Result<SyntheticSealedImage> {
    files::unchanged(inspected._stamp, ops.source_stamp(&inspected._image)?)?;
    let mut image = ops.create()?;
    ops.rewind(&mut inspected._image)?;
    ops.copy(&mut inspected._image, &mut image)?;
    files::unchanged(inspected._stamp, ops.source_stamp(&inspected._image)?)?;
    if !ops.seal(&image)?.contains(required_seals()) {
        return Err(denied("image seals incomplete"));
    }
    ops.rewind(&mut image)?;
    if ops.digest(&mut image)? != inspected._digest {
        return Err(denied("sealed image digest mismatch"));
    }
    Ok(SyntheticSealedImage {
        _image: image,
        _recipe: inspected._recipe,
    })
}

fn copy_bounded(
    source: &mut impl Read,
    destination: &mut impl Write,
    limit: u64,
) -> io::Result<()> {
    let mut reader = source.take(limit + 1);
    let mut count = 0;
    let mut buffer = [0; 65536];
    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|_| denied("image copy read failed"))?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > limit {
            return Err(denied("image copy oversized"));
        }
        destination
            .write_all(&buffer[..n])
            .map_err(|_| denied("image copy write failed"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "sealed_tests.rs"]
mod tests;
