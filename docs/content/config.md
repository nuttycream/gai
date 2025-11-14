---
title: "gai: Configure"
template: config
styles: ["main", "md"]
---

## AI Options {#ai-config}

Configure AI provider settings, prompts, and behavior rules.

### Provider Settings

**`provider`** - Select your AI provider

- Options: `"Gemini"`, `"OpenAI"`, `"Claude"`, `"Gai"`
- Default: `"Gai"`
- Override with CLI: `-p, --provider <PROVIDER>`

**`providers`** - Provider-specific configuration

```toml
[ai.providers.Gemini]
model = "gemini-2.5-flash"
max_tokens = 5000

[ai.providers.OpenAI]
model = "gpt-5"
max_tokens = 5000

[ai.providers.Claude]
model = "claude-3-5-haiku"
max_tokens = 5000
```

### Prompt Options {#prompt-config}

**`system_prompt`** - Custom system prompt override

- Type: Optional string
- Default: `None` (uses built-in prompt)
- Example: `"You are an expert at writing concise commit messages"`

**`commit_convention`** - Custom commit convention override

- Type: Optional string
- Default: `None` (uses Conventional Commits v1)
- Allows you to define your own commit message format

**`include_convention`** - Include commit convention in prompt

- Type: Boolean
- Default: `true`
- Note: Includes full convention spec (uses more tokens)

**`hint`** - Additional hinting for LLMs

- Type: Optional string
- Default: `None`
- CLI: `-H, --hint <TEXT>`
- Example: `"Focus on security-related changes"`
- Provides additional context or instructions to guide the AI

### Context Options {#context-config}

**`include_file_tree`** - Include repository file tree (`.gitignore` respected)

- Type: Boolean
- Default: `true`

**`include_git_status`** - Include git status output

- Type: Boolean
- Default: `true`
- Provides staging and file state information

**`include_untracked`** - Include untracked files

- Type: Boolean
- Default: `true`
- Whether to analyze untracked files

**`files_to_truncate`** - Files to truncate before sending

- Type: Array of strings
- Default: `[]`
- Example: `["Cargo.lock", "package-lock.json"]`
- Saves tokens by truncating large, auto-generated files

### AI Response Rules {#response-rules}

**`group_related_files`** - Group related files by type

- Type: Boolean
- Default: `true`
- Groups files with similar changes into logical commits

**`no_file_splitting`** - Keep files in single commits

- Type: Boolean
- Default: `true`
- Prevents splitting a single file across multiple commits

**`separate_by_purpose`** - Separate commits by purpose

- Type: Boolean
- Default: `true`
- Creates separate commits for unrelated changes

**`verbose_descriptions`** - Use verbose commit descriptions

- Type: Boolean
- Default: `true`
- Generates detailed commit message bodies

**`exclude_extension_in_scope`** - Exclude file extensions in scope

- Type: Boolean
- Default: `true`
- Example: `feat(git)` vs `feat(git.rs)`

**`allow_empty_scope`** - Allow empty scope field

- Type: Boolean
- Default: `true`
- Permits commits without a scope: `feat: message`

**`allow_body`** - Allow commit message bodies

- Type: Boolean
- Default: `true`
- Enables generation of detailed commit message bodies

**`max_header_length`** - Maximum commit header length

- Type: Number (u16)
- Default: `52`
- Enforces header length limit (conventional: 50-72)

**`max_body_length`** - Maximum body line length

- Type: Number (u16)
- Default: `72`
- Enforces body line wrapping

## Gai Options {#gai-config}

Git-specific settings for `gai`.

### Staging Behavior {#staging}

**`only_staged`** - Only generate commits for staged changes

- Type: Boolean
- Default: `false`
- CLI: `gai commit -s, --staged`
- Only analyzes currently staged files/hunks

**`stage_hunks`** - Apply changes as hunks

- Type: Boolean
- Default: `false`
- CLI: `gai commit -H, --hunks`
- Stages individual hunks instead of entire files
- Use `-f, --files` to override back to file staging

### Commit Message Format {#commit-format}

**`capitalize_prefix`** - Capitalize commit type prefix

- Type: Boolean
- Default: `false`
- Example: `Feat:` vs `feat:`

**`include_scope`** - Include scope in commits

- Type: Boolean
- Default: `true`
- Example: `feat(api):` vs `feat:`

**`include_breaking`** - Mark breaking changes

- Type: Boolean
- Default: `true`
- Adds breaking change indicator

**`breaking_symbol`** - Custom breaking change symbol

- Type: Optional character
- Default: `None` (uses `!`)
- Example: `feat!: breaking change`

## TUI Options {#tui-config}

**`auto_request`** - Send request on launch

- Type: Boolean
- Default: `false`
- CLI: `gai tui --auto-request`
- Automatically sends AI request when TUI opens

## Full Example Configuration {#example}

```toml
[ai]
provider = "Gai"
include_convention = false
include_file_tree = true
include_git_status = true
include_untracked = true
files_to_truncate = []

[ai.providers.OpenAI]
model = "gpt-5-nano"
max_tokens = 5000

[ai.providers.Gemini]
model = "gemini-2.5-flash-lite"
max_tokens = 5000

[ai.providers.Claude]
model = "claude-3-5-haiku"
max_tokens = 5000

[ai.providers.Gai]
model = "gemini-2.5-flash"
max_tokens = 5000

[ai.rules]
group_related_files = true
no_file_splitting = true
separate_by_purpose = true
verbose_descriptions = true
exclude_extension_in_scope = true
allow_empty_scope = true
max_header_length = 52
allow_body = false
max_body_length = 72

[gai]
only_staged = false
stage_hunks = false

[gai.commit_config]
capitalize_prefix = false
include_scope = true
include_breaking = true

[tui]
auto_request = false
```

## CLI Usage {#cli}

### Global Flags

These flags work with any command:

- `-c, --compact` - Print with compact outputs (no pretty trees)
- `-i, --interactive` - Launch the TUI interface
- `-p, --provider <PROVIDER>` - Override the configured provider (options:
  `gemini`, `openai`, `claude`, `gai`)
- `-H, --hint <TEXT>` - Provide additional hinting to guide the AI

### Commands

**`gai auth`** - Authenticate with the Gai provider

```bash
gai auth login   # Login via GitHub OAuth
gai auth status  # Check authentication status and request limits
gai auth logout  # Clear stored authentication token
```

**`gai status`** - Display repository status

```bash
gai status          # Show repository status
gai status -v       # Show verbose status with prompt and diffs
```

**`gai commit`** - Generate and apply commits

```bash
gai commit                    # Interactive commit generation
gai commit -y                 # Skip confirmation, apply immediately
gai commit -s                 # Only generate for staged changes
gai commit -H                 # Stage changes as hunks
gai commit -f                 # Stage changes as files (override -H)
gai commit -c KEY=VALUE       # Override config options for this commit
```

## Environment Variables {#env-vars}

Configure API keys using environment variables or a `.env` file:

```bash
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
GEMINI_API_KEY=your_gemini_key
```
