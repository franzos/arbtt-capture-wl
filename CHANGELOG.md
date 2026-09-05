# Changelog

## [0.1.4] - 2026-09-05

### Added
- CI workflow: format, clippy, tests, and dependency audit
- Build provenance attestation on release assets
- README: checksum verification in the install steps
- README: idle time is not detected

### Changed
- `~/.arbtt` is created with mode 0700
- Release builds use `--locked` and pinned packaging tools
- chrono: dropped unused features and their transitive crates
- Rust edition 2024, MSRV 1.85

### Fixed
- Write errors to arbtt-import carry the intended error context
