use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::wizard::config::ProjectConfig;

// Import from vintage_ai_client
use vintage_ai_client::{
    AiService,
    conversation::{
        ConversationManager,
        GameGenerationExt,
        GenerationProgress,
    },
    game_types::GameConfig,
};

pub struct GameGenerator {
    ai_service: AiService,
    conversation_manager: ConversationManager,
    templates_dir: PathBuf,
    project_config: Option<ProjectConfig>, // The "Bible" from wizard
}

impl GameGenerator {
    pub async fn new(templates_dir: PathBuf) -> anyhow::Result<Self> {
        let ai_service = AiService::from_env()?;
        let mut conversation_manager = ai_service.conversation();
        
        // Initialize templates for the conversation manager
        conversation_manager.init_templates(templates_dir.clone()).await?;
        
        Ok(Self {
            ai_service,
            conversation_manager,
            templates_dir,
            project_config: None,
        })
    }
    
    /// Set the project configuration (the "Bible" from wizard)
    pub fn set_project_config(&mut self, config: ProjectConfig) {
        self.project_config = Some(config);
    }
    
    /// Start a game design conversation
    pub async fn start_game_design_conversation(&self, initial_prompt: &str) -> anyhow::Result<(String, String)> {
        // Convert ProjectConfig to serde_json::Value for the conversation API
        let project_json = self.project_config.as_ref()
            .map(|config| serde_json::to_value(config).ok())
            .flatten();
        
        self.conversation_manager
            .start_game_design_conversation(initial_prompt, project_json)
            .await
    }
    
    /// Continue game design conversation
    pub async fn continue_game_design_conversation(
        &self, 
        conversation_id: &str, 
        user_input: &str
    ) -> anyhow::Result<(String, bool, Option<GameConfig>)> {
        self.conversation_manager
            .continue_game_design_conversation(conversation_id, user_input)
            .await
    }
    
    /// Generate full game with progress tracking
    pub async fn generate_full_game<F>(
        &self,
        config: &GameConfig,
        progress_callback: F,
    ) -> anyhow::Result<String>
    where
        F: Fn(GenerationProgress) + Send + 'static,
    {
        // Convert ProjectConfig to serde_json::Value
        let project_json = self.project_config.as_ref()
            .map(|config| serde_json::to_value(config).ok())
            .flatten();
        
        self.conversation_manager
            .generate_full_game(config, project_json, progress_callback)
            .await
    }
    
    /// Load a game template
    pub async fn load_template(&self, name: &str) -> anyhow::Result<GameConfig> {
        let templates_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("rpg-generator")
            .join("templates");
        
        let path = templates_dir.join(format!("{}.toml", name));
        let content = std::fs::read_to_string(path)?;
        
        Ok(toml::from_str(&content)?)
    }
    
    /// Get conversation history for a specific conversation
    pub async fn get_conversation_history(&self, conversation_id: &str) -> anyhow::Result<Vec<(String, String)>> {
        let conversation = self.conversation_manager.get_conversation(conversation_id).await?;
        
        let history: Vec<(String, String)> = conversation.messages
            .into_iter()
            .filter_map(|msg| {
                match msg.role {
                    vintage_ai_client::conversation::MessageRole::User => {
                        Some(("user".to_string(), msg.content))
                    }
                    vintage_ai_client::conversation::MessageRole::Assistant => {
                        Some(("assistant".to_string(), msg.content))
                    }
                    _ => None,
                }
            })
            .collect();
        
        Ok(history)
    }
}

// Helper types for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationState {
    pub conversation_id: String,
}

impl ConversationState {
    pub fn new(conversation_id: String) -> Self {
        Self { conversation_id }
    }
}
