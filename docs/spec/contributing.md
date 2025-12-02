# Contributor Tooling & Automation

## AI Agent Instructions

### GitHub Copilot Instructions

```markdown
# .github/copilot-instructions.md

# Home Assistant Installer - Copilot Instructions

## Project Overview
This is a Tauri desktop application for installing Home Assistant OS.
- Backend: Rust (src-tauri/)
- Frontend: Lit web components + Web Awesome UI library (src/)
- Testing: Playwright for E2E, Web Test Runner for unit tests

## Code Style

### Rust
- Use `rustfmt` defaults
- Prefer `thiserror` for error types
- Use `tokio` for async runtime
- All public functions need doc comments

### TypeScript/Lit
- Use TypeScript strict mode
- Components use `@customElement` decorator
- Styles use CSS-in-JS via Lit's `css` tagged template
- Prefer Web Awesome components (`<wa-*>`) over custom implementations

## Architecture Guidelines
- Tauri commands go in `src-tauri/src/commands/`
- Each command module handles one domain (devices, flash, proxmox, utm)
- Frontend state management uses simple stores in `src/state/`
- Components are in `src/components/`, views in `src/views/`

## Testing
- All Tauri commands need unit tests
- All components need basic render tests
- User flows need Playwright E2E tests
- Use mock mode (`HA_INSTALLER_MOCK=true`) for testing without hardware

## Common Tasks
- Add new device: Update manifest schema and device selector component
- Add new installation path: Create new view folder, add to wizard routing
- Fix flashing issue: Check platform-specific code in `src-tauri/src/platforms/`
```

### Claude Code Instructions

```markdown
# .github/claude-instructions.md

# Home Assistant Installer - Claude Code Instructions

## Project Context
Home Assistant Installer is a cross-platform desktop app built with Tauri.
It helps users install Home Assistant OS on various hardware platforms.

## Tech Stack
- Tauri 2.x (Rust backend + web frontend)
- Lit for web components
- Web Awesome for UI components
- Playwright for E2E testing
- GitHub Actions for CI/CD

## Key Directories
- `src-tauri/src/commands/` - Tauri command handlers
- `src-tauri/src/platforms/` - Platform-specific implementations (disk access)
- `src/components/` - Reusable Lit components
- `src/views/` - Page-level view components
- `test/e2e/` - Playwright tests

## Design Principles
- Visual-first: UI should be usable without reading
- Every option needs an icon or image
- Use Casita mascot for personality (progress, success, error states)
- Follow Home Assistant brand guidelines

## When Making Changes
1. Check if similar patterns exist in codebase
2. Ensure Rust code passes `cargo clippy`
3. Ensure TypeScript passes `npm run lint`
4. Add tests for new functionality
5. Update relevant documentation

## Important Notes
- Mock mode available for testing: `HA_INSTALLER_MOCK=true`
- Manifest data comes from version.home-assistant.io
- No auto-update; version check with download prompt only
- Releases are signed with cosign
```

---

## GitHub Issue Forms

### Bug Report

```yaml
# .github/ISSUE_TEMPLATE/bug_report.yml
name: Bug Report
description: Report a problem with the installer
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug!
        Please fill out the information below to help us investigate.

  - type: dropdown
    id: platform
    attributes:
      label: Operating System
      description: What OS are you running the installer on?
      options:
        - macOS (Apple Silicon)
        - macOS (Intel)
        - Windows 11
        - Windows 10
        - Linux (Ubuntu/Debian)
        - Linux (Fedora/RHEL)
        - Linux (Other)
    validations:
      required: true

  - type: dropdown
    id: installation-path
    attributes:
      label: Installation Path
      description: What were you trying to install?
      options:
        - Raspberry Pi / SBC
        - Mini PC / NUC
        - Home Assistant Yellow
        - Home Assistant Green
        - Proxmox VM
        - UTM VM (macOS)
        - Other
    validations:
      required: true

  - type: input
    id: version
    attributes:
      label: Installer Version
      description: Which version of the installer? (shown in app or filename)
      placeholder: "v1.0.0"
    validations:
      required: true

  - type: textarea
    id: description
    attributes:
      label: What happened?
      description: Describe the bug clearly
      placeholder: "When I clicked 'Flash', the app showed an error..."
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: What did you expect?
      description: What should have happened instead?
    validations:
      required: true

  - type: textarea
    id: steps
    attributes:
      label: Steps to Reproduce
      description: How can we reproduce this?
      placeholder: |
        1. Open the installer
        2. Select Raspberry Pi 5
        3. Insert SD card
        4. Click Flash
        5. See error
    validations:
      required: true

  - type: textarea
    id: logs
    attributes:
      label: Logs or Screenshots
      description: Paste any error messages or attach screenshots
    validations:
      required: false

  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      options:
        - label: I have searched existing issues to avoid duplicates
          required: true
        - label: I am running the latest version of the installer
          required: false
```

