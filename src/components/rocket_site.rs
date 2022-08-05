use crate::{load_state_map, MapError, ProvinceId, StateId};
use std::collections::HashMap;
use std::path::Path;

/// The rocket sites on the map
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RocketSites {
    /// The rocket sites by state
    pub rocket_sites: HashMap<StateId, Vec<ProvinceId>>,
}

impl RocketSites {
    /// Loads the rocket sites from the given path.
    /// # Errors
    /// If the file cannot be read, or if it is invalid.
    #[inline]
    pub fn from_file(path: &Path) -> Result<Self, MapError> {
        let rocket_sites = load_state_map(path)?;
        Ok(Self { rocket_sites })
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn it_reads_the_rocket_sites_file() {
        let rocket_sites = RocketSites::from_file(Path::new("./test/map/rocketsites.txt"))
            .expect("Failed to read rocket_sites.txt");
        assert_eq!(rocket_sites.rocket_sites.len(), 1388);
        assert_eq!(
            rocket_sites.rocket_sites.get(&StateId(1371)),
            Some(&vec![ProvinceId(15230)])
        );
    }
}
