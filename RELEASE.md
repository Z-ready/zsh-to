# Release Checklist

Use this checklist when preparing a tagged release.

## 1. Pick the version

Update every versioned surface together:

- `Cargo.toml`
- `Cargo.lock`
- `bin/to`
- `to.plugin.zsh`
- `Formula/to.rb` tag URL and SHA256

Homebrew infers the formula version from the tag URL, so do not add a separate
`version` field unless the URL cannot be parsed.

## 2. Run local verification

```zsh
zsh scripts/check.zsh
brew style Formula/to.rb
```

## 3. Smoke-test install paths

Homebrew:

```zsh
brew uninstall to || true
brew install to
zsh -fc 'eval "$(to init zsh)"; to --version; to --doctor'
brew reinstall to
brew uninstall to
```

Manual source install:

```zsh
cargo build --release
install -d ~/.local/bin ~/.local/share/to ~/.local/share/zsh/site-functions
install -m 755 bin/to ~/.local/bin/to
install -m 755 target/release/to-helper ~/.local/bin/to-helper
install -m 644 to.plugin.zsh ~/.local/share/to/to.plugin.zsh
install -m 644 completions/_to ~/.local/share/zsh/site-functions/_to
zsh -fc 'eval "$(~/.local/bin/to init zsh)"; to --version; to --doctor'
```

## 4. Tag and archive

```zsh
git tag "vX.Y.Z"
git push origin "vX.Y.Z"
```

Fetch the release tarball and compute the formula SHA256:

```zsh
curl -L -o "to-X.Y.Z.tar.gz" \
  "https://github.com/Z-ready/zsh-to/archive/refs/tags/vX.Y.Z.tar.gz"
shasum -a 256 "to-X.Y.Z.tar.gz"
```

Update `Formula/to.rb`, then run:

```zsh
brew audit --strict --online to
brew test to
```

## 5. Final review

- README installation steps match the released artifacts.
- `to --version` prints the new version before and after shell integration.
- `to --doctor` reports expected dependency status.
- No generated build output is included in the release commit.
