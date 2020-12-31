#[macro_use]
mod macros;

pub mod github;

mod archive;
pub use archive::{Archive, Extract};

mod config;
pub use config::{BinaryTable, Config, DefaultTable};
