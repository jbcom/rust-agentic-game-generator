# AI RPG Generator

A comprehensive Bevy-based game development studio that uses AI to generate complete 16-bit nostalgic RPG games. This application combines a powerful metaprompt system with an integrated development environment (IDE) for creating, previewing, and editing AI-generated games.

## Overview

The AI RPG Generator is a complete game creation suite that:
- Uses OpenAI's GPT-4 for code generation and DALL-E 3 for asset creation
- Provides a visual wizard interface for game configuration
- Generates complete, playable Bevy 0.16.1 games with authentic 16-bit aesthetics
- Includes live preview and editing capabilities with bevy-inspector-egui
- Persists all project data using SeaORM with SQLite

## Architecture

The crate is organized into several key modules:

### Core Modules

- **`database/`** - SeaORM models and database management
- **`generator/`** - AI-powered game generation engine with style consistency
- **`services/`** - Business logic layer for data operations
- **`studio/`** - Main IDE interface with egui dock system
- **`wizard/`** - Project configuration wizard UI

### Supporting Directories

- **`defaults/`** - Default game configurations and ECS presets
- **`docs/`** - Implementation references and metaprompt documentation
- **`prompts/`** - TOML-based prompt templates for code generation
- **`templates/`** - Tera templates for code structure

## Key Features

### 1. Metaprompt System
The generator uses a sophisticated 5-phase metaprompt chain:
- Phase 1: Game concept discovery
- Phase 2: Technical architecture planning
- Phase 3: Asset generation (sprites, tilesets, audio)
- Phase 4: Component implementation
- Phase 5: Integration and polish

### 2. Style Consistency Management
- Enforced 16-bit color palettes (16-256 colors)
- Style guide generation as first step
- Validation and retry pipeline for consistency
- Post-processing to ensure authentic pixel art

### 3. Database-Backed Persistence
- Projects persist across sessions
- Asset versioning and history
- Generation task queue management
- API key storage with encryption

### 4. Integrated Development Environment
- Visual project wizard with validation
- Live game preview with hot reload
- Asset gallery with quality scoring
- Code editor with syntax highlighting
- Export wizard for distribution

## Dependencies

Key dependencies include:
- `bevy` - Game engine framework
- `bevy_egui` - UI integration
- `egui_dock` - Dockable window system
- `async-openai` - AI API integration
- `sea-orm` - Database ORM
- `tokio` - Async runtime
- Image processing: `image`, `imageproc`, `palette`
- Advanced features: `candle-core`, `ort` (optional ML support)

## Usage

```rust
use ai_rpg_generator::GameGeneratorStudioPlugin;

fn main() {
    App::new()
        .add_plugins(GameGeneratorStudioPlugin)
        .run();
}
```

## Project Structure

```
ai_rpg_generator/
├── src/
│   ├── database/      # Database models and migrations
│   ├── generator/     # AI generation engine
│   ├── services/      # Service layer
│   ├── studio/        # Main IDE interface
│   ├── wizard/        # Configuration wizard
│   ├── lib.rs         # Plugin exports
│   └── main.rs        # Application entry point
├── defaults/          # Default configurations
├── docs/              # Documentation and references
├── prompts/           # AI prompt templates
├── templates/         # Code generation templates
└── tests/             # Test suites
```

## Development

The application uses a plugin-based architecture where each major system is a Bevy plugin. The tokio runtime is integrated for async database operations while maintaining Bevy's ECS paradigm.

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

## License

MIT OR Apache-2.0
