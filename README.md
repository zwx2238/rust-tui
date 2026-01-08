# Rust TUI

A terminal user interface (TUI) application built with Rust, providing an interactive command-line experience.

## Features

- Terminal-based interface using `ratatui` and `crossterm`
- Modern Rust implementation with async support
- Markdown rendering with syntax highlighting
- Code execution capabilities
- LLM integration support

## Installation

### Using Cargo

```bash
cargo install --path .
```

### Using Just

```bash
just install
```

This will install the binary and set up the global wrapper script.

## Usage

Run the application:

```bash
deepchat
```

## Development

### Prerequisites

- Rust (latest stable)
- `just` command runner (optional, for convenience commands)

### Development Workflow

1. Before committing code, always run:
   ```bash
   just install
   ```

2. Code quality checks:
   ```bash
   cargo clippy --all-targets --all-features
   ```

3. Generate documentation:
   ```bash
   just doc
   ```

## Project Structure

- `src/` - Main source code
- `scripts/` - Utility scripts
- `tests/` - Test files
- `workspace/` - Workspace files

## License

See `THIRD_PARTY_NOTICES.md` for third-party licenses.
