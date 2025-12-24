//! Image generation module
//!
//! Handles image generation using OpenAI (DALL-E) and potentially other providers.

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::images::{CreateImageRequestArgs, ImageModel, ImageQuality, ImageResponseFormat, ImageSize, Image as OpenAIImage},
};
use base64::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    AiGenerator,
    cache::{AiCache, ImageCache},
    tokens::TokenCounter,
};

/// Image generator
#[derive(Clone)]
pub struct ImageGenerator {
    client: Arc<Client<OpenAIConfig>>,
    cache: Arc<Mutex<AiCache>>,
    image_cache: ImageCache,
    token_counter: Arc<Mutex<TokenCounter>>,
}

/// Configuration for image generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageConfig {
    /// Model to use (e.g., "dall-e-3")
    pub model: String,
    /// Image size
    pub size: String,
    /// Quality (standard or hd for dall-e-3)
    pub quality: String,
    /// Number of images to generate
    pub n: u8,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            model: "dall-e-3".to_string(),
            size: "1024x1024".to_string(),
            quality: "standard".to_string(),
            n: 1,
        }
    }
}

impl ImageGenerator {
    /// Create a new image generator
    pub fn new(
        client: Arc<Client<OpenAIConfig>>,
        cache: Arc<Mutex<AiCache>>,
        token_counter: Arc<Mutex<TokenCounter>>,
    ) -> Self {
        let image_cache = ImageCache::new(cache.clone()
            .try_lock()
            .ok()
            .map(|guard| Arc::new(guard.clone()))
            .unwrap_or_else(|| Arc::new(AiCache::new().unwrap())));

        Self {
            client,
            cache,
            image_cache,
            token_counter,
        }
    }

    /// Generate a single image
    pub async fn generate_single(&self, prompt: &str, config: ImageConfig) -> Result<Vec<u8>> {
        // Check cache first
        let mut params = HashMap::new();
        params.insert("model".to_string(), config.model.clone());
        params.insert("size".to_string(), config.size.clone());
        params.insert("quality".to_string(), config.quality.clone());

        let cache_key = self
            .cache
            .lock()
            .await
            .generate_key("image", prompt, &params);

        if let Some(cached_data) = self
            .image_cache
            .get_image(&cache_key, super::cache::ImageFormat::Png)
            .await
        {
            return Ok(cached_data);
        }

        // Map to OpenAI types
        let model = match config.model.as_str() {
            "dall-e-2" => ImageModel::DallE2,
            _ => ImageModel::DallE3,
        };

        let size = match config.size.as_str() {
            "256x256" => ImageSize::S256x256,
            "512x512" => ImageSize::S512x512,
            "1024x1024" => ImageSize::S1024x1024,
            "1792x1024" => ImageSize::S1792x1024,
            "1024x1792" => ImageSize::S1024x1792,
            _ => ImageSize::S1024x1024,
        };

        let quality = match config.quality.as_str() {
            "hd" => ImageQuality::HD,
            _ => ImageQuality::Standard,
        };

        // Create request
        let request = CreateImageRequestArgs::default()
            .prompt(prompt)
            .model(model)
            .n(config.n)
            .quality(quality)
            .response_format(ImageResponseFormat::B64Json)
            .size(size)
            .build()?;

        // Make API call
        let response = self
            .client
            .images()
            .generate(request)
            .await
            .context("OpenAI image generation failed")?;

        let image_data = response
            .data
            .first()
            .ok_or_else(|| anyhow::anyhow!("No image data in response"))?;

        let image_bytes = match image_data.as_ref() {
            OpenAIImage::B64Json { b64_json, .. } => {
                base64::engine::general_purpose::STANDARD
                    .decode(b64_json.as_ref())
                    .context("Failed to decode base64 image data")?
            }
            OpenAIImage::Url { url, .. } => {
                anyhow::bail!("Expected base64 data but got URL: {url}");
            }
        };

        // Track usage
        let model_name = match config.quality.as_str() {
            "hd" => "dall-e-3-hd",
            _ => "dall-e-3",
        };
        self.token_counter
            .lock()
            .await
            .record_image_generation(model_name, 1)
            .await?;

        // Cache result
        let cache_params: HashMap<String, serde_json::Value> = params
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        self.image_cache
            .put_image(cache_key, image_bytes.clone(), cache_params)
            .await?;

        Ok(image_bytes)
    }
}

#[async_trait::async_trait]
impl AiGenerator for ImageGenerator {
    async fn estimate_tokens(&self, _request: &str) -> Result<usize> {
        Ok(self
            .token_counter
            .lock()
            .await
            .estimate_image_tokens(1024, 1024))
    }

    async fn estimate_cost(&self, _request: &str) -> Result<f64> {
        Ok(0.04)
    }

    async fn is_cached(&self, key: &str) -> bool {
        self.cache.lock().await.get(key).await.is_some()
    }

    async fn clear_cache(&self, key: &str) -> Result<()> {
        self.cache.lock().await.clear(key).await
    }
}
