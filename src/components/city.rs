use crate::components::wrappers::{ColorIndex, Distance, MeshId, PixelDensity, PixelStep};
use crate::MapError;
use jomini::{JominiDeserialize, TextDeserializer};
use serde::Serialize;
use std::fs;
use std::path::Path;

/// The graphical information for depicting large cities on the map.
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct Cities {
    /// The path to the `cities.bmp` file.
    pub types_source: Box<Path>,
    /// TODO: Unknown
    pub pixel_step_x: PixelStep,
    /// TODO: Unknown
    pub pixel_step_y: PixelStep,
    /// The city groups
    #[jomini(duplicated)]
    pub city_group: Vec<CityGroup>,
}

/// A city group
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct CityGroup {
    /// Color index in bmp palette
    pub color_index: ColorIndex,
    /// 0.1 # in fraction of pixels. Negative=less dense.
    pub density: PixelDensity,
    /// Should be sorted by distance (growing)
    #[jomini(duplicated)]
    pub building: Vec<BuildingMesh>,
}

/// The meshes to use for an urban area
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct BuildingMesh {
    /// Distance to the edge of urban area (in map pixels)
    pub distance: Distance,
    /// The id of the mesh to use
    pub mesh: Vec<MeshId>,
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::panic)]
#[allow(clippy::default_numeric_fallback)]
#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::DirectlyDeserialize;

    #[test]
    fn it_loads_cities_from_a_file() {
        let cities_path = Path::new("./test/map/cities.txt");
        let cities = Cities::load_object(&cities_path).expect("Failed to read cities");
        assert_eq!(
            cities.types_source.to_path_buf(),
            Path::new("map/cities.bmp").to_path_buf()
        );
        assert_eq!(cities.pixel_step_x, PixelStep(2));
        assert_eq!(cities.pixel_step_y, PixelStep(2));
        assert_eq!(cities.city_group.len(), 7);
        assert_eq!(cities.city_group[0].color_index, ColorIndex(0));
        assert_eq!(cities.city_group[0].density, PixelDensity(0.9));
        assert_eq!(cities.city_group[0].building.len(), 6);
        assert_eq!(cities.city_group[0].building[0].distance, Distance(1.0));
        assert_eq!(cities.city_group[0].building[0].mesh.len(), 1);
        assert_eq!(
            cities.city_group[0].building[0].mesh[0],
            MeshId("western_citiy_3_entity".to_owned())
        );
    }
}
