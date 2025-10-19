use std::path::Path;

use crate::git::{
    commit::GaiCommit,
    repo::{DiffType, GaiGit},
};

impl GaiGit {
    pub fn apply_commits(&self, commits: &[GaiCommit]) {
        //println!("{:#?}", self.commits);
        for commit in commits {
            self.commit(commit);
        }
    }

    fn commit(&self, commit: &GaiCommit) {
        let mut index = self.repo.index().unwrap();

        index.clear().unwrap();

        if let Ok(head) = self.repo.head()
            && let Ok(tree) = head.peel_to_tree()
        {
            index.read_tree(&tree).unwrap();
        }

        // todo impl validation and add failed hunks
        if self.stage_hunks {
            // going to bypass the index
            // and instead use the stored hunks
            // from create_diffs to create patches
            self.stage_hunks(commit);
        } else {
            self.stage_files(&mut index, commit);
        }

        index.write().unwrap();

        let tree_oid = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_oid).unwrap();

        let parent_commit = match self.repo.revparse_single("HEAD") {
            Ok(obj) => Some(obj.into_commit().unwrap()),
            // ignore first commit
            Err(_) => None,
        };

        let mut parents = Vec::new();
        if let Some(parent) = parent_commit.as_ref() {
            parents.push(parent);
        }

        let sig = self.repo.signature().unwrap();

        let commit_msg = &commit.message;

        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                commit_msg,
                &tree,
                &parents[..],
            )
            .unwrap();
    }

    pub fn stage_hunks(&self, commit: &GaiCommit) {
        let patch = self.create_patches(&commit.hunk_ids);

        match git2::Diff::from_buffer(patch.as_bytes()) {
            Ok(diff) => {
                match self.repo.apply(
                    &diff,
                    git2::ApplyLocation::Index,
                    None,
                ) {
                    Ok(_) => println!("Staged and Applied Hunks!"),
                    Err(e) => {
                        println!("failed to stage hunks: {}", e);

                        std::fs::write("failed.patch", &patch)
                            .unwrap()
                    }
                }
            }

            Err(e) => println!("failed parse patches: {}", e),
        }
    }

    fn stage_files(
        &self,
        index: &mut git2::Index,
        commit: &GaiCommit,
    ) {
        for path in &commit.files {
            let path = Path::new(&path);
            let status = self.repo.status_file(path).unwrap();

            // todo: some changes will implement a combo
            // ex: modified + renamed
            // i think we need to explicitly handle those
            // maybe by storing it in a buffer of some sort
            if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_NEW)
            {
                index.add_path(path).unwrap();
            }
            if status.contains(git2::Status::WT_DELETED) {
                index.remove_path(path).unwrap();
            }
            if status.contains(git2::Status::WT_TYPECHANGE) {
                index.remove_path(path).unwrap();
                index.add_path(path).unwrap();
            }
        }
    }

    fn create_patches(&self, hunk_ids: &[String]) -> String {
        let mut patch = String::new();
        let mut current_file = String::new();

        for hunk_id in hunk_ids {
            // this may be relatively flimsy
            // todo: ideally we want to build out the schemars
            // based on a predetermined set and instead of having
            // an LLM 'write' it out, it can just choose it from the
            // set
            let Some((file_path, index_str)) =
                hunk_id.split_once(':')
            else {
                println!("not a valid hunk_id format:{}", hunk_id);
                continue;
            };

            let Ok(i) = index_str.parse::<usize>() else {
                println!("not a valid hunk_index:{}", index_str);
                continue;
            };

            let Some(file) =
                self.files.iter().find(|f| f.path == file_path)
            else {
                println!("not a valid file: {}", file_path);
                continue;
            };

            let Some(hunk) = file.hunks.get(i) else {
                println!(
                    "le hunk {} not found in file {}",
                    i, file_path
                );

                continue;
            };

            // creating a patch
            // the other methods, using a hunk hash
            // or using headers, kinda sucked
            // manually creating patches
            // helps, since we can also print them out
            // if they lets say fail
            if current_file != file_path {
                patch.push_str(&format!(
                    "diff --git a/{} b/{}\n",
                    file_path, file_path
                ));
                patch.push_str(&format!("--- a/{}\n", file_path));
                patch.push_str(&format!("+++ b/{}\n", file_path));
                current_file = file_path.to_string();
            }

            patch.push_str(&hunk.header);
            if !hunk.header.ends_with('\n') {
                patch.push('\n');
            }

            for line in &hunk.line_diffs {
                let prefix = match line.diff_type {
                    DiffType::Additions => '+',
                    DiffType::Deletions => '-',
                    DiffType::Unchanged => ' ',
                };

                patch.push(prefix);
                patch.push_str(&line.content);
            }
        }

        patch
    }
}
