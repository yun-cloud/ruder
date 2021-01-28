use flate2::read::GzDecoder;
use std::io::{self, Cursor, Read};
use std::mem::MaybeUninit;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncRead;
use zip::ZipArchive;

pub enum Archive<T: AsRef<[u8]>> {
    TarGz(tar::Archive<GzDecoder<Cursor<T>>>),
    Tar(tar::Archive<Cursor<T>>),
    Zip(ZipArchive<Cursor<T>>),
    Raw(Cursor<T>),
}

pub enum Extract<'a, T: AsRef<[u8]> + Send + Sync> {
    TarGz(tar::Entry<'a, GzDecoder<Cursor<T>>>),
    Tar(tar::Entry<'a, Cursor<T>>),
    Zip(zip::read::ZipFile<'a>),
    Raw(&'a mut Cursor<T>),
}

impl<T: AsRef<[u8]>> Archive<T> {
    pub fn new(data: T, name: &str) -> anyhow::Result<Archive<T>> {
        use Archive::*;

        let cur = Cursor::new(data);
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Ok(TarGz(tar::Archive::new(GzDecoder::new(cur))))
        } else if name.ends_with(".tar") {
            Ok(Tar(tar::Archive::new(cur)))
        } else if name.ends_with(".zip") {
            Ok(Zip(ZipArchive::new(cur)?))
        } else {
            log::warn!(
                "Treat {} as raw binary. May caused by unhandled file extension",
                name
            );
            Ok(Raw(cur))
        }
    }
}

impl<T: AsRef<[u8]> + Send + Sync> Archive<T> {
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

impl<'a, T: AsRef<[u8]> + Send + Sync> Read for Extract<'a, T> {
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

impl<'a, T: AsRef<[u8]> + Send + Sync> AsyncRead for Extract<'a, T> {
    unsafe fn prepare_uninitialized_buffer(&self, _buf: &mut [MaybeUninit<u8>]) -> bool {
        false
    }

    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Read::read(self.get_mut(), buf))
    }
}
