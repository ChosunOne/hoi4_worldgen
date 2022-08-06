use crate::components::prelude::*;
use crate::{LoadObject, MapError};
use image::{open, DynamicImage, Pixel, RgbImage};
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// All the components needed to represent a map.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Map {
    /// The provinces.bmp image.
    pub provinces: RgbImage,
    /// The terrain.bmp image
    pub terrain: RgbImage,
    /// The rivers.bmp image
    pub rivers: RgbImage,
    /// The heightmap.bmp image
    pub heightmap: RgbImage,
    /// The trees.bmp image
    pub trees: RgbImage,
    /// The world_normal.bmp image
    /// Remember to invert the Y axis.
    pub normal_map: RgbImage,
    /// The cities.bmp image
    pub cities_map: RgbImage,
    /// The province definitions
    pub definitions: Definitions,
    /// The continent definitions
    pub continents: Continents,
    /// The adjacency rules definitions
    pub adjacency_rules: AdjacencyRules,
    /// The adjacencies between provinces
    pub adjacencies: Adjacencies,
    /// The seasons definitions
    pub seasons: Seasons,
    /// The tree indices
    pub tree_indices: Vec<usize>,
    /// The strategic regions definitions
    pub strategic_regions: StrategicRegions,
    /// The supply nodes on the map
    pub supply_nodes: SupplyNodes,
    /// The railways on the map
    pub railways: Railways,
    /// The buildings on the map
    pub buildings: Buildings,
    /// The graphical information for cities on the map
    pub cities: Cities,
    /// TODO: Unknown
    pub colors: Colors,
    /// The rocket sites on the map
    pub rocket_sites: RocketSites,
    /// The unit stacks on the map
    pub unit_stacks: UnitStacks,
    /// The weather positions on the map
    pub weather_positions: WeatherPositions,
    /// The airports definitions
    pub airports: Airports,
}

impl Map {
    /// Loads a map
    /// # Arguments
    /// * `root_path` - the path to the root Hearts of Iron IV directory
    /// # Errors
    /// * If any of the required files could not be read
    /// * If any of the images are not formatted correctly
    #[inline]
    #[allow(clippy::too_many_lines)]
    pub fn new(root_path: &Path) -> Result<Self, MapError> {
        let default_path = {
            let mut root_path_buf = root_path.to_path_buf();
            root_path_buf.push("map/default.map");
            root_path_buf
        };
        let default_map = DefaultMap::load_object(default_path)?;

        let provinces = load_image(root_path, &default_map.provinces)?;
        let terrain = load_image(root_path, &default_map.terrain)?;
        let rivers = load_image(root_path, &default_map.rivers)?;
        let heightmap = load_image(root_path, &default_map.heightmap)?;
        let trees = load_image(root_path, &default_map.tree_definition)?;
        let normal_map = load_image(root_path, Path::new("world_normal.bmp"))?;
        let cities_map = load_image(root_path, Path::new("cities.bmp"))?;

        verify_images(
            &provinces,
            &terrain,
            &rivers,
            &heightmap,
            &trees,
            &normal_map,
            &cities_map,
        )?;

        let definitions = {
            let terrain_path = {
                let mut root_path_buf = root_path.to_path_buf();
                root_path_buf.push("common/terrain/00_terrain.txt");
                root_path_buf
            };
            let definitions_path = map_file(root_path, &default_map.definitions);
            Definitions::from_files(&definitions_path, &terrain_path)?
        };

        let continents = {
            let continent_path = map_file(root_path, &default_map.continent);
            Continents::load_object(&continent_path)?
        };

        let adjacency_rules = {
            let adjacency_rules_path = map_file(root_path, &default_map.adjacency_rules);
            AdjacencyRules::from_file(&adjacency_rules_path)?
        };

        let adjacencies = {
            let adjacencies_path = map_file(root_path, &default_map.adjacencies);
            Adjacencies::from_file(&adjacencies_path)?
        };

        let seasons = {
            let seasons_path = map_file(root_path, &default_map.seasons);
            Seasons::load_object(&seasons_path)?
        };

        let tree_indices = default_map.tree;

        let strategic_regions = {
            let strategic_regions_path = map_file(root_path, Path::new("strategicregions"));
            StrategicRegions::from_dir(&strategic_regions_path)?
        };

        let supply_nodes = {
            let supply_nodes_path = map_file(root_path, Path::new("supply_nodes.txt"));
            SupplyNodes::from_file(&supply_nodes_path)?
        };

        let railways = {
            let railways_path = map_file(root_path, Path::new("railways.txt"));
            Railways::from_file(&railways_path)?
        };

        let buildings = {
            let types_path = {
                let mut root_path_buf = root_path.to_path_buf();
                root_path_buf.push("common/buildings/00_buildings.txt");
                root_path_buf
            };
            let buildings_path = map_file(root_path, Path::new("buildings.txt"));
            Buildings::from_files(&types_path, &buildings_path)?
        };

        let cities = {
            let cities_path = map_file(root_path, Path::new("cities.txt"));
            Cities::load_object(&cities_path)?
        };

        let colors = {
            let colors_path = map_file(root_path, Path::new("colors.txt"));
            Colors::load_object(&colors_path)?
        };

        let rocket_sites = {
            let rocket_sites_path = map_file(root_path, Path::new("rocketsites.txt"));
            RocketSites::from_file(&rocket_sites_path)?
        };

        let unit_stacks = {
            let unit_stacks_path = map_file(root_path, Path::new("unitstacks.txt"));
            UnitStacks::from_file(&unit_stacks_path)?
        };

        let weather_positions = {
            let weather_positions_path = map_file(root_path, Path::new("weatherpositions.txt"));
            WeatherPositions::from_file(&weather_positions_path)?
        };

        let airports = {
            let airports_path = map_file(root_path, Path::new("airports.txt"));
            Airports::from_file(&airports_path)?
        };

        Ok(Self {
            provinces,
            terrain,
            rivers,
            heightmap,
            trees,
            normal_map,
            cities_map,
            definitions,
            continents,
            adjacency_rules,
            adjacencies,
            seasons,
            tree_indices,
            strategic_regions,
            supply_nodes,
            railways,
            buildings,
            cities,
            colors,
            rocket_sites,
            unit_stacks,
            weather_positions,
            airports,
        })
    }

