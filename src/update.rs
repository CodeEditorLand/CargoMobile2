use crate::util::{
    self,
    cli::{Report, TextWrapper},
    repo::{self, Repo},
};
use std::{
    fmt::{self, Display},
    fs::{self, File},
    io,
    path::PathBuf,
};

static ENABLED_FEATURES: &'static [&'static str] = &[
    #[cfg(feature = "android")]
    "android",
    #[cfg(feature = "apple")]
    "apple",
    #[cfg(feature = "brainium")]
    "brainium",
];

#[derive(Debug)]
pub enum Error {
    NoHomeDir(util::NoHomeDir),
    StatusFailed(repo::Error),
    MarkerCreateFailed { path: PathBuf, cause: io::Error },
    UpdateFailed(repo::Error),
    InstallFailed(bossy::Error),
    MarkerDeleteFailed { path: PathBuf, cause: io::Error },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHomeDir(err) => write!(f, "{}", err),
            Self::StatusFailed(err) => {
                write!(f, "Failed to check status of `cargo-mobile` repo: {}", err)
            }
            Self::MarkerCreateFailed { path, cause } => {
                write!(f, "Failed to create marker file at {:?}: {}", path, cause)
            }
            Self::UpdateFailed(err) => write!(f, "Failed to update `cargo-mobile` repo: {}", err),
            Self::InstallFailed(err) => write!(
                f,
                "Failed to install new version of `cargo-mobile`: {}",
                err
            ),
            Self::MarkerDeleteFailed { path, cause } => {
                write!(f, "Failed to delete marker file at {:?}: {}", path, cause)
            }
        }
    }
}

pub fn update(wrapper: &TextWrapper) -> Result<(), Error> {
    let repo = Repo::checkouts_dir("cargo-mobile").map_err(Error::NoHomeDir)?;
    let marker = repo
        .path()
        .parent()
        .expect("developer error: repo path had no parent")
        .parent()
        .expect("developer error: checkouts dir had no parent")
        .join(".updating");
    let marker_exists = marker.is_file();
    if marker_exists {
        log::info!("marker file present at {:?}", marker);
    } else {
        log::info!("no marker file present at {:?}", marker);
    }
    let msg = if marker_exists || repo.status().map_err(Error::StatusFailed)?.stale() {
        File::create(&marker).map_err(|cause| Error::MarkerCreateFailed {
            path: marker.to_owned(),
            cause,
        })?;
        repo.update("https://github.com/BrainiumLLC/cargo-mobile")
            .map_err(Error::UpdateFailed)?;
        println!("Installing updated `cargo-mobile`...");
        bossy::Command::impure_parse("cargo install --force --path")
            .with_arg(repo.path())
            .with_parsed_args("--no-default-features --features")
            .with_args(ENABLED_FEATURES)
            .run_and_wait()
            .map_err(Error::InstallFailed)?;
        fs::remove_file(&marker).map_err(|cause| Error::MarkerDeleteFailed {
            path: marker.to_owned(),
            cause,
        })?;
        log::info!("deleted marker file at {:?}", marker);
        "installed new version of `cargo-mobile`"
    } else {
        "`cargo-mobile` is already up-to-date"
    };
    let details = match repo.latest_message() {
        Ok(date) => format!("Contains commits up to {:?}", date),
        Err(err) => format!("But we failed to get the latest commit message: {}", err),
    };
    Report::victory(msg, details).print(wrapper);
    Ok(())
}
