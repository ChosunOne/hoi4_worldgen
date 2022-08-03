use crate::components::wrappers::ProvinceId;
use crate::MapError;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// The supply nodes on the map
/// Supply Nodes and Railways are defined in `supply_nodes.txt` and `railways.txt` in the map folder.
/// The format of both files is similar, a list of province IDs with each line defining the location
/// of supply nodes and railways. Empty files should not cause any issues, though of course there
/// will be no predefined supply nodes or railways.  
/// The format of `supply_nodes.txt` is a list of province IDs preceded by a 1, one pair per line.
/// For example:
/// ```text
/// 1 123
/// 1 456
/// ```
/// Note also that ports count as supply nodes and that if no supply node is designated in any of a
/// country's states, the capital victory point will be used as a supply node.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SupplyNodes {
    /// The supply nodes
    pub nodes: HashSet<ProvinceId>,
}

impl SupplyNodes {
    /// Reads the supply nodes from the map folder
    /// # Errors
    /// If the file cannot be read, an error is returned.
    #[inline]
    pub fn from_file(path: &Path) -> Result<Self, MapError> {
        let data = fs::read_to_string(path)?;
        let supply_nodes = data.parse()?;
        Ok(supply_nodes)
    }
}

impl FromStr for SupplyNodes {
    type Err = MapError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nodes = HashSet::new();

        for line in s.lines() {
            let parts = line.trim().split(' ').collect::<Vec<_>>();
            let one = parts
                .get(0)
                .ok_or_else(|| MapError::InvalidSupplyNode(line.to_owned()))?;
            if parts.len() != 2 || *one != "1" {
                return Err(MapError::InvalidSupplyNode(line.to_owned()));
            }
            let province_id = parts
                .get(1)
                .ok_or_else(|| MapError::InvalidSupplyNode(line.to_owned()))?
                .parse()?;
            nodes.insert(province_id);
        }

        Ok(Self { nodes })
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
    fn it_reads_supply_nodes_from_a_file() {
        let path = Path::new("./test/map/supply_nodes.txt");
        let supply_nodes = SupplyNodes::from_file(path).expect("Failed to read supply nodes");
        assert_eq!(supply_nodes.nodes.len(), 1049);
        assert!(supply_nodes.nodes.contains(&ProvinceId(15116)));
        assert!(supply_nodes.nodes.contains(&ProvinceId(6603)));
    }
}
