---
title: "gai: Configure"
template: config
styles: ["main", "md"]
---

## AI Options {#ai-config}

Configure AI provider settings, prompts, and behavior rules.

### Provider Settings

**`provider`** - Select your AI provider

- Options: `"Gemini"`, `"OpenAI"`, `"Claude"`
- Default: `"Gemini"`
- Override with CLI: `--gemini`, `--chatgpt`, `--claude`

**`providers`** - Provider-specific configuration

```toml
[ai.providers.Gemini]
model = "gemini-1.5-flash"
max_tokens = 5000

[ai.providers.OpenAI]
model = "gpt-4"
max_tokens = 5000

[ai.providers.Claude]
model = "claude-3-5-sonnet-20241022"
max_tokens = 5000
```

### Prompt Options {#prompt-config}

**`system_prompt`** - Custom system prompt override

- Type: Optional string
- Default: `None` (uses built-in prompt)
- CLI: `-p, --system-prompt <prompt>`
- Example: `"You are an expert at writing concise commit messages"`

**`commit_convention`** - Custom commit convention override

- Type: Optional string
- Default: `None` (uses Conventional Commits v1)
- Allows you to define your own commit message format

**`include_convention`** - Include commit convention in prompt

- Type: Boolean
- Default: `true`
- CLI: `-C, --include-convention`
- Note: Includes full convention spec (uses a lot more tokens)

### Context Options {#context-config}

**`include_file_tree`** - Include repository file tree `.gitignore` will be
respected

- Type: Boolean
- Default: `true`
- CLI: `-t, --include-file-tree`

**`include_git_status`** - Include git status output

- Type: Boolean
- Default: `true`
- Provides staging and file state information

**`include_untracked`** - Include untracked files

- Type: Boolean
- Default: `true`
- CLI: `-u, --include-untracked`
- Whether to analyze untracked files

**`files_to_truncate`** - Files to truncate before sending

- Type: Array of strings
- Default: `[]`
- CLI: `-T, --truncate-file <file>`
- Example: `["Cargo.lock", "package-lock.json"]`
- Saves tokens by truncating large, auto-generated files

### AI Response Rules {#response-rules}

**`group_related_files`** - Group related files by type

- Type: Boolean
- Default: `true`
- CLI: `-g, --group-related-files`
- Groups files with similar changes into logical commits

**`no_file_splitting`** - Keep files in single commits

- Type: Boolean
- Default: `true`
- CLI: `-S, --no-file-splitting`
- Prevents splitting a single file across multiple commits

**`separate_by_purpose`** - Separate commits by purpose

- Type: Boolean
- Default: `true`
- CLI: `-P, --separate-by-purpose`
- Creates separate commits for unrelated changes

**`verbose_descriptions`** - Use verbose commit descriptions

- Type: Boolean
- Default: `true`
- CLI: `-v, --verbose-descriptions`
- Generates detailed commit message bodies

**`exclude_extension_in_scope`** - Exclude file extensions in scope

- Type: Boolean
- Default: `true`
- CLI: `-e, --exclude-extension-in-scope`
- Example: `feat(git)` vs `feat(git.rs)`

**`allow_empty_scope`** - Allow empty scope field

- Type: Boolean
- Default: `true`
- CLI: `-E, --allow-empty-scope`
- Permits commits without a scope: `feat: message`

**`max_header_length`** - Maximum commit header length

- Type: Number (u16)
- Default: `52`
- CLI: `-m, --max-header-length <u16>`
- Enforces header length limit (conventional: 50-72)

**`max_body_length`** - Maximum body line length

- Type: Number (u16)
- Default: `72`
- CLI: `-M, --max-body-length <u16>`
- Enforces body line wrapping

## Gai Options {#gai-config}

Git-specific settings for `gai`.

### Staging Behavior {#staging}

**`stage_hunks`** - Apply changes as hunks

- Type: Boolean
- Default: `false`
- CLI: `-H, --stage-hunks`
- Stages individual hunks instead of entire files

### Commit Message Format {#commit-format}

**`capitalize_prefix`** - Capitalize commit type prefix

- Type: Boolean
- Default: `true`
- CLI: `-c, --capitalize-prefix`
- Example: `Feat:` vs `feat:`

**`include_scope`** - Include scope in commits

- Type: Boolean
- Default: `true`
- CLI: `-s, --include-scope`
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
- CLI: `tui --auto-request`
- Automatically sends AI request when TUI opens

## Full Example Options {#example}

```toml
[ai]
provider = "Gemini"
include_convention = false
include_file_tree = false
include_git_status = true
include_untracked = true
files_to_truncate = ["Cargo.lock", "package-lock.json"]

[ai.providers.Gemini]
model = "gemini-2.5-flash-lite"
max_tokens = 5000

[ai.providers.Claude]
model = "claude-3-5-haiku"
max_tokens = 5000

[ai.providers.OpenAI]
model = "gpt-5-nano"
max_tokens = 10000

[ai.rules]
group_related_files = true
no_file_splitting = true
separate_by_purpose = true
verbose_descriptions = true
exclude_extension_in_scope = true
allow_empty_scope = true
max_header_length = 52
max_body_length = 72

[gai]
stage_hunks = false

[gai.commit_config]
capitalize_prefix = false
include_scope = true
include_breaking = true

[tui]
auto_request = false
```

## CLI Flags Override {#cli}

All configuration options can be overridden via command-line flags. Use
`gai --help` for a complete list of available flags.

- `-u, --include-untracked` - Include untracked files
- `-H, --stage-hunks` - Apply as hunks
- `-k, --api-key-file <file>` - Path to API key file
- `-t, --include-file-tree` - Include file tree
- `-T, --truncate-file <file>` - Files to truncate
- `-c, --capitalize-prefix` - Capitalize prefix
- `-s, --include-scope` - Include scope
- `-C, --include-convention` - Include convention
- `-g, --group-related-files` - Group related files
- `-S, --no-file-splitting` - No file splitting
- `-P, --separate-by-purpose` - Separate by purpose
- `-v, --verbose-descriptions` - Verbose descriptions
- `-e, --exclude-extension-in-scope` - Exclude extensions
- `-E, --allow-empty-scope` - Allow empty scope
- `-m, --max-header-length <u16>` - Max header length
- `-M, --max-body-length <u16>` - Max body length
- `--chatgpt` - Force ChatGPT
- `--gemini` - Force Gemini
- `--claude` - Force Claude

## Environment Variables {#env-vars}

Configure API keys using environment variables or a `.env` file:

```bash
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
GEMINI_API_KEY=your_gemini_key
```
