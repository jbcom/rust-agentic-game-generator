# Changelog

All notable changes to this project will be documented in this file. See [Conventional Commits](https://conventionalcommits.org) for commit guidelines.

## Initial Releases

This file will be automatically updated by semantic-release based on conventional commits.

### Commit Message Format

This project uses [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Types

- **feat**: A new feature (triggers minor version bump)
- **fix**: A bug fix (triggers patch version bump)
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (formatting, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Performance improvement (triggers patch version bump)
- **test**: Adding or updating tests
- **build**: Changes to build system or dependencies
- **ci**: Changes to CI configuration
- **chore**: Other changes that don't modify src or test files

#### Breaking Changes

Add `!` after type or `BREAKING CHANGE:` in footer to trigger major version bump:

```
feat!: remove deprecated API
```

or

```
feat: update authentication flow

BREAKING CHANGE: authentication now requires OAuth2
```

#### Scopes

Suggested scopes for this project:
- `ai-client`: Changes to vintage_ai_client crate
- `blending`: Changes to vintage_blending_core crate  
- `generator`: Changes to vintage_game_generator crate
- `combat`: Changes to bevy-combat crate
- `build-tools`: Changes to vintage_build_tools crate
- `ci`: CI/CD changes
- `deps`: Dependency updates

#### Examples

```
feat(ai-client): add DALL-E 3 image generation support

Implements DALL-E 3 with quality and size options, caching, and cost tracking.

Resolves #15
```

```
fix(generator): resolve compilation errors in freeform conversation

- Fix lifetime issues in async streaming
- Remove unused imports
- Add missing embedding_model field

Fixes #36
```

```
chore(deps): update bevy to 0.16.1

Updates Bevy framework and related dependencies to latest stable versions.
```
