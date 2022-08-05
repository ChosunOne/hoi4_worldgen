use crate::components::wrappers::{ProvinceId, StateId};
use crate::{load_state_map, MapError};
use std::collections::HashMap;
use std::path::Path;

/// The list of airports in each state
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Airports {
    /// The airports by state
    pub airports: HashMap<StateId, Vec<ProvinceId>>,
}

impl Airports {
    /// Loads the airports from the given path.
    /// # Errors
    /// If the file cannot be read, or if it is invalid.
    #[inline]
    pub fn from_file(path: &Path) -> Result<Self, MapError> {
        let airports = load_state_map(path)?;
        Ok(Self { airports })
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn it_reads_the_airports_file() {
        let airports = Airports::from_file(Path::new("./test/map/airports.txt"))
            .expect("Failed to read airports.txt");
        assert_eq!(airports.airports.len(), 1388);
        assert_eq!(
            airports.airports.get(&StateId(1371)),
            Some(&vec![ProvinceId(15230)])
        );
    }
}
