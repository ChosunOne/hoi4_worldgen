use crate::components::wrappers::{AdjacencyRuleName, Icon, ProvinceId, XCoord, YCoord};
use jomini::JominiDeserialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, JominiDeserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct AdjacencyRule {
    /// The name of the adjacency rule.
    pub name: AdjacencyRuleName,
    /// The logic for when the adjacency is contested.
    pub contested: AdjacencyLogic,
    /// The logic for when the adjacency is controlled by an enemy.
    pub enemy: AdjacencyLogic,
    /// The logic for when the adjacency is controlled by a friend.
    pub friend: AdjacencyLogic,
    /// The logic for when the adjacency is controlled by a neutral.
    pub neutral: AdjacencyLogic,
    /// The provinces for which the rule applies.
    pub required_provinces: Vec<ProvinceId>,
    /// The icon for the adjacency rule.
    pub icon: Icon,
    /// Graphical offsets
    pub offset: Vec<i32>,
    /// Conditions when the rule can be disabled.
    pub is_disabled: Option<IsDisabled>,
}

/// An adjacency rule
#[derive(Clone, Debug, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct RawAdjacencyRules {
    /// The info of the adjacency rule.
    #[jomini(duplicated)]
    adjacency_rule: Vec<AdjacencyRule>,
}

/// Conditions when an adjacency rule can be disabled
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct IsDisabled {
    /// The tooltip to display when the rule is disabled.
    pub tooltip: String,
}

/// The logic for the adjacency rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct AdjacencyLogic {
    /// Whether armies can pass
    pub army: bool,
    /// Whether fleets can pass
    pub navy: bool,
    /// Whether subs can pass
    pub submarine: bool,
    /// Whether trade can pass
    pub trade: bool,
}

/// The Adjacency type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AdjacencyType {
    /// The adjacent province cannot be reached from this province
    #[serde(rename = "impassable")]
    Impassable,
    /// The adjacent province is a sea province
    #[serde(rename = "sea")]
    Sea,
    /// The adjacent province is bordered by a river
    #[serde(rename = "river")]
    River,
    /// The adjacent province is bordered by a large river
    #[serde(rename = "large_river")]
    LargeRiver,
}

/// The type of adjacency between two provinces
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Adjacency {
    /// The ID of the starting province
    #[serde(rename = "From")]
    pub from: ProvinceId,
    /// The ID of the destination province
    #[serde(rename = "To")]
    pub to: ProvinceId,
    /// The type of adjacency
    #[serde(rename = "Type")]
    pub adjacency_type: Option<AdjacencyType>,
    /// Defines a province that can block the adjacency.
    /// While an enemy unit controls this province, the connection will be unavailable. -1 disables
    /// this feature; however, any adjacency with the type "sea" must have a province defined here.
    #[serde(rename = "Through")]
    pub through: Option<ProvinceId>,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub start_x: XCoord,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub stop_x: XCoord,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub start_y: YCoord,
    /// Used to adjust the starting and ending point of the graphic displaying the adjacency. If no
    /// adjustment is needed, use -1 in place of an actual coordinate.
    pub stop_y: YCoord,
    /// An adjacency rule can be referenced that controls access through the adjacency.
    pub adjacency_rule_name: Option<AdjacencyRuleName>,
    /// The comment for the adjacency
    pub comment: Option<String>,
}

/// The adjacencies from the adjacency csv file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Adjacencies {
    /// The adjacencies between provinces
    pub adjacencies: Vec<Adjacency>,
}

/// The adjacency rules from the adjacency rule file
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AdjacencyRules {
    /// The adjacency rules
    pub adjacency_rules: HashMap<AdjacencyRuleName, AdjacencyRule>,
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::append_dir;
    use crate::components::adjacency::AdjacencyType::Impassable;
    use crate::components::default_map::DefaultMap;
    use jomini::TextDeserializer;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_reads_adjacencies_from_the_map() {
        let map = DefaultMap::from_file(Path::new("./test/default.map"))
            .expect("Failed to read default.map");
        let adjacency_rules_path = append_dir(&map.adjacencies, "./test");
        let adjacency_rules_data =
            fs::read_to_string(&adjacency_rules_path).expect("Failed to read adjacency_rules.txt");
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_reader(adjacency_rules_data.as_bytes());
        let mut adjacencies = Vec::new();
        for adjacency in rdr.deserialize() {
            adjacencies.push(adjacency.expect("Failed to deserialize adjacency"));
        }
        let adjacencies = Adjacencies { adjacencies };
        assert_eq!(adjacencies.adjacencies.len(), 486);
        assert_eq!(
            adjacencies.adjacencies[0],
            Adjacency {
                from: ProvinceId(6402),
                to: ProvinceId(6522),
                adjacency_type: Some(Impassable),
                through: Some(ProvinceId(-1)),
                start_x: XCoord(-1),
                stop_x: XCoord(-1),
                start_y: YCoord(-1),
                stop_y: YCoord(-1),
                adjacency_rule_name: None,
                comment: None
            }
        );
    }

    #[test]
    fn it_reads_adjacency_rules_from_the_map() {
        let map = DefaultMap::from_file(Path::new("test/default.map"))
            .expect("Failed to read default.map");
        let adjacency_rules_path = append_dir(&map.adjacency_rules, "./test");
        let adjacency_rules_data =
            fs::read_to_string(&adjacency_rules_path).expect("Failed to read adjacency_rules.txt");
        let rules = TextDeserializer::from_windows1252_slice::<RawAdjacencyRules>(
            adjacency_rules_data.as_bytes(),
        )
        .expect("Failed to deserialize adjacency_rules.txt");
        let mut adjacency_rules = AdjacencyRules {
            adjacency_rules: HashMap::new(),
        };
        for rule in rules.adjacency_rule {
            adjacency_rules
                .adjacency_rules
                .insert(rule.name.clone(), rule);
        }
        assert_eq!(adjacency_rules.adjacency_rules.len(), 11);
        assert_eq!(
            adjacency_rules
                .adjacency_rules
                .get(&AdjacencyRuleName("Veracruz Canal".to_owned())),
            Some(&AdjacencyRule {
                name: AdjacencyRuleName("Veracruz Canal".to_owned()),
                contested: AdjacencyLogic {
                    army: false,
                    navy: false,
                    submarine: false,
                    trade: false
                },
                enemy: AdjacencyLogic {
                    army: false,
                    navy: false,
                    submarine: false,
                    trade: false
                },
                friend: AdjacencyLogic {
                    army: true,
                    navy: true,
                    submarine: true,
                    trade: true
                },
                neutral: AdjacencyLogic {
                    army: false,
                    navy: false,
                    submarine: false,
                    trade: true
                },
                required_provinces: vec![ProvinceId(10033), ProvinceId(10101)],
                icon: Icon(ProvinceId(10101)),
                offset: vec![-3, 0, -6],
                is_disabled: None
            })
        );
    }
}
