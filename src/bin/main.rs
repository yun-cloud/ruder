use std::fs;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use query_the_github_api::extract;
use query_the_github_api::github::{create_github_client, query_latest_release};
use query_the_github_api::Config;

use anyhow::anyhow;
use anyhow::Context;
use log::info;
use regex::Regex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let config: Config =
        toml::from_slice(&fs::read("binary.toml").with_context(|| "Fail to read 'binary.toml'")?)
            .with_context(|| "Fail to deserialize 'binary.toml'")?;
    // info!("binary_data: {:#?}", config);

    let tmp_dir = config.tmp_dir();
    let bin_dir = config.bin_dir();
    info!("tmp_dir: {:?}", tmp_dir);
    info!("bin_dir: {:?}", bin_dir);

    let client = create_github_client()
        .await
        .with_context(|| "Fail to create github client")?;
    for binary in config.binaries() {
        info!("===========================================================================");
        info!("binary: {:#?}", binary);

        let latest_release = query_latest_release(&client, &binary.owner, &binary.repo)
            .await
            .with_context(|| "Fail to query latest release")?;
        // info!("latest_release: {:#?}", latest_release);
        info!("latest_release.tag_name: {:?}", latest_release.tag_name);
        info!("latest_release.name: {:?}", latest_release.name);

        let version_re = Regex::new(r"(\d+).(\d+).(\d+)")?;
        let version = vec![&latest_release.tag_name, &latest_release.name]
            .into_iter()
            .filter_map(|name| version_re.find(&name))
            .next()
            .map(|m| m.as_str().to_owned());
        // warn!("version: {:?}", version);
        if version.is_none() {
            eprintln!("cannot find version from asset tag_name or name");
            continue;
        }
        let version = version.unwrap();

        let asset_download_filename = PathBuf::from(
            binary
                .asset_download_filename
                .replace("{version}", &version),
        );
        // warn!("asset_download_filename: {:?}", asset_download_filename);
        let src = binary.src.replace("{version}", &version);
        // warn!("src: {:?}", src);

        let (download_asset, download_filename) = latest_release
            .assets
            .iter()
            .filter_map(|asset| asset.download_filename().ok().map(|name| (asset, name)))
            .find(|(_, name)| name == &asset_download_filename)
            .ok_or_else(|| {
                anyhow!(
                    "{:?} is not exist in latest release of {}/{}",
                    asset_download_filename,
                    binary.repo,
                    binary.owner
                )
            })?;

        let executable = {
            let data = download_asset.download(&client).await?;

            let mut extracted: Vec<u8> = vec![];
            let filename = download_filename
                .to_str()
                .expect("download_filename failed to convert to &str");
            let _size = extract(Cursor::new(data), filename, &src, &mut extracted)?;
            // warn!("extract _size: {:?}", _size);
            extracted
        };

        let dst = bin_dir.join(&binary.dst);
        fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Fail to create all dir for {:?}", bin_dir))?;
        let mut dst_f = File::create(&dst)?;

        io::copy(&mut Cursor::new(executable), &mut dst_f)?;

        let mut perms = fs::metadata(&dst)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dst, perms)?;
    }

    Ok(())
}
