//! Freeform Mode - AI-assisted game creation through conversation
//!
//! This mode provides an interactive conversation interface where users can
//! describe their game ideas and the AI helps design and generate the game.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::wizard::pipeline::GenerationPipeline;
use crate::wizard::state::{AppState, WizardStep};

mod types;
mod conversation;
mod wizard_steps;
mod ai_interface;

pub use types::*;
pub use conversation::*;
pub use wizard_steps::*;

/// Main entry point for rendering freeform mode
pub fn render_freeform_mode(
    contexts: EguiContexts,
    app_state: ResMut<AppState>,
    freeform_state: ResMut<FreeformModeState>,
    commands: Commands,
    pipeline: Res<GenerationPipeline>,
    stream_res: ResMut<ConversationStream>,
) {
    // Route to appropriate sub-step
    match &freeform_state.current_step {
        FreeformStep::Introduction => {
            wizard_steps::render_introduction(contexts, app_state, freeform_state);
        }
        FreeformStep::BasicInfo => {
            wizard_steps::render_basic_info(contexts, app_state, freeform_state);
        }
        FreeformStep::GameplayDesign => {
            wizard_steps::render_gameplay_design(contexts, app_state, freeform_state);
        }
        FreeformStep::VisualStyle => {
            wizard_steps::render_visual_style(contexts, app_state, freeform_state);
        }
        FreeformStep::Features => {
            wizard_steps::render_features(contexts, app_state, freeform_state);
        }
        FreeformStep::TechnicalSettings => {
            wizard_steps::render_technical_settings(contexts, app_state, freeform_state);
        }
        FreeformStep::Review => {
            wizard_steps::render_review(contexts, app_state, freeform_state);
        }
        FreeformStep::Conversation => {
            conversation::render_conversation(contexts, app_state, freeform_state, commands, pipeline, stream_res);
        }
    }
}

/// Setup resources for freeform mode
pub fn setup_freeform_mode(mut commands: Commands) {
    // Insert the freeform mode state
    commands.insert_resource(FreeformModeState::default());
    commands.insert_resource(ConversationStream::default());

    // Initialize AI client if needed
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        // Initialize AI resources
        info!("OpenAI API key found, AI conversation will be available");
    } else {
        warn!("No OpenAI API key found, AI features will be limited");
    }
}

/// Cleanup resources when leaving freeform mode
pub fn cleanup_freeform_mode(mut commands: Commands) {
    commands.remove_resource::<FreeformModeState>();
    commands.remove_resource::<ConversationStream>();
}
