#[macro_use]
mod macros;

// pub mod extract;
pub mod github;

use std::io::{self, Cursor, Read, Seek};
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

pub fn extract<R: Read + Seek>(
    mut r: R,
    filename: &str,
    src: &str,
    mut buf: &mut Vec<u8>,
) -> anyhow::Result<usize> {
    // TODO: handle src not exist
    let mut size: usize = 0;
    if filename.ends_with(".tar.gz") {
        let mut ar = tar::Archive::new(GzDecoder::new(r));
        for entry in ar.entries()? {
            let mut f = entry?;
            if f.path()? == Path::new(&src) {
                size = f.read_to_end(&mut buf)?;
            }
        }
    } else if filename.ends_with(".tar") {
        let mut ar = tar::Archive::new(r);
        for entry in ar.entries()? {
            let mut f = entry?;
            if f.path()? == Path::new(&src) {
                size = f.read_to_end(&mut buf)?;
            }
        }
    } else if filename.ends_with(".zip") {
        let mut ar = ZipArchive::new(r)?;
        let mut f = ar.by_name(src)?;
        size = f.read_to_end(&mut buf)?;
    } else {
        size = r.read_to_end(&mut buf)?;
    }

    Ok(size)
}

#[derive(Debug)]
pub struct Archive<R> {
    data: R,
    name: String,
}

impl<R> Archive<R> {
    pub fn new(data: R, name: &str) -> Archive<R> {
        let name = name.to_owned();
        Archive { data, name }
    }
}

impl<R> Archive<R> {
    pub fn extract(self, src: &str) -> Extract<R> {
        Extract::new(self.data, &self.name, src)
    }
}

#[derive(Debug)]
pub struct Extract<R> {
    data: R,
    name: String,
    src: String,
}

impl<R> Extract<R> {
    pub fn new(data: R, name: &str, src: &str) -> Extract<R> {
        Extract {
            data,
            name: name.to_owned(),
            src: src.to_owned(),
        }
    }
}

impl<R> Read for Extract<R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.name.ends_with(".tar.gz") {
            let mut tmp: Vec<u8> = vec![];
            let _size = self.data.read_to_end(&mut tmp)?;
            log::error!("[.tar.gz] _size: {:?}", _size);
            let mut ar = tar::Archive::new(GzDecoder::new(Cursor::new(tmp)));
            for entry in ar.entries()? {
                let mut f = entry?;
                if f.path()? == Path::new(&self.src) {
                    return f.read(buf);
                }
            }
        } else if self.name.ends_with(".tar") {
            let mut tmp: Vec<u8> = vec![];
            let _size = self.data.read_to_end(&mut tmp)?;
            log::error!("[.tar] _size: {:?}", _size);
            let mut ar = tar::Archive::new(Cursor::new(tmp));
            for entry in ar.entries()? {
                let mut f = entry?;
                if f.path()? == Path::new(&self.src) {
                    return f.read(buf);
                }
            }
        } else if self.name.ends_with(".zip") {
            let mut tmp: Vec<u8> = vec![];
            let _size = self.data.read_to_end(&mut tmp)?;
            log::error!("[.zip] _size: {:?}", _size);
            let mut ar = ZipArchive::new(Cursor::new(tmp))?;
            let mut f = ar.by_name(&self.src)?;
            return f.read(buf);
        } else {
            return self.data.read(buf);
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "src not found when extracting",
        ))
    }
}
