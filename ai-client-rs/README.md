# ai-client-rs

Multi-provider AI abstraction for Rust - OpenAI, Anthropic with caching.

## Features

- **Unified Interface**: Use OpenAI and Anthropic through a single, consistent API.
- **Intelligent Caching**: Automatic disk and memory caching to reduce API calls and costs.
- **Token Counting**: Accurate token counting using `tiktoken-rs`.
- **Cost Estimation**: Track and estimate costs for all AI operations.
- **Provider Abstraction**: Easily switch between providers or use multiple providers in the same project.

## Support

- **Text Generation**: OpenAI (GPT-4, GPT-3.5) and Anthropic (Claude 3).
- **Image Generation**: OpenAI (DALL-E 3, DALL-E 2).
- **Audio Generation**: OpenAI (TTS).
- **Embeddings**: OpenAI.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ai-client-rs = "0.1.0"
```

## Quick Start

```rust
use ai_client_rs::{AiService, text::TextConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ai = AiService::from_env()?;
    let text_gen = ai.text();
    
    let response = text_gen.generate(
        "Hello, how are you?",
        TextConfig::default()
    ).await?;
    
    println!("Response: {}", response);
    Ok(())
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
