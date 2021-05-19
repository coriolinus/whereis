use lazy_static::lazy_static;
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};
use url::Url;

lazy_static! {
    static ref CRATES_IO: Url = Url::parse("https://crates.io/crates/").unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load cargo metadata")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("no such crate in this workspace: {0}")]
    NotFound(String),
    #[error("cannot represent a remote path without the --url flag")]
    RemoteButNotUrl,
    #[error("failed to compute a relative path from the current location")]
    ComputeRelative,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to compute a URL path from the local path {0}")]
    DirectoryUrl(String),
    #[error("dependency source not found")]
    NoSource,
    #[error("cannot form URL for registry: {0}")]
    UnknownRegistry(String),
    #[error(transparent)]
    Url(#[from] url::ParseError),
}

#[derive(Debug)]
pub enum Location {
    Local(PathBuf),
    Remote(Url),
}

impl Location {
    pub fn show(&self, relative: bool, as_url: bool) -> Result<String, Error> {
        match (self, as_url) {
            // normal local path, maybe relative
            (Self::Local(path), false) => {
                let path = if relative {
                    let path =
                        diff_paths(path, std::env::current_dir()?).ok_or(Error::ComputeRelative)?;
                    if path.as_os_str().is_empty() {
                        ".".into()
                    } else {
                        path
                    }
                } else {
                    path.to_owned()
                };
                Ok(path.display().to_string())
            }
            (Self::Local(path), true) => Ok(Url::from_directory_path(&path)
                .map_err(|_| Error::DirectoryUrl(path.display().to_string()))?
                .into()),
            (Self::Remote(_), false) => Err(Error::RemoteButNotUrl),
            (Self::Remote(url), true) => Ok(url.to_string()),
        }
    }
}

/// Determine where a particular dependency is located.
pub fn where_is(
    crate_name: &str,
    manifest_path: Option<impl AsRef<Path>>,
) -> Result<Location, Error> {
    let mut metadata_command = cargo_metadata::MetadataCommand::new();
    if let Some(path) = manifest_path {
        metadata_command.manifest_path(path.as_ref());
    }

    let metadata = metadata_command.exec()?;

    // first, attempt to locate the requested crate in the workspace
    if let Some(package) = metadata
        .workspace_members
        .iter()
        .map(|package_id| &metadata[package_id])
        .find(|package| package.name == crate_name)
    {
        return Ok(Location::Local(
            package
                .manifest_path
                .parent()
                .and_then(|path| path.canonicalize().ok())
                .unwrap_or_else(|| "/".into()),
        ));
    }

    // if it's not in the workspace, it still might be a dependency
    if let Some(package) = metadata
        .packages
        .iter()
        .find(|package| package.name == crate_name)
    {
        let source = package.source.as_ref().ok_or(Error::NoSource)?;
        return if source.is_crates_io() {
            Ok(Location::Remote(CRATES_IO.join(&crate_name)?))
        } else {
            Err(Error::UnknownRegistry(crate_name.to_string()))
        };
    }

    Err(Error::NotFound(crate_name.to_string()))
}
