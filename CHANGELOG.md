# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by Keep a Changelog, and the project adheres (informally) to semantic versioning starting at v0.

## [Unreleased]

### Initial Release

- Core engine with Game trait, Session, Store trait, Engine orchestrator
- Sixty-six rules (play, closeStock, declare, exchangeTrump, last trick bonus)
- HTTP API via axum (matching go-sixtysix endpoints)
- In-memory session store
- Example server
- Test suite (game, engine, store, API)
- Dockerfile (multi-stage)
- Documentation (rules, API, integration)
