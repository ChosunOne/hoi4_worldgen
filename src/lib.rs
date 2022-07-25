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

use serde::{Deserialize, Serialize};
use std::path::Path;

/// The file default.map references the bitmaps and text files that make up the map.  
/// * All file paths can be changed and are relative to the `map/` directory.  
/// * The map's width and height are taken from provinces.bmp. They both have to be multiples of 256.  
#[derive(Debug)]
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
    pub climate: Option<Box<Path>>,
    pub ambient_object: Box<Path>,
    /// Used to define the color adjustments during the four seasons that pass in game.
    /// There are four seasons: winter, spring, summer and autumn.
    pub seasons: Box<Path>,
    pub tree: Vec<usize>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum ProvinceType {
    Land,
    Sea,
    Water,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
struct Coastal(bool);

/// Terrain type defined in the `common/00_terrain.txt` file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum Terrain {
    Plains,
    Hills,
    Urban,
}

/// The continent is a 1-based index into the continent list. Sea provinces must have the continent of 0.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct Continent(u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct ProvinceId(i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct Red(u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct Green(u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct Blue(u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct XCoord(i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct YCoord(i32);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct AdjacencyRuleName(String);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Definition {
    id: ProvinceId,
    r: Red,
    g: Green,
    b: Blue,
    province_type: ProvinceType,
    coastal: Coastal,
    terrain: Terrain,
    continent: Continent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
enum AdjacencyType {
    Impassable,
    Sea,
    River,
    LargeRiver,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Adjacency {
    from: ProvinceId,
    to: ProvinceId,
    adjacency_type: Option<AdjacencyType>,
    through: ProvinceId,
    start_x: XCoord,
    stop_x: XCoord,
    start_y: YCoord,
    stop_y: YCoord,
    adjacency_rule_name: Option<AdjacencyRuleName>,
    comment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Date(String);

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HSV((f32, f32, f32));

impl PartialEq for HSV {
    fn eq(&self, other: &Self) -> bool {
        self.0 .0 == other.0 .0 && self.0 .1 == other.0 .1 && self.0 .2 == other.0 .2
    }
}

impl Eq for HSV {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Season {
    start_date: Date,
    end_date: Date,
    // Applies to northern hemisphere
    hsv_north: HSV,
    colorbalance_north: HSV,
    // Applies to the equator
    hsv_center: HSV,
    colorbalance_center: HSV,
    // Applies to southern hemisphere
    hsv_south: HSV,
    colorbalance_south: HSV,
}
