use std::fs;
use std::path::{Path, PathBuf};

use typst::diag::{PackageError, PackageResult};
use typst::syntax::PackageSpec;

/// Make a package available in the on-disk cache.
pub fn prepare_package(spec: &PackageSpec) -> PackageResult<PathBuf> {
    let subdir = format!(
        "typst/packages/{}/{}/{}",
        spec.namespace, spec.name, spec.version
    );

    if let Some(data_dir) = dirs::data_dir() {
        let dir = data_dir.join(&subdir);
        if dir.exists() {
            return Ok(dir);
        }
    }

    if let Some(cache_dir) = dirs::cache_dir() {
        let dir = cache_dir.join(&subdir);

        // Download from network if it doesn't exist yet.
        if spec.namespace == "preview" && !dir.exists() {
            download_package(spec, &dir)?;
        }

        if dir.exists() {
            return Ok(dir);
        }
    }

    Err(PackageError::NotFound(spec.clone()))
}

/// Download a package over the network.
fn download_package(spec: &PackageSpec, package_dir: &Path) -> PackageResult<()> {
    // The `@preview` namespace is the only namespace that supports on-demand
    // fetching.
    assert_eq!(spec.namespace, "preview");

    let url = format!(
        "https://packages.typst.org/preview/{}-{}.tar.gz",
        spec.name, spec.version
    );

    let reader = match ureq::get(&url).call() {
        Ok(response) => response.into_reader(),
        Err(ureq::Error::Status(404, _)) => return Err(PackageError::NotFound(spec.clone())),
        Err(_) => return Err(PackageError::NetworkFailed),
    };

    let decompressed = flate2::read::GzDecoder::new(reader);
    tar::Archive::new(decompressed)
        .unpack(package_dir)
        .map_err(|_| {
            fs::remove_dir_all(package_dir).ok();
            PackageError::MalformedArchive
        })
}
