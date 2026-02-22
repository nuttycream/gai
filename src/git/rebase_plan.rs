use git2::Oid;

// lifted from asyncgit
// will use RebaseState progression
// to apply individual RebaseOperationTypes

//
#[derive(PartialEq, Eq, Debug)]
enum RebaseState {
    ///
    Finished,
    ///
    Conflicted,
}

/// rebase
fn rebase(
    repo: &git2::Repository,
    commit: &git2::AnnotatedCommit,
) -> anyhow::Result<RebaseState> {
    let mut rebase = repo.rebase(None, Some(commit), None, None)?;

    let signature = repo.signature()?;

    while let Some(op) = rebase.next() {
        let _op = op?;
        // dbg!(op.id());

        if repo
            .index()?
            .has_conflicts()
        {
            return Ok(RebaseState::Conflicted);
        }

        rebase.commit(None, &signature, None)?;
    }

    if repo
        .index()?
        .has_conflicts()
    {
        return Ok(RebaseState::Conflicted);
    }

    rebase.finish(Some(&signature))?;

    Ok(RebaseState::Finished)
}

/// continue pending rebase
fn continue_rebase(
    repo: &git2::Repository
) -> anyhow::Result<RebaseState> {
    let mut rebase = repo.open_rebase(None)?;
    let signature = repo.signature()?;

    if repo
        .index()?
        .has_conflicts()
    {
        return Ok(RebaseState::Conflicted);
    }

    // try commit current rebase step
    if !repo
        .index()?
        .is_empty()
    {
        rebase.commit(None, &signature, None)?;
    }

    while let Some(op) = rebase.next() {
        let _op = op?;
        // dbg!(op.id());

        if repo
            .index()?
            .has_conflicts()
        {
            return Ok(RebaseState::Conflicted);
        }

        rebase.commit(None, &signature, None)?;
    }

    if repo
        .index()?
        .has_conflicts()
    {
        return Ok(RebaseState::Conflicted);
    }

    rebase.finish(Some(&signature))?;

    Ok(RebaseState::Finished)
}

///
#[derive(PartialEq, Eq, Debug)]
struct RebaseProgress {
    ///
    pub steps: usize,
    ///
    pub current: usize,
    ///
    pub current_commit: Option<Oid>,
}

///
fn get_rebase_progress(
    repo: &git2::Repository
) -> anyhow::Result<RebaseProgress> {
    let mut rebase = repo.open_rebase(None)?;

    let current_commit: Option<Oid> = rebase
        .operation_current()
        .and_then(|idx| rebase.nth(idx))
        .map(|op| op.id());

    let progress = RebaseProgress {
        steps: rebase.len(),
        current: rebase
            .operation_current()
            .unwrap_or_default(),
        current_commit,
    };

    Ok(progress)
}

///
fn abort_rebase(repo: &git2::Repository) -> anyhow::Result<()> {
    let mut rebase = repo.open_rebase(None)?;

    rebase.abort()?;

    Ok(())
}
