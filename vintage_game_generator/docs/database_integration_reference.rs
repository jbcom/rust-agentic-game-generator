// Cargo.toml additions
/*
[dependencies]
# Previous dependencies plus:
sea-orm = { version = "0.12", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
sea-orm-migration = "0.12"
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-native-tls"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }
*/

// main.rs - Updated with database initialization
use bevy::prelude::*;
use bevy_game_studio::{GameGeneratorStudioPlugin, StudioServices};

fn main() {
    // Initialize tokio runtime for async database operations
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    // Initialize database and services
    let services = runtime.block_on(async {
        let db = bevy_game_studio::database::StudioDatabase::new(None).await
            .expect("Failed to initialize database");
        
        StudioServices::new(db).await
            .expect("Failed to initialize services")
    });

    App::new()
        .insert_resource(services)
        .add_plugins(GameGeneratorStudioPlugin)
        .run();
}

// studio.rs - Updated with database integration
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::services::StudioServices;
use crate::wizard::GameConfiguration;

pub struct StudioPlugin;

impl Plugin for StudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<StudioState>()
            .init_state::<StudioPhase>()
            .add_systems(Startup, setup_studio)
            .add_systems(Update, (
                studio_ui_system,
                handle_database_tasks,
            ).chain())
            .add_systems(OnEnter(StudioPhase::Setup), load_or_create_project)
            .add_systems(OnExit(StudioPhase::Setup), save_wizard_state);
    }
}

#[derive(Resource)]
pub struct StudioState {
    pub current_project_id: Option<uuid::Uuid>,
    pub dock_state: Arc<Mutex<DockState<DockTab>>>,
    pub theme: StudioTheme,
    pub ui_state: UIState,
}

#[derive(Default)]
pub struct UIState {
    pub wizard_config: GameConfiguration,
    pub wizard_step: crate::wizard::WizardStep,
    pub validation_errors: Vec<crate::wizard::ValidationError>,
    pub generation_queue: Vec<GenerationTaskView>,
    pub asset_gallery: AssetGalleryState,
}

#[derive(Clone)]
pub struct GenerationTaskView {
    pub id: uuid::Uuid,
    pub name: String,
    pub progress: f32,
    pub status: String,
}

#[derive(Default)]
pub struct AssetGalleryState {
    pub filter: String,
    pub selected_type: Option<String>,
    pub selected_asset: Option<uuid::Uuid>,
    pub assets: Vec<AssetView>,
}

#[derive(Clone)]
pub struct AssetView {
    pub id: uuid::Uuid,
    pub name: String,
    pub asset_type: String,
    pub thumbnail: Option<Handle<Image>>,
    pub quality_score: Option<f32>,
}

// System to load existing project or create new one
fn load_or_create_project(
    mut studio_state: ResMut<StudioState>,
    services: Res<StudioServices>,
    runtime: Res<TokioRuntime>,
) {
    runtime.block_on(async {
        // Try to get active project
        if let Ok(Some(project)) = services.projects.get_active_project().await {
            studio_state.current_project_id = Some(project.id);
            
            // Load configuration
            if let Ok(config) = serde_json::from_str::<GameConfiguration>(&project.config_json) {
                studio_state.ui_state.wizard_config = config;
            }
            
            // Load wizard state
            if let Ok(Some(wizard_state)) = services.wizard.get_wizard_state(project.id).await {
                // Parse and apply wizard state
                if let Ok(step) = wizard_state.step.parse() {
                    studio_state.ui_state.wizard_step = step;
                }
            }
            
            // Load assets
            if let Ok(assets) = services.assets.get_project_assets(project.id).await {
                studio_state.ui_state.asset_gallery.assets = assets.into_iter()
                    .map(|a| AssetView {
                        id: a.id,
                        name: a.name,
                        asset_type: a.asset_type,
                        thumbnail: None, // TODO: Load actual thumbnails
                        quality_score: a.quality_score,
                    })
                    .collect();
            }
        }
    });
}

