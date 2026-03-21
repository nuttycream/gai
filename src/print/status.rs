use crate::git::status::{FileStatus, StatusItemType};

use super::tree::{Tree, TreeItem};

pub fn print(
    branch: &str,
    staged_statuses: &[FileStatus],
    working_dir_statuses: &[FileStatus],
    compact: bool,
) -> anyhow::Result<()> {
    Ok(())
}
