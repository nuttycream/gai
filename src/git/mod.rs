pub mod branch;
pub mod checkout;
pub mod commit;
pub mod diffs;
pub mod errors;
pub mod lines;
pub mod log;
pub mod patches;
pub mod rebase;
pub mod rebase_plan;
pub mod repo;
pub mod reset;
pub mod staging;
pub mod status;
pub mod utils;

pub use diffs::{DiffStrategy, Diffs};
pub use repo::GitRepo;
pub use staging::StagingStrategy;
pub use status::StatusStrategy;

#[cfg(test)]
mod tests {
    use git2::Repository;
    use tempfile::TempDir;

    pub fn repo_init() -> (TempDir, Repository) {
        let td = TempDir::new().unwrap();
        let repo = Repository::init(td.path()).unwrap();
        {
            let mut config = repo
                .config()
                .unwrap();

            config
                .set_str("user.name", "name")
                .unwrap();
            config
                .set_str("user.email", "email")
                .unwrap();

            let mut index = repo
                .index()
                .unwrap();

            let id = index
                .write_tree()
                .unwrap();

            let tree = repo
                .find_tree(id)
                .unwrap();

            let sig = repo
                .signature()
                .unwrap();

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "initial",
                &tree,
                &[],
            )
            .unwrap();
        }

        (td, repo)
    }

    /// modified from asyncgit
    pub fn write_commit_file(
        repo: &Repository,
        filename: &str,
        content: &str,
        message: &str,
    ) -> git2::Oid {
        let path = repo
            .workdir()
            .unwrap()
            .join(filename);

        std::fs::write(&path, content).unwrap();

        let mut index = repo
            .index()
            .unwrap();

        index
            .add_path(std::path::Path::new(filename))
            .unwrap();

        index
            .write()
            .unwrap();

        let tree_oid = index
            .write_tree()
            .unwrap();

        let tree = repo
            .find_tree(tree_oid)
            .unwrap();

        let sig = repo
            .signature()
            .unwrap();

        let parent = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent],
        )
        .unwrap()
    }
}
