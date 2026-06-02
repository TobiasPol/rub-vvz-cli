# RUB VVZ CLI

`rub-vvz` is a small Rust command-line client for the public course catalogue of
Ruhr-Universitaet Bochum (RUB).

It reads the public VVZ/CampusOffice HTML pages at
[`vvz.ruhr-uni-bochum.de`](https://vvz.ruhr-uni-bochum.de/) and does not use,
store, or request private eCampus credentials. Private eCampus data such as
registrations, exams, grades, or personal timetables are intentionally out of
scope.

## Status

This project is currently in beta.

The latest beta release is `v0.1.0-beta.2`. The CLI is usable for public VVZ
lookups, but RUB can change CampusOffice HTML without notice. Parser fixes are
expected during beta.

## Features

- Search public RUB lectures and courses by title, lecturer, or course number
- Show details for a course `gguid`
- List top-level VVZ areas and faculties
- List all events in a VVZ area
- Print either readable text or machine-readable JSON
- Text output explains identifiers and useful next commands for agents
- Detect the current semester `tguid` from the public VVZ
- No external Rust crate dependencies
- Uses `curl` for HTTP transport, which keeps builds simple and dependency-free

## Installation

### Homebrew on macOS

The repository contains a Homebrew formula after the beta release is published.
Because the repository is not named as a conventional `homebrew-*` tap, tap it
with an explicit URL:

```bash
brew tap TobiasPol/rub-vvz-cli https://github.com/TobiasPol/rub-vvz-cli
brew install rub-vvz
```

If your Homebrew installation enforces tap trust, trust only this formula before
installing:

```bash
brew trust --formula TobiasPol/rub-vvz-cli/rub-vvz
```

Upgrade later with:

```bash
brew update
brew upgrade rub-vvz
```

### Cargo

```bash
cargo install --git https://github.com/TobiasPol/rub-vvz-cli --tag v0.1.0-beta.2
```

### From Source

```bash
git clone https://github.com/TobiasPol/rub-vvz-cli.git
cd rub-vvz-cli
cargo install --path .
```

## Requirements

- macOS or Linux
- `curl` in `PATH`
- Rust only when building from source or installing through the current
  source-based Homebrew formula

## Usage

Search for public courses:

```bash
rub-vvz help
rub-vvz help search
rub-vvz search Cryptography
rub-vvz search "Software Engineering" --limit 5
rub-vvz search Cryptography --json
```

Show a course detail page:

```bash
rub-vvz event 0xACEEDD74DF204A70B6ED84BACDA481CC
rub-vvz event 0xACEEDD74DF204A70B6ED84BACDA481CC --json
```

List VVZ areas and faculties:

```bash
rub-vvz fields
rub-vvz fields --json
```

List events inside a VVZ area:

```bash
rub-vvz events 0x9666E9D9803C46EB9ABF547944989092 --limit 10
rub-vvz events 0x9666E9D9803C46EB9ABF547944989092 --json
```

Use a specific semester:

```bash
rub-vvz search Cryptography --term-guid 0x32F038F554334DC3AB9024E476ECAE2E
```

## Commands

### `help [command]`

Shows root help or command-specific help. The explicit command form is intended
for agents and scripts that should not infer usage from failures.

```bash
rub-vvz help
rub-vvz help search
rub-vvz event --help
```

### `search <query>`

Searches public VVZ events.

Options:

- `--json` prints JSON
- `--limit <n>` or `--limit=<n>` limits result count, default `20`
- `--term-guid <tguid>` or `--term-guid=<tguid>` uses a specific semester
- `--lang <lang>` or `--lang=<lang>` sets the VVZ language, default `de`

### `event <gguid-or-url>`

Shows details for a course event. The argument can be a raw event `gguid` or a
complete `event.asp` URL.

Options:

- `--json` prints JSON
- `--term-guid <tguid>` or `--term-guid=<tguid>` is needed when a raw `gguid`
  should be resolved for a specific semester
- `--lang <lang>` or `--lang=<lang>` sets the VVZ language, default `de`

### `fields`

Lists top-level public VVZ areas and faculties.

Options:

- `--json` prints JSON
- `--term-guid <tguid>` or `--term-guid=<tguid>` uses a specific semester
- `--lang <lang>` or `--lang=<lang>` sets the VVZ language, default `de`

### `events <field-gguid-or-url>`

Lists public events for a VVZ area or faculty. The argument can be a raw
`gguid` from `fields` or a complete `eventlist.asp` URL.

Options:

- `--json` prints JSON
- `--limit <n>` or `--limit=<n>` limits result count, default `100`
- `--term-guid <tguid>` or `--term-guid=<tguid>` uses a specific semester
- `--lang <lang>` or `--lang=<lang>` sets the VVZ language, default `de`

## Examples

Find cryptography courses:

```bash
rub-vvz search Cryptography --limit 3
```

Fetch details as JSON:

```bash
rub-vvz event 0xACEEDD74DF204A70B6ED84BACDA481CC --json
```

Find the Informatics faculty identifier:

```bash
rub-vvz fields | grep Informatik
```

List Informatics events:

```bash
rub-vvz events 0x9666E9D9803C46EB9ABF547944989092 --limit 20
```

## JSON Output

Text output is optimized for humans and agents reading terminal logs. It
includes an `Interpretation:` paragraph that explains `gguid`, `tguid`, and a
useful follow-up command. Use `--json` whenever another program should parse the
result. JSON output intentionally does not include explanatory prose.

`search` and `events` return arrays of course references:

```json
[
  {
    "title": "Introduction to Cryptography 2",
    "url": "https://vvz.ruhr-uni-bochum.de/campus/all/event.asp?...",
    "event_guid": "0xACEEDD74DF204A70B6ED84BACDA481CC",
    "term_guid": "0x32F038F554334DC3AB9024E476ECAE2E",
    "summary": "211009 | Paar, Christof | Lecture with exercise"
  }
]
```

`event --json` returns:

- `title`
- `url`
- `event_guid`
- `term_guid`
- `fields`, such as course number, title, type, credits, semester and SWS
- `sections`, such as appointments, lecturers, modules and audience groups
- `description` when available

## CI/CD

GitHub Actions are configured for:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --locked`
- release builds for Linux and macOS when pushing `v*` tags
- GitHub prereleases for tags containing `alpha`, `beta`, or `rc`

## Releases

Release tags use this format:

```text
vMAJOR.MINOR.PATCH[-PRERELEASE.N]
```

Example:

```text
v0.1.0-beta.2
```

## Development

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- search Cryptography --limit 1
```

The CLI intentionally has no external Rust crate dependencies. If richer HTML
parsing or native HTTP becomes necessary later, add it deliberately and keep the
existing parser fixtures as regression tests.

## Limitations

- Public VVZ pages are parsed from HTML, not a stable JSON API.
- Personal eCampus data is not supported.
- Write actions such as registrations or timetable changes are not supported.
- RUB can change CampusOffice markup at any time; parser regressions should be
  covered with fixtures before fixing.

## License

MIT. See [LICENSE](LICENSE).
