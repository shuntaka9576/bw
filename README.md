# bw

A worktree management tool based on bare clone.

This project is created for personal use.

## Install

```bash
git clone git@github.com:shuntaka9576/bw.git
cd bw
cargo install --path .
```

## Usage

Clone a repository

```bash
bw get git@github.com:user/repo.git
```

Add a worktree

```bash
bw add feature/new-feature
```

List worktrees

```bash
bw list
```

Remove a worktree

```bash
bw remove feature-new-feature
```

## Configuration

~/.config/bw/config.toml

```toml
root = "~/repos"
clone_method = "ssh"
```

bw.toml (per repository)

```toml
base_branch = "main"
post_add_commands = '''
npm install
'''
```
