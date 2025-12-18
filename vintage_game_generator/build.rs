use anyhow::Result;
use vintage_build_tools::VintageBuildTools;

const TIMELINE_START: i32 = 1980; // Pac-Man year, genres established
const TIMELINE_END: i32 = 1995; // Before 3D dominance

#[tokio::main]
async fn main() -> Result<()> {
    // Only rerun if templates change - those affect the generated output
    println!("cargo:rerun-if-changed=templates/giantbomb/mod.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/games.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/platforms.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/eras.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/graph.rs.jinja");

    // Build the vintage game data - the build tools handle all validation
    match VintageBuildTools::from_env(TIMELINE_START, TIMELINE_END) {
        Ok(build_tools) => {
            build_tools.build().await?;
        }
        Err(e) => {
            // Print warning but don't fail - allows building without API key
            // The generated code will be stubs/placeholders
            eprintln!("cargo:warning=Skipping game data generation: {e}");
            eprintln!("cargo:warning=Set GIANTBOMB_API_KEY to generate real game data");

            // Generate stub modules so the crate compiles
            generate_stub_modules()?;
        }
    }

    Ok(())
}

/// Generate stub modules when API key is not available
fn generate_stub_modules() -> Result<()> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Generate minimal stub for giantbomb module
    let stub_content = r#"
// Auto-generated stub - run with GIANTBOMB_API_KEY to generate real data

pub mod games {
    pub fn get_all_games() -> Vec<()> { Vec::new() }
}

pub mod platforms {
    pub fn get_all_platforms() -> Vec<()> { Vec::new() }
}

pub mod eras {
    pub fn get_all_eras() -> Vec<()> { Vec::new() }
}

pub mod graph {
    pub fn get_game_graph() -> () { () }
}
"#;

    fs::write(out_dir.join("giantbomb.rs"), stub_content)?;

    Ok(())
}
