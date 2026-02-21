# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by Keep a Changelog, and the project adheres to semantic versioning.

## [Unreleased]

## [1.0.0] - 2026-02-21

### Added

- Core engine with `Game` trait, `Session`, `Store` trait, and `Engine` orchestrator
- Full Sixty-six game rules implementation:
  - Card play with trick resolution
  - Stock management (open/closed)
  - Marriage declarations (20/40 points)
  - Trump exchange mechanics
  - Last trick bonus
- HTTP API via axum (fully compatible with go-sixtysix endpoints)
- In-memory session store with pluggable `Store` trait interface
- Example demo server (port 8080)
- Comprehensive test suite (8 tests covering game, engine, store, API)
- Multi-stage Dockerfile for production deployment
- Full documentation (rules, API guide, integration notes)
- GitHub Actions CI/CD workflows (build, test, coverage, security audit, Docker)
- Dependabot configuration for dependency updates
- Community files (LICENSE, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, FUNDING)
