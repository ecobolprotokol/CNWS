# Changelog

Semua perubahan penting pada proyek ini akan didokumentasikan dalam file ini.

Format berdasarkan [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
dan versi mengikuti [Semantic Versioning](https://semver.org/lang/id/).

## [Unreleased]

### Added
- Inisialisasi workspace Rust dengan struktur lengkap
- Core library (`cnws-core`) dengan module:
  - Substrate layer (storage, integrity, revision, gc, recovery, conversion)
  - Lattice layer (runtime, memory, routing, learning, cache)
  - API layer (storage, conversion, runtime, revision, memory, admin)
  - Telemetry layer (metrics, logging, tracing)
- CLI binary (`cnws-cli`) dengan commands: init, import, diag, revision, memory, query, metrics
- Conformance test runner (`cnws-conformance`) dengan 10 test suites (CS-01 sampai CS-10)
- 17 spesifikasi teknis di `docs/specs/`
- GitHub Actions CI workflows
- Dependabot configuration
- Development tools (setup.sh, build.sh)

### Documentation
- README.md dengan struktur repository dan quick start
- CONTRIBUTING.md dengan panduan kontribusi
- .claude/instructions.md dengan development instructions

## [0.1.0] - 2024-01-01

### Added
- Initial release preparation
- Project structure setup
- Specification documents
