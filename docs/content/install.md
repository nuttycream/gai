---
title: "gai: Installation"
template: install
styles: ["main", "md"]
---

## Installation

### Using Cargo (Recommended)

If you have Rust installed, you can install **gai** directly from the
repository:

```bash
cargo install --git https://github.com/nuttycream/gai
```

### Pre-built Binaries

Don't have Rust installed? Download a pre-built binary for your platform:

- [GitHub Releases](https://github.com/nuttycream/gai/releases)

After downloading, make the binary executable and move it to your PATH:

```bash
# Linux/macOS
chmod +x gai
sudo mv gai /usr/local/bin/

# or to a local user directory 
mkdir -p ~/.local/bin
mv gai ~/.local/bin/

# verify
gai --version
```

## Usage

### Quick Start

Navigate to any git repository and run:

```bash
gai commit
```

**gai** will analyze your changes and generate intelligent commit messages based
on your diffs.

### Basic Commands

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

### Common Workflows

```bash
# Let gai analyze and create commits
# No need to stage changes at this point
gai commit

# Review suggested commits and choose to apply, edit, or retry
```

**Interactive workflow with TUI**:

```bash
# Open TUI to review diffs and manage generated commits interactively
gai tui
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
