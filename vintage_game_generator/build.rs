use anyhow::Result;
use vintage_build_tools::VintageBuildTools;

const TIMELINE_START: i32 = 1980;  // Pac-Man year, genres established
const TIMELINE_END: i32 = 1995;    // Before 3D dominance

#[tokio::main]
async fn main() -> Result<()> {
    // Only rerun if templates change - those affect the generated output
    println!("cargo:rerun-if-changed=templates/giantbomb/mod.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/games.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/platforms.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/eras.rs.jinja");
    println!("cargo:rerun-if-changed=templates/giantbomb/graph.rs.jinja");

    // Build the vintage game data - the build tools handle all validation
    let build_tools = VintageBuildTools::from_env(TIMELINE_START, TIMELINE_END)?;
    build_tools.build().await?;
    
    Ok(())
}
