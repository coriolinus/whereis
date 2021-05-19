use cargo_whereis::{where_is, Error};
use color_eyre::eyre::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Location of workspace `Cargo.toml`.
    #[structopt(long, parse(from_str))]
    manifest_path: Option<PathBuf>,

    /// Output relative to the current directory.
    ///
    /// Default: output is absolute.
    #[structopt(short, long, conflicts_with("url"))]
    relative: bool,

    /// Output a URL.
    ///
    /// This uses `file://` syntax for local crates, and links to the appropriate registry
    /// for remote crates.
    #[structopt(short, long)]
    url: bool,

    /// Output a URL instead of an error if the crate is an external dependency.
    ///
    /// The default behavior without `--url` is to emit a local filesystem path,
    /// and with `--url` it is to emit a URL. This behavior is consistent, for scripting,
    /// but for human use it can be convenient to accept either a path or a URL according to the
    /// nature of the dependency. This flag enables that behavior.
    #[structopt(short, long, conflicts_with("url"))]
    force: bool,

    /// Which crate to find.
    ///
    /// Note: this must be the official crate name, even if it has an alias.
    #[structopt(name = "CRATE")]
    package: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();
    let location = where_is(&opt.package, opt.manifest_path.as_ref())?;
    println!(
        "{}",
        location.show(opt.relative, opt.url).or_else(|err| {
            if opt.force && matches!(err, Error::RemoteButNotUrl) {
                location.show(opt.relative, true)
            } else {
                Err(err)
            }
        })?
    );
    Ok(())
}
