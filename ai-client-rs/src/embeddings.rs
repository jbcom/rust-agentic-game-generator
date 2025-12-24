//! Embeddings generation module
//!
//! Handles generation of vector embeddings for semantic similarity.

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::embeddings::CreateEmbeddingRequestArgs,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    AiGenerator,
    cache::{AiCache, CachedData},
    tokens::TokenCounter,
};

/// Embeddings generator for semantic similarity
#[derive(Clone)]
pub struct EmbeddingsGenerator {
    client: Arc<Client<OpenAIConfig>>,
    cache: Arc<Mutex<AiCache>>,
    token_counter: Arc<Mutex<TokenCounter>>,
}

impl EmbeddingsGenerator {
    /// Create a new embeddings generator
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

    /// Generate embeddings for a single text
    pub async fn generate(&self, text: &str, model: &str) -> Result<Vec<f32>> {
        // Check cache first
        let cache_key = format!("embedding:{}:{}", model, text);

        if let Some(cached) = self.cache.lock().await.get(&cache_key).await {
            if let CachedData::Embedding(embedding) = cached.data {
                return Ok(embedding);
            }
        }

        let request = CreateEmbeddingRequestArgs::default()
            .model(model)
            .input(text)
            .build()?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .context("Failed to generate embedding")?;

        let embedding = response
            .data
            .first()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?
            .embedding
            .clone();

        // Track usage
        self.token_counter
            .lock()
            .await
            .record_usage(
                model,
                response.usage.prompt_tokens as usize,
                0,
            )
            .await?;

        // Cache result
        self.cache
            .lock()
            .await
            .put(
                cache_key,
                CachedData::Embedding(embedding.clone()),
                std::collections::HashMap::new(),
            )
            .await?;

        Ok(embedding)
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len() {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        norm_a = norm_a.sqrt();
        norm_b = norm_b.sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait::async_trait]
impl AiGenerator for EmbeddingsGenerator {
    async fn estimate_tokens(&self, request: &str) -> Result<usize> {
        let counter = self.token_counter.lock().await;
        counter.count_tokens(request, "text-embedding-3-small")
    }

    async fn estimate_cost(&self, request: &str) -> Result<f64> {
        let counter = self.token_counter.lock().await;
        counter.estimate_cost(request, "text-embedding-3-small", 0)
    }

    async fn is_cached(&self, key: &str) -> bool {
        self.cache.lock().await.get(key).await.is_some()
    }

    async fn clear_cache(&self, key: &str) -> Result<()> {
        self.cache.lock().await.clear(key).await
    }
}
