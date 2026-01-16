# Contributing to Porta

Thank you for your interest in contributing to Porta! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust** 1.70 or later
- **Node.js** 20 or later
- **SQLite** 3
- **Git**

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/porta.git
   cd porta
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/porta-app/porta.git
   ```

## Development Setup

### Backend (Rust)

```bash
# Build the backend library
cd backend
cargo build

# Run tests
cargo test

# Run clippy for linting
cargo clippy
```

### Server

```bash
# Build the server
cd server
cargo build

# Run in development mode
cargo run -- --config porta.toml.example

# Run tests
cargo test
```

### Frontend (Vue.js)

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev

# Run tests
npm run test

# Type check
npm run type-check
```

### Desktop App (Tauri)

```bash
# Development mode
npm run tauri:dev

# Build release
npm run build:desktop
```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/add-new-protocol` - New features
- `fix/connection-timeout` - Bug fixes
- `docs/update-readme` - Documentation
- `refactor/simplify-state` - Code refactoring

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(p2p): add QUIC transport support
fix(proxy): handle connection timeout properly
docs(readme): add installation instructions
```

## Pull Request Process

1. **Update your fork**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature
   ```

3. **Make your changes** and commit them

4. **Run tests**:
   ```bash
   npm test
   ```

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature
   ```

6. **Create a Pull Request** on GitHub

### PR Requirements

- [ ] All tests pass
- [ ] Code follows project style guidelines
- [ ] Documentation is updated if needed
- [ ] Commit messages follow conventions
- [ ] PR description explains the changes

## Coding Standards

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting:
  ```bash
  cargo fmt
  ```
- Use `clippy` for linting:
  ```bash
  cargo clippy -- -D warnings
  ```
- Write doc comments for public APIs
- Handle errors properly with `anyhow` or `thiserror`

### TypeScript/Vue

- Use TypeScript for type safety
- Follow Vue 3 Composition API patterns
- Use meaningful variable and function names
- Add JSDoc comments for complex functions

### General

- Keep functions small and focused
- Write self-documenting code
- Add comments for complex logic
- Avoid premature optimization

## Testing

### Backend Tests

```bash
cd backend
cargo test
```

### Server Tests

```bash
cd server
cargo test
```

### Frontend Tests

```bash
cd frontend
npm run test
```

### System Tests

```bash
# Start test server
npm run test:setup

# Run API tests
npm run test:system

# Stop test server
npm run test:teardown
```

### Writing Tests

- Write unit tests for new functions
- Add integration tests for API endpoints
- Test edge cases and error conditions
- Aim for meaningful coverage, not 100%

## Project Structure

```
porta-app/
├── backend/          # Shared Rust library
│   ├── src/
│   │   ├── lib.rs    # Library exports
│   │   ├── state.rs  # Application state
│   │   ├── app.rs    # Business logic
│   │   ├── routes/   # HTTP API routes
│   │   ├── p2p/      # P2P networking
│   │   └── proxy/    # Proxy implementation
│   └── tests/        # Integration tests
├── server/           # Server binary
│   └── src/
│       ├── main.rs   # Entry point
│       └── config.rs # Configuration
├── src-tauri/        # Desktop app
├── frontend/         # Vue.js frontend
│   ├── src/
│   │   ├── pages/    # Page components
│   │   ├── services/ # API client
│   │   └── types.ts  # Type definitions
│   └── tests/        # Frontend tests
└── scripts/          # Build scripts
```

## Need Help?

- Check existing [issues](https://github.com/porta-app/porta/issues)
- Create a new issue for questions
- Join discussions in the repository

Thank you for contributing!
