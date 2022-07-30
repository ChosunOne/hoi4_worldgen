use crate::components::adjacency::{Adjacencies, AdjacencyRules};
use crate::components::airport::Airports;
use crate::components::continent::Continents;
use crate::components::province::Definitions;
use crate::components::season::Seasons;
use image::RgbImage;

/// All the components needed to represent a map.
#[derive(Debug)]
pub struct Map {
    provinces: RgbImage,
    terrain: RgbImage,
    rivers: RgbImage,
    heightmap: RgbImage,
    trees: RgbImage,
    normal_map: RgbImage,
    cities_map: RgbImage,
    definitions: Definitions,
    continents: Continents,
    adjacency_rules: AdjacencyRules,
    adjacencies: Adjacencies,
    seasons: Seasons,
    tree_indices: Vec<usize>,
    // strategic_regions: StrategicRegions,
    // supply_nodes: SupplyNodes,
    // railways: Railways,
    // buildings: Buildings,
    // cities: Cities,
    // colors: Colors,
    // rocket_sites: RocketSites,
    // unit_stacks: UnitStacks,
    // weather_positions: WeatherPositions,
    airports: Airports,
}
