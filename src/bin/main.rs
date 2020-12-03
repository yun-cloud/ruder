use std::path::Path;

use query_the_github_api::extract::unpack_tar_gz;
use query_the_github_api::github::{create_github_client, query_latest_release};

use log::info;
use skip_error::skip_error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let owner = "BurntSushi";
    let repo = "ripgrep";
    let tmpdir = Path::new("./output");
    let asset_download_filename = Path::new("ripgrep-12.1.1-x86_64-unknown-linux-musl.tar.gz");
    info!("owner: {:?}", owner);
    info!("repo: {:?}", repo);
    info!("tmpdir: {:?}", tmpdir);
    info!("asset_download_filename: {:?}", asset_download_filename);

    let client = create_github_client().await?;
    let latest_release = query_latest_release(&client, owner, repo).await?;
    // info!("latest_release: {:#?}", latest_release);

    for asset in &latest_release.assets {
        let download_filename = skip_error!(asset.download_filename());
        /*
         * info!(
         *     "filepath: {:?}, {:?}, {:?}",
         *     download_filename,
         *     download_filename.file_stem(),
         *     download_filename.extension()
         * );
         */
        if download_filename == asset_download_filename {
            let filepath = asset.download(&client, tmpdir).await?;
            unpack_tar_gz(filepath, tmpdir)?;
            break;
        }
    }

    Ok(())
}
