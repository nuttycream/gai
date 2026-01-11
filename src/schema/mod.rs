pub mod builder;
pub mod commit;
pub mod find;
pub mod rebase;

pub use builder::{SchemaBuilder, SchemaSettings};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn basic() {
        let schema = SchemaBuilder::new().build();

        let expected = json!({
            "type": "object",
            "properties": {}
        });

        assert_eq!(schema, expected);
    }

    #[test]
    fn enum_arr() {
        let arr = vec!["foo".to_owned(), "bar".to_owned()];
        let schema = SchemaBuilder::new()
            .insert_enum_array("test", None, true, &arr)
            .build();

        let expected = json!({
            "type": "object",
            "properties": {
                "test": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["foo", "bar"]
                    }
                }
            },
            "required": ["test"]
        });

        assert_eq!(schema, expected);
    }

    #[test]
    fn nested_schema() {
        let address = SchemaBuilder::new()
            .insert_str("street", None, true)
            .insert_str("city", None, true)
            .build_inner();

        let schema = SchemaBuilder::new()
            .insert_str("name", None, true)
            .insert_object(
                "address",
                Some("home address"),
                true,
                address,
            )
            .build();

        assert_eq!(schema["properties"]["address"]["type"], "object");
        assert!(
            schema["properties"]["address"]["properties"]["street"]
                .is_object()
        );
    }

    #[test]
    fn commit_schema() {
        let prefix_types = vec!["feat".to_owned(), "fix".to_owned()];
        let schema_settings =
            SchemaSettings::default().additional_properties(false);

        let schema = SchemaBuilder::new()
            .settings(schema_settings)
            .insert_str("reasoning", None, true)
            .insert_str_array("files", None, true)
            .insert_str_array("hunk_ids", None, true)
            .insert_enum(
                "prefix",
                Some("prefix_type"),
                true,
                &prefix_types,
            )
            .insert_str("scope", None, true)
            .insert_bool("breaking", None, true)
            .insert_str("header", None, true)
            .insert_str("body", None, true)
            .build();

        let expected = json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "reasoning": {
                    "type": "string"
                },
                "files": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
                "hunk_ids": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
                "prefix": {
                    "description": "prefix_type",
                    "type": "string",
                    "enum": ["feat", "fix"]
                },
                "scope": {
                    "type": "string"
                },
                "breaking": {
                    "type": "boolean"
                },
                "header": {
                    "type": "string"
                },
                "body": {
                    "type": "string"
                }
            },
            "required": [
                "reasoning", "files", "hunk_ids", "prefix",
                "scope", "breaking", "header", "body"
            ]
        });

        assert_eq!(schema, expected);
    }
}

/* "schema": {
    "additionalProperties": false,
    "description": "response object that a provider will respond with",
    "properties": {
      "commits": {
        "description": "list of commits to create staged changes",
        "items": {
          "additionalProperties": false,
          "properties": {
            "body": {
              "description": "extended description",
              "type": "string"
            },
            "breaking": {
              "description": "is a breaking change?",
              "type": "boolean"
            },
            "files": {
              "description": "paths to apply commit to\nex. main.rs doubloon.rs",
              "items": {
                "type": "string"
              },
              "type": "array"
            },
            "header": {
              "description": "short commit description\nused as a initial view",
              "type": "string"
            },
            "hunk_ids": {
              "description": "hunk \"ids\" per file\nusing format file:index\nex: src/main.rs:0",
              "items": {
                "type": "string"
              },
              "type": "array"
            },
            "prefix": {
              "description": "commit type",
              "enum": [
                "feat",
                "fix",
                "refactor",
                "style",
                "test",
                "docs",
                "build",
                "ci",
                "ops",
                "chore"
              ],
              "type": "string"
            },
            "reasoning": {
              "description": "reason why you decided to make this\ncommit. ex. why are they grouped together?\nor why decide on this type of change for the\ndiffs",
              "type": "string"
            },
            "scope": {
              "description": "scope of the change",
              "type": "string"
            }
          },
          "required": [
            "reasoning",
            "files",
            "hunk_ids",
            "prefix",
            "scope",
            "breaking",
            "header",
            "body"
          ],
          "type": "object"
        },
        "type": "array"
      }
    },
    "required": [
      "commits"
    ],
    "title": "ResponseSchema",
    "type": "object"
  },
  "strict": true,
  "type": "json_schema"
} */
