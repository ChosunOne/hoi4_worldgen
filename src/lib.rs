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

use jomini::JominiDeserialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The file default.map references the bitmaps and text files that make up the map.  
/// * All file paths can be changed and are relative to the `map/` directory.  
/// * The map's width and height are taken from provinces.bmp. They both have to be multiples of 256.  
#[derive(Debug, JominiDeserialize, PartialEq)]
#[non_exhaustive]
pub struct DefaultMap {
    /// Contains the provinces the game recognizes.
    /// The province ID, as well as the RGB value, must be unique. The valid terrain types are
    /// defined in `/Hearts of Iron IV/common/terrain`. The continent is a 1-based index into the
    /// continent list. Sea provinces must have the continent of 0.
    /// * **The file must have Windows-style line endings (CRLF!)**
    pub definitions: Box<Path>,
    /// Controls the location and shape of the provinces on the map. Each pixel belongs to the province with the matching RGB value.
    /// Provinces that share a pixel edge neighbor each other and are connected.  
    /// Height and width needs to be multiples of 256.  
    /// Provinces should be kept contiguous as much as possible. Large gaps will cause the game to crash.  
    /// When debug mode is enabled, it will warn about the following conditions:
    /// * "Map invalid X crossing. Please fix pixels at coords": Four provinces share a common corner. The game connects the bottom left and the top right provinces but this situation is confusing to the player and should be avoided.
    /// * "Province X has TOO LARGE BOX. Perhaps pixels are spread around the world in provinces.bmp": The province has a width/height of more than 1/8th of the total map width/height. This might be an indication that two provinces inadvertently share a color.
    /// * "Province X has only N pixels": The province consists of no more than NGraphics.MINIMUM_PROVINCE_SIZE_IN_PIXELS (8 by default). This is likely too small to be easily usable by the player.
    /// The provinces.bmp file should be in RGB mode and saved as a 24-bit bitmap image file (.BMP).
    /// * If the map is saved with a 32-bit format, the game will crash with a 'warning X4008: floating point division by zero' error.
    pub provinces: Box<Path>,
    /// Seems to be unused.
    pub positions: Box<Path>,
    /// An 8-bit indexed mode BMP file that controls the terrain assignment and textures.
    /// The indexes refer to the terrains at the bottom of `/Hearts of Iron IV/common/terrain/00_terrain.txt`.  
    /// The terrain only affects the visuals of the map and paths between provinces; the provinces themselves use the assigned terrains from definitions.csv.
    /// Needs to be the same size as provinces.bmp.
    pub terrain: Box<Path>,
    /// Controls the river placement on the map. Rivers must always be 1 pixel thick.
    /// The rivers.bmp file should be in Indexed mode and saved as a 8-bit bitmap image file (.BMP).
    /// Needs to be the same size as provinces.bmp.  
    ///
    /// | Index | Color         | Function                                                             |  
    /// |-------|---------------|----------------------------------------------------------------------|  
    /// | 0     | (0, 255, 0)   | The source of a river                                                |  
    /// | 1     | (255, 0, 0)   | Flow-in source. Used to join multiple 'source' paths into one river. |  
    /// | 2     | (255, 252, 0) | Flow-out source. Used to branch outwards from one river.             |
    /// | 3     | (0, 225, 255) | River with narrowest texture.                                        |
    /// | 4     | (0, 200, 255) | River with narrow texture.                                           |
    /// | 5     | (0, 155, 255) |                                                                      |
    /// | 6     | (0, 100, 255) | River with wide texture.                                             |
    /// | 7     | (0, 0, 255)   |                                                                      |
    /// | 8     | (0, 0, 225)   |                                                                      |
    /// | 9     | (0, 0, 200)   |                                                                      |
    /// | 10    | (0, 0, 150)   |                                                                      |
    /// | 11    | (0, 0, 100)   | River with widest texture.                                           |
    ///
    /// * Indexes 0 up to including 6 are treated as small rivers for game mechanics, indexes up to including 11 as large rivers.
    /// * To correctly render, each river must have exactly one marker, either a start marker (green/yellow) or an end marker (red).  
    /// If the path between two provinces overlaps at least one river pixel, it is considered a river crossing.
    /// If it intersects multiple river pixels of different types, the crossing type is implementation defined.
    /// To avoid player confusion, province paths should either clearly cut or stay clear of a river.
    /// * Do NOT place a green source pixel at the beginning of a river that ends in a red merge pixel.
    /// This will cause the river to use the VFX for emptying into an ocean at the merge point rather than merging into the other river.
    pub rivers: Box<Path>,
    /// Determines the 3D mesh of the map. ( 0, 0, 0 ) is the lowest point, with (255, 255, 255) being the highest.
    /// * The sea level is set at (95, 95, 95), so any values below that will be submerged.
    /// * Make the transitions between heights smooth, otherwise you will create noticeable jagged edges.
    /// * The heightmap.bmp file should be in Greyscale mode and saved as a 8-bit bitmap image file (.BMP).
    /// * Needs to be the same size as provinces.bmp.
    pub heightmap: Box<Path>,
    /// Controls the tree placement on the map. The resolution of the trees.bmp file affects the density of trees placed.
    /// The trees.bmp file should be in Indexed mode and saved as a 8-bit bitmap image file (.BMP).
    pub tree_definition: Box<Path>,
    /// Found in continent.txt, located in the map folder, continents are used to group large swathes
    /// of provinces together as a traditional continent. Continents are used to define AI areas.  
    /// All land provinces must belong to a continent, otherwise you may experience errors/crashes.  
    /// The continents in the base game are (the number before the continent name is the ID):  
    /// 1. Europe  
    /// 2. North America   
    /// 3. South America  
    /// 4. Australia   
    /// 5. Africa   
    /// 6. Asia   
    /// 7. Middle East  
    pub continent: Box<Path>,
    /// The names of the Adjacency Rules
    pub adjacency_rules: Box<Path>,
    /// The adjacencies file is found at `/Hearts of Iron IV/map/adjacencies.csv`. As a comma-separated file,
    /// you may open it with Excel or other similar programs, or a text editor. The default encoding is ANSI.  
    /// * The file modifies and adds custom adjacencies between provinces on top of the normal connections
    /// defined by the provinces and rivers maps. For example it controls which provinces non-
    /// contiguously connect to other provinces. An island is normally not connected to any other
    /// land provinces, as there are sea provinces in the way.
    /// * The adjacencies file tells the game to connect such provinces, allowing land units to walk
    /// between them. It also allows changing the properties of an existing connection, e.g. making
    /// them impassable, changing their type, or defining which provinces are gated by straits.
    /// * The type may be empty for a normal land connection, or "river"/"large_river"/"sea"/"impassable"
    /// for a connection of the respective type. The "through" field defines a province that can block
    /// the adjacency. While an enemy unit controls this province, the connection will be unavailable.
    /// -1 disables this feature; however, any adjacency with the type "sea" must have a province
    /// defined here. The map coordinates are used to adjust the starting and ending point of the
    /// graphic displaying the adjacency. If no adjustment is needed, use -1 in place of an actual
    /// coordinate. Optionally an adjacency rule can be referenced that controls access through the
    /// adjacency.
    /// * Even when otherwise empty, the file must be terminated with a line containing a negative
    /// from-field and a semicolon to prevent an infinite hang on start-up.
    pub adjacencies: Box<Path>,
    /// Unused
    pub climate: Option<Box<Path>>,
    /// Defines the cosmetic 3D objects found in the map. This includes the map frame, so don't
    /// simply empty the file if you want to remove the other objects.
    pub ambient_object: Box<Path>,
    /// Used to define the color adjustments during the four seasons that pass in game.
    /// There are four seasons: winter, spring, summer and autumn.
    pub seasons: Box<Path>,
    /// Define which indices in trees.bmp palette which should count as trees for automatic terrain
    /// assignment
    pub tree: Vec<usize>,
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

/// Whether a province is coastal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Coastal(bool);

/// Terrain type defined in the `common/00_terrain.txt` file.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash, PartialOrd, Ord)]
pub struct Terrain(String);

/// The continent is a 1-based index into the continent list. Sea provinces must have the continent of 0.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Continent(i32);

/// The ID for a province.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ProvinceId(i32);

/// A red value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Red(u8);

/// A green value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Green(u8);

/// A blue value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Blue(u8);

/// An x coordinate on the map.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct XCoord(i32);

/// A y coordinate on the map.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct YCoord(i32);

/// An adjacency rule name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash)]
pub struct AdjacencyRuleName(String);

/// An adjacency rule
#[derive(Clone, Debug, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct RawAdjacencyRules {
    /// The info of the adjacency rule.
    #[jomini(duplicated)]
    adjacency_rule: Vec<AdjacencyRule>,
}

