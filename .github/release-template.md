# RUB VVZ CLI v0.1.0-beta.1

This is the first beta release of `rub-vvz`, a Rust CLI for the public
Ruhr-Universitaet Bochum course catalogue.

## Highlights

- Search public VVZ courses.
- Show event details by `gguid` or URL.
- List VVZ fields/faculties.
- List events within a VVZ field.
- Print text or JSON.

## Install

```bash
cargo install --git https://github.com/TobiasPol/rub-vvz-cli --tag v0.1.0-beta.1
```

Homebrew setup is documented in `README.md` and `docs/homebrew.md`.

## Notes

This beta parses public CampusOffice HTML. Markup changes on RUB systems can
require parser updates.
