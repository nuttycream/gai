---
title: "gai: Usage"
template: install
styles: ["main", "md"]
---

## Usage

### Quick Start

Navigate to any git repository and run:

```bash
gai commit
```

**gai** will analyze your changes and generate intelligent commit messages based
on your diffs.

### Basic Usage

```bash
# Generate commits
gai commit

# Skip confirmation and apply immediately**:
gai commit -y

# Launch the Terminal User Interface
gai tui

# Launch TUI and send request automatically
gai tui --auto-request
```

### TUI

```bash
# Open TUI to review diffs and manage generated commits interactively
gai tui
```

### Advanced Usage

```bash
# Provide additional context to guide the AI with hinting
# -A or --hint
gai commit -A "This is a fix with performance improvements"

# Use specific provider with custom hint
gai commit --claude -A "Explain breaking changes clearly"

# Disable commit bodies for concise messages (only header)
gai commit -B

# Combine multiple options
gai commit -v -A "Emphasize performance optimizations" --gemini

# Skip confirmation and apply immediately
gai commit -y -A "Bug fixes only"
```

### Configuration

On first run, **gai** creates a default configuration file at:

- **Linux/macOS**: `~/.config/gai/config.toml`

See the [Configuration Guide](/config) for detailed customization options.

### Setting Up API Keys

**gai** requires an API key from your chosen AI provider. Set it using
environment variables:

```bash
# For Gemini (default)
export GEMINI_API_KEY="your_api_key_here"

# For OpenAI
export OPENAI_API_KEY="your_api_key_here"

# For Claude
export ANTHROPIC_API_KEY="your_api_key_here"
```

Add these to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make them
permanent.

### Authentication with Gai Provider

To use the **Gai** provider (powered by gemini-flash-2.5 ), authenticate using
GitHub OAuth:

```bash
# Login via GitHub OAuth
gai auth login
```

This will open your browser to authenticate with GitHub. After authorization,
you'll receive a token to paste back into the terminal.

```bash
# Check authentication status and request limits
gai auth status

# Logout and clear stored token
gai auth logout
```

The Gai provider offers 10 free requests that reset periodically.

### Getting Help

View all available commands and options:

```bash
gai --help
```

View help for specific commands:

```bash
gai commit --help
gai tui --help
```

### Contributing

For contributors or those who want to build from source:

```bash
git clone https://github.com/nuttycream/gai.git
cd gai
cargo build --release
```

The project comes with a `flake.nix` and a `.envrc` that automatically drops you
in a known working nix environment using
[direnv](https://github.com/nix-community/nix-direnv)
