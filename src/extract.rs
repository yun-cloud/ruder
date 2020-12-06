use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;
use flate2::read::GzDecoder;
use tar::Archive;
use thiserror::Error;
use zip::ZipArchive;

pub fn unpack<P: AsRef<Path>, Q: AsRef<Path>>(path: P, dst: Q) -> anyhow::Result<()> {
    let ext: TarExt = path
        .as_ref()
        .to_str()
        .ok_or_else(|| anyhow!("failed to convert path to &str"))?
        .parse()?;

    match ext {
        TarExt::TarGz => {
            let tar_gz = File::open(path)?;
            let tar = GzDecoder::new(tar_gz);
            let mut ar = Archive::new(tar);
            ar.unpack(dst)?;
        }
        TarExt::Zip => {
            let f = File::open(path)?;
            let mut ar = ZipArchive::new(f)?;
            ar.extract(dst)?;
        }
        TarExt::Tar => {
            let tar = File::open(path)?;
            let mut ar = Archive::new(tar);
            ar.unpack(dst)?;
        }
    }

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum TarExt {
    TarGz,
    Tar,
    Zip,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("We can not handle this kind of tar, try .tar.gz, .tar, or .zip")]
pub struct TarExtParseError(());

impl FromStr for TarExt {
    type Err = TarExtParseError;

    fn from_str(s: &str) -> Result<Self, TarExtParseError> {
        if s.ends_with(".tar.gz") {
            Ok(TarExt::TarGz)
        } else if s.ends_with(".tar") {
            Ok(TarExt::Tar)
        } else if s.ends_with(".zip") {
            Ok(TarExt::Zip)
        } else {
            Err(TarExtParseError(()))
        }
    }
}

pub fn unpack_tar_gz<P: AsRef<Path>, Q: AsRef<Path>>(path: P, dst: Q) -> anyhow::Result<()> {
    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut ar = Archive::new(tar);
    ar.unpack(dst)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tar_ext_test() {
        assert_eq!(
            "ripgrep-12.1.1-x86_64-unknown-linux-musl.tar.gz".parse::<TarExt>(),
            Ok(TarExt::TarGz)
        );
        assert_eq!(
            "ripgrep-12.1.1-x86_64-pc-windows-msvc.zip".parse::<TarExt>(),
            Ok(TarExt::Zip)
        );
        assert_eq!("foo.tar".parse::<TarExt>(), Ok(TarExt::Tar));
        assert_eq!("foo.tar.xz".parse::<TarExt>(), Err(TarExtParseError(())));
    }
}
