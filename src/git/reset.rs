use git2::{Oid, Repository, ResetType};

/// modified from asyncgit
/// resets to commit HARD (deletes changes)
pub fn reset_repo_hard(
    repo: &Repository,
    commit: &str,
) -> anyhow::Result<()> {
    let commit = Oid::from_str(commit)?;

    reset_repo(repo, commit, ResetType::Hard)
}

/// reset repo to commit, mixed (keep changes)
pub fn reset_repo_mixed(
    repo: &Repository,
    commit: &str,
) -> anyhow::Result<()> {
    let commit = Oid::from_str(commit)?;

    reset_repo(repo, commit, ResetType::Mixed)
}

fn reset_repo(
    repo: &Repository,
    commit: Oid,
    kind: ResetType,
) -> anyhow::Result<()> {
    let c = repo.find_commit(commit)?;

    repo.reset(c.as_object(), kind, None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::repo_init;
    use crate::git::tests::write_commit_file;

    #[test]
    fn test_reset_hard() {
        let (_dir, repo) = repo_init();
        let initial = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string();

        println!("initial:{}", initial);

        let c1 = write_commit_file(&repo, "a.txt", "hell", "add a");
        println!("c1: {}", c1);

        assert!(
            repo.workdir()
                .unwrap()
                .join("a.txt")
                .exists()
        );

        reset_repo_hard(&repo, &initial).unwrap();

        let head = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        println!(
            "head after hard:{} {}",
            head.id(),
            head.message()
                .unwrap()
                .trim()
        );

        assert_eq!(
            head.id()
                .to_string(),
            initial
        );

        assert!(
            !repo
                .workdir()
                .unwrap()
                .join("a.txt")
                .exists()
        );
    }

    #[test]
    fn test_reset_mixed() {
        let (_dir, repo) = repo_init();
        let initial = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string();

        let _c1 = write_commit_file(&repo, "a.txt", "foo", "add a");

        reset_repo_mixed(&repo, &initial).unwrap();

        let head = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        assert_eq!(
            head.id()
                .to_string(),
            initial
        );

        // should see the file in
        // the working dir
        assert!(
            repo.workdir()
                .unwrap()
                .join("a.txt")
                .exists()
        );
    }
}