// System to save wizard state when leaving setup phase
fn save_wizard_state(
    studio_state: Res<StudioState>,
    services: Res<StudioServices>,
    runtime: Res<TokioRuntime>,
) {
    if let Some(project_id) = studio_state.current_project_id {
        let config = studio_state.ui_state.wizard_config.clone();
        let step = studio_state.ui_state.wizard_step;
        let errors = studio_state.ui_state.validation_errors.clone();
        
        runtime.block_on(async move {
            // Save project configuration
            if let Err(e) = services.projects.update_config(project_id, config).await {
                error!("Failed to save project config: {}", e);
            }
            
            // Save wizard state
            let step_data = serde_json::json!({}); // TODO: Serialize actual step data
            if let Err(e) = services.wizard.save_wizard_state(
                project_id,
                step,
                step_data,
                errors,
                false,
            ).await {
                error!("Failed to save wizard state: {}", e);
            }
        });
    }
}

// System to handle background database tasks
fn handle_database_tasks(
    mut studio_state: ResMut<StudioState>,
    services: Res<StudioServices>,
    runtime: Res<TokioRuntime>,
    mut task_events: EventReader<DatabaseTaskEvent>,
) {
    for event in task_events.read() {
        match event {
            DatabaseTaskEvent::CreateAsset { name, asset_type, data } => {
                let project_id = studio_state.current_project_id.unwrap();
                let services = services.clone();
                let name = name.clone();
                let asset_type = asset_type.clone();
                let data = data.clone();
                
                runtime.spawn(async move {
                    // Save asset to filesystem
                    let file_path = save_asset_to_disk(&name, &asset_type, &data).await;
                    
                    // Create database record
                    if let Ok(asset) = services.assets.create_asset(
                        project_id,
                        name,
                        asset_type,
                        file_path,
                        serde_json::json!({}),
                        serde_json::json!({}),
                        None,
                    ).await {
                        info!("Created asset: {}", asset.id);
                    }
                });
            }
            DatabaseTaskEvent::UpdateAssetQuality { id, score } => {
                let services = services.clone();
                let id = *id;
                let score = *score;
                
                runtime.spawn(async move {
                    if let Err(e) = services.assets.update_quality_score(id, score).await {
                        error!("Failed to update quality score: {}", e);
                    }
                });
            }
            DatabaseTaskEvent::QueueGeneration { task_type, prompt, params } => {
                if let Some(project_id) = studio_state.current_project_id {
                    let services = services.clone();
                    let task_type = task_type.clone();
                    let prompt = prompt.clone();
                    let params = params.clone();
                    
                    runtime.spawn(async move {
                        if let Ok(task) = services.generation.create_task(
                            project_id,
                            task_type,
                            prompt,
                            params,
                            0,
                        ).await {
                            info!("Queued generation task: {}", task.id);
                        }
                    });
                }
            }
        }
    }
}

// Event system for database operations
#[derive(Event)]
pub enum DatabaseTaskEvent {
    CreateAsset {
        name: String,
        asset_type: String,
        data: Vec<u8>,
    },
    UpdateAssetQuality {
        id: uuid::Uuid,
        score: f32,
    },
    QueueGeneration {
        task_type: String,
        prompt: String,
        params: serde_json::Value,
    },
}

// Resource for tokio runtime
#[derive(Resource)]
pub struct TokioRuntime(tokio::runtime::Runtime);

