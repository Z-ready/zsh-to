# Contributing

Thanks for helping improve `to`. This project is a zsh-first CLI, so changes
should be tested through the shell function whenever behavior affects users.

## Development setup

Install the runtime tools used by the test suite:

```zsh
brew install fd fzf sqlite zsh
```

On Linux, install the equivalent packages from your distribution. Some
distributions package `fd` as `fdfind`; add an `fd` symlink when needed.

Build the Rust helper:

```zsh
cargo build --release
```

Load the plugin from a checkout:

```zsh
source ./to.plugin.zsh
```

## Verification

Run the full local gate before opening a PR:

```zsh
zsh scripts/check.zsh
brew style Formula/to.rb
```

`brew audit --strict --online to` should be run from an installed tap when
preparing a release.

## Change guidelines

- Preserve public behavior unless the existing behavior is clearly broken.
- Add regression tests for every behavior fix.
- Keep README examples executable.
- Prefer small shell changes with observable tests over broad rewrites.
- Keep paths quoted; tests should cover spaces and unicode when touching path
  handling.
- Treat `TO_AI_COMMAND` and `TO_AI_RANK_COMMAND` as explicit user-controlled
  hooks. Do not run generated or untrusted command strings implicitly.

## Pull request checklist

- [ ] User-visible behavior is tested through `zsh tests/run.zsh`.
- [ ] Rust helper changes pass `cargo fmt`, `cargo test`, and `cargo clippy`.
- [ ] README and completions are updated for new commands or flags.
- [ ] Homebrew formula changes pass `ruby -c` and `brew style`.
- [ ] Fresh-install behavior still works with `eval "$(to init zsh)"`.
