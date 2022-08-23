use crate::components::prelude::*;
use crate::{LoadObject, MapError};
use jomini::JominiDeserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// The collection of states on the map
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct States {
    /// The collection of states
    pub states: HashMap<StateId, State>,
}

impl States {
    /// Loads the states from the `history/states/` directory.
    /// # Errors
    /// If the states directory does not exist, or if any of the states fail to load.
    #[inline]
    pub fn from_dir(path: &Path) -> Result<Self, MapError> {
        let state_files = fs::read_dir(path)?;
        let mut states = HashMap::new();
        for state_file in state_files.flatten() {
            let state_path = state_file.path();
            let state = RawState::load_object(&state_path)?.state;
            states.insert(state.id, state);
        }
        Ok(States { states })
    }
}

/// Container for a state
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
struct RawState {
    /// The inner state
    state: State,
}

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
    /// How much manpower the state starts with.  
    /// Duplicated because people tend to make mistakes.  The game only considers the last entry however.
    #[jomini(duplicated)]
    pub manpower: Vec<Manpower>,
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
    /// Duplicated because people tend to make mistakes.  The game only considers the last entry however.
    #[jomini(duplicated)]
    pub state_category: Vec<StateCategoryName>,
    /// The state's history
    pub history: Option<StateHistory>,
    /// The provinces that belong to the state
    pub provinces: HashSet<ProvinceId>,
    /// The base supply of the state
    pub local_supplies: Option<LocalSupplies>,
    /// Whether or not the state is impassable
    pub impassable: Option<bool>,
    /// Adds an additional multiplier on the amount of unlocked shared building slots. Recommended
    /// to avoid, instead using state categories.
    pub buildings_max_level_factor: Option<BuildingsMaxLevelFactor>,
}

/// A state's history.
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct StateHistory {
    /// defines the initial owner of the state. If a state does not have an owner, the game will run
    /// without issues; however, executing nearly any effect on that state, such as transferring it
    /// to a country, will crash the game.
    pub owner: CountryTag,
    /// defines the initial controller of the state. Optional to define - only necessary if the
    /// owner differs from the controller.
    pub controller: Option<CountryTag>,
    /// defines the amount of victory points on a specified province, where the first number is the
    /// province and the second number is the amount of victory points. Only one province can be
    /// defined within one victory_points. In order to have multiple provinces with victory points
    /// in one state, several instances of victory_points = { ... } need to be put in.
    #[jomini(duplicated)]
    pub victory_points: Vec<(ProvinceId, VictoryPoints)>,
    // TODO: State resources
}

#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::LoadObject;

    #[test]
    fn it_loads_a_state() {
        let state = RawState::load_object(Path::new("./test/history/states/1-State.txt"))
            .expect("Failed to load state")
            .state;

        assert_eq!(state.id, StateId(1));
        assert_eq!(state.name, StateName("STATE_1".to_owned()));
        assert_eq!(*state.manpower.last().unwrap(), Manpower(25000));
        assert_eq!(
            *state.state_category.last().unwrap(),
            StateCategoryName("metropolis".to_owned())
        );
        assert_eq!(
            state.buildings_max_level_factor,
            Some(BuildingsMaxLevelFactor(1.0))
        );
        assert_eq!(
            state.provinces,
            HashSet::from([
                ProvinceId(951),
                ProvinceId(1780),
                ProvinceId(2001),
                ProvinceId(2409),
                ProvinceId(2410),
                ProvinceId(2411),
                ProvinceId(2412),
                ProvinceId(2413),
                ProvinceId(2414),
                ProvinceId(2415),
                ProvinceId(4622),
                ProvinceId(4785),
                ProvinceId(4786),
                ProvinceId(4787),
                ProvinceId(4953),
                ProvinceId(4954),
            ])
        );
    }

    #[test]
    fn it_loads_states() {
        let states =
            States::from_dir(Path::new("./test/history/states")).expect("Failed to load states");
        assert_eq!(states.states.len(), 1388);
    }
}
