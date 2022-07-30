use crate::components::wrappers::{Blue, Coastal, ContinentIndex, Green, ProvinceId, Red, Terrain};
use serde::{Deserialize, Serialize};

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::append_dir;
    use crate::components::default_map::DefaultMap;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_reads_definitions_from_the_map() {
        let map = DefaultMap::from_file(Path::new("test/default.map")).expect("Failed to read map");
        let definitions_path = map.definitions.to_path_buf();
        let definitions_path = append_dir(&definitions_path, "./test");
        let definitions_data =
            fs::read_to_string(&definitions_path).expect("Failed to read definition.csv");

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b';')
            .from_reader(definitions_data.as_bytes());
        let mut definitions = Vec::new();
        for definition in rdr.deserialize() {
            definitions.push(definition.expect("Failed to deserialize definition"));
        }
        let definitions = Definitions { definitions };
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
