use crate::StrategicRegionId;
use serde::{Deserialize, Serialize};

/// The positions for weather effects on the map.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WeatherPositions {
    /// The weather positions
    pub positions: Vec<WeatherPosition>,
}

/// A position for a weather effect.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WeatherPosition {
    /// The strategic region for the effect
    pub id: StrategicRegionId,
    /// The x position on the map
    pub x: f32,
    /// The y position on the map
    pub y: f32,
    /// The z position on the map
    pub z: f32,
    /// The graphics definition to use for the effect
    pub weather_type: WeatherType,
}

/// Whether the effect is big or small
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum WeatherType {
    /// The default effect
    #[serde(rename = "big")]
    Big,
    /// The small effect
    #[serde(rename = "small")]
    Small,
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

    #[test]
    fn it_loads_weather_positions_from_a_file() {
        let positions = WeatherPosition::load_csv("./test/map/weatherpositions.txt", false)
            .expect("Failed to read weather positions");
        let weather_positions = WeatherPositions { positions };
        assert_eq!(weather_positions.positions.len(), 265);
        assert_eq!(weather_positions.positions[0].id, StrategicRegionId(1));
        assert!((weather_positions.positions[0].x - 3339.0).abs() < f32::EPSILON);
        assert!((weather_positions.positions[0].y - 12.2).abs() < f32::EPSILON);
        assert!((weather_positions.positions[0].z - 1519.0).abs() < f32::EPSILON);
        assert_eq!(
            weather_positions.positions[0].weather_type,
            WeatherType::Small
        );
    }
}
