use crate::components::prelude::*;
use jomini::JominiDeserialize;
use serde::Serialize;

/// A state.  
/// The state borders must follow strategic regions, defined in
/// `/Hearts of Iron IV/map/strategicregions/*.txt`. If one province in the state belongs to one
/// strategic region, while a different province in the same state belongs to a different strategic
/// region, a map error will be created, which will cause a game crash on launch if the debug mode
/// is not turned on. Make sure that strategic region borders are followed, either by adjusting the
/// state or the strategic regions.
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct State {
    /// The state id
    pub id: StateId,
    /// The name of the state
    pub name: StateName,
    /// How much manpower the state starts with
    pub manpower: Manpower,
    /// The state category.  
    /// State categories can be added in /Hearts of Iron IV/common/state_category/*.txt. Each state
    /// category is contained within the state_categories = { ... }, as a code block with the name
    /// of the state category's ID.  
    /// A state category is a modifier block, where any state-scoped modifier can be used. The only
    /// modifier that the base game uses is local_building_slots, set to an integer, but any can be
    /// used.
    /// Additionally, the color = { 0 0 255 } block corresponds to the state's colour in the
    /// state map mode. It is defined in the RGB format, where each value is defined as an integer
    /// on the scale from 0 to 255.
    pub state_category: StateCategoryName,
    pub history: StateHistory,
    pub provinces: Vec<ProvinceId>,
    pub local_supplies: Option<LocalSupplies>,
}
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
pub struct StateHistory {
    pub owner: CountryTag,
    #[jomini(duplicated)]
    pub victory_points: Vec<(ProvinceId, VictoryPoints)>,
}