impl TokioRuntime {
    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
    
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

// Helper function for asset storage
async fn save_asset_to_disk(name: &str, asset_type: &str, data: &[u8]) -> String {
    use tokio::fs;
    use std::path::PathBuf;
    
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("bevy-game-studio");
    path.push("assets");
    path.push(asset_type);
    
    fs::create_dir_all(&path).await.ok();
    
    let file_name = format!("{}_{}.dat", name, uuid::Uuid::new_v4());
    path.push(&file_name);
    
    fs::write(&path, data).await.ok();
    
    path.to_string_lossy().to_string()
}

// Updated wizard UI to work with database-backed state
pub fn render_wizard_with_db(
    ui: &mut egui::Ui,
    studio_state: &mut StudioState,
    services: &StudioServices,
    runtime: &TokioRuntime,
    next_phase: &mut NextState<StudioPhase>,
) {
    ui.heading("🎮 Game Project Setup");
    ui.separator();
    
    // Progress bar
    render_wizard_progress(ui, studio_state.ui_state.wizard_step);
    ui.separator();
    
    // Current step content
    egui::ScrollArea::vertical().show(ui, |ui| {
        match studio_state.ui_state.wizard_step {
            WizardStep::Welcome => render_welcome_step(ui, studio_state),
            WizardStep::BasicInfo => render_basic_info_step(ui, &mut studio_state.ui_state.wizard_config),
            WizardStep::GameplayDesign => render_gameplay_design_step(ui, &mut studio_state.ui_state.wizard_config),
            WizardStep::VisualStyle => render_visual_style_step(ui, &mut studio_state.ui_state.wizard_config),
            WizardStep::Features => render_features_step(ui, &mut studio_state.ui_state.wizard_config),
            WizardStep::TechnicalSettings => render_technical_settings_step(ui, &mut studio_state.ui_state.wizard_config),
            WizardStep::Review => render_review_step(ui, &studio_state.ui_state.wizard_config),
        }
    });
    
    ui.separator();
    
    // Navigation with auto-save
    ui.horizontal(|ui| {
        if studio_state.ui_state.wizard_step != WizardStep::Welcome {
            if ui.button("← Previous").clicked() {
                save_current_step(studio_state, services, runtime);
                studio_state.ui_state.wizard_step = studio_state.ui_state.wizard_step.previous();
            }
        }
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if studio_state.ui_state.wizard_step == WizardStep::Review {
                if ui.button("🚀 Generate Game").clicked() {
                    if validate_and_save_final(studio_state, services, runtime) {
                        next_phase.set(StudioPhase::Generation);
                    }
                }
            } else {
                if ui.button("Next →").clicked() {
                    if validate_current_step(studio_state, services, runtime) {
                        save_current_step(studio_state, services, runtime);
                        studio_state.ui_state.wizard_step = studio_state.ui_state.wizard_step.next();
                    }
                }
            }
        });
    });
}

fn save_current_step(
    studio_state: &StudioState,
    services: &StudioServices,
    runtime: &TokioRuntime,
) {
    if let Some(project_id) = studio_state.current_project_id {
        let step = studio_state.ui_state.wizard_step;
        let config = studio_state.ui_state.wizard_config.clone();
        let services = services.clone();
        
        runtime.spawn(async move {
            let step_data = serde_json::to_value(&config).unwrap_or_default();
            if let Err(e) = services.wizard.save_wizard_state(
                project_id,
                step,
                step_data,
                vec![],
                false,
            ).await {
                error!("Failed to save wizard state: {}", e);
            }
        });
    }
}

fn validate_current_step(
    studio_state: &mut StudioState,
    services: &StudioServices,
    runtime: &TokioRuntime,
) -> bool {
    if let Some(project_id) = studio_state.current_project_id {
        let errors = runtime.block_on(async {
            services.wizard.validate_step(project_id, studio_state.ui_state.wizard_step).await
                .unwrap_or_default()
        });
        
        studio_state.ui_state.validation_errors = errors;
        studio_state.ui_state.validation_errors.is_empty()
    } else {
        false
    }
}

fn validate_and_save_final(
    studio_state: &mut StudioState,
    services: &StudioServices,
    runtime: &TokioRuntime,
) -> bool {
    // Validate all steps
    let all_valid = validate_current_step(studio_state, services, runtime);
    
    if all_valid {
        // Save final state
        save_current_step(studio_state, services, runtime);
        
        // Mark wizard as complete
        if let Some(project_id) = studio_state.current_project_id {
            let services = services.clone();
            runtime.spawn(async move {
                // Update wizard completion status
                // This would trigger the generation phase
            });
        }
    }
    
    all_valid
}