/// An adjacency rule
#[derive(Clone, Debug, JominiDeserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct AdjacencyRule {
    /// The name of the adjacency rule.
    pub name: AdjacencyRuleName,
    /// The logic for when the adjacency is contested.
    pub contested: AdjacencyLogic,
    /// The logic for when the adjacency is controlled by an enemy.
    pub enemy: AdjacencyLogic,
    /// The logic for when the adjacency is controlled by a friend.
    pub friend: AdjacencyLogic,
    /// The logic for when the adjacency is controlled by a neutral.
    pub neutral: AdjacencyLogic,
    /// The provinces for which the rule applies.
    pub required_provinces: Vec<ProvinceId>,
    /// The icon for the adjacency rule.
    pub icon: Icon,
    /// Graphical offsets
    pub offset: Vec<i32>,
    /// Conditions when the rule can be disabled.
    pub is_disabled: Option<IsDisabled>,
}

/// Conditions when an adjacency rule can be disabled
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct IsDisabled {
    /// The tooltip to display when the rule is disabled.
    pub tooltip: String,
}

/// The logic for the adjacency rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct AdjacencyLogic {
    /// Whether armies can pass
    pub army: bool,
    /// Whether fleets can pass
    pub navy: bool,
    /// Whether subs can pass
    pub submarine: bool,
    /// Whether trade can pass
    pub trade: bool,
}

