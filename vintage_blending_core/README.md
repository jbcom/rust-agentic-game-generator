# Vintage Blending Core

Core library for game blending functionality, used both at build time and runtime.

## Architecture

This crate provides:

1. **Types** (`types.rs`): Core data structures like `FeatureVector`, `GameMetadata`, `CompatibilityEdge`
2. **Graph** (`graph.rs`): Weighted graph implementation using `petgraph` for game relationships
3. **Metadata Builder** (`metadata.rs`): Used by `build.rs` to pre-compute feature vectors
4. **Similarity Engine** (`similarity.rs`): Algorithms for computing game compatibility

## Build-time Usage

In `build.rs`, we can now pre-compute metadata:

```rust
use vintage_blending_core::{MetadataBuilder, GameMetadata};

// During build
let builder = MetadataBuilder::new();
let mut all_metadata = HashMap::new();

for game in timeline_games {
    let metadata = builder.build_from_json(&game)?;
    all_metadata.insert(metadata.game_id.clone(), metadata);
}

// Pre-compute common pairings
builder.update_common_pairings(&mut all_metadata);

// Save to generated code or binary format
```

## Runtime Usage

At runtime, the pre-computed metadata enables fast blending:

```rust
use vintage_blending_core::{GameGraph, SimilarityEngine};

// Load pre-computed metadata
let metadata = load_metadata();

// Create graph
let graph = GameGraph::new(metadata)?;

// Find optimal blend
let blend_path = graph.find_blend_path(&selected_games)?;
```

## Graph Structure

The blending system uses a **weighted undirected graph** where:
- **Nodes** = Games
- **Edges** = Compatibility scores (0.0-1.0)
- **Metadata** = Pre-computed feature vectors

This allows:
- O(1) compatibility lookups for pre-computed pairs
- O(V²) worst-case for new combinations
- Efficient pathfinding with minimum spanning trees

## Why Petgraph?

[petgraph](https://docs.rs/petgraph) is the de facto standard for graph algorithms in Rust:
- Efficient graph representations
- Built-in algorithms (Dijkstra, MST, etc.)
- Zero-copy iterators
- Type-safe node/edge indices

## Feature Vectors

Each game has a feature vector containing:
- Genre weights (normalized)
- Mechanic flags (binary)
- Platform generation (1-5)
- Complexity score (0.0-1.0)
- Action/Strategy balance (-1.0 to 1.0)
- Single/Multiplayer balance (-1.0 to 1.0)

Similarity is computed using:
- Cosine similarity for genre weights
- Jaccard similarity for mechanics
- Linear interpolation for scalar values
