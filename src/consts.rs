pub const CHATGPT_DEFAULT: &str = "gpt-5-nano";
pub const CLAUDE_DEFAULT: &str = "claude-3-5-haiku";
pub const GEMINI_DEFAULT: &str = "gemini-2.5-flash";

pub const DEFAULT_SYS_PROMPT: &str = "You are an expert at git operations. Create git a logical list of git commits based on diffs and structure.";

pub const LOGO: &str = r#""#;

pub const PROMPT_STAGE_HUNKS: &str = "Fill hunk_ids with the HUNK_ID values shown in the diffs (format: \"filepath:index\").\
    Each hunk can only appear in ONE commit.\
    Ex.: [\"src/main.rs:0\", \"src/git/repo.rs:1\"]";

pub const PROMPT_STAGE_FILES: &str =
    "Fill out files with valid paths and leave hunk_headers empty";

pub const RULE_GROUP_FILES: &str = "- GROUP related files into LOGICAL commits based on the type of change\n\
- Examples of files that should be grouped together:\n\
  * Multiple files implementing the same feature\n\
  * Files modified for the same bug fix\n\
  * Related configuration and code changes\n\
  * Test files with the code they test\n";

pub const RULE_NO_FILE_SPLITTING: &str =
    "- CRITICAL: Each file should appear in ONLY ONE commit\n";

pub const RULE_SEPARATE_BY_PURPOSE: &str = "- CRITICAL: Create SEPARATE commits when changes serve DIFFERENT purposes\n";

pub const RULE_COMMIT_MESSAGE_HEADER: &str =
    "\n## CommitMessage Field Requirements:\n";

pub const RULE_PREFIX: &str = "  * prefix: Select the appropriate type from the PrefixType enum\n";

pub const RULE_BREAKING: &str =
    "  * breaking: Set to true if breaking change, false otherwise\n";

// Base instructions - will be combined with length/scope rules
// ideally we include examples atp
pub const RULE_HEADER_BASE: &str = "  * header: CRITICAL - This field contains ONLY the description text\n\
    - NEVER include the prefix (like 'feat:', 'fix:') in this field\n\
    - NEVER include the scope (like '(api)', '(parser)') in this field\n\
    - Example WRONG: 'feat: add new parser' or 'feat(api): add endpoint'\n\
    - Example CORRECT: 'add new parser' or 'add endpoint'\n";

pub const RULE_BODY_BASE: &str = "  * body: Provide detailed explanation of what changed and why\n";

pub const RULE_MESSAGE_VERBOSE: &str = "    - Make descriptions ACCURATE and VERBOSE\n\
    - Descriptions must closely align with the actual code changes\n";

pub const RULE_MESSAGE_CONCISE: &str =
    "    - Write clear, concise descriptions\n";

pub const RULE_SCOPE_ALLOW_EMPTY_WITH_EXTENSION: &str = "  * scope: Component name with extension (e.g., 'main.rs', 'parser') or empty string \"\"\n";
pub const RULE_SCOPE_ALLOW_EMPTY_NO_EXTENSION: &str = "  * scope: Component name WITHOUT extension (e.g., 'main', 'parser') or empty string \"\"\n";
pub const RULE_SCOPE_REQUIRED_WITH_EXTENSION: &str = "  * scope: Component name with extension (e.g., 'main.rs', 'parser')\n";
pub const RULE_SCOPE_REQUIRED_NO_EXTENSION: &str = "  * scope: Component name WITHOUT extension (e.g., 'main', 'parser')\n";

pub const HUNK_INSTRUCTIONS: &str = "\n## File/Hunk Instructions:\n\
Fill hunk_ids with the HUNK_ID values shown in the diffs (format: \"filepath:index\").\n\
Each hunk can only appear in ONE commit.\n\
Example: [\"src/main.rs:0\", \"src/git/repo.rs:1\"]\n";

pub const FILE_INSTRUCTIONS: &str = "\n## File Instructions:\n\
Fill out the 'files' array with valid file paths.\n\
Leave 'hunk_ids' as an empty array.\n";

pub const COMMIT_CONVENTION: &str = "
# Conventional Commits 1.0.0

## Summary

