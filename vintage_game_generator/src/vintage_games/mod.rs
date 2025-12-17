//! Vintage game timeline module (1980-1995)
//! 
//! This module contains a curated timeline of exemplar games from the golden and retro eras.
//! Each year features the highest-rated game from up to 3 different genres.
//! Games are selected to serve as creative inspiration for the AI RPG generator.

pub mod games;
pub mod platforms;
pub mod eras;
pub mod graph;

// Re-export commonly used items
pub use games::{TimelineGame, TIMELINE_GAMES, games_by_year, games_by_genre, search_games, all_genres};
pub use platforms::{PlatformInfo, PLATFORM_INFO, get_platform_info};
pub use eras::{Era, era_for_year, era_description, games_by_era};
pub use graph::{build_game_graph, GameNode};

/// Timeline span
pub const TIMELINE_START: i32 = 1980;
pub const TIMELINE_END: i32 = 1995;
