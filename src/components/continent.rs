use crate::components::wrappers::Continent;
use jomini::JominiDeserialize;
use serde::Serialize;

/// The list of continents
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct Continents {
    /// The list of continents
    pub continents: Vec<Continent>,
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
    fn it_reads_continents_from_the_map() {
        let map = DefaultMap::from_file(Path::new("test/default.map"))
            .expect("Failed to read default.map");
        let continents_path = append_dir(&map.continent, "./test");
        let continents_data =
            fs::read_to_string(&continents_path).expect("Failed to read continents.txt");
        let continents =
            TextDeserializer::from_windows1252_slice::<Continents>(continents_data.as_bytes())
                .expect("Failed to deserialize continents.txt");
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
