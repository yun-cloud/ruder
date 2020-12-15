#[macro_use]
mod macros;

// pub mod extract;
pub mod github;

use std::io::{Read, Seek};
use std::path::Path;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use serde::Deserialize;
use tar::Archive;
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

pub fn extract<R: Read + Seek>(mut r: R, filename: &str, src: &str) -> anyhow::Result<Vec<u8>> {
    // TODO: handle src not exist
    let mut _size: usize = 0;
    let mut ret: Vec<u8> = vec![];
    if filename.ends_with(".tar.gz") {
        let mut ar = Archive::new(GzDecoder::new(r));
        for entry in ar.entries()? {
            let mut f = entry?;
            if f.path()? == Path::new(&src) {
                _size = f.read_to_end(&mut ret)?;
            }
        }
    } else if filename.ends_with(".tar") {
        let mut ar = Archive::new(r);
        for entry in ar.entries()? {
            let mut f = entry?;
            if f.path()? == Path::new(&src) {
                _size = f.read_to_end(&mut ret)?;
            }
        }
    } else if filename.ends_with(".zip") {
        let mut ar = ZipArchive::new(r)?;
        let mut f = ar.by_name(src)?;
        _size = f.read_to_end(&mut ret)?;
    } else {
        _size = r.read_to_end(&mut ret)?;
    }

    Ok(ret)
}
