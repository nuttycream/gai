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

### What's Next?

- [Understand how to use gai and see common workflows](/usage)
- [View all configurable options](/config)
