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

use git2::{Oid, Repository, Sort};

use super::errors::GitError;

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
    commits: &[String],
) -> anyhow::Result<()> {
    for oid in commits {
        let oid = Oid::from_str(oid)?;

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

    // should force sync curr tree to match new HEAD
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::new().force(),
    ))?;

    Ok(())
}

/// helper func to get a list of trailing commits
/// from a specified oid, this just walks from that commit
/// back
pub fn trailing_commits(
    repo: &Repository,
    from: &str,
) -> anyhow::Result<Vec<String>> {
    let mut trails = Vec::new();

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;

    let from_oid = Oid::from_str(from)?;

    for oid in revwalk {
        let oid = oid?;

        if oid == from_oid {
            break;
        }

        trails.push(oid.to_string());
    }

    trails.reverse();

    Ok(trails)
}

///
pub fn squash_to_head(
    repo: &Repository,
    commit: &str,
    message: &str,
) -> anyhow::Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::repo_init;
    use crate::git::tests::write_commit_file;

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

        cherry_pick_commits(&repo, &[pick_oid.to_string()]).unwrap();

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
        cherry_pick_commits(&repo, &[c1.to_string(), c2.to_string()])
            .unwrap();

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

    #[test]
    fn test_trailing_commits() {
        let (_dir, repo) = repo_init();

        let initial = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string();

        println!("initial: {}", initial);

        let c1 = write_commit_file(&repo, "a.txt", "testa", "add a");
        let c2 = write_commit_file(&repo, "b.txt", "testb", "add b");
        let c3 = write_commit_file(&repo, "c.txt", "testc", "add c");

        println!("c1 {}\n c2 {}\n c3 {}", c1, c2, c3);

        let trails = trailing_commits(&repo, &initial).unwrap();

        println!("from initial:");
        for t in &trails {
            println!("{}", t);
        }

        assert_eq!(trails.len(), 3);
        assert_eq!(trails[2], c3.to_string());
        assert_eq!(trails[1], c2.to_string());
        assert_eq!(trails[0], c1.to_string());

        let trails =
            trailing_commits(&repo, &c3.to_string()).unwrap();

        assert!(trails.is_empty());
    }
}
