# Technology Context

**Last Updated**: 2025-08-01 02:34 CST

## Core Technology Stack

### Language & Framework
- **Rust** (2024 Edition) - Primary language
- **Bevy 0.16.1** - Game engine and application framework
- **SeaORM** - Database ORM for SQLite
- **MinJinja** - Template engine for prompt and code generation

### UI Technologies
- **Bevy UI** - Native Bevy UI system (no egui)
- **bevy_mod_picking** - Enhanced UI interaction
- **bevy_asset_loader** - Declarative asset loading

### AI Integration
- **async-openai 0.29** - OpenAI API client
- **reqwest** - HTTP client for other AI providers
- **MinJinja templates** - Prompt generation
- **Metaprompt Architecture** - Dynamic prompt generation system

### Database
- **SQLite** - Embedded database for all game state
- **SeaORM** - Async ORM with migration support
- **sea-orm-cli** - Database migration tooling

### Asset Generation
- **DALL-E 3** - Sprite and asset generation
- **image** crate - Image manipulation
- **Custom pixel art enhancer** - Upscaling and style consistency

### Game-Specific Libraries
- **bevy_ecs_tilemap** - Efficient tilemap rendering
- **bevy_kira_audio** - Audio playback
- **bevy_yarnspinner** - Dialogue system
- **bevy_rapier2d** - Physics (optional)
- **bevy-combat** - (Internal) Generic combat system template

### Development Tools
- **cargo-watch** - Auto-reload during development
- **pytest + pytest-mock** - Python testing (legacy)
- **reqwest-vcr** - VCR-style API testing
- **cached** - Function-level caching

## Architecture Decisions

### Metaprompt Architecture (NEW)
- Three-level hierarchy: Config → Metaprompts → Prompts → Content
- Dynamic prompt generation based on game features
- Infinite flexibility without hardcoded templates
- Feature-driven prompt adaptation

### Database-First Design
- All game state stored in SQLite
- ECS components are simple UUID markers
- Complex logic in database, not Rust structs
- AI excels at SQL generation

### Multi-Provider AI Strategy
- Provider-agnostic interface
- Support for OpenAI, Anthropic, Google, OpenRouter
- Local LLM support via Ollama
- Automatic fallback on errors

### Event-Driven Architecture
- Bevy events for all communication
- Loose coupling between modules
- Testable and maintainable

### Plugin-Based Modularity
- Wizard, Generator, Studio as plugins
- Optional features via feature flags
- Clear dependency boundaries

## Technology Constraints

### Bevy 0.16.1 Specific
- Use component tuples, not bundles
- `Text::from_section()` not `TextBundle`
- No `NodeBundle`, use `(Node::default(), Style { ... })`
- Rust 2024 module system (no mod.rs needed)

### Performance Considerations
- Batch database operations
- Cache metaprompt outputs
- Async for all I/O operations
- Stream large AI responses

### Testing Requirements
- VCR for deterministic API tests
- In-memory SQLite for unit tests
- GUI test harness for integration
- Screenshot-based regression tests

## Integration Points

### Wizard → Generator
- `WizardCompleteEvent` triggers generation
- Game config passed via event
- Progress tracked in database

### Generator → Studio
- Generated assets hot-reload
- Progress events update UI
- Error events show notifications

### Database → ECS
- Simple mapper pattern
- UUID components reference DB
- Systems query DB for complex data

## Future Technology Additions

### Planned Integrations
- **bevy_mod_debugdump** - ECS visualization
- **bevy_hanabi** - Particle effects
- **bevy_ui_navigation** - Gamepad support
- **wgpu** direct access - Custom shaders

### AI Provider Expansions
- Anthropic Claude integration
- Google Gemini support
- OpenRouter for model variety
- Ollama for local models

### Asset Pipeline Enhancements
- Sprite sheet generation
- Audio synthesis integration
- Procedural music generation
- Voice synthesis for NPCs

## Migration History

### From Tera to MinJinja (Completed)
- All templates migrated
- Better error messages
- Template inheritance support
- Smaller binary size

### From Custom to Ecosystem (In Progress)
- VCR: Custom → reqwest-vcr ✅
- Templates: Tera → MinJinja ✅
- Caching: Custom → cached crate ✅
- Assets: Manual → bevy_asset_loader (planned)

### From Hardcoded to Metaprompts (Active)
- Static prompts → Dynamic generation
- Feature analysis → Tailored prompts
- Infinite game combinations supported

## Development Environment

### Required Tools
- Rust 1.88+ (2024 edition)
- cargo and rustup
- SQLite CLI tools
- Git for version control

### Recommended Setup
- VS Code with rust-analyzer
- cargo-watch for hot reload
- Just for task automation
- direnv for environment variables

### Environment Variables
- `OPENAI_API_KEY` - Required for generation
- `DATABASE_URL` - SQLite connection
- `RUST_LOG` - Logging configuration
- `ASSET_DIR` - Generated asset location
