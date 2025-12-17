//! AI conversation interface for freeform mode

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::wizard::state::AppState;
use super::{FreeformModeState, ConversationEntry, ConversationRole};

/// Render the AI conversation interface
pub fn render_conversation(
    mut contexts: EguiContexts,
    mut app_state: ResMut<AppState>,
    mut freeform_state: ResMut<FreeformModeState>,
    _commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    
    egui::CentralPanel::default().show(ctx, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("🤖 AI Game Design Conversation");
            ui.separator();
            ui.label("Let's bring your game to life!");
        });
        ui.separator();
        
        // Context summary
        if !freeform_state.conversation.context_summary.is_empty() {
            ui.group(|ui| {
                ui.label("Current Context:");
                ui.label(&freeform_state.conversation.context_summary);
            });
            ui.separator();
        }
        
        // Conversation history
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(ui.available_height() - 100.0)
            .show(ui, |ui| {
                for entry in &freeform_state.conversation.history {
                    render_conversation_entry(ui, entry);
                    ui.add_space(10.0);
                }
                
                // Show processing indicator
                if freeform_state.conversation.is_processing {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("AI is thinking...");
                    });
                }
                
                // Show error if any
                if let Some(error) = &freeform_state.conversation.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                }
            });
        
        ui.separator();
        
        // Input area
        ui.horizontal(|ui| {
            let response = ui.text_edit_multiline(&mut freeform_state.conversation.current_input);
            
            // Focus on the text input
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !freeform_state.conversation.current_input.trim().is_empty() {
                    send_message(&mut freeform_state);
                }
            }
            
            ui.vertical(|ui| {
                if ui.button("Send").clicked() && !freeform_state.conversation.current_input.trim().is_empty() {
                    send_message(&mut freeform_state);
                }
                
                if ui.button("Export").clicked() {
                    // TODO: Export conversation and config
                    info!("Exporting freeform configuration...");
                }
            });
        });
        
        // Navigation
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("← Back to Review").clicked() {
                freeform_state.current_step = super::FreeformStep::Review;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Generate Game →").clicked() {
                    // TODO: Start generation
                    app_state.set_wizard_step(crate::wizard::state::WizardStep::Complete);
                }
            });
        });
    });
}

fn render_conversation_entry(ui: &mut egui::Ui, entry: &ConversationEntry) {
    let (icon, color) = match entry.role {
        ConversationRole::User => ("👤", egui::Color32::from_rgb(100, 150, 255)),
        ConversationRole::Assistant => ("🤖", egui::Color32::from_rgb(100, 255, 150)),
        ConversationRole::System => ("⚙️", egui::Color32::from_rgb(200, 200, 200)),
    };
    
    ui.horizontal(|ui| {
        ui.label(icon);
        ui.colored_label(color, &entry.content);
    });
    
    if let Some(metadata) = &entry.metadata {
        ui.indent("metadata", |ui| {
            ui.label(format!("Topic: {}", metadata.topic));
            if !metadata.decisions_made.is_empty() {
                ui.label("Decisions made:");
                for decision in &metadata.decisions_made {
                    ui.label(format!("  • {}", decision));
                }
            }
        });
    }
}

fn send_message(freeform_state: &mut FreeformModeState) {
    let message = freeform_state.conversation.current_input.trim().to_string();
    
    // Add user message to history
    freeform_state.conversation.history.push(ConversationEntry {
        role: ConversationRole::User,
        content: message.clone(),
        timestamp: std::time::SystemTime::now(),
        metadata: None,
    });
    
    // Clear input
    freeform_state.conversation.current_input.clear();
    
    // Set processing state
    freeform_state.conversation.is_processing = true;
    
    // TODO: Actually send to AI and process response
    // For now, just simulate a response
    simulate_ai_response(freeform_state);
}

fn simulate_ai_response(freeform_state: &mut FreeformModeState) {
    // Simulate AI processing delay
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Add simulated response
    freeform_state.conversation.history.push(ConversationEntry {
        role: ConversationRole::Assistant,
        content: format!(
            "I understand you want to create a {} called '{}'. Let me help you design the unique features that will make it stand out!",
            match &freeform_state.game_config.genre {
                super::GameGenre::ActionRPG => "fast-paced Action RPG",
                super::GameGenre::TurnBasedRPG => "strategic Turn-Based RPG",
                super::GameGenre::PuzzleRPG => "mind-bending Puzzle RPG",
                super::GameGenre::PlatformRPG => "acrobatic Platform RPG",
                super::GameGenre::RoguelikeRPG => "challenging Roguelike RPG",
                super::GameGenre::Custom(s) => s,
            },
            freeform_state.game_config.game_name
        ),
        timestamp: std::time::SystemTime::now(),
        metadata: Some(super::ConversationMetadata {
            topic: "game_overview".to_string(),
            decisions_made: vec![],
            alternatives_considered: vec![],
        }),
    });
    
    // Update context summary
    freeform_state.conversation.context_summary = format!(
        "Designing '{}' - A {} for {} players",
        freeform_state.game_config.game_name,
        match &freeform_state.game_config.genre {
            super::GameGenre::Custom(s) => s.clone(),
            _ => format!("{:?}", freeform_state.game_config.genre),
        },
        match freeform_state.game_config.target_audience {
            super::TargetAudience::Casual => "casual",
            super::TargetAudience::Core => "core",
            super::TargetAudience::Hardcore => "hardcore",
        }
    );
    
    // Clear processing state
    freeform_state.conversation.is_processing = false;
}
