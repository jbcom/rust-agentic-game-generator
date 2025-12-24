use ai_client_rs::{AiService, text::TextConfig};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    // Initialize the AI service from environment variables
    // Requires OPENAI_API_KEY and optionally ANTHROPIC_API_KEY
    let ai = AiService::from_env()?;
    
    // Get the text generator
    let text_gen = ai.text();
    
    // 1. Generate text using OpenAI
    println!("Generating with OpenAI...");
    let openai_config = TextConfig {
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    
    let response = text_gen.generate(
        "Write a short haiku about Rust programming.",
        openai_config
    ).await?;
    
    println!("OpenAI Haiku:\n{}\n", response);
    
    // 2. Generate text using Anthropic (if key provided)
    if ai.anthropic_key.is_some() {
        println!("Generating with Anthropic...");
        let anthropic_config = TextConfig {
            model: "claude-3-haiku-20240307".to_string(),
            ..Default::default()
        };
        
        let response = text_gen.generate(
            "Write a short haiku about Rust programming.",
            anthropic_config
        ).await?;
        
        println!("Anthropic Haiku:\n{}\n", response);
    }
    
    // 3. Caching check
    println!("Checking cache...");
    let start = std::time::Instant::now();
    let _ = text_gen.generate(
        "Write a short haiku about Rust programming.",
        TextConfig {
            model: "gpt-4".to_string(),
            ..Default::default()
        }
    ).await?;
    println!("Cached response took: {:?}", start.elapsed());
    
    // 4. Statistics
    let stats = ai.token_counter.lock().await.get_stats().await;
    println!("\nUsage Stats:");
    println!("Total Tokens: {}", stats.prompt_tokens + stats.completion_tokens);
    println!("Total Cost: ${:.4}", stats.total_cost);
    
    Ok(())
}
