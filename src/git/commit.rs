use crate::ai::response::ResponseCommit;

#[derive(Debug)]
pub struct GaiCommit {
    pub files: Vec<String>,
    pub hunk_headers: Vec<String>,
    pub message: String,
}

impl GaiCommit {
    pub fn from_response(
        response: &ResponseCommit,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> Self {
        let message = {
            let prefix = if capitalize_prefix {
                format!("{:?}", response.message.prefix)
            } else {
                format!("{:?}", response.message.prefix)
                    .to_lowercase()
            };

            let breaking =
                if response.message.breaking { "!" } else { "" };
            let scope = if include_scope
                && !response.message.scope.is_empty()
            {
                // gonna set it to lowercase PERMA
                // sometimes the AI responds with a scope
                // that includes the file extension and is capitalized
                // like (Respfileonse.rs) which looks ridiculous imo
                // the only way i can think of is to make it a rule to not include
                // extension names
                format!("({})", response.message.scope.to_lowercase())
            } else {
                "".to_owned()
            };

            format!(
                "{}{}{}: {}",
                prefix, breaking, scope, response.message.description
            )
        };
        GaiCommit {
            files: response.files.to_owned(),
            hunk_headers: response.hunk_headers.to_owned(),
            message,
        }
    }
}