    /// Verifies the province colors against the provinces image
    /// # Errors
    /// * If the province definitions are not valid
    #[inline]
    pub fn verify_province_colors(&self) -> Result<(), MapError> {
        let mut color_set = HashSet::new();
        color_set.insert((Red(0), Green(0), Blue(0)));
        for pixel in self.provinces.pixels() {
            if let [r, g, b] = pixel.channels() {
                let red = Red(*r);
                let green = Green(*g);
                let blue = Blue(*b);
                color_set.insert((red, green, blue));
            }
        }
        debug!("{} colors found", color_set.len());
        for definition in &self.definitions.definitions {
            let color = (definition.r, definition.g, definition.b);
            if !color_set.contains(&color) {
                return Err(MapError::InvalidProvinceColor(color));
            }
            color_set.remove(&color);
        }
        if !color_set.is_empty() {
            return Err(MapError::IncompleteProvinceDefinitions(
                color_set.into_iter().collect(),
            ));
        }

        Ok(())
    }
}

/// Checks the image sizes and aspect ratios
fn verify_images(
    provinces: &RgbImage,
    terrain: &RgbImage,
    rivers: &RgbImage,
    heightmap: &RgbImage,
    trees: &RgbImage,
    normal_map: &RgbImage,
    cities: &RgbImage,
) -> Result<(), MapError> {
    if provinces.width() != heightmap.width() || provinces.height() != heightmap.height() {
        return Err(MapError::ImageSizeMismatch(
            "provinces map does not match heightmap".to_owned(),
        ));
    }
    if terrain.width() != heightmap.width() || terrain.height() != heightmap.height() {
        return Err(MapError::ImageSizeMismatch(
            "terrain map does not match heightmap".to_owned(),
        ));
    }
    if rivers.width() != heightmap.width() || rivers.height() != heightmap.height() {
        return Err(MapError::ImageSizeMismatch(
            "rivers map does not match heightmap".to_owned(),
        ));
    }
    if cities.width() != heightmap.width() || cities.height() != heightmap.height() {
        return Err(MapError::ImageSizeMismatch(
            "cities map does not match heightmap".to_owned(),
        ));
    }

    let heightmap_aspect_ratio = f64::from(heightmap.width()) / f64::from(heightmap.height());
    let trees_aspect_ratio = f64::from(trees.width()) / f64::from(trees.height());
    if (heightmap_aspect_ratio - trees_aspect_ratio).abs() > 0.01_f64 {
        return Err(MapError::ImageSizeMismatch(
            "heightmap aspect ratio does not match trees aspect ratio".to_owned(),
        ));
    }
    let normal_aspect_ratio = f64::from(normal_map.width()) / f64::from(normal_map.height());
    if (heightmap_aspect_ratio - normal_aspect_ratio).abs() > 0.01_f64 {
        return Err(MapError::ImageSizeMismatch(
            "heightmap aspect ratio does not match normal aspect ratio".to_owned(),
        ));
    }

    Ok(())
}

/// Loads the bmp image and verifies it is in the correct format.
fn load_image(root_path: &Path, image_path: &Path) -> Result<RgbImage, MapError> {
    let image_bmp_path = map_file(root_path, image_path);
    info!("Loading {}", image_bmp_path.display());
    let provinces_bmp: DynamicImage = open(&image_bmp_path)?;
    if let DynamicImage::ImageRgb8(image) = provinces_bmp {
        let is_trees = image_path.display().to_string().contains("trees");
        let is_normal = image_path.display().to_string().contains("world_normal");
        if is_trees || is_normal {
            return Ok(image);
        }
        let is_correct_height = image.height() % 256 == 0;
        let is_correct_width = image.width() % 256 == 0;
        if !is_correct_height || !is_correct_width {
            return Err(MapError::InvalidImageSize(image_bmp_path));
        }
        Ok(image)
    } else {
        Err(MapError::InvalidImageType(image_bmp_path))
    }
}

/// Generates the path to the root/map/ directory
fn map_path(root_path: &Path) -> PathBuf {
    let mut root_path_buf = root_path.to_path_buf();
    root_path_buf.push("map");
    root_path_buf
}

/// Generates a path to a file in the root/map/ directory
fn map_file(root_path: &Path, file_path: &Path) -> PathBuf {
    let mut map_path = map_path(root_path);
    map_path.push(file_path);
    map_path
}

#[allow(clippy::expect_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_loads_a_map() {
        let map = Map::new(Path::new("./test"));
        assert!(map.is_ok());
    }

    #[test]
    fn it_verifies_province_colors() {
        let map = Map::new(Path::new("./test")).expect("Failed to load map");
        map.verify_province_colors()
            .expect("Failed to verify provinces");
    }
}
