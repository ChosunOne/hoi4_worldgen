//! Map generator for Hearts of Iron IV by Paradox Interactive.
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_docs
)]
#![allow(clippy::module_inception)]
#![allow(clippy::implicit_return)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::shadow_same)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::separated_literal_suffix)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::use_self)]
#![allow(clippy::pattern_type_mismatch)]

use crate::components::wrappers::{StrategicRegionId, StrategicRegionName};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Holds the components of the map
pub mod components;
/// Holds the components together into one struct
pub mod map;

/// Errors that may occur when loading/verifying/creating a map.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MapError {
    /// Error while reading/writing to a file on disk.
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    /// Error parsing a value
    #[error("{0}")]
    ParseError(#[from] jomini::Error),
    #[error("{0}")]
    DeserializeError(#[from] jomini::DeserializeError),
    /// Error finding a file
    #[error("File not found")]
    FileNotFoundError(PathBuf),
    /// An invalid strategic region id
    #[error("{0}")]
    InvalidStrategicRegionId(#[from] std::num::ParseIntError),
    /// An invalid strategic region name
    #[error("{0}")]
    InvalidStrategicRegionName(StrategicRegionName),
    /// An invalid strategic region
    #[error("{0}")]
    InvalidStrategicRegion(StrategicRegionId),
    /// An invalid strategic region file name
    #[error("{0}")]
    InvalidStrategicRegionFileName(String),
    /// An invalid supply node
    #[error("{0}")]
    InvalidSupplyNode(String),
    /// An invalid railway
    #[error("{0}")]
    InvalidRailway(String),
}

/// Appends a directory to the front of a given path.
#[inline]
#[must_use]
pub fn append_dir(p: &Path, d: &str) -> PathBuf {
    let dirs = p.parent().expect("Failed to get parent dir");
    dirs.join(d)
        .join(p.file_name().expect("Failed to get file name"))
}
