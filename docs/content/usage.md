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

### Authentication with Gai Provider

To use the **Gai** provider (uses Gemini Flash 2.5 or 2.5-lite), first
authenticate using GitHub OAuth:

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

Our Gai provider offers free requests that reset every 24 hours.

### Basic Commands

```bash
# Generate commits interactively
gai commit

# Skip confirmation and apply immediately
gai commit -y

# Launch the Terminal User Interface
gai -i commit
```

### Working with Staged Changes

```bash
# Only generate commits for staged changes
gai commit -s

# Stage changes as individual hunks
gai commit -H

# Stage changes as complete files
gai commit -f

# Combine staged-only with hunks
gai commit -s -H
```

### Using AI Providers

```bash
# Use a specific provider
gai -p gemini commit
gai -p claude commit
gai -p openai commit

# Use the Gai provider (requires authentication)
gai -p gai commit
```

### Adding Context with Hints

```bash
# Provide additional context to guide the AI
gai -H "These are a list of chore changes" commit
```

### Overriding Configuration

```bash
# Temporarily override config options
gai commit -c ai.rules.verbose_descriptions=false

# Override multiple options
gai commit -c ai.rules.max_header_length=80 -c ai.rules.allow_body=false

# Change model for this commit only
gai commit -c ai.providers.Gemini.model=gemini-2.5-flash-lite

# Override commit format settings
gai commit -c gai.commit_config.capitalize_prefix=true
```

### Repository Status

```bash
# Show current repository status
gai status

# Show verbose status (includes prompt and diffs that will be sent to AI)
gai status -v
```

### Advanced Examples

```bash
# Compact output without confirmation 
gai -c commit -y

# Override multiple settings for a quick commit
gai -H "Quick fixes" commit \
  -c ai.rules.allow_body=false \
  -c ai.rules.max_header_length=50 \
  -y

# Generate commits from hunks with verbose descriptions
gai commit -H -c ai.rules.verbose_descriptions=true

# Use OpenAI with specific model and max header length
gai commit -p openai \
  -c ai.providers.OpenAI.model=gpt-5 \
  -c ai.rules.max_header_length=67
```

### Getting Help

View all available commands and options:

```bash
gai --help
```

View help for specific commands:

```bash
gai commit --help
gai auth --help
gai status --help
```

### Tips and Best Practices

**Use hints effectively**: Provide context that helps the AI understand the
purpose of your changes.

```bash
gai -H "Refactoring for better testability" commit
```

**Stage incrementally**: Use `-s` to commit already-staged changes separately
from unstaged work.

```bash
git add file1.rs file2.rs
gai commit -s
```

### Notes:

- Always review generated commits before applying them (unless using `-y`).
- Most options are better set in `config.toml` for consistency. Use `-c`
  overrides for one-off situations.
- Before committing, use `gai status -v` to see exactly what will be sent to the
  AI.
