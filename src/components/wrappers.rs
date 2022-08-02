use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

/// Whether a province is coastal.
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Coastal(pub bool);

/// Terrain type defined in the `common/00_terrain.txt` file.
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Terrain(pub String);

/// The continent is a 1-based index into the continent list. Sea provinces must have the continent of 0.
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ContinentIndex(pub i32);

/// A continent identifier
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub struct Continent(pub String);

/// The ID for a province.
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
    Hash,
    FromStr,
)]
#[non_exhaustive]
pub struct ProvinceId(pub i32);

/// A temperature value.
#[derive(Copy, Clone, Debug, Display, PartialEq, PartialOrd, Deserialize, Serialize, FromStr)]
#[non_exhaustive]
pub struct Temperature(pub f32);

/// A weight value.
#[derive(Copy, Clone, Debug, Display, PartialEq, PartialOrd, Deserialize, Serialize, FromStr)]
#[non_exhaustive]
pub struct Weight(pub f32);

/// A snow level value.
#[derive(Copy, Clone, Debug, Display, PartialEq, PartialOrd, Deserialize, Serialize, FromStr)]
#[non_exhaustive]
pub struct SnowLevel(pub f32);

/// The ID for a state.
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
    Hash,
    FromStr,
)]
#[non_exhaustive]
pub struct StateId(pub i32);

/// The ID for a strategic region.
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
    Hash,
    FromStr,
)]
#[non_exhaustive]
pub struct StrategicRegionId(pub i32);

/// The level of the railroad.
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
    Hash,
    FromStr,
)]
#[non_exhaustive]
pub struct RailLevel(pub i32);

/// A red value.
#[derive(
    Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash,
)]
#[non_exhaustive]
pub struct Red(pub u8);

/// A green value.
#[derive(
    Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash,
)]
#[non_exhaustive]
pub struct Green(pub u8);

/// A blue value.
#[derive(
    Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash,
)]
#[non_exhaustive]
pub struct Blue(pub u8);

/// An x coordinate on the map.
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[non_exhaustive]
pub struct XCoord(pub i32);

/// A y coordinate on the map.
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[non_exhaustive]
pub struct YCoord(pub i32);

/// An adjacency rule name.
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub struct AdjacencyRuleName(pub String);

/// An adjacency rule name.
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub struct StrategicRegionName(pub String);

/// The the province on which to show the crossing icon
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Icon(pub ProvinceId);

/// An HSV value.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Hsv(pub (f32, f32, f32));

impl PartialEq for Hsv {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 .0 == other.0 .0 && self.0 .1 == other.0 .1 && self.0 .2 == other.0 .2
    }
}

impl Eq for Hsv {}
