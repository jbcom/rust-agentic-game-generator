pub mod metadata;
pub mod analysis;
pub mod engine;
pub mod visualization;
pub mod export;

// Re-export key functions
pub use engine::create_blend;
pub use visualization::render_blend_visualization;
pub use export::{export_blend_to_config, render_export_ui};

use bevy_egui::egui;
use crate::wizard::steps::guided::GuidedModeState;

/// Main blend UI that combines visualization and export
pub fn render_blend_ui(ui: &mut egui::Ui, state: &mut GuidedModeState) {
    ui.columns(2, |columns| {
        // Left column - visualization
        columns[0].group(|ui| {
            render_blend_visualization(ui, state);
        });
        
        // Right column - export options
        columns[1].group(|ui| {
            render_export_ui(ui, state);
        });
    });
}