The Conventional Commits specification is a lightweight convention on top of
commit messages. It provides an easy set of rules for creating an explicit
commit history; which makes it easier to write automated tools on top of. This
convention dovetails with [SemVer](http://semver.org), by describing the
features, fixes, and breaking changes made in commit messages.

The commit message should be structured as follows:

---

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

---

The commit contains the following structural elements, to communicate intent to
the consumers of your library:

1. **fix:** a commit of the _type_ `fix` patches a bug in your codebase (this
correlates with [`PATCH`](http://semver.org/#summary) in Semantic
Versioning).
1. **feat:** a commit of the _type_ `feat` introduces a new feature to the
codebase (this correlates with [`MINOR`](http://semver.org/#summary) in
Semantic Versioning).
1. **BREAKING CHANGE:** a commit that has a footer `BREAKING CHANGE:`, or
appends a `!` after the type/scope, introduces a breaking API change
(correlating with [`MAJOR`](http://semver.org/#summary) in Semantic
Versioning). A BREAKING CHANGE can be part of commits of any _type_.
1. _types_ other than `fix:` and `feat:` are allowed, for example
[@commitlint/config-conventional](https://github.com/conventional-changelog/commitlint/tree/master/%40commitlint/config-conventional)
(based on the
[Angular convention](https://github.com/angular/angular/blob/22b96b9/CONTRIBUTING.md#-commit-message-guidelines))
recommends `build:`, `chore:`, `ci:`, `docs:`, `style:`, `refactor:`,
`perf:`, `test:`, and others.
1. _footers_ other than `BREAKING CHANGE: <description>` may be provided and
follow a convention similar to
[git trailer format](https://git-scm.com/docs/git-interpret-trailers).

Additional types are not mandated by the Conventional Commits specification, and
have no implicit effect in Semantic Versioning (unless they include a BREAKING
CHANGE).
<br /><br /> A scope may be provided to a commit's type, to provide additional
contextual information and is contained within parenthesis, e.g.,
`feat(parser): add ability to parse arrays`.

## Examples

### Commit message with description and breaking change footer

```
feat: allow provided config object to extend other configs

BREAKING CHANGE: `extends` key in config file is now used for extending other config files
```

### Commit message with `!` to draw attention to breaking change

```
feat!: send an email to the customer when a product is shipped
```

### Commit message with scope and `!` to draw attention to breaking change

```
feat(api)!: send an email to the customer when a product is shipped
```

### Commit message with both `!` and BREAKING CHANGE footer

```
chore!: drop support for Node 6

BREAKING CHANGE: use JavaScript features not available in Node 6.
```

### Commit message with no body

```
docs: correct spelling of CHANGELOG
```

### Commit message with scope

```
feat(lang): add Polish language
```

### Commit message with multi-paragraph body and multiple footers

```
fix: prevent racing of requests

Introduce a request id and a reference to latest request. Dismiss
incoming responses other than from latest request.

Remove timeouts which were used to mitigate the racing issue but are
obsolete now.

Reviewed-by: Z
Refs: #123
```
## Specification
The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”,
“SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

1. Commits MUST be prefixed with a type, which consists of a noun, `feat`,
`fix`, etc., followed by the OPTIONAL scope, OPTIONAL `!`, and REQUIRED
terminal colon and space.
1. The type `feat` MUST be used when a commit adds a new feature to your
application or library.
1. The type `fix` MUST be used when a commit represents a bug fix for your
application.
1. A scope MAY be provided after a type. A scope MUST consist of a noun
describing a section of the codebase surrounded by parenthesis, e.g.,
`fix(parser):`
1. A description MUST immediately follow the colon and space after the
type/scope prefix. The description is a short summary of the code changes,
e.g., _fix: array parsing issue when multiple spaces were contained in
string_.
1. A longer commit body MAY be provided after the short description, providing
additional contextual information about the code changes. The body MUST begin
one blank line after the description.
1. A commit body is free-form and MAY consist of any number of newline separated
paragraphs.
1. One or more footers MAY be provided one blank line after the body. Each
footer MUST consist of a word token, followed by either a `:<space>` or
`<space>#` separator, followed by a string value (this is inspired by the
[git trailer convention](https://git-scm.com/docs/git-interpret-trailers)).
1. A footer's token MUST use `-` in place of whitespace characters, e.g.,
`Acked-by` (this helps differentiate the footer section from a
multi-paragraph body). An exception is made for `BREAKING CHANGE`, which MAY
also be used as a token.
1. A footer's value MAY contain spaces and newlines, and parsing MUST terminate
when the next valid footer token/separator pair is observed.
1. Breaking changes MUST be indicated in the type/scope prefix of a commit, or
as an entry in the footer.
1. If included as a footer, a breaking change MUST consist of the uppercase text
BREAKING CHANGE, followed by a colon, space, and description, e.g., _BREAKING
CHANGE: environment variables now take precedence over config files_.
1. If included in the type/scope prefix, breaking changes MUST be indicated by a
`!` immediately before the `:`. If `!` is used, `BREAKING CHANGE:` MAY be
omitted from the footer section, and the commit description SHALL be used to
describe the breaking change.
1. Types other than `feat` and `fix` MAY be used in your commit messages, e.g.,
_docs: update ref docs._
1. The units of information that make up Conventional Commits MUST NOT be
treated as case sensitive by implementors, with the exception of BREAKING
CHANGE which MUST be uppercase.
1. BREAKING-CHANGE MUST be synonymous with BREAKING CHANGE, when used as a token
in a footer.
";
