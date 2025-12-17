// Module for wizard steps - new flow with visual mode selection
pub mod welcome;
pub mod language;
pub mod guided;

// Re-export step types and functions
pub use welcome::{draw_welcome_step, WelcomeAction};
pub use language::{draw_language_step, LanguageChoice};
