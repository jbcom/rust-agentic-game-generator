use crate::core::images::raster::ai::{load_generation_request, GenerationRequest};

fn main() {
    let toml_content = r#"
[prompt]
description = "RPG wizard logo"
size = "128x128"
style = "pixel_art"
output_path = "logo.png"
"#;

    println!("Testing TOML parsing...");
    match load_generation_request(toml_content, Some("prompt")) {
        Ok(request) => {
            println!("✅ Successfully parsed TOML:");
            println!("  Description: {}", request.description);
            println!("  Size: {:?}", request.size);
            println!("  Output path: {:?}", request.output_path);
            println!("  Metadata: {:?}", request.metadata);
            println!("  Formatted prompt: {}", request.format_prompt());
        }
        Err(e) => {
            println!("❌ Failed to parse TOML: {}", e);
        }
    }
}
