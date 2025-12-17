//! Self-Contained Bevy Test Application
//! 
//! This test harness provides a complete Bevy application environment
//! for testing individual wizard steps in isolation. Each step can be
//! cloned into this harness and run independently for rapid development.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_dock::{DockArea, DockState, Style as DockStyle};
use vintage_game_generator::{
    wizard::{
        WizardStep, WizardState, GenerateMode,
        config::{ProjectConfig, GameSpecification},
    }
};
use std::sync::Arc;

/// Test harness application
pub struct TestHarnessApp;

impl TestHarnessApp {
    /// Run the test harness with a specific wizard step
    pub fn run_with_step(step: Box<dyn WizardStep>) {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(EguiPlugin)
            .insert_resource(TestHarnessState::new(step))
            .add_systems(Startup, setup_test_environment)
            .add_systems(Update, render_test_ui)
            .run();
    }
    
    /// Run the test harness with the timeline step
    pub fn run_timeline_test() {
        let timeline_step = Box::new(TimelineTestStep::default());
        Self::run_with_step(timeline_step);
    }
}

/// Test harness state
#[derive(Resource)]
struct TestHarnessState {
    /// The wizard step being tested
    step: Box<dyn WizardStep>,
    /// Mock wizard state
    wizard_state: Arc<WizardState>,
    /// Test dock state
    dock_state: DockState<String>,
    /// Test control panel visibility
    show_controls: bool,
}

impl TestHarnessState {
    fn new(step: Box<dyn WizardStep>) -> Self {
        // Create a mock wizard state
        let wizard_state = Arc::new(WizardState {
            mode: GenerateMode::Guided,
            project_config: ProjectConfig::default(),
            game_spec: GameSpecification::default(),
            current_step: 0,
            total_steps: 1,
            is_complete: false,
            error_message: None,
        });
        
        // Create dock state with test panels
        let mut dock_state = DockState::new(vec!["main".to_string()]);
        dock_state.push_to_focused_leaf("controls".to_string());
        dock_state.push_to_focused_leaf("state".to_string());
        
        Self {
            step,
            wizard_state,
            dock_state,
            show_controls: true,
        }
    }
}

/// Setup test environment
fn setup_test_environment(mut commands: Commands) {
    // Add a camera for any potential 2D/3D content
    commands.spawn(Camera2d);
    
    // Add test environment markers
    commands.spawn((
        Name::new("TestHarness"),
        TestEnvironmentMarker,
    ));
}

/// Marker component for test environment
#[derive(Component)]
struct TestEnvironmentMarker;

/// Main test UI rendering
fn render_test_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<TestHarnessState>,
    mut exit: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    
    // Top menu bar
    egui::TopBottomPanel::top("test_menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Test Harness", |ui| {
                if ui.button("Reset Step").clicked() {
                    state.wizard_state = Arc::new(WizardState::default());
                }
                
                ui.separator();
                
                if ui.button("Exit").clicked() {
                    exit.send(AppExit::Success);
                }
            });
            
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut state.show_controls, "Show Controls");
            });
            
            ui.separator();
            
            ui.label(format!("Testing: {}", state.step.title()));
        });
    });
    
    // Main dock area
    egui::CentralPanel::default().show(ctx, |ui| {
        DockArea::new(&mut state.dock_state)
            .style(DockStyle::from_egui(ui.style()))
            .show_inside(ui, &mut TabViewer {
                step: &mut state.step,
                wizard_state: &mut state.wizard_state,
                show_controls: state.show_controls,
            });
    });
}

/// Tab viewer for dock area
struct TabViewer<'a> {
    step: &'a mut Box<dyn WizardStep>,
    wizard_state: &'a mut Arc<WizardState>,
    show_controls: bool,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = String;
    
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab.as_str() {
            "main" => "Step UI".into(),
            "controls" => "Test Controls".into(),
            "state" => "Wizard State".into(),
            _ => tab.as_str().into(),
        }
    }
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "main" => {
                // Render the actual step
                ui.group(|ui| {
                    self.step.ui(ui, Arc::clone(self.wizard_state));
                });
            }
            "controls" => {
                if self.show_controls {
                    render_test_controls(ui, self.wizard_state);
                }
            }
            "state" => {
                render_state_inspector(ui, self.wizard_state);
            }
            _ => {
                ui.label("Unknown tab");
            }
        }
    }
}

/// Render test controls
fn render_test_controls(ui: &mut egui::Ui, wizard_state: &mut Arc<WizardState>) {
    ui.heading("Test Controls");
    ui.separator();
    
    // Mock data generators
    ui.collapsing("Generate Mock Data", |ui| {
        if ui.button("Random Project Config").clicked() {
            if let Some(state) = Arc::get_mut(wizard_state) {
                state.project_config.project_name = format!("Test Project {}", rand::random::<u32>() % 1000);
                state.project_config.author_name = "Test Author".to_string();
                state.project_config.version = "0.1.0".to_string();
            }
        }
        
        if ui.button("Sample Game Spec").clicked() {
            if let Some(state) = Arc::get_mut(wizard_state) {
                state.game_spec.title = "Retro Quest".to_string();
                state.game_spec.genre = "RPG".to_string();
                state.game_spec.art_style = "16-bit pixel".to_string();
                state.game_spec.inspirations = vec![
                    "Final Fantasy".to_string(),
                    "Chrono Trigger".to_string(),
                ];
            }
        }
    });
    
    ui.separator();
    
    // State manipulation
    ui.collapsing("State Manipulation", |ui| {
        if let Some(state) = Arc::get_mut(wizard_state) {
            ui.horizontal(|ui| {
                ui.label("Current Step:");
                ui.add(egui::DragValue::new(&mut state.current_step));
            });
            
            ui.horizontal(|ui| {
                ui.label("Total Steps:");
                ui.add(egui::DragValue::new(&mut state.total_steps));
            });
            
            ui.checkbox(&mut state.is_complete, "Is Complete");
            
            ui.horizontal(|ui| {
                ui.label("Error:");
                let mut error_text = state.error_message.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut error_text).changed() {
                    state.error_message = if error_text.is_empty() {
                        None
                    } else {
                        Some(error_text)
                    };
                }
            });
        }
    });
}

