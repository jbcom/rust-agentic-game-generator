use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use crate::wizard::{
    AppDirectories, 
    config::ConfigManager, 
    state::{AppState, WizardStep, GenerationStatus, LogLevel},
    SwitchModeEvent, 
    AppMode,
    steps::{
        draw_welcome_step,
        guided::{GuidedModeState, render_guided_mode, setup_guided_mode},
    },
};
use crate::wizard::pipeline::GenerationPipeline;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectedTab {
    Conversation,
    Prompts,
    Assets,
    Build,
    Logs,
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: std::time::SystemTime,
    pub level: LogLevel,
    pub message: String,
}

pub fn draw_generate_ui(
    mut contexts: EguiContexts,
    mut app_state: ResMut<AppState>,
    directories: Res<AppDirectories>,
    _pipeline: Res<GenerationPipeline>,
    _switch_mode_events: EventWriter<SwitchModeEvent>,
    guided_state: Option<ResMut<GuidedModeState>>,
    commands: Commands,
    mut exit_events: EventWriter<AppExit>,
) {
    trace!("draw_generate_ui called");
    
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => {
            // EguiContext not ready yet - this happens in the first frame
            // Don't log as error since this is expected behavior
            trace!("EguiContext not ready yet, skipping frame");
            return;
        }
    };
    
    debug!("Successfully got EguiContext, current wizard step: {:?}", app_state.wizard_step);
    
    // Initialize config manager if not present
    if app_state.config_manager.is_none() {
        if let Ok(config_manager) = ConfigManager::new(&directories.project_dir, None) {
            app_state.set_config_manager(config_manager);
        }
    }
    
    // Handle exit dialog
    if app_state.show_exit_dialog {
        egui::Window::new("Exit Confirmation")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Are you sure you want to exit?");
                ui.add_space(10.0);
                ui.label("Any unsaved progress will be lost.");
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        app_state.show_exit_dialog = false;
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("Exit").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                            exit_events.write(AppExit::Success);
                        }
                    });
                });
            });
    }
    
    // Draw wizard steps based on current state - ONLY WELCOME → GUIDED flow
    match &app_state.wizard_step {
        WizardStep::Welcome => {
            debug!("Drawing welcome step");
            draw_wizard_frame_with_state(ctx, &mut app_state, |ui, state| {
                let wizard_mode_action = draw_welcome_step(ui, &mut state.config_manager);
                if let Some(action) = wizard_mode_action {
                    info!("Welcome action selected: {:?}", action);
                    state.set_wizard_mode(action);
                }
            });
        }
        WizardStep::GuidedMode => {
            debug!("Drawing guided mode step");
            // Guided mode handles its own UI completely
            // This is where the user browses and blends vintage games
            if let Some(guided_state) = guided_state {
                debug!("Guided state exists, rendering guided mode");
                render_guided_mode(contexts, app_state, guided_state);
            } else {
                // Need to setup guided mode resources
                warn!("No guided state found, setting up guided mode");
                setup_guided_mode(commands);
                
                // Show loading state for this frame
                draw_wizard_frame(ctx, &mut app_state, |ui| {
                    ui.label("Loading guided mode...");
                });
            }
        }
        WizardStep::Complete => {
            draw_wizard_frame_with_state(ctx, &mut app_state, |ui, state| {
                ui.heading("🎉 Export Complete!");
                ui.separator();
                
                if let Some(export) = &state.guided_export {
                    ui.label("Successfully exported game configuration");
                    ui.label(format!("Blend: {}", export.blend_name));
                    
                    ui.separator();
                    
                    if ui.button("Start New Blend").clicked() {
                        state.set_wizard_step(WizardStep::Welcome);
                        state.guided_export = None;
                    }
                } else {
                    ui.label("Export completed successfully!");
                    
                    if ui.button("Return to Welcome").clicked() {
                        state.set_wizard_step(WizardStep::Welcome);
                    }
                }
            });
        }
        _ => {
            warn!("Unhandled wizard step: {:?}", app_state.wizard_step);
            // Any other step that isn't implemented yet
            draw_wizard_frame_with_state(ctx, &mut app_state, |ui, state| {
                ui.heading("🚧 Under Construction");
                ui.label("This feature is not implemented yet.");
                ui.add_space(20.0);
                
                // Show pipeline status for debugging
                ui.separator();
                ui.label("Pipeline initialized");
                
                if ui.button("← Back to Welcome").clicked() {
                    state.set_wizard_step(WizardStep::Welcome);
                }
            });
        }
    }
}

