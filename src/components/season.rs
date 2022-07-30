use crate::components::wrappers::Hsv;
use jomini::common::Date;
use jomini::JominiDeserialize;
use serde::Serialize;

/// Defines the color adjustment for a season.
#[derive(Debug, Clone, PartialEq, Eq, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct Season {
    /// The starting date of the season.
    /// Format is 00.\<month\>.\<day\>  
    /// Ex. 00.12.01
    pub start_date: Date,
    /// The ending date of the season.
    pub end_date: Date,
    /// Applies HSV to northern hemisphere
    pub hsv_north: Hsv,
    /// Applies colorbalance to northern hemisphere
    pub colorbalance_north: Hsv,
    /// Applies HSV to the equator
    pub hsv_center: Hsv,
    /// Applies colorbalance to the equator
    pub colorbalance_center: Hsv,
    /// Applies HSV to southern hemisphere
    pub hsv_south: Hsv,
    /// Applies colorbalance to southern hemisphere
    pub colorbalance_south: Hsv,
}

/// The dates when to show the seasons on the trees.
#[derive(Debug, Clone, PartialEq, Eq, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct TreeSeason {
    /// The starting date
    pub start_date: Date,
    /// The ending date
    pub end_date: Date,
}

/// The season definitions
#[derive(Debug, Clone, PartialEq, Eq, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct Seasons {
    /// Winter
    pub winter: Season,
    /// Spring
    pub spring: Season,
    /// Summer
    pub summer: Season,
    /// Autumn
    pub autumn: Season,
    /// Primary winter tree
    pub tree_winter: TreeSeason,
    /// Secondary winter tree
    pub tree_winter2: TreeSeason,
    /// Primary spring tree
    pub tree_spring: TreeSeason,
    /// Secondary spring tree
    pub tree_spring2: TreeSeason,
    /// Primary summer tree
    pub tree_summer: TreeSeason,
    /// Secondary summer tree
    pub tree_summer2: TreeSeason,
    /// Primary autumn tree
    pub tree_autumn: TreeSeason,
    /// Secondary autumn tree
    pub tree_autumn2: TreeSeason,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::append_dir;
    use crate::components::default_map::DefaultMap;
    use jomini::TextDeserializer;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_loads_seasons_from_the_map() {
        let map = DefaultMap::from_file(Path::new("test/default.map"))
            .expect("Failed to read default.map");
        let seasons_path = append_dir(&map.seasons, "./test");
        let seasons_data = fs::read_to_string(&seasons_path).expect("Failed to read seasons.txt");
        let seasons = TextDeserializer::from_windows1252_slice::<Seasons>(seasons_data.as_bytes())
            .expect("Failed to deserialize seasons.txt");
        assert_eq!(
            seasons.winter,
            Season {
                start_date: Date::from_ymd(0, 12, 1),
                end_date: Date::from_ymd(0, 2, 28),
                hsv_north: Hsv((0.0, 0.4, 0.7)),
                colorbalance_north: Hsv((0.8, 0.8, 1.1)),
                hsv_center: Hsv((0.0, 0.85, 1.0)),
                colorbalance_center: Hsv((1.1, 1.0, 1.0)),
                hsv_south: Hsv((0.0, 0.85, 1.0)),
                colorbalance_south: Hsv((1.1, 1.0, 1.0)),
            }
        );
        assert_eq!(
            seasons.tree_spring2,
            TreeSeason {
                start_date: Date::from_ymd(0, 3, 20),
                end_date: Date::from_ymd(0, 4, 20),
            }
        );
    }
}
