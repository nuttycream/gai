use crate::settings::Settings;

use super::{CommitSettings, ContextSettings, PromptRules};

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider:
                crate::providers::provider::ProviderKind::Gemini,
            providers: Default::default(),
            staging_type: Default::default(),
            status_type: Default::default(),
            prompt: Default::default(),
            rules: Default::default(),
            context: Default::default(),
            commit: Default::default(),
        }
    }
}

impl Default for PromptRules {
    fn default() -> Self {
        Self {
            group_related_files: true,
            separate_by_purpose: true,
            verbose_descriptions: true,
            extension_in_scope: false,
            allow_empty_scope: true,
            max_header_length: 52,
            allow_body: false,
            max_body_length: 72,
        }
    }
}
impl Default for ContextSettings {
    fn default() -> Self {
        Self {
            include_convention: false,
            include_file_tree: false,
            include_git_status: true,
            include_untracked: true,
            ignore_files: None,
            truncate_files: None,
            include_log: false,
            log_amount: 10,
        }
    }
}

impl Default for CommitSettings {
    fn default() -> Self {
        Self {
            only_staged: false,
            capitalize_prefix: false,
            include_scope: true,
            include_breaking: true,
            breaking_symbol: '!',
        }
    }
}
