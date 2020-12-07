#[macro_use]
mod macros;

pub mod extract;
pub mod github;

use std::path::PathBuf;

use serde::Deserialize;

pub_fields! {
    #[derive(Debug, Deserialize)]
    struct Config {
        default: Option<DefaultTable>,
        binary: Option<Vec<BinaryTable>>,
    }
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
