# Contributing to Home Assistant Installer

Thank you for your interest in contributing! This document provides guidelines
and instructions for contributing.

## Getting Started

### Prerequisites

- Rust (via rustup)
- Node.js 22+
- Platform-specific Tauri dependencies

### Setup

```bash
# Clone the repository
git clone https://github.com/home-assistant/hai.git
cd hai

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Running Tests

```bash
# All tests
npm test

# Unit tests only
npm run test:unit

# E2E tests (with mock mode)
HA_INSTALLER_MOCK=true npm run test:e2e

# Rust tests
cd src-tauri && cargo test
```

## Development Guidelines

### Code Style

- **Rust**: Follow `rustfmt` defaults, run `cargo fmt` before committing
- **TypeScript**: Follow ESLint config, run `npm run lint` before committing
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(proxmox): add node selection dropdown`
- `fix(flash): handle USB disconnect during write`
- `docs: update installation instructions`

### Branch Naming

- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring

### Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure all tests pass
5. Submit a pull request

### Design Principles

When contributing UI changes, remember:

1. **Visual-first**: The UI should be understandable without reading text
2. **Use icons and images**: Every option should have a visual identity
3. **Follow HA branding**: Use Home Assistant colors and style
4. **Include Casita**: Use the mascot for personality in appropriate places

## Getting Help

- [GitHub Discussions](https://github.com/home-assistant/hai/discussions)
- [Home Assistant Discord](https://discord.gg/home-assistant)
- [Community Forum](https://community.home-assistant.io/)

## License

By contributing, you agree that your contributions will be licensed under the
Apache 2.0 License.
