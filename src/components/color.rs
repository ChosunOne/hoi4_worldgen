use crate::components::wrappers::{Blue, Green, Red};
use jomini::JominiDeserialize;
use serde::{Deserialize, Serialize};

/// Colors on the map
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct Colors {
    /// The colors
    #[jomini(duplicated)]
    pub color: Vec<Color>,
}

/// An RGB Color value
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Color(pub Red, pub Green, pub Blue);

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::panic)]
#[allow(clippy::default_numeric_fallback)]
#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::DirectlyDeserialize;
    use std::path::Path;

    #[test]
    fn it_loads_colors_from_file() {
        let colors_path = Path::new("./test/map/colors.txt");
        let colors = Colors::load_object(&colors_path).expect("Failed to read colors");
        assert_eq!(colors.color.len(), 200);
        assert_eq!(colors.color[0], Color(Red(4), Green(144), Blue(178)));
        assert_eq!(colors.color[75], Color(Red(107), Green(170), Blue(77)));
    }
}
