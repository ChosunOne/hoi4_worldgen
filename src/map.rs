use crate::components::prelude::*;
use crate::components::state::{State, States};
use crate::{LoadObject, MapDisplayMode, MapError};
use actix::{Actor, AsyncContext, Context, Handler, Message};
use egui::Pos2;
use image::{open, DynamicImage, Pixel, Rgb, RgbImage};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle, TermLike};
use log::{debug, error, info, trace, warn};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;
use tokio::try_join;

/// All the components needed to represent a map.
#[derive(Debug)]
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
    /// The map of strategic regions
    pub strategic_region_map: Option<RgbImage>,
    /// The map of states
    pub state_map: Option<RgbImage>,
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
    /// The map of colors to province ids
    pub provinces_by_color: HashMap<Rgb<u8>, ProvinceId>,
    /// The map of province ids to strategic regions
    pub strategic_regions_by_province: HashMap<ProvinceId, StrategicRegionId>,
    /// The map of state ids to States
    pub states: HashMap<StateId, State>,
    /// The map of province ids to states
    pub states_by_province: HashMap<ProvinceId, StateId>,
    strategic_region_map_handle: Option<JoinHandle<()>>,
    state_map_handle: Option<JoinHandle<()>>,
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
    pub fn new<T: TermLike + Clone + 'static>(
        root_path: &Path,
        term: &Option<T>,
    ) -> Result<Self, MapError> {
        let progress = {
            let dt = draw_target(term);
            let p = MultiProgress::new();
            p.set_draw_target(dt);
            p
        };
        let progress_style = ProgressStyle::with_template("{wide_msg}")?;
        let default_path = {
            let mut root_path_buf = root_path.to_path_buf();
            root_path_buf.push("map/default.map");
            root_path_buf
        };
        let default_map = DefaultMap::load_object(&default_path)?;

        let provinces_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.provinces,
        );

        let terrain_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.terrain,
        );

        let rivers_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.rivers,
        );

        let heightmap_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.heightmap,
        );

        let trees_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            &default_map.tree_definition,
        );

        let normal_map_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            Path::new("world_normal.bmp"),
        );

        let cities_map_handle = Self::spawn_image_loading_thread(
            root_path,
            &progress,
            &progress_style,
            Path::new("cities.bmp"),
        );

        let rt = tokio::runtime::Handle::current();
        let (
            provinces_result,
            terrain_result,
            rivers_result,
            heightmap_result,
            trees_result,
            normal_map_result,
            cities_map_result,
        ) = rt.block_on(async move {
            try_join!(
                provinces_handle,
                terrain_handle,
                rivers_handle,
                heightmap_handle,
                trees_handle,
                normal_map_handle,
                cities_map_handle
            )
        })?;
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
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            tokio::task::spawn_blocking(move || {
                pb.set_message("Verifying images...\n");
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
                pb.finish();
                result
            })
        };

        let definitions_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let terrain_path = {
                let mut root_path_buf = root_path.to_path_buf();
                root_path_buf.push("common/terrain/00_terrain.txt");
                root_path_buf
            };
            let definitions_path = map_file(root_path, &default_map.definitions);
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading definitions and terrain...\n");
                let result = Definitions::from_files(&definitions_path, &terrain_path);
                if result.is_err() {
                    error!(
                        "Error loading definitions and terrain from {} and {}",
                        definitions_path.display(),
                        terrain_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let continents_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let continent_path = map_file(root_path, &default_map.continent);
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading continents...\n");
                let result = Continents::load_object(&continent_path);
                if result.is_err() {
                    error!("Error loading continents from {}", continent_path.display());
                }
                pb.finish();
                result
            })
        };

        let adjacency_rules_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let adjacency_rules_path = map_file(root_path, &default_map.adjacency_rules);
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading adjacency rules...\n");
                let result = AdjacencyRules::from_file(&adjacency_rules_path);
                pb.finish();
                match result {
                    Ok(rules) => Ok(rules),
                    Err(e) => {
                        error!(
                            "Error loading adjacency rules from {}: {:?}",
                            adjacency_rules_path.display(),
                            e
                        );
                        Err(e)
                    }
                }
            })
        };

        let adjacencies_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let adjacencies_path = map_file(root_path, &default_map.adjacencies);
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading adjacencies...\n");
                let result = Adjacencies::from_file(&adjacencies_path);
                if result.is_err() {
                    error!(
                        "Error loading adjacencies from {}",
                        adjacencies_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let seasons_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let seasons_path = map_file(root_path, &default_map.seasons);
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading seasons...\n");
                let result = Seasons::load_object(&seasons_path);
                if result.is_err() {
                    error!("Error loading seasons from {}", seasons_path.display());
                }
                pb.finish();
                result
            })
        };

        let tree_indices = default_map.tree;

        let strategic_regions_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let strategic_regions_path = map_file(root_path, Path::new("strategicregions"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading strategic regions...\n");
                let result = StrategicRegions::from_dir(&strategic_regions_path);
                pb.finish();
                match result {
                    Ok(regions) => Ok(regions),
                    Err(e) => {
                        error!(
                            "Error loading strategic regions from {}: {:?}",
                            strategic_regions_path.display(),
                            e
                        );
                        Err(e)
                    }
                }
            })
        };

        let supply_nodes_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let supply_nodes_path = map_file(root_path, Path::new("supply_nodes.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading supply nodes...\n");
                let result = SupplyNodes::from_file(&supply_nodes_path);
                if result.is_err() {
                    error!(
                        "Error loading supply nodes from {}",
                        supply_nodes_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let railways_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let railways_path = map_file(root_path, Path::new("railways.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading railways...\n");
                let result = Railways::from_file(&railways_path);
                if result.is_err() {
                    error!("Error loading railways from {}", railways_path.display());
                }
                pb.finish();
                result
            })
        };

        let buildings_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let types_path = {
                let mut root_path_buf = root_path.to_path_buf();
                root_path_buf.push("common/buildings/00_buildings.txt");
                root_path_buf
            };
            let buildings_path = map_file(root_path, Path::new("buildings.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading buildings and building types...\n");
                let result = Buildings::from_files(&types_path, &buildings_path);
                if result.is_err() {
                    error!(
                        "Error loading buildings from {} and {}",
                        buildings_path.display(),
                        types_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let cities_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let cities_path = map_file(root_path, Path::new("cities.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading cities...\n");
                let result = Cities::load_object(&cities_path);
                if result.is_err() {
                    error!("Error loading cities from {}", cities_path.display());
                }
                pb.finish();
                result
            })
        };

        let colors_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let colors_path = map_file(root_path, Path::new("colors.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading colors...\n");
                let result = Colors::load_object(&colors_path);
                if result.is_err() {
                    error!("Error loading colors from {}", colors_path.display());
                }
                pb.finish();
                result
            })
        };

        let rocket_sites_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let rocket_sites_path = map_file(root_path, Path::new("rocketsites.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading rocket sites...\n");
                let result = RocketSites::from_file(&rocket_sites_path);
                if result.is_err() {
                    error!(
                        "Error loading rocket sites from {}",
                        rocket_sites_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let unit_stacks_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let unit_stacks_path = map_file(root_path, Path::new("unitstacks.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading unit stacks...\n");
                let result = UnitStacks::from_file(&unit_stacks_path);
                if result.is_err() {
                    error!(
                        "Error loading unit stacks from {}",
                        unit_stacks_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let weather_positions_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let weather_positions_path = map_file(root_path, Path::new("weatherpositions.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading weather positions...\n");
                let result = WeatherPositions::from_file(&weather_positions_path);
                if result.is_err() {
                    error!(
                        "Failed to load weather positions from {}",
                        weather_positions_path.display()
                    );
                }
                pb.finish();
                result
            })
        };

        let airports_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let airports_path = map_file(root_path, Path::new("airports.txt"));
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading airports...\n");
                let result = Airports::from_file(&airports_path);
                if result.is_err() {
                    error!("Failed to load airports from {}", airports_path.display());
                }
                pb.finish();
                result
            })
        };

        let states_handle = {
            let pb = Self::create_map_progress_indicator(&progress, &progress_style);
            let states_path = {
                let mut states = root_path.to_path_buf();
                states.push("history/states");
                states
            };
            tokio::task::spawn_blocking(move || {
                pb.set_message("Loading states...\n");
                let result = States::from_dir(&states_path);
                if result.is_err() {
                    error!("Failed to load states from {}", states_path.display());
                }
                pb.finish();
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
            states_result,
        ) = rt.block_on(async move {
            try_join!(
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
                airports_handle,
                states_handle
            )
        })?;

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
        let states = states_result?.states;

        let provinces_by_color = definitions
            .definitions
            .iter()
            .map(|(id, province)| {
                (
                    Rgb::from([province.r.into(), province.g.into(), province.b.into()]),
                    *id,
                )
            })
            .collect();

        let strategic_regions_by_province = strategic_regions
            .strategic_regions
            .iter()
            .flat_map(|(id, sr)| sr.provinces.iter().map(|p| (*p, *id)).collect::<Vec<_>>())
            .collect();

        let states_by_province = states
            .iter()
            .flat_map(|(id, sr)| sr.provinces.iter().map(|p| (*p, *id)).collect::<Vec<_>>())
            .collect();

        progress.println("Loading map complete")?;
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
            strategic_region_map: None,
            supply_nodes,
            railways,
            buildings,
            cities,
            colors,
            rocket_sites,
            unit_stacks,
            weather_positions,
            airports,
            provinces_by_color,
            strategic_regions_by_province,
            strategic_region_map_handle: None,
            states,
            state_map_handle: None,
            state_map: None,
            states_by_province,
        })
    }

    /// Spawns a thread to load an image
    fn spawn_image_loading_thread(
        root_path: &Path,
        progress: &MultiProgress,
        progress_style: &ProgressStyle,
        image_path: &Path,
    ) -> JoinHandle<Result<RgbImage, MapError>> {
        let path = root_path.to_path_buf();
        let pb = Self::create_map_progress_indicator(progress, progress_style);
        let ip = image_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            pb.set_message(format!("Loading {} \n", ip.display()));
            let image_result = load_image(&path, &ip);
            if image_result.is_err() {
                error!("Error loading {}", ip.display());
            }
            pb.finish();
            image_result
        })
    }

    /// Creates a map progress indicator
    fn create_map_progress_indicator(
        progress: &MultiProgress,
        progress_style: &ProgressStyle,
    ) -> ProgressBar {
        progress
            .add(ProgressBar::new(1))
            .with_style(progress_style.clone())
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
        trace!("{} colors found", color_set.len());
        for definition in self.definitions.definitions.values() {
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

    /// Gets the province id from a given point.
    fn province_id_from_point(&self, point: Pos2) -> Option<ProvinceId> {
        let color = self.provinces.get_pixel(point.x as u32, point.y as u32);
        self.provinces_by_color.get(color).copied()
    }
}

impl Actor for Map {
    type Context = Context<Self>;
}

/// A request to get a `ProvinceId` from a supplied texture uv coordinate
#[derive(Message, Debug)]
#[rtype(result = "Option<ProvinceId>")]
#[non_exhaustive]
pub struct GetProvinceIdFromPoint(pub Pos2);

impl GetProvinceIdFromPoint {
    /// Creates a new request for a province id
    #[inline]
    #[must_use]
    pub const fn new(pos: Pos2) -> Self {
        Self(pos)
    }
}

/// A request to get a `StrategicRegionId` from a supplied texture uv coordinate
#[derive(Message, Debug)]
#[rtype(result = "Option<StrategicRegionId>")]
#[non_exhaustive]
pub struct GetStrategicRegionIdFromPoint(pub Pos2);

impl GetStrategicRegionIdFromPoint {
    /// Creates a new request for a strategic region id
    #[inline]
    #[must_use]
    pub const fn new(pos: Pos2) -> Self {
        Self(pos)
    }
}

/// A request to get a `StrategicRegionId` from a supplied texture uv coordinate
#[derive(Message, Debug)]
#[rtype(result = "Option<StateId>")]
#[non_exhaustive]
pub struct GetStateIdFromPoint(pub Pos2);

impl GetStateIdFromPoint {
    /// Creates a new request for a state id
    #[inline]
    #[must_use]
    pub const fn new(pos: Pos2) -> Self {
        Self(pos)
    }
}

/// A request to get a `Definition` from a supplied `ProvinceId`
#[derive(Message, Debug)]
#[rtype(result = "Option<Definition>")]
#[non_exhaustive]
pub struct GetProvinceDefinitionFromId(pub ProvinceId);

impl GetProvinceDefinitionFromId {
    /// Creates a new request for a province id
    #[inline]
    #[must_use]
    pub const fn new(id: ProvinceId) -> Self {
        Self(id)
    }
}

/// A request to get a `StrategicRegion` from a given `StrategicRegionId`
#[derive(Message, Debug)]
#[rtype(result = "Option<StrategicRegion>")]
#[non_exhaustive]
pub struct GetStrategicRegionFromId(pub StrategicRegionId);

impl GetStrategicRegionFromId {
    /// Creates a new request for a strategic region id
    #[inline]
    #[must_use]
    pub const fn new(id: StrategicRegionId) -> Self {
        Self(id)
    }
}

/// A request to get a `State` from a given `StateId`.
#[derive(Message, Debug)]
#[rtype(result = "Option<State>")]
#[non_exhaustive]
pub struct GetStateFromId(pub StateId);

impl GetStateFromId {
    /// Creates a new request for a state id
    #[inline]
    #[must_use]
    pub const fn new(id: StateId) -> Self {
        Self(id)
    }
}

/// A request to get a `Continent` from a supplied `ContinentIndex`
#[derive(Message, Debug)]
#[rtype(result = "Option<Continent>")]
#[non_exhaustive]
pub struct GetContinentFromIndex(pub ContinentIndex);

impl GetContinentFromIndex {
    /// Creates a new request for a province id
    #[inline]
    #[must_use]
    pub const fn new(index: ContinentIndex) -> Self {
        Self(index)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct GenerateStrategicRegionMap;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct GenerateStateMap;

#[derive(Message)]
#[rtype(result = "()")]
struct UpdateStrategicRegionMap(RgbImage);

#[derive(Message)]
#[rtype(result = "()")]
struct UpdateStateMap(RgbImage);

/// A request to get an `RgbImage` from a supplied `MapDisplayMode`
#[allow(clippy::exhaustive_enums)]
#[derive(Message, Debug)]
#[rtype(result = "Option<RgbImage>")]
pub enum GetMapImage {
    HeightMap,
    Terrain,
    Provinces,
    Rivers,
    StrategicRegions,
    States,
}

impl From<MapDisplayMode> for GetMapImage {
    #[inline]
    fn from(mode: MapDisplayMode) -> Self {
        match mode {
            MapDisplayMode::HeightMap => Self::HeightMap,
            MapDisplayMode::Terrain => Self::Terrain,
            MapDisplayMode::Provinces => Self::Provinces,
            MapDisplayMode::Rivers => Self::Rivers,
            MapDisplayMode::StrategicRegions => Self::StrategicRegions,
            MapDisplayMode::States => Self::States,
        }
    }
}

impl Handler<GetMapImage> for Map {
    type Result = Option<RgbImage>;

    #[inline]
    fn handle(&mut self, msg: GetMapImage, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            GetMapImage::HeightMap => Some(self.heightmap.clone()),
            GetMapImage::Terrain => Some(self.terrain.clone()),
            GetMapImage::Provinces => Some(self.provinces.clone()),
            GetMapImage::Rivers => Some(self.rivers.clone()),
            GetMapImage::StrategicRegions => self.strategic_region_map.clone(),
            GetMapImage::States => self.state_map.clone(),
        }
    }
}

impl Handler<GetProvinceIdFromPoint> for Map {
    type Result = Option<ProvinceId>;

    #[inline]
    fn handle(&mut self, msg: GetProvinceIdFromPoint, _ctx: &mut Context<Self>) -> Self::Result {
        let point = msg.0;
        self.province_id_from_point(point)
    }
}

impl Handler<GetStrategicRegionIdFromPoint> for Map {
    type Result = Option<StrategicRegionId>;
    #[inline]
    fn handle(
        &mut self,
        msg: GetStrategicRegionIdFromPoint,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        let point = msg.0;
        if self.strategic_region_map.is_some() {
            let color = self.provinces.get_pixel(point.x as u32, point.y as u32);
            let province_id = self.provinces_by_color.get(color).copied();
            if let Some(id) = province_id {
                return self.strategic_regions_by_province.get(&id).copied();
            }
        }

        None
    }
}

impl Handler<GetStateIdFromPoint> for Map {
    type Result = Option<StateId>;

    #[inline]
    fn handle(&mut self, msg: GetStateIdFromPoint, _ctx: &mut Self::Context) -> Self::Result {
        let point = msg.0;
        if self.state_map.is_some() {
            let color = self.provinces.get_pixel(point.x as u32, point.y as u32);
            let province_id = self.provinces_by_color.get(color).copied();
            if let Some(id) = province_id {
                return self.states_by_province.get(&id).copied();
            }
        }
        None
    }
}

impl Handler<GetStrategicRegionFromId> for Map {
    type Result = Option<StrategicRegion>;
    #[inline]
    fn handle(&mut self, msg: GetStrategicRegionFromId, _ctx: &mut Context<Self>) -> Self::Result {
        self.strategic_regions
            .strategic_regions
            .get(&msg.0)
            .cloned()
    }
}

impl Handler<GetStateFromId> for Map {
    type Result = Option<State>;
    #[inline]
    fn handle(&mut self, msg: GetStateFromId, _ctx: &mut Context<Self>) -> Self::Result {
        self.states.get(&msg.0).cloned()
    }
}

impl Handler<GetProvinceDefinitionFromId> for Map {
    type Result = Option<Definition>;

    #[inline]
    fn handle(
        &mut self,
        msg: GetProvinceDefinitionFromId,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        self.definitions.definitions.get(&msg.0).cloned()
    }
}

impl Handler<GetContinentFromIndex> for Map {
    type Result = Option<Continent>;

    #[inline]
    fn handle(&mut self, msg: GetContinentFromIndex, _ctx: &mut Context<Self>) -> Self::Result {
        let index = msg.0;
        if index.0 < 1 {
            return None;
        }
        self.continents.continents.get(index.0 - 1).cloned()
    }
}

impl Handler<GenerateStrategicRegionMap> for Map {
    type Result = ();

    #[inline]
    fn handle(
        &mut self,
        _msg: GenerateStrategicRegionMap,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        if self.strategic_region_map.is_some() {
            return;
        }
        let strategic_regions = self.strategic_regions.strategic_regions.clone();
        let provinces = self.provinces.clone();
        let provinces_by_color = self.provinces_by_color.clone();
        let definitions = self.definitions.definitions.clone();
        let strategic_regions_by_province = self.strategic_regions_by_province.clone();
        let self_addr = ctx.address();
        let strategic_region_map_handle = tokio::task::spawn_blocking(move || {
            match generate_region_map(
                &strategic_regions,
                &provinces,
                &provinces_by_color,
                &definitions,
                &strategic_regions_by_province,
            ) {
                Ok(m) => {
                    if let Err(e) = self_addr.try_send(UpdateStrategicRegionMap(m)) {
                        error!("Failed to send strategic region map update: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to generate strategic region map: {:?}", e);
                }
            }
        });

        self.strategic_region_map_handle = Some(strategic_region_map_handle);
    }
}

impl Handler<UpdateStrategicRegionMap> for Map {
    type Result = ();

    #[inline]
    fn handle(&mut self, msg: UpdateStrategicRegionMap, _ctx: &mut Self::Context) -> Self::Result {
        self.strategic_region_map = Some(msg.0);
        self.strategic_region_map_handle.take();
    }
}

impl Handler<GenerateStateMap> for Map {
    type Result = ();

    #[inline]
    fn handle(&mut self, _msg: GenerateStateMap, ctx: &mut Self::Context) -> Self::Result {
        if self.state_map.is_some() {
            return;
        }
        let states = self.states.clone();
        let provinces = self.provinces.clone();
        let provinces_by_color = self.provinces_by_color.clone();
        let definitions = self.definitions.definitions.clone();
        let states_by_province = self.states_by_province.clone();
        let self_addr = ctx.address();
        let state_map_handle = tokio::task::spawn_blocking(move || {
            match generate_region_map(
                &states,
                &provinces,
                &provinces_by_color,
                &definitions,
                &states_by_province,
            ) {
                Ok(m) => {
                    if let Err(e) = self_addr.try_send(UpdateStateMap(m)) {
                        error!("Failed to send state map update: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to generate state map: {:?}", e);
                }
            }
        });

        self.state_map_handle = Some(state_map_handle);
    }
}

impl Handler<UpdateStateMap> for Map {
    type Result = ();

    #[inline]
    fn handle(&mut self, msg: UpdateStateMap, _ctx: &mut Self::Context) -> Self::Result {
        self.state_map = Some(msg.0);
        self.state_map_handle.take();
    }
}

/// Generates an `RgbImage` from the regions
/// # Errors
/// * If the regions are not valid
#[inline]
fn generate_region_map<RegionId: Copy + Eq + Hash, Region>(
    regions: &HashMap<RegionId, Region>,
    provinces: &RgbImage,
    provinces_by_color: &HashMap<Rgb<u8>, ProvinceId>,
    definitions: &HashMap<ProvinceId, Definition>,
    regions_by_province: &HashMap<ProvinceId, RegionId>,
) -> Result<RgbImage, MapError> {
    let region_colors = {
        let mut rng = thread_rng();
        regions
            .keys()
            .copied()
            .map(|id| {
                let r = rng.gen();
                let g = rng.gen();
                let b = rng.gen();
                let color = Rgb::<u8>::from([r, g, b]);
                (id, color)
            })
            .collect::<HashMap<_, _>>()
    };
    let mut region_map = RgbImage::new(provinces.width(), provinces.height());
    for (x, y, pixel) in provinces.enumerate_pixels() {
        let province_id = provinces_by_color.get(pixel).ok_or_else(|| {
            MapError::InvalidProvinceColor((Red(pixel.0[0]), Green(pixel.0[1]), Blue(pixel.0[2])))
        })?;
        let province = definitions
            .get(province_id)
            .ok_or(MapError::DefinitionNotFound(*province_id))?;
        let region_id = regions_by_province.get(&province.id);
        let color = region_id.map_or(Rgb::<u8>::from([0, 0, 0]), |rid| {
            *region_colors
                .get(rid)
                .expect("Regions are inconsistent with assigned colors")
        });
        region_map.put_pixel(x, y, color);
    }
    Ok(region_map)
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
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use indicatif::InMemoryTerm;

    #[test]
    fn it_loads_a_map() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let handle = rt.spawn_blocking(|| Map::new::<InMemoryTerm>(Path::new("./test"), &None));
        let map = rt.block_on(handle).unwrap();
        assert!(map.is_ok());
    }

    #[test]
    fn it_verifies_province_colors() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let handle = rt.spawn_blocking(|| Map::new::<InMemoryTerm>(Path::new("./test"), &None));
        let map = rt.block_on(handle).unwrap().expect("Failed to load map");
        map.verify_province_colors()
            .expect("Failed to verify provinces");
    }
}
