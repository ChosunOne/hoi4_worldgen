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
#![allow(clippy::pub_use)]

use crate::components::prelude::*;
use image::ImageError;
use indicatif::style::TemplateError;
use jomini::{TextDeserializer, TextTape};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::task::JoinError;

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
    /// Error while parsing a file
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
    /// An invalid buildings file
    #[error("{0}")]
    InvalidBuildingsFile(String),
    /// Duplicate building type
    #[error("{0}")]
    DuplicateBuildingType(BuildingId),
    /// Invalid building id
    #[error("{0}")]
    InvalidBuildingId(BuildingId),
    /// Invalid terrain file
    #[error("{0}")]
    InvalidKeyFile(String),
    /// Duplicate terrain type
    #[error("0")]
    DuplicateKeyType(String),
    /// Invalid image file
    #[error("{0}")]
    InvalidImageFile(#[from] ImageError),
    /// Invalid image type
    #[error("{0}")]
    InvalidImageType(PathBuf),
    /// Invalid image size
    #[error("{0}")]
    InvalidImageSize(PathBuf),
    /// Image size mismatch
    #[error("{0}")]
    ImageSizeMismatch(String),
    /// Invalid province color
    #[error("{0:?}")]
    InvalidProvinceColor((Red, Green, Blue)),
    /// Incomplete province definitions
    #[error("{0:?}")]
    IncompleteProvinceDefinitions(Vec<(Red, Green, Blue)>),
    /// Invalid province terrain
    #[error("{0:?}")]
    InvalidProvinceTerrain(Definition),
    /// A join error
    #[error("{0}")]
    JoinError(#[from] JoinError),
    /// An `indicatif` template error
    #[error("{0}")]
    TemplateError(#[from] TemplateError),
}

/// Appends a directory to the front of a given path.
/// # Errors
/// * If the path has no parent directory
/// * If the path has no file name
#[inline]
pub fn append_dir(p: &Path, d: &str) -> Result<PathBuf, MapError> {
    let dirs = p
        .parent()
        .ok_or_else(|| MapError::FileNotFoundError(p.to_path_buf()))?;
    Ok(dirs.join(d).join(
        p.file_name()
            .ok_or_else(|| MapError::FileNotFoundError(p.to_path_buf()))?,
    ))
}

/// Returns a vector of rows from a CSV file.
pub trait LoadCsv
where
    Self: Sized,
{
    /// Returns a vector of rows from a CSV file.
    /// # Errors
    /// Returns an error if the file cannot be read.
    fn load_csv<P: AsRef<Path>>(path: P, has_headers: bool) -> Result<Vec<Self>, MapError>;
}

impl<T: Sized + for<'de> Deserialize<'de>> LoadCsv for T {
    #[inline]
    fn load_csv<P: AsRef<Path>>(path: P, has_headers: bool) -> Result<Vec<Self>, MapError> {
        let data = fs::read_to_string(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .delimiter(b';')
            .from_reader(data.as_bytes());
        let rows = rdr.deserialize().flatten().collect();
        Ok(rows)
    }
}

/// Returns a set of all the keys in the first object of the file.
pub trait LoadKeys
where
    Self: Sized,
{
    /// Returns a set of all the keys in the first object of the file.
    /// # Errors
    /// If the file is not found or if the file is empty.
    fn load_keys(path: &Path) -> Result<HashSet<Self>, MapError>;
}

impl<T: Sized + From<String> + Eq + Hash> LoadKeys for T {
    #[inline]
    fn load_keys(path: &Path) -> Result<HashSet<T>, MapError> {
        let data = fs::read_to_string(&path)?;
        let tape = TextTape::from_slice(data.as_bytes())?;
        let reader = tape.windows1252_reader();
        let fields = reader.fields().collect::<Vec<_>>();
        let (_key, _op, value) = fields
            .get(0)
            .ok_or_else(|| MapError::InvalidKeyFile(path.to_string_lossy().to_string()))?;
        let types_container = value.read_object()?;
        let types_objects = types_container.fields().collect::<Vec<_>>();
        let mut types = HashSet::new();
        for (key, _op, _value) in types_objects {
            let terrain_type = key.read_string().into();
            if types.contains(&terrain_type) {
                return Err(MapError::DuplicateKeyType(key.read_string()));
            }
            types.insert(terrain_type);
        }
        Ok(types)
    }
}

/// A trait for when a structure can easily be converted from a string directly via `jomini`'s
/// `TextDeserializer`..
pub trait LoadObject
where
    Self: Sized,
{
    /// Deserializes a string into a structure.  Only works if the string requires no modification
    /// prior to deserialization.
    /// # Errors
    /// Returns an error if the file cannot be read.
    fn load_object<P: AsRef<Path>>(path: P) -> Result<Self, MapError>;
}

impl<T: Sized + for<'de> Deserialize<'de>> LoadObject for T {
    #[inline]
    fn load_object<P: AsRef<Path>>(path: P) -> Result<Self, MapError> {
        let data = fs::read_to_string(path)?;
        let object = TextDeserializer::from_windows1252_slice(data.as_bytes())?;
        Ok(object)
    }
}

/// Loads a map where the keys are `StateId`s and the values are `Vec<ProvinceId>`s.
/// # Errors
/// Returns an error if the file cannot be read.
#[inline]
pub fn load_state_map<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<StateId, Vec<ProvinceId>>, MapError> {
    let data = fs::read_to_string(path)?;
    let mut state_map = HashMap::new();

    for line in data.lines() {
        let tape = TextTape::from_slice(line.as_bytes())?;
        let reader = tape.windows1252_reader();
        for (key, _op, value) in reader.fields() {
            let state_id = key.read_str().parse::<StateId>()?;
            let province_ids = {
                let array = value.read_array()?;
                let mut ids = Vec::new();
                for id in array.values() {
                    let id_string = id.read_string()?;
                    ids.push(id_string.parse::<ProvinceId>()?);
                }
                ids
            };
            state_map.insert(state_id, province_ids);
        }
    }

    Ok(state_map)
}

#[cfg(test)]
mod tests {
    use indicatif::{InMemoryTerm, ProgressBar, ProgressDrawTarget, TermLike};

    #[test]
    fn it_paints_to_an_in_memory_term() {
        let term = InMemoryTerm::new(64, 120);
        let mut bar = ProgressBar::new(100);
        let draw_target =
            ProgressDrawTarget::term_like(Box::new(term.clone()) as Box<dyn TermLike>);
        bar.set_draw_target(draw_target);
        bar.set_style(indicatif::ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} {bytes_per_sec} {msg}").expect("invalid style"));
        bar.set_message("Loading...");
        bar.set_position(50);
        bar.finish_with_message("Done!");
        let output = term.contents();
        assert_eq!(
            output,
            "Loading... [00:00:00] [====================] 0/100 0.00B Done!\n"
        );
    }
}
