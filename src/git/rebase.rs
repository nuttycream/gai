// functions here are not so much related
// to git rebase as or relevant rebase
// mechanics as gai rebase will not
// operate similar to traditional git rebase
// in that it won't transplant commits
// to another branch, unless
// specifically specified.
// I want to avoid having any sort of
// conflict that will popup in those scenarios
// while we can check if conflictgs
// exist in the first place
//
// if that were the case, then using an --onto
// flag, and checking if conflicts exist.
// if they exist, then we bail early
// rather than leaving the
// repo in a half-rebased state
//
// gai rebase is more to "recreate" commits
// in-place, but restructed, somewhat similar
// to a git rebase -i, but less about doing
// operations (might be an option) and more
// to do with generating commits from the diff
// of the specified divergent point

use git2::{Oid, Repository};

use crate::git::errors::GitError;

use super::{
    StagingStrategy,
    commit::{GitCommit, apply_commits},
    diffs::FileDiff,
};

/// recreate commits from diverged_from commit
/// to head. Pass in an optional commit to rebase from
/// essentially recreates/"rebases" commits
/// from commit -> to commit, erasing all commits
/// in between, for the new commits
pub fn rebase_commits(
    repo: &Repository,
    diverged_from: Oid,
    commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_strategy: &StagingStrategy,
) -> anyhow::Result<()> {
    let commit_diverged = repo.find_commit(diverged_from)?;

    // reset to the diverged commit
    // mixed, to keep changes "unstaged"
    repo.reset(
        commit_diverged.as_object(),
        git2::ResetType::Mixed,
        None,
    )?;

    // call apply commits
    apply_commits(repo, commits, og_file_diffs, staging_strategy)?;

    Ok(())
}

/// cherry pick commits, this would take in a list
/// of commits OID that should've been captured
/// before sending out diffs and soft resetting
/// this would fail IF the repo from the point of applying
/// the new commits has conflicts from the first cherry picked
/// commit if this happens, oh lord,
/// validation (check if the two trees match) would happen elsewhere
/// ideally before this
pub fn cherry_pick_commits(
    repo: &Repository,
    commits: &[Oid],
) -> anyhow::Result<()> {
    for &oid in commits {
        let commit = repo.find_commit(oid)?;

        let head = repo
            .head()?
            .peel_to_commit()?;

        let mut index =
            repo.cherrypick_commit(&commit, &head, 0, None)?;

        // should validate elsewhere
        // exit early regardless
        if index.has_conflicts() {
            return Err(GitError::Generic(
                "Cannot cherry pick, repo has conflicts".to_string(),
            )
            .into());
        }

        let tree_oid = index.write_tree_to(repo)?;
        let tree = repo.find_tree(tree_oid)?;

        let sig = repo.signature()?;

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            //
            commit
                .message()
                .unwrap_or(""),
            &tree,
            &[&head],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use tempfile::TempDir;

    fn repo_init() -> (TempDir, Repository) {
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
    fn write_commit_file(
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

    #[test]
    fn test_cherry_pick_single_commit() {
        let (_dir, repo) = repo_init();

        let head_commit = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        println!(
            "initial HEAD:{} {}",
            head_commit.id(),
            head_commit
                .message()
                .unwrap()
        );

        let pick_oid = write_commit_file(
            &repo,
            "test.txt",
            "test input text",
            "add test",
        );

        println!("pick: {}", pick_oid);

        repo.reset(
            head_commit.as_object(),
            git2::ResetType::Hard,
            None,
        )
        .unwrap();

        println!(
            "HEAD after resetting:{} {}",
            //fuggit
            repo.head()
                .unwrap()
                .peel_to_commit()
                .unwrap()
                .id(),
            repo.head()
                .unwrap()
                .peel_to_commit()
                .unwrap()
                .message()
                .unwrap()
                .trim()
        );

        cherry_pick_commits(&repo, &[pick_oid]).unwrap();

        // verify HEAD
        let new_head = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        println!(
            "HEAD after cherrypick: {} {}",
            new_head.id(),
            new_head
                .message()
                .unwrap()
                .trim()
        );

        println!(
            "parent of HEAD:{}",
            new_head
                .parent_id(0)
                .unwrap()
        );

        assert_ne!(new_head.id(), head_commit.id());

        assert_eq!(
            new_head
                .message()
                .unwrap(),
            "add test"
        );

        // very file in new tree
        let tree = new_head
            .tree()
            .unwrap();

        assert!(
            tree.get_name("test.txt")
                .is_some()
        );
    }

    #[test]
    fn test_cherry_pick_multiple_commits() {
        let (_dir, repo) = repo_init();

        let head_commit = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        println!(
            "initial HEAD:{} {}",
            head_commit.id(),
            head_commit
                .message()
                .unwrap()
        );

        let c1 = write_commit_file(&repo, "a.txt", "aaa", "add a");
        let c2 = write_commit_file(&repo, "b.txt", "bbb", "add b");

        // HARD reset to initial commit, before both c1 and c2
        let before = repo
            .find_commit(
                repo.head()
                    .unwrap()
                    .peel_to_commit()
                    .unwrap()
                    .parent_id(0)
                    .unwrap(),
            )
            .unwrap();

        println!(
            "HEAD before: {} {}",
            before.id(),
            before
                .message()
                .unwrap()
                .trim()
        );

        let initial = repo
            .revparse_single("HEAD~2")
            .unwrap();

        println!("resetting to: {}", initial.id());

        repo.reset(&initial, git2::ResetType::Hard, None)
            .unwrap();

        let after = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        println!(
            "HEAD after reset:  {} \"{}\"",
            after.id(),
            after
                .message()
                .unwrap()
                .trim()
        );

        println!("log before cherrypick");

        let mut walk = repo
            .revwalk()
            .unwrap();
        walk.push_head()
            .unwrap();

        for oid in walk {
            let oid = oid.unwrap();
            let c = repo
                .find_commit(oid)
                .unwrap();

            println!(
                "  {} {}",
                oid,
                c.message()
                    .unwrap()
                    .trim()
            );
        }

        // cherry pick c1 and c2
        cherry_pick_commits(&repo, &[c1, c2]).unwrap();

        println!("log after cherrypick");

        let mut walk = repo
            .revwalk()
            .unwrap();
        walk.push_head()
            .unwrap();

        for oid in walk {
            let oid = oid.unwrap();
            let c = repo
                .find_commit(oid)
                .unwrap();

            println!(
                "  {} {}",
                oid,
                c.message()
                    .unwrap()
                    .trim()
            );
        }

        let final_h = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap();

        assert_eq!(
            final_h
                .message()
                .unwrap(),
            "add b"
        );

        let tree = final_h
            .tree()
            .unwrap();

        assert!(
            tree.get_name("a.txt")
                .is_some()
        );

        assert!(
            tree.get_name("b.txt")
                .is_some()
        );
    }
}
