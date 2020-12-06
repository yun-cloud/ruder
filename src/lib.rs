#[macro_use]
mod macros;

pub mod extract;
pub mod github;

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