### Feature Request

```yaml
# .github/ISSUE_TEMPLATE/feature_request.yml
name: Feature Request
description: Suggest a new feature or improvement
labels: ["enhancement", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for your suggestion! Please describe your idea below.

  - type: textarea
    id: problem
    attributes:
      label: Problem or Use Case
      description: What problem does this solve? What are you trying to do?
      placeholder: "I want to install Home Assistant on my XYZ device, but..."
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: How would you like this to work?
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: Any other approaches you've thought about?
    validations:
      required: false

  - type: dropdown
    id: area
    attributes:
      label: Area
      description: What part of the installer does this relate to?
      options:
        - New device support
        - New installation method
        - User interface
        - Performance
        - Documentation
        - Other
    validations:
      required: true

  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      options:
        - label: I have searched existing issues to avoid duplicates
          required: true
```

### Device Support Request

```yaml
# .github/ISSUE_TEMPLATE/device_request.yml
name: Device Support Request
description: Request support for a new device or board
labels: ["device-support", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Want to see a new device supported? Let us know!

  - type: input
    id: device-name
    attributes:
      label: Device Name
      description: Full name of the device
      placeholder: "ODROID-M2"
    validations:
      required: true

  - type: input
    id: manufacturer
    attributes:
      label: Manufacturer
      placeholder: "Hardkernel"
    validations:
      required: true

  - type: dropdown
    id: device-type
    attributes:
      label: Device Type
      options:
        - Single Board Computer (SBC)
        - Mini PC / NUC
        - Official Home Assistant Hardware
        - Other
    validations:
      required: true

  - type: input
    id: haos-support
    attributes:
      label: Home Assistant OS Support
      description: Is there already a HAOS image for this device?
      placeholder: "Yes - https://github.com/home-assistant/operating-system/releases"
    validations:
      required: true

  - type: textarea
    id: details
    attributes:
      label: Additional Details
      description: Any other relevant information (links, specs, etc.)
    validations:
      required: false
```

### Issue Config

```yaml
# .github/ISSUE_TEMPLATE/config.yml
blank_issues_enabled: false
contact_links:
  - name: Home Assistant Community
    url: https://community.home-assistant.io/
    about: Ask questions and get help from the community
  - name: Home Assistant Discord
    url: https://discord.gg/home-assistant
    about: Chat with the community in real-time
  - name: Documentation
    url: https://www.home-assistant.io/installation/
    about: Official installation documentation
```

---

## Pull Request Template

```markdown
# .github/PULL_REQUEST_TEMPLATE.md

## Description

<!-- Describe your changes -->

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] CI/CD changes

## Related Issues

<!-- Link any related issues: Fixes #123, Relates to #456 -->

## Testing

<!-- How did you test this? -->

- [ ] Unit tests added/updated
- [ ] E2E tests added/updated
- [ ] Manually tested on macOS
- [ ] Manually tested on Windows
- [ ] Manually tested on Linux

## Screenshots

<!-- If UI changes, add before/after screenshots -->

## Checklist

- [ ] My code follows the project's code style
- [ ] I have run `cargo fmt` and `npm run lint`
- [ ] I have added tests for my changes
- [ ] All new and existing tests pass
- [ ] I have updated documentation if needed
- [ ] My commits follow conventional commit format
```

---

## Copilot Code Review

```yaml
# .github/copilot-review.yml
# Configuration for GitHub Copilot code review

reviews:
  # Auto-review settings
  auto_review:
    enabled: true
    
  # What to focus on
  focus_areas:
    - security
    - error_handling
    - performance
    - accessibility
    
  # Language-specific guidance
  languages:
    rust:
      - Check for proper error handling with Result types
      - Ensure no unwrap() in production code paths
      - Verify async code properly handles cancellation
      - Check for potential panics
    typescript:
      - Ensure proper typing (no `any`)
      - Check for accessibility in components
      - Verify event cleanup in Lit lifecycle
      - Check for memory leaks in subscriptions

  # Custom review prompts
  prompts:
    - "Does this change handle offline/network error scenarios?"
    - "Are there any potential security issues with user input?"
    - "Is the UI accessible (keyboard navigation, screen readers)?"
    - "Does this follow the visual-first design principle?"
```

---

## Renovate Configuration