/// The the province on which to show the crossing icon
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Icon(ProvinceId);

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
    pub continent: Continent,
}

/// The Adjacency type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AdjacencyType {
    /// The adjacent province cannot be reached from this province
    #[serde(rename = "impassable")]
    Impassable,
    /// The adjacent province is a sea province
    #[serde(rename = "sea")]
    Sea,
    /// The adjacent province is bordered by a river
    #[serde(rename = "river")]
    River,
    /// The adjacent province is bordered by a large river
    #[serde(rename = "large_river")]
    LargeRiver,
}

/// The type of adjacency between two provinces
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Adjacency {
    /// The ID of the starting province
    #[serde(rename = "From")]
    pub from: ProvinceId,
    /// The ID of the destination province
    #[serde(rename = "To")]
    pub to: ProvinceId,
    /// The type of adjacency
    #[serde(rename = "Type")]
    pub adjacency_type: Option<AdjacencyType>,
    /// Defines a province that can block the adjacency.
    /// While an enemy unit controls this province, the connection will be unavailable. -1 disables
    /// this feature; however, any adjacency with the type "sea" must have a province defined here.
    #[serde(rename = "Through")]
    pub through: Option<ProvinceId>,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub start_x: XCoord,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub stop_x: XCoord,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub start_y: YCoord,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub stop_y: YCoord,
    /// An adjacency rule can be referenced that controls access through the adjacency.
    pub adjacency_rule_name: Option<AdjacencyRuleName>,
    /// The comment for the adjacency
    pub comment: Option<String>,
}

/// A date in the format YYYY.MM.DD
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Date(String);

/// An HSV value.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Hsv((f32, f32, f32));

impl PartialEq for Hsv {
    fn eq(&self, other: &Self) -> bool {
        self.0 .0 == other.0 .0 && self.0 .1 == other.0 .1 && self.0 .2 == other.0 .2
    }
}

impl Eq for Hsv {}

/// Defines the color adjustment for a season.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Season {
    /// The starting date of the season.
    /// Format is 00.\<month\>.\<day\>  
    /// Ex. 00.12.01
    start_date: Date,
    /// The ending date of the season.
    end_date: Date,
    /// Applies HSV to northern hemisphere
    hsv_north: Hsv,
    /// Applies colorbalance to northern hemisphere
    colorbalance_north: Hsv,
    /// Applies HSV to the equator
    hsv_center: Hsv,
    /// Applies colorbalance to the equator
    colorbalance_center: Hsv,
    /// Applies HSV to southern hemisphere
    hsv_south: Hsv,
    /// Applies colorbalance to southern hemisphere
    colorbalance_south: Hsv,
}

/// The definitions from the definition csv file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Definitions {
    /// The definitions for the provinces
    pub definitions: Vec<Definition>,
}

/// The adjacencies from the adjacency csv file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Adjacencies {
    /// The adjacencies between provinces
    pub adjacencies: Vec<Adjacency>,
}

