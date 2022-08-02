use crate::components::adjacency::{Adjacencies, AdjacencyRules};
use crate::components::airport::Airports;
use crate::components::continent::Continents;
use crate::components::province::Definitions;
use crate::components::railway::Railways;
use crate::components::season::Seasons;
use crate::components::strategic_region::StrategicRegions;
use crate::components::supply_node::SupplyNodes;
use image::RgbImage;

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
    /// The worldnormal.bmp image
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
    // buildings: Buildings,
    // cities: Cities,
    // colors: Colors,
    // rocket_sites: RocketSites,
    // unit_stacks: UnitStacks,
    // weather_positions: WeatherPositions,
    /// The airports definitions
    pub airports: Airports,
}
