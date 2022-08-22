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
use derive_more::Display;
use image::ImageError;
use indicatif::style::TemplateError;
use jomini::{ScalarError, TextDeserializer, TextTape};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use tokio::task::JoinError;

/// Holds the components of the map
pub mod components;
/// Holds the components together into one struct
pub mod map;

/// The map display mode
#[allow(clippy::exhaustive_enums)]
#[derive(Default, Display, Copy, Clone, Debug, PartialEq, Eq)]
pub enum MapDisplayMode {
    #[default]
    HeightMap,
    Terrain,
    Provinces,
    Rivers,
    StrategicRegions,
    States,
}

/// Errors that may occur when loading/verifying/creating a map.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MapError {
    /// Error while reading/writing to a file on disk.
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    /// Error loading a value
    #[error("{0}")]
    LoadError(#[from] jomini::Error),
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
    /// A definition could be found with the given province id
    #[error("{0}")]
    DefinitionNotFound(ProvinceId),
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
    /// An invalid key type
    #[error("{0}")]
    InvalidKey(String),
    /// An invalid value type
    #[error("{0}")]
    InvalidValue(String),
    /// An invalid `DayMonth`
    #[error("{0}")]
    InvalidDayMonth(#[from] DayMonthParseError),
    /// An invalid float
    #[error("{0}")]
    InvalidFloat(#[from] std::num::ParseFloatError),
    /// An invalid scalar
    #[error("{0}")]
    InvalidScalar(#[from] ScalarError),
    /// An invalid int conversion
    #[error("{0}")]
    InvalidInt(#[from] std::num::TryFromIntError),
    /// An invalid continent index
    #[error("{0}")]
    InvalidContinentIndex(ContinentIndex),
    /// An `actix` `MailBoxError`
    #[error("{0}")]
    MailBoxError(#[from] actix::MailboxError),
    /// The `UiRenderer` is not initialized
    #[error("The UI Renderer is not initialized")]
    UiRendererNotInitialized,
    /// An error receiving data from a channel
    #[error("{0}")]
    RecvError(#[from] std::sync::mpsc::RecvError),
    #[error("{0}")]
    RegionNotFoundForProvince(ProvinceId),
    #[error("Invalid Period")]
    InvalidPeriod,
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
    /// Returns a set of all the keys in the given object of the file.
    /// # Errors
    /// If the file is not found or if the file is empty.
    fn load_keys(path: &Path, object_name: &str) -> Result<HashSet<Self>, MapError>;
}

impl<T: Sized + From<String> + Eq + Hash> LoadKeys for T {
    #[inline]
    fn load_keys(path: &Path, object_name: &str) -> Result<HashSet<T>, MapError> {
        let data = fs::read_to_string(&path)?;
        let tape = TextTape::from_slice(data.as_bytes())?;
        let reader = tape.windows1252_reader();
        let fields = reader
            .fields()
            .filter(|f| {
                let (raw_key, _op, _value) = f;
                raw_key.read_str() == object_name
            })
            .collect::<Vec<_>>();
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

/// Loads a map where the keys and values are deserializable from strings.
/// # Errors
/// Returns an error if the file cannot be read.
#[inline]
pub fn load_map<
    P: AsRef<Path>,
    K: Eq + Hash + FromStr<Err = E>,
    E: Display,
    V: FromStr<Err = E2>,
    E2: Display,
>(
    path: P,
) -> Result<HashMap<K, Vec<V>>, MapError> {
    let data = fs::read_to_string(path)?;
    let mut map = HashMap::new();

    for line in data.lines() {
        let tape = TextTape::from_slice(line.as_bytes())?;
        let reader = tape.windows1252_reader();
        for (key, _op, value) in reader.fields() {
            let id = match key.read_str().parse::<K>() {
                Ok(i) => i,
                Err(e) => return Err(MapError::InvalidKey(e.to_string())),
            };
            let values = {
                let array = value.read_array()?;
                let mut ids = Vec::new();
                for val in array.values() {
                    let v_string = val.read_string()?;
                    let v = match v_string.parse::<V>() {
                        Ok(v) => v,
                        Err(e) => return Err(MapError::InvalidValue(e.to_string())),
                    };
                    ids.push(v);
                }
                ids
            };
            map.insert(id, values);
        }
    }

    Ok(map)
}
