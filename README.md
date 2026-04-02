<p align="center">
  <img src="https://github.com/cube-cult/gai-site/blob/main/static/gai_logo.svg" alt="logo"/>
  <hr>
</p>

gai, is a git + AI powered TUI that uses the intelligence of LLM's to simplify
the process of creating entire commits. From staging files or chunks of code, to
creating a verbose and well defined commit message. It simplifies the process of
version control with the added benefit of a clean commit history.

<p align="center">
  <img src="https://github.com/cube-cult/gai-site/blob/main/static/demo.gif" width=60% alt="gif"/>
</p>

## Planned features

- [x] Per File Staging [#4](https://github.com/cube-cult/gai/issues/4)
- [x] CLI [#8](https://github.com/cube-cult/gai/issues/8)
- [x] Per Hunk Staging [#5](https://github.com/cube-cult/gai/issues/5)
- [x] Magic Rebasing [#6](https://github.com/cube-cult/gai/issues/6)
- [x] Magic Find [#12](https://github.com/cube-cult/gai/issues/12)
- [ ] Magic Undo [#72](https://github.com/cube-cult/gai/issues/72)
- [ ] Magic Sync [#29](https://github.com/cube-cult/gai/issues/29)
- [ ] Magic Blame [#73](https://github.com/cube-cult/gai/issues/73)
- [x] Recreate/Amend Existing Commits
      [#7](https://github.com/cube-cult/gai/issues/7)

# Attribution

[`asyncgit`](https://github.com/gitui-org/gitui/tree/master/asyncgit)

Copied and modified from their `sync` module, generating diffs, status, commit,
and staging. Ideally, I didn't want to introduce an async runtime as well as the
dependencies needed from using the crate.

[`tui-rs-tree-widget`](https://github.com/EdJoPaTo/tui-rs-tree-widget)

Copied and modified the `TreeItem` struct and `Tree` widget to create a similar
customizable tree. Removed most of the `ratatui` related implementations, in
favor of printing directly to the terminal or as a string.
