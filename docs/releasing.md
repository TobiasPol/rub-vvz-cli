# Releasing

This project uses Git tags to drive release automation.

## Versioning

Use semantic versions with prerelease suffixes during beta:

```text
v0.1.0-beta.1
v0.1.0-beta.2
v0.1.0
```

The Cargo package version should match the tag without the leading `v`.

## Checklist

1. Update `Cargo.toml`.
2. Update `CHANGELOG.md`.
3. Run local checks:

   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test --locked
   ```

4. Commit the changes.
5. Create and push a tag:

   ```bash
   git tag v0.1.0-beta.1
   git push origin main --tags
   ```

6. GitHub Actions will build release artifacts and publish a GitHub release.
7. Update the Homebrew formula SHA if the formula points at a new source
   archive or release asset.

## Homebrew SHA

For a source-based formula:

```bash
curl -L https://github.com/TobiasPol/rub-vvz-cli/archive/refs/tags/v0.1.0-beta.1.tar.gz | shasum -a 256
```
