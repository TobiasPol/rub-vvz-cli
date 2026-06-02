# Changelog

All notable changes to `rub-vvz` will be documented here.

## [0.1.0-beta.2] - 2026-06-02

### Added

- Explicit `help [command]` support for root and command-specific help.
- Explanatory text output for search results, event details, fields, and empty
  states to make agent-read terminal output easier to interpret.
- Support for `--flag=value` option syntax.

### Changed

- Switched CLI-generated help, errors, examples, and explanations to English.
- Kept `--json` output free of explanatory prose for stable machine parsing.

### Fixed

- Unknown options now fail with a clear error instead of being ignored.

## [0.1.0-beta.1] - 2026-06-02

### Added

- Initial Rust CLI for the public RUB VVZ.
- Commands: `search`, `event`, `fields`, and `events`.
- Text and JSON output modes.
- Current semester `tguid` discovery.
- Parser fixtures for CampusOffice search results, detail fields, and public
  subfield links.
- GitHub Actions CI for format, lint, and tests.
- GitHub Actions release workflow for tagged builds.
- Homebrew installation documentation.
