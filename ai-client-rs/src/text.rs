//! Text generation module
//!
//! Handles generation using OpenAI and Anthropic providers.

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    AiGenerator,
    cache::{AiCache, CachedData},
    tokens::TokenCounter,
};

/// Text generator for all text-based content
#[derive(Clone)]
pub struct TextGenerator {
    openai_client: Arc<Client<OpenAIConfig>>,
    anthropic_key: Option<String>,
    cache: Arc<Mutex<AiCache>>,
    token_counter: Arc<Mutex<TokenCounter>>,
}

/// Configuration for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// Model to use (e.g., "gpt-4", "claude-3-opus-20240229")
    pub model: String,
    /// Temperature for creativity (0.0-2.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u16,
    /// Top-p nucleus sampling
    pub top_p: f32,
    /// System prompt for context
    pub system_prompt: Option<String>,
    /// Provider to use (if None, inferred from model name)
    pub provider: Option<String>,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            top_p: 1.0,
            system_prompt: None,
            provider: None,
        }
    }
}

impl TextGenerator {
    /// Create a new text generator
    pub fn new(
        openai_client: Arc<Client<OpenAIConfig>>,
        anthropic_key: Option<String>,
        cache: Arc<Mutex<AiCache>>,
        token_counter: Arc<Mutex<TokenCounter>>,
    ) -> Self {
        Self {
            openai_client,
            anthropic_key,
            cache,
            token_counter,
        }
    }

    /// Generate text with caching and token tracking
    pub async fn generate(&self, prompt: &str, config: TextConfig) -> Result<String> {
        let provider = config.provider.clone().unwrap_or_else(|| {
            if config.model.starts_with("claude") {
                "anthropic".to_string()
            } else {
                "openai".to_string()
            }
        });

        // Generate cache key
        let mut params = HashMap::new();
        params.insert("model".to_string(), config.model.clone());
        params.insert("provider".to_string(), provider.clone());
        params.insert("temperature".to_string(), config.temperature.to_string());
        params.insert("max_tokens".to_string(), config.max_tokens.to_string());

        let cache_key = self
            .cache
            .lock()
            .await
            .generate_key("text", prompt, &params);

        // Check cache first
        if let Some(cached) = self.cache.lock().await.get(&cache_key).await {
            if let CachedData::Text(text) = cached.data {
                return Ok(text);
            }
        }

        let (text, prompt_tokens, completion_tokens) = match provider.as_str() {
            "anthropic" => self.generate_anthropic(prompt, &config).await?,
            _ => self.generate_openai(prompt, &config).await?,
        };

        // Track tokens
        self.token_counter
            .lock()
            .await
            .record_usage(&config.model, prompt_tokens, completion_tokens)
            .await?;

        // Cache result
        let cache_params: HashMap<String, serde_json::Value> = params
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        self.cache
            .lock()
            .await
            .put(cache_key, CachedData::Text(text.clone()), cache_params)
            .await?;

        Ok(text)
    }

    async fn generate_openai(
        &self,
        prompt: &str,
        config: &TextConfig,
    ) -> Result<(String, usize, usize)> {
        let mut messages = Vec::new();

        if let Some(system) = &config.system_prompt {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system.as_str())
                    .build()?
                    .into(),
            );
        }

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&config.model)
            .messages(messages)
            .temperature(config.temperature)
            .max_tokens(config.max_tokens)
            .top_p(config.top_p)
            .build()?;

        let response = self
            .openai_client
            .chat()
            .create(request)
            .await
            .context("OpenAI API call failed")?;

        let text = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let usage = response.usage.context("No usage info from OpenAI")?;

        Ok((
            text,
            usage.prompt_tokens as usize,
            usage.completion_tokens as usize,
        ))
    }

    async fn generate_anthropic(
        &self,
        prompt: &str,
        config: &TextConfig,
    ) -> Result<(String, usize, usize)> {
        let api_key = self
            .anthropic_key
            .as_ref()
            .context("Anthropic API key not provided")?;

        let client = reqwest::Client::new();
        
        let mut body = serde_json::json!({
            "model": config.model,
            "max_tokens": config.max_tokens,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": config.temperature,
            "top_p": config.top_p,
        });

        if let Some(system) = &config.system_prompt {
            body.as_object_mut().unwrap().insert("system".to_string(), serde_json::json!(system));
        }

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Anthropic API request failed")?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Anthropic API error: {}", error);
        }

        let result: serde_json::Value = response.json().await?;
        
        let text = result["content"][0]["text"]
            .as_str()
            .context("Failed to extract text from Anthropic response")?
            .to_string();
            
        let prompt_tokens = result["usage"]["input_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = result["usage"]["output_tokens"].as_u64().unwrap_or(0) as usize;

        Ok((text, prompt_tokens, completion_tokens))
    }
}

#[async_trait::async_trait]
impl AiGenerator for TextGenerator {
    async fn estimate_tokens(&self, request: &str) -> Result<usize> {
        let counter = self.token_counter.lock().await;
        counter.count_tokens(request, "gpt-4")
    }

    async fn estimate_cost(&self, request: &str) -> Result<f64> {
        let counter = self.token_counter.lock().await;
        counter.estimate_cost(request, "gpt-4", 1000)
    }

    async fn is_cached(&self, key: &str) -> bool {
        self.cache.lock().await.get(key).await.is_some()
    }

    async fn clear_cache(&self, key: &str) -> Result<()> {
        self.cache.lock().await.clear(key).await
    }
}
