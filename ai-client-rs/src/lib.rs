//! Multi-provider AI abstraction for Rust - OpenAI, Anthropic with caching
//!
//! This crate provides a unified interface for various AI providers,
//! with built-in support for caching, token counting, and cost estimation.

pub mod audio;
pub mod cache;
pub mod embeddings;
pub mod image;
pub mod text;
pub mod tokens;

use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Common trait for all AI generators
#[async_trait::async_trait]
pub trait AiGenerator: Send + Sync {
    /// Get estimated tokens for a request
    async fn estimate_tokens(&self, request: &str) -> Result<usize>;

    /// Get estimated cost for a request
    async fn estimate_cost(&self, request: &str) -> Result<f64>;

    /// Check if result is cached
    async fn is_cached(&self, key: &str) -> bool;

    /// Clear cache for specific key
    async fn clear_cache(&self, key: &str) -> Result<()>;
}

/// Global AI service manager that coordinates different AI providers and services.
/// 
/// It maintains shared state like the OpenAI client, caching engine, and token counter.
#[derive(Clone)]
pub struct AiService {
    /// OpenAI client for API calls.
    pub openai_client: Arc<Client<OpenAIConfig>>,
    /// Optional Anthropic API key. If provided, Claude models can be used.
    pub anthropic_key: Option<String>,
    /// Thread-safe cache manager for all AI operations.
    pub cache: Arc<Mutex<cache::AiCache>>,
    /// Thread-safe token counter for cost and usage tracking.
    pub token_counter: Arc<Mutex<tokens::TokenCounter>>,
}

impl AiService {
    /// Create a new AI service instance
    pub fn new() -> Result<Self> {
        let openai_config = OpenAIConfig::new();
        let openai_client = Arc::new(Client::with_config(openai_config));
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();

        Ok(Self {
            openai_client,
            anthropic_key,
            cache: Arc::new(Mutex::new(cache::AiCache::new()?)),
            token_counter: Arc::new(Mutex::new(tokens::TokenCounter::new())),
        })
    }

    /// Initialize from environment variables
    pub fn from_env() -> Result<Self> {
        Self::new()
    }

    /// Get a reference to the text generation service
    pub fn text(&self) -> text::TextGenerator {
        text::TextGenerator::new(
            self.openai_client.clone(),
            self.anthropic_key.clone(),
            self.cache.clone(),
            self.token_counter.clone(),
        )
    }

    /// Get a reference to the image generation service
    pub fn image(&self) -> image::ImageGenerator {
        image::ImageGenerator::new(
            self.openai_client.clone(),
            self.cache.clone(),
            self.token_counter.clone(),
        )
    }

    /// Get a reference to the audio generation service
    pub fn audio(&self) -> audio::AudioGenerator {
        audio::AudioGenerator::new(
            self.openai_client.clone(),
            self.cache.clone(),
            self.token_counter.clone(),
        )
    }

    /// Get a reference to the embeddings service
    pub fn embeddings(&self) -> embeddings::EmbeddingsGenerator {
        embeddings::EmbeddingsGenerator::new(
            self.openai_client.clone(),
            self.cache.clone(),
            self.token_counter.clone(),
        )
    }
}

/// Configuration for AI services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiConfig {
    // Model Selection
    /// Text generation model (e.g., gpt-4, claude-3-opus)
    pub text_model: String,
    /// Image generation model (e.g., dall-e-3)
    pub image_model: String,
    /// Audio generation model (e.g., tts-1)
    pub audio_model: String,

    // Generation Parameters
    /// Temperature for generation (0.0 - 2.0)
    pub temperature: f32,
    /// Top-p for text generation (0.0-1.0)
    pub top_p: f32,
    /// Maximum tokens per request
    pub max_tokens: u32,

    // Provider Settings
    /// AI provider (openai, anthropic)
    pub ai_provider: String,

    // Cache and Performance
    /// Enable AI response caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            text_model: "gpt-4".to_string(),
            image_model: "dall-e-3".to_string(),
            audio_model: "tts-1".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            max_tokens: 2000,
            ai_provider: "openai".to_string(),
            cache_enabled: true,
            cache_ttl: 3600 * 24 * 7, // 1 week
        }
    }
}
