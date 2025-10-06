<p align="center">
  <img src="https://github.com/nuttycream/gai/blob/main/docs/logo.svg" />
</p>

# 

gai, is a git + AI powered TUI that automatically generates commits, commit
messages, and branches when appropriate.

gai is yet another entry to the **A**rtificial (intelligence) **S**lop
**S**oftware initiative (aka. ASS). 

It works by taking a diff of your repo and
sending an API request to an LLM AI provider, where it takes that response and
builds out git operations along with messages for you to review and send out -
simplifying the process greatly.

> [!NOTE]
> This is not a complete git replacement. In fact, I recommend heavily relying
> on the git cli and using this primarily when you don't want to create commit
> messages.

## Planned features

- [x] Per File Staging - stage and commit per file/s commit messages
- [ ] Per Hunk Staging - similar to `git add -p`, stage on a per hunk basis,
      with relevant hunks placed together.
- [ ] Magic Rebasing - per @water-sucks:
  - having a ton of staged changes that split cleanly into multiple commits
  - reworking a branch by formulating some form of rebase plan
    `git rebase —edit-todo style` or by creating new commits out of a list of
    hunks or commits on a given branch
- [ ] Recreate/Amend Existing Commits - take a diff from one point in the commit
      history to another point, and have the LLM create a clean history, either
      by recreating/splitting the commits, or by amending them in place.
- [ ] CLI - optionally skip the tui and provide an (optional)
      confirmation/dialog to stdout.
- [ ] GitHub Marketplace App - GH actions bot
