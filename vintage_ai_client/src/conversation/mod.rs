//! Conversation management for interactive AI assistance
//! 
//! Handles multi-turn conversations for:
//! - Game design discussions
//! - Iterative refinement of concepts
//! - Q&A about generated content
//! - Blend exploration and explanation
//! - Phased game generation workflows

mod manager;
mod types;
mod starters;
mod game_generation;

pub use manager::ConversationManager;
pub use types::{
    Conversation,
    ConversationMessage,
    MessageRole,
    ConversationContext,
    GenerationPhase,
    GenerationProgress,
    GameConceptContext,
    BlendContext,
    ConversationSummary,
    MessageConfig,
};
pub use starters::{
    game_design_context,
    blend_exploration_context,
    technical_assistance_context,
    game_generation_context,
};

// Re-export game generation methods
pub use game_generation::GameGenerationExt;
