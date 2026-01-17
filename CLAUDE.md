# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

ghqb は bare clone と worktree をサポートした ghq ライクなツールです。リポジトリを `.bare` ディレクトリに bare clone し、後から worktree を追加できる構造でクローンします。

## ビルドコマンド

```bash
# 開発ビルド
cargo build

# リリースビルド（LTO有効、バイナリストリップ）
cargo build --release

# テスト実行
cargo test

# 単一テスト実行
cargo test test_parse_ssh_url

# クリッパーによるリント
cargo clippy

# フォーマット
cargo fmt
```

## アーキテクチャ

### モジュール構成

- `src/main.rs` - CLIエントリポイント。clap によるサブコマンド定義（`get`, `config`, `add`, `list`, `rm`）
- `src/commands/` - サブコマンドの実装
  - `get.rs` - リポジトリのクローン処理。URL解析→bare clone→post_clone_commands実行
  - `config.rs` - 設定ファイルをエディタで開く
  - `bw.rs` - worktree管理（add/list/rm）
- `src/git/clone.rs` - git2クレートを使用したbare clone実装。SSH/HTTPSの認証対応
- `src/url/parser.rs` - リポジトリURL解析。SSH、HTTPS、短縮形式（github.com/user/repo）をサポート
- `src/config/mod.rs` - TOML設定ファイル管理。`~/.config/ghqb/config.toml`
- `src/error.rs` - thiserrorによるエラー型定義

### クローン処理の流れ

1. URL解析 (`url::parse_repo_url`) → `RepoInfo` 構造体へ
2. クローン先パスを `{root}/{host}/{owner}/{repo}` 形式で構築
3. `.bare` サブディレクトリに bare clone を実行
4. `post_clone_commands` を実行:
   - `.git` ファイル作成（`gitdir: .bare`）
   - fetch 設定
   - HEADブランチ（main等）を自動でworktreeに追加

### クローン後のディレクトリ構造

```
~/repos/github.com/user/repo/
├── .bare/           # bare clone
├── .git             # gitdir: .bare
└── main/            # HEADブランチのworktree（自動作成）
```

### 設定ファイル

場所: `$XDG_CONFIG_HOME/ghqb/config.toml` または `~/.config/ghqb/config.toml`

```toml
root = "~/repos"           # クローン先のルートディレクトリ
clone_method = "ssh"       # デフォルトのクローン方式
suffix = ".work"           # ディレクトリ名のサフィックス（オプション）
post_clone_commands = '''  # bare clone後に実行するコマンド
echo 'gitdir: .bare' > .git
git config --file .bare/config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
git fetch origin
HEAD_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'); [ -n "$HEAD_BRANCH" ] && git worktree add "$HEAD_BRANCH" "$HEAD_BRANCH"
'''
```

## 認証

- SSH: ssh-agent から認証情報を取得
- HTTPS: 環境変数 `GIT_USERNAME`, `GIT_PASSWORD` を使用

## Worktree管理

### コマンド

```bash
# worktree作成（新規ブランチ）
bw add feature/test

# worktree作成（既存ブランチ）
bw add main

# worktree一覧（fzf選択→パス出力）
bw list

# worktree削除
bw rm feature-test
```

### 設定ファイル（bw.toml）

リポジトリルート（.bareと同じ階層）に配置:

```toml
base_branch = "main"
post_add_commands = '''
npm install
'''
```

### 処理の流れ

1. `.bare` ディレクトリを探してリポジトリルートを特定
2. 無効なworktree登録があれば自動でprune
3. ブランチ名のスラッシュをハイフンに変換してディレクトリ名に（`feature/test` → `feature-test`）
4. 既存ブランチなら `git worktree add <path> <branch>`、新規なら `git worktree add -b <branch> <path> <base>`
5. `post_add_commands` を実行
