//! Similarity engine for game comparisons
//!
//! Provides various similarity metrics and algorithms

use crate::types::GameMetadata;

/// Engine for computing game similarities
pub struct SimilarityEngine {
    /// Weight for genre similarity (0.0-1.0)
    pub genre_weight: f32,
    /// Weight for mechanic similarity (0.0-1.0)
    pub mechanic_weight: f32,
    /// Weight for era similarity (0.0-1.0)
    pub era_weight: f32,
    /// Weight for complexity similarity (0.0-1.0)
    pub complexity_weight: f32,
}

impl SimilarityEngine {
    pub fn new() -> Self {
        Self {
            genre_weight: 0.3,
            mechanic_weight: 0.3,
            era_weight: 0.2,
            complexity_weight: 0.2,
        }
    }

    /// Compute similarity between two games
    pub fn compute_similarity(&self, game1: &GameMetadata, game2: &GameMetadata) -> f32 {
        let vector_sim = game1.feature_vector.similarity(&game2.feature_vector);
        let era_sim = self.compute_era_similarity(game1, game2);

        // Weighted combination
        let weighted_sim = vector_sim
            * (self.genre_weight + self.mechanic_weight + self.complexity_weight)
            + era_sim * self.era_weight;

        // Normalize
        weighted_sim
            / (self.genre_weight + self.mechanic_weight + self.era_weight + self.complexity_weight)
    }

    /// Compute era similarity
    fn compute_era_similarity(&self, game1: &GameMetadata, game2: &GameMetadata) -> f32 {
        if game1.era_category == game2.era_category {
            1.0
        } else {
            // Adjacent eras have some similarity
            let year_diff = (game1.year as i32 - game2.year as i32).abs();
            match year_diff {
                0..=2 => 0.9,
                3..=5 => 0.6,
                6..=8 => 0.3,
                _ => 0.1,
            }
        }
    }

    /// Find games most similar to a target
    pub fn find_similar_games(
        &self,
        target: &GameMetadata,
        candidates: &[GameMetadata],
        limit: usize,
    ) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = candidates
            .iter()
            .filter(|g| g.game_id != target.game_id)
            .map(|game| {
                let sim = self.compute_similarity(target, game);
                (game.game_id.clone(), sim)
            })
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);
        similarities
    }
}

impl Default for SimilarityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FeatureVector;
    use std::collections::HashMap;

    #[test]
    fn test_similarity_engine() {
        let engine = SimilarityEngine::new();

        let game1 = GameMetadata {
            game_id: "game1".to_string(),
            name: "Test Game 1".to_string(),
            year: 1985,
            feature_vector: FeatureVector {
                genre_weights: vec![1.0, 0.0, 0.0],
                mechanic_flags: vec![true, false, true],
                platform_generation: 2,
                complexity: 0.5,
                action_strategy_balance: -0.5,
                single_multi_balance: -0.8,
                semantic_embedding: None,
            },
            common_pairings: HashMap::new(),
            genre_affinities: HashMap::new(),
            mechanic_tags: vec!["Combat".to_string()],
            era_category: "mid_80s".to_string(),
            mood_tags: Vec::new(),
        };

        let game2 = GameMetadata {
            game_id: "game2".to_string(),
            name: "Test Game 2".to_string(),
            year: 1986,
            feature_vector: FeatureVector {
                genre_weights: vec![0.8, 0.2, 0.0],
                mechanic_flags: vec![true, false, false],
                platform_generation: 2,
                complexity: 0.6,
                action_strategy_balance: -0.4,
                single_multi_balance: -0.7,
                semantic_embedding: None,
            },
            common_pairings: HashMap::new(),
            genre_affinities: HashMap::new(),
            mechanic_tags: vec!["Combat".to_string()],
            era_category: "mid_80s".to_string(),
            mood_tags: Vec::new(),
        };

        let similarity = engine.compute_similarity(&game1, &game2);
        assert!(similarity > 0.5); // Should be quite similar
        assert!(similarity < 1.0); // But not identical
    }
}
