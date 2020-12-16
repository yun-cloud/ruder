#[macro_use]
mod macros;

pub mod github;

use std::io::{self, Cursor, Read};
use std::path::Path;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use serde::Deserialize;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub struct Config {
    default: Option<DefaultTable>,
    binary: Option<Vec<BinaryTable>>,
}

pub_fields! {
    #[derive(Debug, Deserialize)]
    struct DefaultTable {
        tmp_dir: Option<String>,
        bin_dir: Option<String>,
    }
}

pub_fields! {
    #[derive(Debug, Deserialize)]
    struct BinaryTable {
        owner: String,
        repo: String,
        asset_download_filename: String,
        src: String,
        dst: String,
    }
}

impl Config {
    pub fn tmp_dir(&self) -> PathBuf {
        self.default
            .as_ref()
            .map(|x| x.tmp_dir.as_ref())
            .flatten()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./output"))
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.default
            .as_ref()
            .map(|x| x.bin_dir.as_ref())
            .flatten()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./bin"))
    }
}

impl<'a> Config {
    pub fn binaries(&'a self) -> impl Iterator<Item = &BinaryTable> + 'a {
        self.binary.iter().flat_map(|v| v.iter())
    }
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