/// Render state inspector
fn render_state_inspector(ui: &mut egui::Ui, wizard_state: &Arc<WizardState>) {
    ui.heading("Wizard State Inspector");
    ui.separator();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.collapsing("Project Config", |ui| {
            ui.label(format!("Name: {}", wizard_state.project_config.project_name));
            ui.label(format!("Author: {}", wizard_state.project_config.author_name));
            ui.label(format!("Version: {}", wizard_state.project_config.version));
            ui.label(format!("Description: {}", wizard_state.project_config.description));
        });
        
        ui.collapsing("Game Specification", |ui| {
            ui.label(format!("Title: {}", wizard_state.game_spec.title));
            ui.label(format!("Genre: {}", wizard_state.game_spec.genre));
            ui.label(format!("Art Style: {}", wizard_state.game_spec.art_style));
            ui.label(format!("Inspirations: {:?}", wizard_state.game_spec.inspirations));
        });
        
        ui.collapsing("Raw JSON", |ui| {
            if let Ok(json) = serde_json::to_string_pretty(&wizard_state.as_ref()) {
                ui.code(&json);
            }
        });
    });
}

/// Timeline test step implementation
#[derive(Default)]
pub struct TimelineTestStep {
    selected_year: Option<i32>,
    selected_game: Option<String>,
    timeline_zoom: f32,
}

impl TimelineTestStep {
    pub fn new() -> Self {
        Self {
            selected_year: None,
            selected_game: None,
            timeline_zoom: 1.0,
        }
    }
}

impl WizardStep for TimelineTestStep {
    fn title(&self) -> &str {
        "Vintage Game Timeline"
    }
    
    fn ui(&mut self, ui: &mut egui::Ui, _wizard_state: Arc<WizardState>) {
        ui.heading("Select Games for Inspiration");
        ui.separator();
        
        // Timeline controls
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut self.timeline_zoom, 0.5..=2.0));
            
            if ui.button("Reset View").clicked() {
                self.timeline_zoom = 1.0;
                self.selected_year = None;
            }
        });
        
        ui.separator();
        
        // Mock timeline UI (will be replaced with egui_timeline)
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for year in 1980..=1995 {
                    let is_selected = self.selected_year == Some(year);
                    
                    ui.vertical(|ui| {
                        ui.set_width(100.0 * self.timeline_zoom);
                        
                        // Year header
                        let year_text = if is_selected {
                            egui::RichText::new(year.to_string()).strong()
                        } else {
                            egui::RichText::new(year.to_string())
                        };
                        
                        if ui.selectable_label(is_selected, year_text).clicked() {
                            self.selected_year = Some(year);
                        }
                        
                        // Mock games for this year
                        ui.separator();
                        
                        for game_idx in 0..3 {
                            let game_name = format!("Game {} ({})", game_idx + 1, year);
                            let is_game_selected = self.selected_game.as_ref() == Some(&game_name);
                            
                            if ui.selectable_label(is_game_selected, &game_name).clicked() {
                                self.selected_game = Some(game_name);
                            }
                        }
                    });
                    
                    ui.separator();
                }
            });
        });
        
        // Selected game details
        if let Some(game) = &self.selected_game {
            ui.separator();
            ui.group(|ui| {
                ui.heading("Selected Game");
                ui.label(format!("Title: {}", game));
                ui.label("Genre: RPG");
                ui.label("Platform: NES");
                ui.label("This is a mock game entry for testing.");
            });
        }
    }
    
    fn validate(&self, _wizard_state: &WizardState) -> Result<(), String> {
        if self.selected_game.is_none() {
            Err("Please select at least one game for inspiration".to_string())
        } else {
            Ok(())
        }
    }
}

/// Integration test utilities
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test that the timeline step can be created and validated
    #[test]
    fn test_timeline_step_creation() {
        let mut step = TimelineTestStep::new();
        let wizard_state = WizardState::default();
        
        // Should fail validation without selection
        assert!(step.validate(&wizard_state).is_err());
        
        // Should pass with selection
        step.selected_game = Some("Test Game".to_string());
        assert!(step.validate(&wizard_state).is_ok());
    }
}

/// Example of how to run specific tests
/// 
/// ```bash
/// # Run the timeline test harness
/// cargo test --test test_harness -- --nocapture timeline_test::run
/// 
/// # Run with custom step
/// cargo test --test test_harness -- --nocapture custom_step::run
/// ```
#[cfg(test)]
mod timeline_test {
    use super::*;
    
    #[test]
    #[ignore] // Run manually with --ignored flag
    fn run() {
        TestHarnessApp::run_timeline_test();
    }
}
