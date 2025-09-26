<p align="center">
  <img src="https://github.com/nuttycream/gai/blob/main/docs/logo.svg" />
</p>

# 

gai, pronounced guy, is a git + AI powered TUI that automatically generates
commits, commit messages, and branches when appropriate.

## What it is:

It works by taking a diff of your repo and sending an API request to an LLM AI
provider, where it takes that response and builds out git operations along with
messages for you to review and send out - simplifying the process greatly.

## What it isn't:

This is not a complete git replacement. In fact, I recommend heavily relying on
the git cli and using this primarily when you don't want to create commit
messages. If you're looking for an amazing git tui:

- [gitui](https://github.com/gitui-org/gitui)
- [lazygit](https://github.com/jesseduffield/lazygit)
