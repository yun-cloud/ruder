#[macro_use]
mod macros;

pub mod github;

mod archive;
pub use archive::{Archive, Extract};

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use lazy_static::lazy_static;
use regex::Regex;
use semver::Version;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    default: Option<DefaultTable>,
    binary: Option<Vec<BinaryTable>>,
}

#[derive(Debug, Deserialize)]
pub struct DefaultTable {
    bin_dir: Option<String>,
    #[serde(default)]
    upgrade_policy: UpgradePolicy,
}

#[derive(Debug, Deserialize)]
pub struct BinaryTable {
    repo: String,
    asset_download_filename: String,
    src: String,
    dst: String,
}

impl Config {
    pub fn bin_dir(&self) -> PathBuf {
        self.default
            .as_ref()
            .map(|x| x.bin_dir.as_ref())
            .flatten()
            .map(|s| shellexpand::tilde(s).into_owned())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./bin"))
    }

    pub fn need_to_upgrade(&self, binary: &BinaryTable, latest_version: &Version) -> bool {
        let policy = self.upgrade_policy();
        let dst = self.bin_dir().join(binary.dst());
        let bin_status = binary_status(dst)
            .map_err(|e| log::warn!("binary_status() failed: {:?}", e))
            .unwrap_or(BinaryStatus::NotFound);
        log::info!("bin_status: {:?}", bin_status);

        policy.need_to_upgrade(&bin_status, latest_version)
    }

    fn upgrade_policy(&self) -> UpgradePolicy {
        // TODO: remove this, for debug usage now //
        self.default
            .as_ref()
            .map(|x| x.upgrade_policy)
            .unwrap_or_default()
    }
}

impl<'a> Config {
    pub fn binaries(&'a self) -> impl Iterator<Item = &BinaryTable> + 'a {
        self.binary.iter().flat_map(|v| v.iter())
    }
}

impl BinaryTable {
    pub fn src(&self, version: &Version) -> String {
        self.src.replace("{version}", &format!("{}", version))
    }

    pub fn asset_download_filename(&self, version: &Version) -> String {
        self.asset_download_filename
            .replace("{version}", &format!("{}", version))
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn dst(&self) -> &str {
        &self.dst
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
enum UpgradePolicy {
    #[serde(rename(deserialize = "always"))]
    Always,
    #[serde(rename(deserialize = "upgrade"))]
    Upgrade,
    #[serde(rename(deserialize = "skip_when_exist"))]
    SkipWhenExist,
}

impl Default for UpgradePolicy {
    fn default() -> Self {
        UpgradePolicy::Upgrade
    }
}

impl UpgradePolicy {
    fn need_to_upgrade(&self, bin_status: &BinaryStatus, latest_version: &Version) -> bool {
        use BinaryStatus::*;
        use UpgradePolicy::*;

        match self {
            Always => true,
            Upgrade => match bin_status {
                ExistWithVersion(version) => version < latest_version,
                Exist => {
                    eprintln!("Can not get version from binary, ugprade anyway");
                    true
                }
                _ => true,
            },
            SkipWhenExist => matches!(bin_status, NotFound),
        }
    }
}

fn binary_status<P: AsRef<Path>>(path: P) -> anyhow::Result<BinaryStatus> {
    lazy_static! {
        static ref VERSION_RE: Regex = Regex::new(r"(\d+).(\d+).(\d+)").unwrap();
    }
    fn inner(path: &Path) -> anyhow::Result<BinaryStatus> {
        if path.exists() {
            let output = Command::new(path).arg("--version").output()?;
            let stdout = String::from_utf8(output.stdout)?;
            let stderr = String::from_utf8(output.stderr)?;
            let version = vec![&stdout, &stderr]
                .into_iter()
                .filter_map(|name| VERSION_RE.find(&name))
                .map(|m| Version::parse(m.as_str()).unwrap())
                .next();
            match version {
                Some(version) => Ok(BinaryStatus::ExistWithVersion(version)),
                None => {
                    eprintln!("cannot get version from {:?}", path);
                    Ok(BinaryStatus::Exist)
                }
            }
        } else {
            Ok(BinaryStatus::NotFound)
        }
    }
    inner(path.as_ref())
}

#[derive(Debug)]
enum BinaryStatus {
    NotFound,
    Exist,
    ExistWithVersion(Version),
}
