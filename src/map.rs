use crate::components::prelude::*;
use crate::{LoadObject, MapError};
use image::{open, DynamicImage, Pixel, RgbImage};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle, TermLike};
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::try_join;

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
    #[allow(clippy::integer_arithmetic)]
    pub async fn new<T: TermLike + Clone + 'static>(
        root_path: &Path,
        term: &Option<T>,
    ) -> Result<Self, MapError> {
        let progress = {
            let dt = draw_target(term);
            MultiProgress::with_draw_target(dt)
        };
        let progress_style = ProgressStyle::with_template("{wide_msg}")?;
        let default_path = {
            let mut root_path_buf = root_path.to_path_buf();
            root_path_buf.push("map/default.map");
            root_path_buf
        };
        let default_map = DefaultMap::load_object(default_path)?;

        let provinces_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.provinces,
            term,
        );

        let terrain_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.terrain,
            term,
        );

        let rivers_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.rivers,
            term,
        );

        let heightmap_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.heightmap,
            term,
        );

        let trees_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.tree_definition,
            term,
        );

        let normal_map_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            Path::new("world_normal.bmp"),
            term,
        );

        let cities_map_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            Path::new("cities.bmp"),
            term,
        );

        let (
            provinces_result,
            terrain_result,
            rivers_result,
            heightmap_result,
            trees_result,
            normal_map_result,
            cities_map_result,
        ) = try_join!(
            provinces_handle,
            terrain_handle,
            rivers_handle,
            heightmap_handle,
            trees_handle,
            normal_map_handle,
            cities_map_handle
        )?;
        let provinces = provinces_result?;
        let terrain = terrain_result?;
        let rivers = rivers_result?;
        let heightmap = heightmap_result?;
        let trees = trees_result?;
        let normal_map = normal_map_result?;
        let cities_map = cities_map_result?;

        let verify_images_handle = {
            let provinces_clone = provinces.clone();
            let terrain_clone = terrain.clone();
            let rivers_clone = rivers.clone();
            let heightmap_clone = heightmap.clone();
            let trees_clone = trees.clone();
            let normal_map_clone = normal_map.clone();
            let cities_map_clone = cities_map.clone();
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            tokio::spawn(async move {
                pb.set_message("Verifying images...");
                let result = verify_images(
                    &provinces_clone,
                    &terrain_clone,
                    &rivers_clone,
                    &heightmap_clone,
                    &trees_clone,
                    &normal_map_clone,
                    &cities_map_clone,
                );
                if result.is_err() {
                    error!("Error verifying images");
                }
                pb.finish_with_message("done");
                result
            })
        };

        let definitions_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let terrain_path = {
                let mut root_path_buf = root_path.to_path_buf();
                root_path_buf.push("common/terrain/00_terrain.txt");
                root_path_buf
            };
            let definitions_path = map_file(root_path, &default_map.definitions);
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading definitions and terrain from {} and {}...",
                    definitions_path.display(),
                    terrain_path.display()
                ));
                let result = Definitions::from_files(&definitions_path, &terrain_path);
                if result.is_err() {
                    error!(
                        "Error loading definitions and terrain from {} and {}",
                        definitions_path.display(),
                        terrain_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let continents_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let continent_path = map_file(root_path, &default_map.continent);
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading continents from {}...",
                    continent_path.display()
                ));
                let result = Continents::load_object(&continent_path);
                if result.is_err() {
                    error!("Error loading continents from {}", continent_path.display());
                }
                pb.finish_with_message("done");
                result
            })
        };

        let adjacency_rules_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let adjacency_rules_path = map_file(root_path, &default_map.adjacency_rules);
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading adjacency rules from {}...",
                    adjacency_rules_path.display()
                ));
                let result = AdjacencyRules::from_file(&adjacency_rules_path);
                if result.is_err() {
                    error!(
                        "Error loading adjacency rules from {}",
                        adjacency_rules_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let adjacencies_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let adjacencies_path = map_file(root_path, &default_map.adjacencies);
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading adjacencies from {}...",
                    adjacencies_path.display()
                ));
                let result = Adjacencies::from_file(&adjacencies_path);
                if result.is_err() {
                    error!(
                        "Error loading adjacencies from {}",
                        adjacencies_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let seasons_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let seasons_path = map_file(root_path, &default_map.seasons);
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading seasons from {}...",
                    seasons_path.display()
                ));
                let result = Seasons::load_object(&seasons_path);
                if result.is_err() {
                    error!("Error loading seasons from {}", seasons_path.display());
                }
                pb.finish_with_message("done");
                result
            })
        };

        let tree_indices = default_map.tree;

        let strategic_regions_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let strategic_regions_path = map_file(root_path, Path::new("strategicregions"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading strategic regions from {}...",
                    strategic_regions_path.display()
                ));
                let result = StrategicRegions::from_dir(&strategic_regions_path);
                if result.is_err() {
                    error!(
                        "Error loading strategic regions from {}",
                        strategic_regions_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let supply_nodes_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let supply_nodes_path = map_file(root_path, Path::new("supply_nodes.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading supply nodes from {}...",
                    supply_nodes_path.display()
                ));
                let result = SupplyNodes::from_file(&supply_nodes_path);
                if result.is_err() {
                    error!(
                        "Error loading supply nodes from {}",
                        supply_nodes_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let railways_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let railways_path = map_file(root_path, Path::new("railways.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading railways from {}...",
                    railways_path.display()
                ));
                let result = Railways::from_file(&railways_path);
                if result.is_err() {
                    error!("Error loading railways from {}", railways_path.display());
                }
                pb.finish_with_message("done");
                result
            })
        };

        let buildings_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let types_path = {
                let mut root_path_buf = root_path.to_path_buf();
                root_path_buf.push("common/buildings/00_buildings.txt");
                root_path_buf
            };
            let buildings_path = map_file(root_path, Path::new("buildings.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading buildings and building types from {} and {}...",
                    buildings_path.display(),
                    types_path.display()
                ));
                let result = Buildings::from_files(&types_path, &buildings_path);
                if result.is_err() {
                    error!(
                        "Error loading buildings from {} and {}",
                        buildings_path.display(),
                        types_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let cities_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let cities_path = map_file(root_path, Path::new("cities.txt"));
            tokio::spawn(async move {
                pb.set_message(format!("Loading cities from {}...", cities_path.display()));
                let result = Cities::load_object(&cities_path);
                if result.is_err() {
                    error!("Error loading cities from {}", cities_path.display());
                }
                pb.finish_with_message("done");
                result
            })
        };

        let colors_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let colors_path = map_file(root_path, Path::new("colors.txt"));
            tokio::spawn(async move {
                pb.set_message(format!("Loading colors from {}...", colors_path.display()));
                let result = Colors::load_object(&colors_path);
                if result.is_err() {
                    error!("Error loading colors from {}", colors_path.display());
                }
                pb.finish_with_message("done");
                result
            })
        };

        let rocket_sites_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let rocket_sites_path = map_file(root_path, Path::new("rocketsites.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading rocket sites from {}...",
                    rocket_sites_path.display()
                ));
                let result = RocketSites::from_file(&rocket_sites_path);
                if result.is_err() {
                    error!(
                        "Error loading rocket sites from {}",
                        rocket_sites_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let unit_stacks_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let unit_stacks_path = map_file(root_path, Path::new("unitstacks.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading unit stacks from {}...",
                    unit_stacks_path.display()
                ));
                let result = UnitStacks::from_file(&unit_stacks_path);
                if result.is_err() {
                    error!(
                        "Error loading unit stacks from {}",
                        unit_stacks_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let weather_positions_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let weather_positions_path = map_file(root_path, Path::new("weatherpositions.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading weather positions from {}...",
                    weather_positions_path.display()
                ));
                let result = WeatherPositions::from_file(&weather_positions_path);
                if result.is_err() {
                    error!(
                        "Failed to load weather positions from {}",
                        weather_positions_path.display()
                    );
                }
                pb.finish_with_message("done");
                result
            })
        };

        let airports_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style, term);
            let airports_path = map_file(root_path, Path::new("airports.txt"));
            tokio::spawn(async move {
                pb.set_message(format!(
                    "Loading airports from {}...",
                    airports_path.display()
                ));
                let result = Airports::from_file(&airports_path);
                pb.finish_with_message("done");
                result
            })
        };

        let (
            verify_result,
            definitions_result,
            continents_result,
            adjacency_rules_result,
            adjacencies_result,
            seasons_result,
            strategic_regions_result,
            supply_nodes_result,
            railways_result,
            buildings_result,
            cities_result,
            colors_result,
            rocket_sites_result,
            unit_stacks_result,
            weather_positions_result,
            airports_result,
        ) = try_join!(
            verify_images_handle,
            definitions_handle,
            continents_handle,
            adjacency_rules_handle,
            adjacencies_handle,
            seasons_handle,
            strategic_regions_handle,
            supply_nodes_handle,
            railways_handle,
            buildings_handle,
            cities_handle,
            colors_handle,
            rocket_sites_handle,
            unit_stacks_handle,
            weather_positions_handle,
            airports_handle
        )?;

        verify_result?;
        let definitions = definitions_result?;
        let continents = continents_result?;
        let adjacency_rules = adjacency_rules_result?;
        let adjacencies = adjacencies_result?;
        let seasons = seasons_result?;
        let strategic_regions = strategic_regions_result?;
        let supply_nodes = supply_nodes_result?;
        let railways = railways_result?;
        let buildings = buildings_result?;
        let cities = cities_result?;
        let colors = colors_result?;
        let rocket_sites = rocket_sites_result?;
        let unit_stacks = unit_stacks_result?;
        let weather_positions = weather_positions_result?;
        let airports = airports_result?;

        progress.clear()?;

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

    /// Spawns a thread to load an image
    fn spawn_image_loading_thread<T: TermLike + Clone + 'static>(
        root_path: &Path,
        progress: &MultiProgress,
        progress_style: &ProgressStyle,
        image_path: &Path,
        term: &Option<T>,
    ) -> JoinHandle<Result<RgbImage, MapError>> {
        let path = root_path.to_path_buf();
        let pb = Self::create_map_progress_indicator(progress, progress_style, term);
        let ip = image_path.to_path_buf();
        tokio::spawn(async move {
            pb.set_message(format!("Loading {}", ip.display()));
            let image_result = load_image(&path, &ip);
            if image_result.is_err() {
                error!("Error loading {}", ip.display());
            }
            pb.finish_with_message("done");
            image_result
        })
    }

    /// Creates a map progress indicator
    fn create_map_progress_indicator<T: TermLike + Clone + 'static>(
        progress: &MultiProgress,
        progress_style: &ProgressStyle,
        term: &Option<T>,
    ) -> ProgressBar {
        let dt = draw_target(term);
        let pb = progress
            .add(ProgressBar::new(1))
            .with_style(progress_style.clone());
        pb.set_draw_target(dt);
        pb
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

/// Creates a draw target
fn draw_target<T: TermLike + Clone + Sized + 'static>(term: &Option<T>) -> ProgressDrawTarget {
    let draw_target = term.as_ref().map_or_else(ProgressDrawTarget::stdout, |t| {
        let target: Box<dyn TermLike> = Box::new(t.clone());
        ProgressDrawTarget::term_like(target)
    });
    draw_target
}

#[allow(clippy::expect_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use indicatif::InMemoryTerm;

    #[tokio::test]
    async fn it_loads_a_map() {
        let map = Map::new::<InMemoryTerm>(Path::new("./test"), &None).await;
        assert!(map.is_ok());
    }

    #[tokio::test]
    async fn it_verifies_province_colors() {
        let map = Map::new::<InMemoryTerm>(Path::new("./test"), &None)
            .await
            .expect("Failed to load map");
        map.verify_province_colors()
            .expect("Failed to verify provinces");
    }
}
