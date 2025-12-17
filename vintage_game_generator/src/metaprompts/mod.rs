// Module declarations for metaprompts - Rust 2024 style (no mod.rs)
pub mod generator;
pub mod types;
pub mod conversation;
pub mod validation;
pub mod watcher;

// Re-exports for convenience
pub use generator::GameGenerator;
// Re-export from vintage_ai_client
pub use vintage_ai_client::conversation::{GenerationProgress, GenerationPhase};
pub use types::{GameConfig, ArtStyle, ColorPalette, WorldConfig};
pub use conversation::{ConversationState, ConversationMessage};
pub use validation::{PromptValidator, ValidationResult};
pub use watcher::{PromptWatcher, GenerationQueue};
