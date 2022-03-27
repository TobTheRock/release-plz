use crate::{tmp_repo::TempRepo, PackagePath, UpdateRequest, UpdateResult};
use anyhow::Context;
use cargo_edit::LocalManifest;
use cargo_metadata::{Package, Version};
use std::{fs, path::Path};

use tracing::{debug, instrument};

/// Update a local rust project
#[instrument]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, UpdateResult)>, TempRepo)> {
    let (packages_to_update, repository) = crate::next_versions(input)?;
    update_versions(&packages_to_update)?;
    update_changelogs(&packages_to_update)?;
    Ok((packages_to_update, repository))
}

#[instrument(skip_all)]
fn update_versions(local_packages: &[(Package, UpdateResult)]) -> anyhow::Result<()> {
    for (package, update) in local_packages {
        let package_path = package.package_path()?;
        set_version(package_path, &update.version);
    }
    Ok(())
}

#[instrument(skip_all)]
fn update_changelogs(local_packages: &[(Package, UpdateResult)]) -> anyhow::Result<()> {
    for (package, update) in local_packages {
        let changelog_path = package.changelog_path()?;
        let changelog = update.changelog.as_ref().unwrap();
        fs::write(&changelog_path, changelog)
            .with_context(|| format!("cannot write changelog to {:?}", &changelog_path))?;
    }
    Ok(())
}

#[instrument]
fn set_version(package_path: &Path, version: &Version) {
    debug!("updating version");
    let mut local_manifest =
        LocalManifest::try_new(&package_path.join("Cargo.toml")).expect("cannot read manifest");
    local_manifest.set_package_version(version);
    local_manifest.write().expect("cannot update manifest");
}
