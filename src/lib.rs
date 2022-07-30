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

use std::path::{Path, PathBuf};

/// Holds the components of the map
pub mod components;
/// Holds the components together into one struct
pub mod map;

/// Appends a directory to the front of a given path.
#[inline]
#[must_use]
pub fn append_dir(p: &Path, d: &str) -> PathBuf {
    let dirs = p.parent().expect("Failed to get parent dir");
    dirs.join(d)
        .join(p.file_name().expect("Failed to get file name"))
}