fn draw_wizard_frame<T>(
    ctx: &egui::Context,
    app_state: &mut AppState,
    content: impl FnOnce(&mut egui::Ui) -> T,
) -> T {
    let mut result = None;
    egui::CentralPanel::default().show(ctx, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("🎮 Vintage Game Generator");
            ui.separator();
            ui.label(app_state.get_step_title());
        });
        ui.separator();
        
        // Progress bar
        let progress = app_state.get_progress();
        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .animate(true);
        ui.add(progress_bar);
        ui.separator();
        
        // Main content area
        egui::ScrollArea::vertical().show(ui, |ui| {
            result = Some(content(ui));
        });
        
        ui.separator();
        
        // Navigation
        ui.horizontal(|ui| {
            if app_state.can_go_back() {
                if ui.button("← Back").clicked() {
                    app_state.go_back();
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Exit").clicked() {
                    app_state.show_exit_dialog = true;
                }
            });
        });
    });
    result.unwrap()
}

fn draw_wizard_frame_with_state(
    ctx: &egui::Context,
    app_state: &mut AppState,
    content: impl FnOnce(&mut egui::Ui, &mut AppState),
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("🎮 Vintage Game Generator");
            ui.separator();
            ui.label(app_state.get_step_title());
        });
        ui.separator();
        
        // Progress bar
        let progress = app_state.get_progress();
        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .animate(true);
        ui.add(progress_bar);
        ui.separator();
        
        // Main content area
        egui::ScrollArea::vertical().show(ui, |ui| {
            content(ui, app_state);
        });
        
        ui.separator();
        
        // Navigation
        ui.horizontal(|ui| {
            if app_state.can_go_back() {
                if ui.button("← Back").clicked() {
                    app_state.go_back();
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Exit").clicked() {
                    app_state.show_exit_dialog = true;
                }
            });
        });
    });
}

fn draw_main_ui(
    ctx: &egui::Context,
    app_state: &mut ResMut<AppState>,
    directories: &AppDirectories,
    pipeline: &GenerationPipeline,
    switch_mode_events: &mut EventWriter<SwitchModeEvent>,
) {
    // Show pipeline information in the status bar
    let pipeline_info = "Pipeline: Ready for generation".to_string();
    
    // Top panel
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Vintage Game Generator");
            ui.separator();
            
            // Show current mode
            ui.label(format!("Mode: {:?}", app_state.wizard_mode));
            
            ui.separator();
            
            // Generation status
            match &app_state.generation_status {
                GenerationStatus::Idle => {
                    if ui.button("▶ Start Generation").clicked() {
                        app_state.start_generation("Initializing...".to_string());
                    }
                }
                GenerationStatus::Generating { current, progress } => {
                    ui.label(format!("Generating: {}", current));
                    ui.add(egui::ProgressBar::new(*progress).show_percentage());
                    if ui.button("⏸ Pause").clicked() {
                        // Pause generation by setting status to idle
                        app_state.generation_status = GenerationStatus::Idle;
                        info!("Generation paused by user");
                    }
                }
                GenerationStatus::Complete => {
                    ui.colored_label(egui::Color32::GREEN, "✓ Generation Complete");
                }
                GenerationStatus::Failed(error) => {
                    ui.colored_label(egui::Color32::RED, format!("✗ Failed: {}", error));
                }
            }
            
            ui.separator();
            
            // Mode switch button
            if ui.button("📂 Browse Projects").clicked() {
                switch_mode_events.write(SwitchModeEvent {
                    new_mode: AppMode::List,
                    project_path: None,
                });
            }
        });
    });
    
    // Main content
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Main Generation Interface");
        ui.separator();
        
        // Show current configuration
        ui.group(|ui| {
            ui.label("Current Configuration:");
            ui.label(format!("Mode: {:?}", app_state.wizard_mode));
        });
        
        ui.separator();
        
        // Show generation pipeline status
        if let GenerationStatus::Generating { .. } = &app_state.generation_status {
            ui.heading("Generation Pipeline");
            ui.separator();
            
            // Show pipeline stages placeholder
            ui.group(|ui| {
                ui.label("Pipeline Status:");
                ui.label("Generation in progress...");
            });
            
            ui.separator();
            
            // Show logs placeholder
            ui.heading("Generation Logs");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.label("Logs will appear here...");
                });
        } else {
            // Show configuration summary when not generating
            ui.heading("Current Configuration");
            ui.separator();
            
            if let Some(config_manager) = &app_state.config_manager {
                ui.group(|ui| {
                    ui.label(format!("Mode: {:?}", app_state.wizard_mode));
                    ui.label(format!("Directory: {}", directories.project_dir.display()));
                    ui.label(format!("Config loaded: {}", config_manager.config_loaded()));
                    ui.label(pipeline_info);
                });
            }
            
            ui.separator();
            ui.label("Use the wizard above to configure your game, then start generation.");
        }
    });
}
