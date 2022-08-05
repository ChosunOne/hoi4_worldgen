use jomini::{JominiDeserialize, TextDeserializer};
use std::error::Error;
use std::fs;
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

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{append_dir, LoadObject};
    use image::{open, DynamicImage};

    #[test]
    fn it_reads_a_default_map_file() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read map");
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
    fn it_loads_provinces_bmp_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read default.map");
        let provinces_bmp_path = append_dir(&map.provinces, "./test/map");
        let provinces_bmp: DynamicImage =
            open(&provinces_bmp_path).expect("Failed to read provinces.bmp");
        match provinces_bmp {
            DynamicImage::ImageRgb8(image) => {
                assert_eq!(image.width(), 5632);
                assert_eq!(image.height(), 2304);
            }
            _ => panic!("Failed to read provinces.bmp"),
        }
    }

    #[test]
    fn it_reads_terrain_bmp_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read default.map");
        let terrain_bmp_path = append_dir(&map.terrain, "./test/map");
        let terrain_bmp: DynamicImage =
            open(&terrain_bmp_path).expect("Failed to read terrain.bmp");
        match terrain_bmp {
            DynamicImage::ImageRgb8(image) => {
                assert_eq!(image.width(), 5632);
                assert_eq!(image.height(), 2304);
            }
            _ => panic!("Failed to read terrain.bmp"),
        }
    }

    #[test]
    fn it_reads_rivers_bmp_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read default.map");
        let rivers_bmp_path = append_dir(&map.rivers, "./test/map");
        let rivers_bmp: DynamicImage = open(&rivers_bmp_path).expect("Failed to read rivers.bmp");
        match rivers_bmp {
            DynamicImage::ImageRgb8(image) => {
                assert_eq!(image.width(), 5632);
                assert_eq!(image.height(), 2304);
            }
            _ => panic!("Failed to read rivers.bmp"),
        }
    }

    #[test]
    fn it_reads_heightmap_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read default.map");
        let heightmap_bmp_path = append_dir(&map.heightmap, "./test/map");
        let heightmap_bmp: DynamicImage =
            open(&heightmap_bmp_path).expect("Failed to read heightmap.bmp");
        match heightmap_bmp {
            DynamicImage::ImageRgb8(image) => {
                assert_eq!(image.width(), 5632);
                assert_eq!(image.height(), 2304);
            }
            _ => panic!("Failed to read heightmap.bmp"),
        }
    }

    #[test]
    fn it_reads_trees_bmp_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read default.map");
        let tree_bmp_path = append_dir(&map.tree_definition, "./test/map");
        let tree_bmp: DynamicImage = open(&tree_bmp_path).expect("Failed to read trees.bmp");
        match tree_bmp {
            DynamicImage::ImageRgb8(image) => {
                assert_eq!(image.width(), 1650);
                assert_eq!(image.height(), 675);
            }
            _ => panic!("Failed to read trees.bmp"),
        }
    }
}
