use std::{fs, path::Path};

use git2::Repository;

pub fn get_head_repo(repo: &Repository) -> anyhow::Result<git2::Oid> {
    let head = repo
        .head()?
        .target();

    head.ok_or(super::errors::GitError::NoHead.into())
}

pub const fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

pub fn new_file_content(path: &Path) -> Option<Vec<u8>> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta
            .file_type()
            .is_symlink()
        {
            if let Ok(path) = fs::read_link(path) {
                return Some(
                    path.to_str()?
                        .to_string()
                        .as_bytes()
                        .into(),
                );
            }
        } else if !meta
            .file_type()
            .is_dir()
            && let Ok(content) = fs::read(path)
        {
            return Some(content);
        }
    }

    None
}

pub(super) fn bytes2string(bytes: &[u8]) -> anyhow::Result<String> {
    Ok(String::from_utf8(bytes.to_vec())?)
}