```json5
// renovate.json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": [
    "config:recommended",
    ":semanticCommits",
    ":preserveSemverRanges",
    "group:allNonMajor"
  ],
  
  "labels": ["dependencies"],
  
  "schedule": ["before 6am on monday"],
  
  "timezone": "UTC",
  
  "prHourlyLimit": 2,
  "prConcurrentLimit": 5,
  
  "packageRules": [
    {
      "description": "Group Tauri packages",
      "matchPackagePatterns": ["^@tauri-apps/", "^tauri"],
      "groupName": "tauri",
      "groupSlug": "tauri"
    },
    {
      "description": "Group Lit packages",
      "matchPackagePatterns": ["^lit", "^@lit/"],
      "groupName": "lit",
      "groupSlug": "lit"
    },
    {
      "description": "Group Web Awesome packages",
      "matchPackagePatterns": ["webawesome", "shoelace"],
      "groupName": "web-awesome",
      "groupSlug": "web-awesome"
    },
    {
      "description": "Group Playwright packages",
      "matchPackagePatterns": ["^@playwright/", "^playwright"],
      "groupName": "playwright",
      "groupSlug": "playwright"
    },
    {
      "description": "Group testing packages",
      "matchPackagePatterns": ["^@web/test-runner", "^@open-wc/testing", "^sinon"],
      "groupName": "testing",
      "groupSlug": "testing"
    },
    {
      "description": "Group Rust crates",
      "matchManagers": ["cargo"],
      "groupName": "rust-dependencies",
      "groupSlug": "rust-deps"
    },
    {
      "description": "Group GitHub Actions",
      "matchManagers": ["github-actions"],
      "groupName": "github-actions",
      "groupSlug": "gha"
    },
    {
      "description": "Auto-merge patch updates",
      "matchUpdateTypes": ["patch"],
      "automerge": true,
      "automergeType": "pr",
      "platformAutomerge": true
    },
    {
      "description": "Auto-merge minor dev dependencies",
      "matchDepTypes": ["devDependencies"],
      "matchUpdateTypes": ["minor"],
      "automerge": true
    }
  ],
  
  "vulnerabilityAlerts": {
    "enabled": true,
    "labels": ["security"]
  },
  
  "rust": {
    "enabled": true
  }
}
```

---

## CODEOWNERS

```
# .github/CODEOWNERS

# Default owners for everything
* @home-assistant/installer-team

# Rust backend
/src-tauri/ @home-assistant/installer-backend

# Frontend components
/src/ @home-assistant/installer-frontend

# CI/CD and infrastructure
/.github/ @home-assistant/installer-infra
/renovate.json @home-assistant/installer-infra

# Documentation
/docs/ @home-assistant/docs-team
*.md @home-assistant/docs-team
```

---

## Contributing Guide

```markdown
# CONTRIBUTING.md

# Contributing to Home Assistant Installer

Thank you for your interest in contributing! This document provides guidelines
and instructions for contributing.

## Getting Started

### Prerequisites

- Rust (via rustup)
- Node.js 20+
- Platform-specific Tauri dependencies

### Setup

\`\`\`bash
# Clone the repository
git clone https://github.com/home-assistant/hai.git
cd hai

# Install dependencies
npm install

# Start development server
npm run tauri dev
\`\`\`

### Running Tests

\`\`\`bash
# All tests
npm test

# Unit tests only
npm run test:unit

# E2E tests (with mock mode)
HA_INSTALLER_MOCK=true npm run test:e2e

# Rust tests
cd src-tauri && cargo test
\`\`\`

## Development Guidelines

### Code Style

- **Rust**: Follow \`rustfmt\` defaults, run \`cargo fmt\` before committing
- **TypeScript**: Follow ESLint config, run \`npm run lint\` before committing
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)

### Commit Message Format

\`\`\`
type(scope): description

[optional body]

[optional footer]
\`\`\`

Types: \`feat\`, \`fix\`, \`docs\`, \`style\`, \`refactor\`, \`test\`, \`chore\`

Examples:
- \`feat(proxmox): add node selection dropdown\`
- \`fix(flash): handle USB disconnect during write\`
- \`docs: update installation instructions\`

### Branch Naming

- \`feat/description\` - New features
- \`fix/description\` - Bug fixes
- \`docs/description\` - Documentation
- \`refactor/description\` - Code refactoring

### Pull Request Process

1. Fork the repository
2. Create a feature branch from \`main\`
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
```

---

## Repository Structure Summary

```
hai/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.yml
│   │   ├── feature_request.yml
│   │   ├── device_request.yml
│   │   └── config.yml
│   ├── workflows/
│   │   ├── test.yml
│   │   └── release.yml
│   ├── CODEOWNERS
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── copilot-instructions.md
│   ├── copilot-review.yml
│   └── claude-instructions.md
├── docs/
│   ├── spec/
│   │   ├── README.md
│   │   ├── architecture.md
│   │   ├── ui-design.md
│   │   ├── user-flows.md
│   │   ├── backend.md
│   │   ├── testing.md
│   │   ├── ci-cd.md
│   │   └── contributing.md
│   └── project.md
├── src-tauri/
│   └── ...
├── src/
│   └── ...
├── test/
│   └── ...
├── renovate.json
├── CONTRIBUTING.md
├── README.md
└── LICENSE
```
