# to

`to` is an exploratory directory jumper for zsh.

zoxide optimizes history: it gets better after you visit directories.
`to` optimizes discovery: it indexes configured roots, then jumps to matching
folders even if you have never visited them before.

```zsh
to backend
to src/components
to app backend
to repo nginx
```

Because `cd` must run in the current shell, `to` is loaded as a zsh function.
The optional Rust helper accelerates SQLite queries, while zsh still performs
the final `cd`.

## Getting Started

Install with Homebrew:

```zsh
brew tap Z-ready/zsh-to https://github.com/Z-ready/zsh-to
brew install to
```

Add this to `~/.zshrc`:

```zsh
eval "$(to init zsh)"
```

Reload zsh and add a focused search root:

```zsh
source ~/.zshrc
to use ~/Projects
to --reindex
to backend
```

Homebrew installs the normal runtime environment for the full experience:
`fd`, `fzf`, `sqlite`, and a filesystem watcher (`fswatch` on macOS,
`inotify-tools` on Linux). Rust is used only while building `to-helper`.

## What It Does

`to` searches directory trees you choose:

```zsh
to use ~/Projects
to use ~/Downloads
to roots
```

Then it resolves queries in this order:

1. Built-in aliases, such as `download` and `desktop`.
2. User aliases from `to add`.
3. Workspaces from `to workspace`.
4. SQLite exact-name matches.
5. SQLite Git repo matches.
6. SQLite token matches for multi-word queries.
7. SQLite path-fragment matches.
8. `fd` discovery, with `find` fallback.

If a database path is stale, `to` removes it, falls back to filesystem
discovery, and writes the fresh result back into the index.

## Commands

```zsh
to <query...>             # search and cd to a matching directory
to -i <query...>          # force interactive selection
to -r <root> <query...>   # temporarily search from one root
to --from <root> <query>  # same as -r

to use .                  # add the current directory as a search root
to use ~/Projects         # add a specific search root
to unuse ~/Projects       # remove a search root
to roots                  # list search roots

to add blog ~/Notes/blog  # add a user alias
to remove blog            # remove a user alias
to aliases                # list user aliases

to workspace work ~/Work  # add a workspace
to work work              # jump to a workspace
to unwork work            # remove a workspace
to workspaces             # list workspaces

to repo nginx             # jump to a matching Git repository
to recent                 # jump from recent destinations
to ai docker              # use TO_AI_COMMAND, or broad fallback search

to --reindex              # refresh the SQLite/TSV directory index
to --watch                # watch roots and reindex after filesystem changes
to --doctor               # check dependencies and config
```

When multiple matches are found, `to` opens `fzf`. If `fzf` is unavailable, it
prints a numbered list.

## Matching Examples

Built-in aliases:

```text
download / downloads -> ~/Downloads
desktop              -> ~/Desktop
document/documents   -> ~/Documents
project/projects     -> ~/Projects
code                 -> ~/Code
```

Exact directory names win for plain single-word queries:

```zsh
to assignment
```

If `~/Downloads/Assignment` exists, `to` jumps there instead of listing every
child directory below it.

Path fragments work when the query contains a slash:

```zsh
to src/components
```

Multi-word queries require every token to appear somewhere in the path:

```zsh
to app backend
```

Git-aware search uses `.git` detected during indexing:

```zsh
to repo nginx
```

User shortcuts:

```zsh
to add blog ~/Documents/obsidian/blog
to blog

to workspace work ~/Documents/work
to work work
```

Recent destinations:

```zsh
to recent
```

## Performance

`to` does not run a background daemon. Idle cost is effectively zero.

Normal jumps are index-first:

```text
query -> SQLite -> validate path -> cd
                  -> fallback to fd/find on miss or stale path
                  -> write fresh result back to SQLite
```

The SQLite index stores:

```sql
dirs(id, path, name, parent, depth, is_git, last_seen, last_used, hit_count)
tokens(token, dir_id)
roots(path, mtime, config_key, last_indexed)
aliases(name, path)
workspaces(name, path)
recent(path, last_used)
```

That makes common developer queries cheap:

```zsh
to backend        # exact-name or token query
to app backend    # token query
to repo nginx     # Git repo + token query
```

Successful jumps update `hit_count` and `last_used`, so ranking improves with
use. Results are ordered roughly as:

```text
exact name > recent/frequent > shallower depth > shorter path
```

`to --reindex` is incremental for SQLite. It records root mtime and
index-affecting config, skips unchanged roots, refreshes changed roots, adds
new directories, and prunes stale directories under refreshed roots.

