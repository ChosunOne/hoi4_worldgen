use crate::components::wrappers::{ProvinceId, RailLevel};
use crate::MapError;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// A railway.
/// The format of railways.txt is a list of province IDs preceded by the level of the railway (1-5),
/// and the number of provinces in the railway, each line representing one railway. For example:
/// ```text
/// 1 3 10 21 32
/// 2 4 43 54 65 78
/// ```
/// It's important that the provinces are listed in order from beginning to end of the railway and
/// that they are adjacent. Also, if the same provinces are listed between two railways, the levels
/// are added together.  
// Rivers can act as supply routes, as long as there is a supply node (or port) in a province
// adjacent to the river.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Railway {
    /// The level of the railway
    pub level: RailLevel,
    /// The length of the railway
    pub length: usize,
    /// The provinces that are connected by this railway
    pub provinces: Vec<ProvinceId>,
}

impl FromStr for Railway {
    type Err = MapError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<_>>();
        let level = parts
            .get(0)
            .ok_or_else(|| MapError::InvalidRailway(s.to_owned()))?
            .parse::<RailLevel>()?;
        let length = parts
            .get(1)
            .ok_or_else(|| MapError::InvalidRailway(s.to_owned()))?
            .parse::<usize>()?;
        let provinces = parts
            .iter()
            .skip(2)
            .flat_map(|s| s.parse::<ProvinceId>())
            .collect::<Vec<_>>();
        if length != provinces.len() {
            return Err(MapError::InvalidRailway(s.to_owned()));
        }
        Ok(Self {
            level,
            length,
            provinces,
        })
    }
}

/// The collection of railways on the map.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Railways {
    /// The railways
    pub railways: Vec<Railway>,
}

impl Railways {
    /// Reads the railways from the map folder
    /// # Errors
    /// If the file cannot be read, an error is returned.
    #[inline]
    pub fn from_file(path: &Path) -> Result<Self, MapError> {
        let data = fs::read_to_string(path)?;
        let railways = data.parse()?;
        Ok(railways)
    }
}

impl FromStr for Railways {
    type Err = MapError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let railways = s.lines().flat_map(str::parse).collect();
        Ok(Self { railways })
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
    use std::path::Path;

    #[test]
    fn it_reads_railways_from_a_file() {
        let path = Path::new("./test/railways.txt");
        let railways = Railways::from_file(path).expect("Failed to read railways");
        assert_eq!(railways.railways.len(), 1520);
    }
}
