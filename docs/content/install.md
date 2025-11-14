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
# Linux
chmod +x gai
sudo mv gai /usr/local/bin/

# or to a local user directory 
mkdir -p ~/.local/bin
mv gai ~/.local/bin/

# verify installation
gai --version
```

### Building from Source

For contributors or those who want to build from source:

```bash
git clone https://github.com/nuttycream/gai.git
cd gai
cargo build --release
```

The project includes a `flake.nix` and `.envrc` that automatically sets up a
known working development environment using
[direnv](https://github.com/nix-community/nix-direnv).

### Initial Setup

After installation, set up your API keys for your preferred AI provider:

```bash
# For Gemini (default provider in config)
export GEMINI_API_KEY="your_api_key_here"

# For OpenAI
export OPENAI_API_KEY="your_api_key_here"

# For Claude
export ANTHROPIC_API_KEY="your_api_key_here"
```

Add these to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make them
permanent.

Alternatively, use the **Gai** provider which offers free requests:

```bash
gai auth login
```

### What's Next?

- [Understand how to use gai and see common workflows](/usage)
- [View all configurable options](/config)
