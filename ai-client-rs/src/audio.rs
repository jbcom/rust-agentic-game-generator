//! Audio generation module
//!
//! Handles text-to-speech generation using OpenAI.

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::audio::{CreateSpeechRequestArgs, SpeechModel, Voice},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    AiGenerator,
    cache::{AiCache, CachedData},
    tokens::TokenCounter,
};

/// Audio generator for text-to-speech
#[derive(Clone)]
pub struct AudioGenerator {
    client: Arc<Client<OpenAIConfig>>,
    cache: Arc<Mutex<AiCache>>,
    token_counter: Arc<Mutex<TokenCounter>>,
}

/// Configuration for audio generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfig {
    /// Model to use (e.g., "tts-1", "tts-1-hd")
    pub model: String,
    /// Voice to use (alloy, echo, fable, onyx, nova, shimmer)
    pub voice: String,
    /// Response format (mp3, opus, aac, flac)
    pub response_format: String,
    /// Speed (0.25 to 4.0)
    pub speed: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            model: "tts-1".to_string(),
            voice: "alloy".to_string(),
            response_format: "mp3".to_string(),
            speed: 1.0,
        }
    }
}

impl AudioGenerator {
    /// Create a new audio generator
    pub fn new(
        client: Arc<Client<OpenAIConfig>>,
        cache: Arc<Mutex<AiCache>>,
        token_counter: Arc<Mutex<TokenCounter>>,
    ) -> Self {
        Self {
            client,
            cache,
            token_counter,
        }
    }

    /// Generate audio from text
    pub async fn generate(&self, text: &str, config: AudioConfig) -> Result<Vec<u8>> {
        // Check cache first
        let mut params = HashMap::new();
        params.insert("model".to_string(), config.model.clone());
        params.insert("voice".to_string(), config.voice.clone());
        params.insert("speed".to_string(), config.speed.to_string());

        let cache_key = self
            .cache
            .lock()
            .await
            .generate_key("audio", text, &params);

        if let Some(cached) = self.cache.lock().await.get(&cache_key).await {
            if let CachedData::Audio(audio) = cached.data {
                return Ok(audio);
            }
        }

        // Map to OpenAI types
        let model = match config.model.as_str() {
            "tts-1-hd" => SpeechModel::Tts1Hd,
            _ => SpeechModel::Tts1,
        };

        let voice = match config.voice.as_str() {
            "echo" => Voice::Echo,
            "fable" => Voice::Fable,
            "onyx" => Voice::Onyx,
            "nova" => Voice::Nova,
            "shimmer" => Voice::Shimmer,
            _ => Voice::Alloy,
        };

        // Create request
        let request = CreateSpeechRequestArgs::default()
            .input(text)
            .model(model)
            .voice(voice)
            .speed(config.speed)
            .build()?;

        // Make API call
        let response = self
            .client
            .audio()
            .speech()
            .create(request)
            .await
            .context("OpenAI speech generation failed")?;

        let audio_bytes = response.bytes.to_vec();

        // Track usage (OpenAI doesn't return tokens for TTS, using chars as proxy)
        self.token_counter
            .lock()
            .await
            .record_usage(&config.model, text.len() / 4, 0)
            .await?;

        // Cache result
        let cache_params: HashMap<String, serde_json::Value> = params
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        self.cache
            .lock()
            .await
            .put(cache_key, CachedData::Audio(audio_bytes.clone()), cache_params)
            .await?;

        Ok(audio_bytes)
    }
}

#[async_trait::async_trait]
impl AiGenerator for AudioGenerator {
    async fn estimate_tokens(&self, request: &str) -> Result<usize> {
        Ok(request.len() / 4)
    }

    async fn estimate_cost(&self, request: &str) -> Result<f64> {
        // TTS pricing is usually per char, but this is a rough estimate
        Ok(request.len() as f64 * 0.000015)
    }

    async fn is_cached(&self, key: &str) -> bool {
        self.cache.lock().await.get(key).await.is_some()
    }

    async fn clear_cache(&self, key: &str) -> Result<()> {
        self.cache.lock().await.clear(key).await
    }
}
