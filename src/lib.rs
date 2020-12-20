#[macro_use]
mod macros;

pub mod github;

use std::io::{self, Cursor, Read};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use regex::Regex;
use semver::Version;
use serde::Deserialize;
use zip::ZipArchive;

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
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./bin"))
    }

    pub fn upgrade_policy(&self) -> UpgradePolicy {
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
pub enum UpgradePolicy {
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

pub fn binary_status<P: AsRef<Path>>(path: P) -> anyhow::Result<BinaryStatus> {
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
                None => Ok(BinaryStatus::Exist),
            }
        } else {
            Ok(BinaryStatus::NotFound)
        }
    }
    inner(path.as_ref())
}

#[derive(Debug)]
pub enum BinaryStatus {
    NotFound,
    Exist,
    ExistWithVersion(Version),
}

pub enum Archive<T: AsRef<[u8]>> {
    TarGz(tar::Archive<GzDecoder<Cursor<T>>>),
    Tar(tar::Archive<Cursor<T>>),
    Zip(ZipArchive<Cursor<T>>),
    Raw(Cursor<T>),
}

pub enum Extract<'a, T: AsRef<[u8]>> {
    TarGz(tar::Entry<'a, GzDecoder<Cursor<T>>>),
    Tar(tar::Entry<'a, Cursor<T>>),
    Zip(zip::read::ZipFile<'a>),
    Raw(&'a mut Cursor<T>),
}

impl<T: AsRef<[u8]>> Archive<T> {
    pub fn new(data: T, name: &str) -> anyhow::Result<Archive<T>> {
        use Archive::*;

        let cur = Cursor::new(data);
        if name.ends_with(".tar.gz") {
            Ok(TarGz(tar::Archive::new(GzDecoder::new(cur))))
        } else if name.ends_with(".tar") {
            Ok(Tar(tar::Archive::new(cur)))
        } else if name.ends_with(".zip") {
            Ok(Zip(ZipArchive::new(cur)?))
        } else {
            Ok(Raw(cur))
        }
    }

    pub fn extract<'a>(&'a mut self, src: &str) -> anyhow::Result<Extract<'a, T>> {
        use Archive::*;

        match self {
            TarGz(ar) => {
                for entry in ar.entries()? {
                    let f = entry?;
                    if f.path()? == Path::new(src) {
                        return Ok(Extract::TarGz(f));
                    }
                }
            }
            Tar(ar) => {
                for entry in ar.entries()? {
                    let f = entry?;
                    if f.path()? == Path::new(src) {
                        return Ok(Extract::Tar(f));
                    }
                }
            }
            Zip(ar) => {
                return Ok(Extract::Zip(ar.by_name(src)?));
            }
            Raw(cur) => {
                return Ok(Extract::Raw(cur));
            }
        }

        Err(anyhow::anyhow!("'{}' not found in extract()", src))
    }
}

impl<'a, T: AsRef<[u8]>> Read for Extract<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use Extract::*;

        match self {
            TarGz(r) => r.read(buf),
            Tar(r) => r.read(buf),
            Zip(r) => r.read(buf),
            Raw(r) => r.read(buf),
        }
    }
}
