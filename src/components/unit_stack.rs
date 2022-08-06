use crate::components::wrappers::ModelIndex;
use crate::{LoadCsv, MapError, ProvinceId};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The unit stack information for displaying units on the map.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnitStacks {
    /// The unit stacks
    pub stacks: Vec<UnitStack>,
}

/// A unit stack
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnitStack {
    /// The province ID
    pub province_id: ProvinceId,
    /// The model index
    pub model_index: ModelIndex,
    /// The x offset
    pub x: f32,
    /// The y offset
    pub y: f32,
    /// The z offset
    pub z: f32,
    /// This is a guess, perhaps rotation?
    rotation: f32,
    /// This is a guess, perhaps scale?
    scale: f32,
}

impl UnitStacks {
    /// Loads the `UnitStacks` from a given path
    /// # Errors
    /// If the file cannot be read, or if it is invalid
    #[inline]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MapError> {
        let stacks = UnitStack::load_csv(path, false)?;
        Ok(Self { stacks })
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
    use crate::LoadCsv;
    use std::path::Path;

    #[test]
    fn it_loads_unit_stacks_from_file() {
        let unit_stacks_path = Path::new("./test/map/unitstacks.txt");
        let unit_stacks =
            UnitStacks::from_file(unit_stacks_path).expect("Failed to load unit stacks");
        assert_eq!(unit_stacks.stacks.len(), 307_834);
        assert_eq!(unit_stacks.stacks[307_592].province_id, ProvinceId(16765));
        assert_eq!(unit_stacks.stacks[307_592].model_index, ModelIndex(38));
        assert!((unit_stacks.stacks[307_592].x - 3272.88).abs() < f32::EPSILON);
        assert!((unit_stacks.stacks[307_592].y - 9.5).abs() < f32::EPSILON);
        assert!((unit_stacks.stacks[307_592].z - 939.0).abs() < f32::EPSILON);
        assert!((unit_stacks.stacks[307_592].rotation - -1.57).abs() < f32::EPSILON);
        assert!((unit_stacks.stacks[307_592].scale - 0.28).abs() < f32::EPSILON);
    }
}
