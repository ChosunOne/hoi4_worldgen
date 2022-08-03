use crate::components::wrappers::{Blue, Coastal, ContinentIndex, Green, ProvinceId, Red, Terrain};
use crate::MapError;
use jomini::TextTape;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
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
        let definitions = Self::load_definitions(definitions_path)?;
        let terrain = Self::load_terrain(terrain_path)?;
        Ok(Self {
            definitions,
            terrain,
        })
    }

    /// Load the definitions from the given path.
    fn load_definitions(definitions_path: &Path) -> Result<Vec<Definition>, MapError> {
        let definitions_data = fs::read_to_string(&definitions_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b';')
            .from_reader(definitions_data.as_bytes());
        let definitions = rdr.deserialize().flatten().collect();
        Ok(definitions)
    }

    /// Load the terrain types from the given path.
    fn load_terrain(path: &Path) -> Result<HashSet<Terrain>, MapError> {
        let data = fs::read_to_string(&path)?;
        let tape = TextTape::from_slice(data.as_bytes())?;
        let reader = tape.windows1252_reader();
        let fields = reader.fields().collect::<Vec<_>>();
        let (_key, _op, value) = fields
            .get(0)
            .ok_or_else(|| MapError::InvalidTerrainFile(path.to_string_lossy().to_string()))?;
        let types_container = value.read_object()?;
        let types_objects = types_container.fields().collect::<Vec<_>>();
        let mut types = HashSet::new();
        for (key, _op, _value) in types_objects {
            let terrain_type = key.read_string().into();
            if types.contains(&terrain_type) {
                return Err(MapError::DuplicateTerrainType(terrain_type));
            }
            types.insert(terrain_type);
        }
        Ok(types)
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::append_dir;
    use crate::components::default_map::DefaultMap;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_reads_definitions_from_the_map() {
        let map =
            DefaultMap::from_file(Path::new("./test/map/default.map")).expect("Failed to read map");
        let definitions_path = map.definitions.to_path_buf();
        let definitions_path = append_dir(&definitions_path, "./test/map");
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
}