Broader modes cost more:

- `to src/components` searches a path fragment.
- `to -i backend` collects all matches for selection.
- `TO_SEARCH_PATH_FRAGMENTS=1` lets single words match anywhere in a path.
- `to --watch` runs a foreground watcher and reindexes after filesystem events.

For large machines, prefer focused roots:

```zsh
to use ~/Projects
to use ~/Downloads
```

Avoid using your whole home directory as the only root unless you really want
that scan:

```zsh
to use ~
```

Low-energy defaults:

```zsh
TO_MAX_DEPTH=5
TO_SEARCH_PATH_FRAGMENTS=0
TO_FOLLOW_SYMLINKS=0
```

## Configuration

Configuration is loaded from:

```zsh
~/.config/to/config.zsh
```

Example:

```zsh
TO_ROOTS=(
  "$HOME/Projects"
  "$HOME/Documents"
)

TO_EXCLUDES=(
  ".git"
  "node_modules"
  "target"
  ".venv"
  "Library"
)

TO_MAX_DEPTH=8
TO_INTERACTIVE_THRESHOLD=3
TO_SEARCH_PATH_FRAGMENTS=0
TO_FOLLOW_SYMLINKS=0
TO_WATCH_DEBOUNCE=2
TO_AI_COMMAND=""
TO_AI_RANK_COMMAND=""
TO_HELPER=""
```

Important options:

- `TO_MAX_DEPTH`: maximum scan depth for each root.
- `TO_SEARCH_PATH_FRAGMENTS`: set to `1` to let plain single words match
  anywhere in a path.
- `TO_FOLLOW_SYMLINKS`: set to `1` to scan through symlinked directories.
- `TO_WATCH_DEBOUNCE`: seconds to wait before reindexing after watcher events.
- `TO_AI_COMMAND`: external command for `to ai <query...>`.
- `TO_AI_RANK_COMMAND`: external command that ranks normal candidate paths from
  stdin. The query is passed as its first argument.
- `TO_HELPER`: explicit path to `to-helper`; otherwise `to` auto-detects it.

Persistent state lives under `~/.config/to` by default:

```text
roots         configured search roots
index.sqlite3 SQLite dirs, tokens, aliases, workspaces, recent, root metadata
aliases       text fallback for aliases
workspaces    text fallback for workspaces
recent        text fallback for recent jumps
index.tsv     fallback directory index when sqlite3 is unavailable
```

Set `TO_CONFIG_HOME` before sourcing the plugin to use another config
directory.

## Shell Integration

`to` is a zsh plugin:

```zsh
eval "$(to init zsh)"
```

If you prefer to source the plugin directly:

```zsh
source "$(brew --prefix to)/share/to/to.plugin.zsh"
```

Completions are installed by Homebrew. If completions do not appear, make sure
your zsh config runs `compinit`, then rebuild the completion cache:

```zsh
rm -f ~/.zcompdump*
autoload -Uz compinit
compinit
```

## AI Hook

`to` does not bundle a hosted AI model. Instead, `TO_AI_COMMAND` and
`TO_AI_RANK_COMMAND` are extension points for users who want to connect a local
or remote ranker.

When set, `to ai <query...>` runs that command with the query and expects it to
print candidate directories, one per line. `to` validates candidates before
jumping.

Without `TO_AI_COMMAND`, `to ai` uses the broad built-in fallback search.

When `TO_AI_RANK_COMMAND` is set, normal filesystem fallback candidates are
sent to the command on stdin and the query is passed as the first argument. The
command should print ranked candidate paths, one per line. `to` validates the
output and appends any omitted original candidates after the ranked results.

## Troubleshooting

Check your environment:

```zsh
to --doctor
```

Refresh the index:

```zsh
to --reindex
```

Force interactive selection:

```zsh
to -i backend
```

Temporarily search a specific tree:

```zsh
to -r ~/Projects backend
```

If `to` finds too many matches, keep path-fragment search disabled:

```zsh
TO_SEARCH_PATH_FRAGMENTS=0
```

If `to` is slow, reduce roots and depth:

```zsh
to roots
TO_MAX_DEPTH=5
```

## Development

Run the zsh test suite:

```zsh
zsh tests/run.zsh
```

Run the Rust helper checks:

```zsh
cargo fmt -- --check
cargo test
cargo build --release
cargo clippy --all-targets -- -D warnings
```

For local formula development:

```zsh
brew tap-new local/to
cp Formula/to.rb "$(brew --repository local/to)/Formula/to.rb"
brew install local/to/to
```
