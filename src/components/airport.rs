use crate::components::wrappers::{ProvinceId, StateId};
use crate::MapError;
use jomini::TextTape;
use std::collections::HashMap;
use std::str::FromStr;

/// The list of airports in each state
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Airports {
    /// The airports by state
    pub airports: HashMap<StateId, Vec<ProvinceId>>,
}

impl FromStr for Airports {
    type Err = MapError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut airports = Airports {
            airports: HashMap::new(),
        };

        for line in s.lines() {
            let tape = TextTape::from_slice(line.as_bytes())?;
            let reader = tape.windows1252_reader();
            for (key, _op, value) in reader.fields() {
                let state_id = key.read_str().parse::<StateId>()?;
                let province_ids = {
                    let array = value.read_array()?;
                    let mut ids = Vec::new();
                    for id in array.values() {
                        let id_string = id.read_string()?;
                        ids.push(id_string.parse::<ProvinceId>()?);
                    }
                    ids
                };
                airports.airports.insert(state_id, province_ids);
            }
        }

        Ok(airports)
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_reads_the_airports_file() {
        let airports_path = Path::new("./test/airports.txt");
        let airports_data = fs::read_to_string(airports_path).expect("Failed to read airports.txt");
        let airports = airports_data
            .parse::<Airports>()
            .expect("Failed to parse airports.txt");
        assert_eq!(airports.airports.len(), 1388);
        assert_eq!(
            airports.airports.get(&StateId(1371)),
            Some(&vec![ProvinceId(15230)])
        );
    }
}
