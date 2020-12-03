use std::fs::File;
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

pub fn unpack_tar_gz<P: AsRef<Path>, Q: AsRef<Path>>(path: P, dst: Q) -> anyhow::Result<()> {
    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut ar = Archive::new(tar);
    ar.unpack(dst)?;

    Ok(())
}
