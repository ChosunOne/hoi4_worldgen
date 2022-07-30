use crate::{Adjacencies, AdjacencyRules, Airports, Continents, Definitions, Seasons};
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
