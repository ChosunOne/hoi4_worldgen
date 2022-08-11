use crate::components::wrappers::{Blue, Coastal, ContinentIndex, Green, ProvinceId, Red, Terrain};
use crate::{LoadCsv, LoadKeys, MapError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// An entry in the definitions file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Definition {
    /// The ID of the province
    pub id: ProvinceId,
    /// The red value of the province on the provinces map
    pub r: Red,
    /// The green value of the province on the provinces map
    pub g: Green,
    /// The blue value of the province on the provinces map
    pub b: Blue,
    /// The type of the province
    pub province_type: ProvinceType,
    /// Whether the province is coastal
    pub coastal: Coastal,
    /// The terrain type of the province
    pub terrain: Terrain,
    /// The continent of the province
    pub continent: ContinentIndex,
}

/// The type of the province.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ProvinceType {
    /// A land province
    #[serde(rename = "land")]
    Land,
    /// A sea province
    #[serde(rename = "sea")]
    Sea,
    /// A water province
    #[serde(rename = "lake")]
    Lake,
}

/// The definitions from the definition csv file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Definitions {
    /// The definitions for the provinces
    pub definitions: Vec<Definition>,
    /// The terrain types
    pub terrain: HashSet<Terrain>,
}

impl Definitions {
    /// Load the definitions from the given path.
    /// # Errors
    /// If the file cannot be read, or if the file is not a valid csv file, then an error is returned.
    #[inline]
    pub fn from_files(definitions_path: &Path, terrain_path: &Path) -> Result<Self, MapError> {
        let definitions = Definition::load_csv(definitions_path, false)?;
        let terrain = Terrain::load_keys(terrain_path, "categories")?;
        Ok(Self {
            definitions,
            terrain,
        })
    }

    /// Verifies the province terrain types against the `common/terrain/00_terrain.txt` file
    /// # Errors
    /// * If the provinces contain terrain not defined in the `common/terrain/00_terrain.txt` file
    #[inline]
    pub fn verify_province_terrain(&self) -> Result<(), Vec<MapError>> {
        let errors = self
            .definitions
            .iter()
            .filter(|def| !self.terrain.contains(&def.terrain))
            .map(|def| MapError::InvalidProvinceTerrain(def.clone()))
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(())
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::default_map::DefaultMap;
    use crate::{append_dir, LoadObject};
    use std::path::Path;

    #[test]
    fn it_reads_definitions_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read map");
        let definitions_path = map.definitions.to_path_buf();
        let definitions_path =
            append_dir(&definitions_path, "./test/map").expect("Failed to find definitions");
        let terrain_path = Path::new("./test/common/terrain/00_terrain.txt");
        let definitions = Definitions::from_files(&definitions_path, terrain_path)
            .expect("Failed to read definitions");
        assert_eq!(definitions.definitions.len(), 17007);
        assert_eq!(
            definitions.definitions[0],
            Definition {
                id: ProvinceId(0),
                r: Red(0),
                g: Green(0),
                b: Blue(0),
                province_type: ProvinceType::Land,
                coastal: Coastal(false),
                terrain: Terrain("hills".to_owned()),
                continent: ContinentIndex(2)
            }
        );

        assert_eq!(
            definitions.definitions[16999],
            Definition {
                id: ProvinceId(16999),
                r: Red(189),
                g: Green(48),
                b: Blue(218),
                province_type: ProvinceType::Land,
                coastal: Coastal(false),
                terrain: Terrain("hills".to_owned()),
                continent: ContinentIndex(2)
            }
        );
    }

    #[test]
    fn it_verifies_province_terrain() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read map");
        let definitions_path = map.definitions.to_path_buf();
        let definitions_path =
            append_dir(&definitions_path, "./test/map").expect("Failed to find definitions");
        let terrain_path = Path::new("./test/common/terrain/00_terrain.txt");
        let definitions = Definitions::from_files(&definitions_path, terrain_path)
            .expect("Failed to read definitions");
        if let Err(errors) = definitions.verify_province_terrain() {
            println!("{:#?}", errors);
            assert_eq!(errors.len(), 32);
        } else {
            panic!("Failed to detect invalid terrain in provinces");
        }
    }
}
