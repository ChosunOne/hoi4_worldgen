use crate::components::wrappers::Continent;
use crate::MapError;
use jomini::{JominiDeserialize, TextDeserializer};
use serde::Serialize;
use std::fs;
use std::path::Path;

/// The list of continents
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct Continents {
    /// The list of continents
    pub continents: Vec<Continent>,
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::default_map::DefaultMap;
    use crate::{append_dir, DirectlyDeserialize};
    use jomini::TextDeserializer;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_reads_continents_from_the_map() {
        let map = DefaultMap::load_object(Path::new("./test/map/default.map"))
            .expect("Failed to read default.map");
        let continents_path = append_dir(&map.continent, "./test/map");
        let continents =
            Continents::load_object(&continents_path).expect("Failed to read continents");
        assert_eq!(continents.continents.len(), 6);
        assert_eq!(continents.continents[0], Continent("west_coast".to_owned()));
        assert_eq!(
            continents.continents[1],
            Continent("northern_reaches".to_owned())
        );
        assert_eq!(
            continents.continents[2],
            Continent("land_of_titans".to_owned())
        );
        assert_eq!(continents.continents[3], Continent("midwest".to_owned()));
        assert_eq!(continents.continents[4], Continent("east_coast".to_owned()));
        assert_eq!(
            continents.continents[5],
            Continent("caribbean_expanse".to_owned())
        );
    }
}
