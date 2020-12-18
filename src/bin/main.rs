use std::fs;
use std::fs::File;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use query_the_github_api::github::{create_github_client, query_latest_release};
use query_the_github_api::Archive;
use query_the_github_api::Config;

use anyhow::anyhow;
use anyhow::Context;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let config: Config =
        toml::from_slice(&fs::read("binary.toml").with_context(|| "Fail to read 'binary.toml'")?)
            .with_context(|| "Fail to deserialize 'binary.toml'")?;
    // info!("binary_data: {:#?}", config);

    let bin_dir = config.bin_dir();
    info!("bin_dir: {:?}", bin_dir);

    let client = create_github_client()
        .await
        .with_context(|| "Fail to create github client")?;

    for binary in config.binaries() {
        info!("===========================================================================");
        info!("binary: {:#?}", binary);

        let latest_release = query_latest_release(&client, binary.repo())
            .await
            .with_context(|| "Fail to query latest release")?;
        // info!("latest_release: {:#?}", latest_release);
        info!("latest_release.tag_name: {:?}", latest_release.tag_name);
        info!("latest_release.name: {:?}", latest_release.name);

        let version = match latest_release.version() {
            Err(err) => {
                eprintln!("Fail to get version of release, {}", err);
                continue;
            }
            Ok(version) => version,
        };

        let src = binary.src(&version);
        let asset_download_filename = PathBuf::from(&binary.asset_download_filename(&version));

        let (download_asset, download_filename) = latest_release
            .assets
            .iter()
            .filter_map(|asset| asset.download_filename().ok().map(|name| (asset, name)))
            .find(|(_, name)| name == &asset_download_filename)
            .ok_or_else(|| {
                anyhow!(
                    "{:?} is not exist in latest release of {}",
                    asset_download_filename,
                    binary.repo(),
                )
            })?;
        let data = download_asset
            .download(&client)
            .await
            .with_context(|| "Failed to download asset")?;

        let filename = download_filename
            .to_str()
            .expect("download_filename fail to convert to &str");
        let mut ar = Archive::new(data, filename).with_context(|| "Fail to create archive")?;
        let mut executable = ar
            .extract(&src)
            .with_context(|| "Fail to extract archive")?;

        fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Fail to create all dir for {:?}", bin_dir))?;
        let dst = bin_dir.join(binary.dst());
        let mut dst_f = File::create(&dst).with_context(|| "Fail to create destination file")?;
        io::copy(&mut executable, &mut dst_f)
            .with_context(|| "fail to copy download executable to destination file")?;

        let mut perms = fs::metadata(&dst)
            .with_context(|| "Fail to get metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dst, perms).with_context(|| "Fail to set permissions")?;
    }

    Ok(())
}
