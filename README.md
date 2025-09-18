<p align="center">
  <img src="https://github.com/nuttycream/gai/blob/main/docs/logo.svg" />
</p>

# gai

gai, pronounced guy, is a git + AI powered TUI that automatically generates
commits, commit messages, branches, and pull requests when appropriate.

It works by taking a diff of your repo and sending an API request to an LLM AI
provider, where it takes that response and builds out git operations along with
messages for you to review and send out - simplifying the process greatly.

On launch -> load git repo -> get repo status
