use crate::components::wrappers::{BuildingId, ProvinceId, StateId};
use crate::{LoadCsv, LoadKeys, MapError};
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// The locations of building models for each state are defined in
/// `/Hearts of Iron IV/map/buildings.txt`. An entry in that file is defined as such (If
/// unspecified, assume a number with up to 2 decimal digits):  
/// ```csv
/// State ID (integer); building ID (string); X position; Y position; Z position; Rotation; Adjacent sea province (integer)
/// ```
/// * State ID defines which state the building is located in. Even for provincial buildings, this
/// is the ID of the state, not the province.
/// * Building ID is defines which model is being located. While this includes each building, this
/// also includes floating harbours as `floating_harbor`.
/// * X, Y, and Z position represent the position on the map of the building model. The X and Z
/// positions are equal to the X and Y axes on the province bitmap with 1 pixel equalling 1 unit,
/// left-to-right and down-to-up respectively. This is also what the game uses to know which province
/// it's for for provincial buildings. The Y position, on the scale of 0 to 25.5, can be calculated
/// with the heightmap by taking the value of the pixel at that position and making it fit on the
/// scale of 0 to 25.5 (Such as by dividing it by 10 if it's on the scale of 0 to 255).
/// * Rotation is measured in radians. A rotation of 0 will result in the building model pointing in
/// the same direction as the model is set, while positives will rotate it counter-clockwise and
/// negatives will rotate it clockwise. A full rotation resulting in the same position as 0 is equal
/// to the number π multiplied by 2, roughly 6.28.
/// * Adjacent sea province is only necessary to define for naval bases and floating harbours, in
/// order to let the game know from which sea province ships or convoys can access the land province
/// where it is located. If the building type is not a naval base, it should be left at 0.  
/// It is preferable to generate the building models in the building section in the nudger, rather
/// than filling it out manually. However, note that the game will crash if the currently-existing
/// `/Hearts of Iron IV/map/buildings.txt` file is entirely empty, so there should be at least one
/// definition, even if incorrect.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StateBuilding {
    /// The state that the building is located in.
    pub state_id: StateId,
    /// The type of building
    pub building_id: BuildingId,
    /// The X position of the building model
    pub x: f32,
    /// The Y position of the building model
    pub y: f32,
    /// The Z position of the building model
    pub z: f32,
    /// The rotation of the building model in radians
    pub rotation: f32,
    /// The ID of the adjacent sea province, if any
    pub adjacent_sea_province: ProvinceId,
}

/// The buildings on the map
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Buildings {
    /// The building types
    pub types: HashSet<BuildingId>,
    /// The buildings
    pub buildings: Vec<StateBuilding>,
}

impl Buildings {
    /// Creates a new `BuildingTypes` from a file
    /// # Errors
    /// If the file cannot be read, or if it is invalid, returns an error.
    #[inline]
    pub fn from_files(types_path: &Path, buildings_path: &Path) -> Result<Self, MapError> {
        let mut types = BuildingId::load_keys(types_path, "buildings")?;
        // Floating harbors appear to be a building type that is hard coded into the game.
        types.insert(BuildingId("floating_harbor".to_owned()));
        let raw_buildings = StateBuilding::load_csv(buildings_path, false)?;

        // Verify that all building ids are defined in types
        for building in &raw_buildings {
            if !types.contains(&building.building_id) {
                warn!(
                    "BuildingId {:?} is not defined in types",
                    building.building_id
                );
            }
        }

        let buildings = raw_buildings
            .into_iter()
            .filter(|b| types.contains(&b.building_id))
            .collect();

        Ok(Self { types, buildings })
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::panic)]
#[allow(clippy::default_numeric_fallback)]
#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_buildings_from_files() {
        let types_path = Path::new("./test/common/buildings/00_buildings.txt");
        let buildings_path = Path::new("./test/map/buildings.txt");
        let buildings = Buildings::from_files(types_path, buildings_path)
            .expect("Failed to read building types");
        assert_eq!(buildings.types.len(), 17);
        assert!(buildings
            .types
            .contains(&BuildingId("circuitry_generator".to_owned())));
        assert_eq!(buildings.buildings.len(), 47522);
        assert_eq!(
            buildings.buildings[12].building_id,
            BuildingId("coastal_bunker".to_owned())
        );
        assert!((buildings.buildings[12].x - 1672.0_f32).abs() < f32::EPSILON);
        assert!((buildings.buildings[12].y - 9.68_f32).abs() < f32::EPSILON);
        assert!((buildings.buildings[12].z - 1559.0_f32).abs() < f32::EPSILON);
        assert!((buildings.buildings[12].rotation - -3.93_f32).abs() < f32::EPSILON);
        assert_eq!(buildings.buildings[12].adjacent_sea_province, ProvinceId(0));
    }
}
