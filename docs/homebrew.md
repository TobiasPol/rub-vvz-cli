# Homebrew

The current formula is source-based. Homebrew downloads the GitHub tag archive,
uses Rust as a build dependency, and installs the `rub-vvz` binary.

## Install

```bash
brew tap TobiasPol/rub-vvz-cli https://github.com/TobiasPol/rub-vvz-cli
brew install rub-vvz
```

## Upgrade

```bash
brew update
brew upgrade rub-vvz
```

## Formula Maintenance

After a new release:

1. Update `Formula/rub-vvz.rb` with the new tag URL.
2. Recalculate the SHA:

   ```bash
   curl -L https://github.com/TobiasPol/rub-vvz-cli/archive/refs/tags/<tag>.tar.gz | shasum -a 256
   ```

3. Run:

   ```bash
   brew audit --strict --online rub-vvz
   brew test rub-vvz
   ```