/// The adjacency rules from the adjacency rule file
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AdjacencyRules {
    /// The adjacency rules
    pub adjacency_rules: HashMap<AdjacencyRuleName, AdjacencyRule>,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::AdjacencyType::Impassable;
    use jomini::TextDeserializer;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn it_reads_a_default_map_file() {
        let map_data =
            fs::read_to_string("./test/default.map").expect("Failed to read default.map");
        let map = TextDeserializer::from_windows1252_slice::<DefaultMap>(map_data.as_bytes())
            .expect("Failed to deserialize default.map");
        assert_eq!(
            map.definitions
                .to_str()
                .expect("Failed to get map definitions"),
            "definition.csv"
        );
        assert_eq!(
            map.provinces.to_str().expect("Failed to get map provinces"),
            "provinces.bmp"
        );
        assert_eq!(
            map.terrain.to_str().expect("Failed to get map terrain"),
            "terrain.bmp"
        );
        assert_eq!(
            map.rivers.to_str().expect("Failed to get map rivers"),
            "rivers.bmp"
        );
        assert_eq!(
            map.heightmap.to_str().expect("Failed to get map heightmap"),
            "heightmap.bmp"
        );
        assert_eq!(
            map.tree_definition
                .to_str()
                .expect("Failed to get map tree definition"),
            "trees.bmp"
        );
        assert_eq!(
            map.continent
                .to_str()
                .expect("Failed to get map continents"),
            "continent.txt"
        );
        assert_eq!(
            map.adjacency_rules
                .to_str()
                .expect("Failed to get map adjacency rules"),
            "adjacency_rules.txt"
        );
        assert!(map.climate.is_none());
        assert_eq!(
            map.ambient_object
                .to_str()
                .expect("Failed to get map ambient objects"),
            "ambient_object.txt"
        );
        assert_eq!(
            map.seasons.to_str().expect("Failed to get map seasons"),
            "seasons.txt"
        );
        assert_eq!(map.tree, vec![3, 4, 7, 10]);
    }

    #[test]
    fn it_reads_definitions_from_the_map() {
        let map_data =
            fs::read_to_string("./test/default.map").expect("Failed to read default.map");
        let map = TextDeserializer::from_windows1252_slice::<DefaultMap>(map_data.as_bytes())
            .expect("Failed to deserialize default.map");
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
                continent: Continent(2)
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
                continent: Continent(2)
            }
        );
    }

    #[test]
    fn it_reads_adjacencies_from_the_map() {
        let map_data =
            fs::read_to_string("./test/default.map").expect("Failed to read default.map");
        let map = TextDeserializer::from_windows1252_slice::<DefaultMap>(map_data.as_bytes())
            .expect("Failed to deserialize default.map");
        let adjacency_rules_path = append_dir(&map.adjacencies, "./test");
        let adjacency_rules_data =
            fs::read_to_string(&adjacency_rules_path).expect("Failed to read adjacency_rules.txt");
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_reader(adjacency_rules_data.as_bytes());
        let mut adjacencies = Vec::new();
        for adjacency in rdr.deserialize() {
            adjacencies.push(adjacency.expect("Failed to deserialize adjacency"));
        }
        let adjacencies = Adjacencies { adjacencies };
        assert_eq!(adjacencies.adjacencies.len(), 486);
        assert_eq!(
            adjacencies.adjacencies[0],
            Adjacency {
                from: ProvinceId(6402),
                to: ProvinceId(6522),
                adjacency_type: Some(Impassable),
                through: Some(ProvinceId(-1)),
                start_x: XCoord(-1),
                stop_x: XCoord(-1),
                start_y: YCoord(-1),
                stop_y: YCoord(-1),
                adjacency_rule_name: None,
                comment: None
            }
        );
    }

    #[test]
    fn it_reads_adjacency_rules_from_the_map() {
        let map_data =
            fs::read_to_string("./test/default.map").expect("Failed to read default.map");
        let map = TextDeserializer::from_windows1252_slice::<DefaultMap>(map_data.as_bytes())
            .expect("Failed to deserialize default.map");
        let adjacency_rules_path = append_dir(&map.adjacency_rules, "./test");
        let adjacency_rules_data =
            fs::read_to_string(&adjacency_rules_path).expect("Failed to read adjacency_rules.txt");
        let rules = TextDeserializer::from_windows1252_slice::<RawAdjacencyRules>(
            adjacency_rules_data.as_bytes(),
        )
        .expect("Failed to deserialize adjacency_rules.txt");
        let mut adjacency_rules = AdjacencyRules {
            adjacency_rules: HashMap::new(),
        };
        for rule in rules.adjacency_rule {
            adjacency_rules
                .adjacency_rules
                .insert(rule.name.clone(), rule);
        }
        assert_eq!(adjacency_rules.adjacency_rules.len(), 11);
        assert_eq!(
            adjacency_rules
                .adjacency_rules
                .get(&AdjacencyRuleName("Veracruz Canal".to_owned())),
            Some(&AdjacencyRule {
                name: AdjacencyRuleName("Veracruz Canal".to_owned()),
                contested: AdjacencyLogic {
                    army: false,
                    navy: false,
                    submarine: false,
                    trade: false
                },
                enemy: AdjacencyLogic {
                    army: false,
                    navy: false,
                    submarine: false,
                    trade: false
                },
                friend: AdjacencyLogic {
                    army: true,
                    navy: true,
                    submarine: true,
                    trade: true
                },
                neutral: AdjacencyLogic {
                    army: false,
                    navy: false,
                    submarine: false,
                    trade: true
                },
                required_provinces: vec![ProvinceId(10033), ProvinceId(10101)],
                icon: Icon(ProvinceId(10101)),
                offset: vec![-3, 0, -6],
                is_disabled: None
            })
        );
    }

    fn append_dir(p: &Path, d: &str) -> PathBuf {
        let dirs = p.parent().unwrap();
        dirs.join(d).join(p.file_name().unwrap())
    }
}
