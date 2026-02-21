# rust-sixtysix

[![CI](https://github.com/RumenDamyanov/rust-sixtysix/actions/workflows/ci.yml/badge.svg)](https://github.com/RumenDamyanov/rust-sixtysix/actions/workflows/ci.yml)
[![CodeQL](https://github.com/RumenDamyanov/rust-sixtysix/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/RumenDamyanov/rust-sixtysix/actions/workflows/github-code-scanning/codeql)
[![Dependabot](https://github.com/RumenDamyanov/rust-sixtysix/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/RumenDamyanov/rust-sixtysix/actions/workflows/dependabot/dependabot-updates)
[![codecov](https://codecov.io/gh/RumenDamyanov/rust-sixtysix/graph/badge.svg)](https://codecov.io/gh/RumenDamyanov/rust-sixtysix)
[![crates.io](https://img.shields.io/crates/v/rumenx-sixtysix.svg)](https://crates.io/crates/rumenx-sixtysix)
[![docs.rs](https://docs.rs/rumenx-sixtysix/badge.svg)](https://docs.rs/rumenx-sixtysix)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

Minimal backend engine + HTTP API for the traditional 24-card trick-taking game **Sixty-six** (AKA *Schnapsen* variant family). Built in Rust for frontend clients (web, mobile, CLI bots) that want a stateless, deterministic core.

This is a Rust adaptation of [go-sixtysix](https://github.com/rumendamyanov/go-sixtysix), matching its functionality and API surface.

Rules reference: [Wikipedia – Sixty-six](https://en.wikipedia.org/wiki/Sixty-six_(card_game)). This implementation models a standard two-player deal with marriages, closing the stock, trump exchange, and last trick bonus.

## Contents

- [Features](#features)
- [Install](#install)
- [Quick Start](#quick-start)
- [Container Image](#container-image)
- [Concepts](#concepts)
- [HTTP API](#http-api)
- [Game Rules Summary](#game-rules-summary)
- [Frontend Integration Ideas](#frontend-integration-ideas)
- [Project Layout](#project-layout)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Features

- Deterministic game state creation (seeded RNG) for reproducible replays
- Lightweight in-memory session store (pluggable trait interface)
- Clear `Game` trait (validate + apply immutable state transitions)
- HTTP API with small surface (sessions + actions) via axum
- Test coverage across engine, store, rules, and API
- Simple deployment (single binary)

## Install

Requires Rust 1.70+.

Add as dependency:

```toml
[dependencies]
sixtysix = { git = "https://github.com/rumendamyanov/rust-sixtysix" }
```

Or clone and use directly:

```bash
git clone https://github.com/rumendamyanov/rust-sixtysix
cd rust-sixtysix
cargo build
```

Typical usage:

```rust
use sixtysix::engine::Engine;
use sixtysix::game::SixtySix;
use sixtysix::store::Memory;
use std::sync::Arc;

fn main() {
    let mem = Arc::new(Memory::new());
    let engine = Arc::new(Engine::new(mem));
    engine.register(Arc::new(SixtySix));
    // create and use sessions via engine API
}
```

## Quick Start

1. Build the project: `cargo build`
2. Run the demo server:

```bash
cargo run --example server
```

3. Create a session (seed optional):

```bash
curl -s -X POST 'http://localhost:8080/sessions?game=sixtysix&seed=42' | jq
```

4. List sessions:

```bash
curl -s 'http://localhost:8080/sessions?game=sixtysix' | jq
```

5. Play a card (`card` is an encoded int `suit*100+rankValue`):

```bash
curl -s -X POST http://localhost:8080/sessions/{id} \
  -H 'Content-Type: application/json' \
  -d '{"type":"play","payload":{"card":211}}' | jq
```

6. Close stock:

```bash
curl -s -X POST http://localhost:8080/sessions/{id} \
  -d '{"type":"closeStock"}'
```

More examples: see [docs/api.md](docs/api.md).

## Container Image

Build locally (multi-stage):

```bash
docker build -t rust-sixtysix:dev .
```

Run:

```bash
docker run --rm -p 8080:8080 rust-sixtysix:dev
```

Pass a different listen port:

```bash
docker run --rm -e PORT=9090 -p 9090:9090 rust-sixtysix:dev
```

## Concepts

Engine pieces:

| Piece | Purpose |
|-------|---------|
| `engine::Game` | Rule set trait: `initial_state`, `validate`, `apply` |
| `engine::Engine` | Registers games, manages sessions, dispatches actions |
| `store::Store` | Persistence trait (memory impl provided) |
| `api::create_router` | Minimal HTTP adapter (serves JSON via axum) |

Card encoding: `suit*100 + rankValue` where suits: Clubs=0, Diamonds=1, Hearts=2, Spades=3; rank values: A=11, 10=10, K=4, Q=3, J=2, 9=0.

## HTTP API

Core endpoints:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/healthz` | Liveness probe |
| GET | `/games` | List registered games |
| POST | `/sessions?game=sixtysix&seed=SEED` | Create session |
| GET | `/sessions?game=sixtysix&offset=0&limit=20` | Page sessions |
| GET | `/sessions/{id}` | Fetch session (state snapshot) |
| POST | `/sessions/{id}` | Apply action `{type,payload}` |
| DELETE | `/sessions/{id}` | Delete session |

Schemas + examples: [docs/api.md](docs/api.md).

### Actions

| Type | Payload | Effect |
|------|---------|--------|
| `play` | `{card:int}` | Play a card; resolves trick after 2 plays |
| `closeStock` | - | Close stock: no further drawing; must follow suit |
| `declare` | `{suit:int}` | Marriage (K+Q) scoring (20 / 40 trump) at lead |
| `exchangeTrump` | - | Swap 9 of trump with upcard (while stock open, at lead) |

## Game Rules Summary

Short form (see [docs/rules.md](docs/rules.md) for detail):

1. 24-card deck (A 10 K Q J 9 in four suits). Deal 6 each (3+3), stock remainder, last card face-up = trump.
2. Leader plays any card when stock open; follower may play any card until stock closed or empty; then must follow suit if possible.
3. Trick winner: higher of suit led; trumps beat non-trumps.
4. Winner scores captured card values; first to 66 ends deal; +10 last trick bonus.
5. Marriage declaration at lead (holding K+Q) scores 20 (non-trump) or 40 (trump).
6. Trump 9 exchange allowed at lead while stock open.

## Frontend Integration Ideas

- Maintain local optimistic state while posting actions (server returns authoritative state version).
- Use the seed to recreate initial hands client-side for replay / spectator mode.
- Visual mapping for encoded cards: `suit = c/100`, `rankValue = c%100` -> show face; build a lookup table.
- Implement WebSocket push wrapper watching session updates for real-time UI (out of scope here, easy extension).

See [docs/integration.md](docs/integration.md) for architecture suggestions.

## Project Layout

```text
src/
  lib.rs           # Crate root, module exports
  game.rs          # Sixty-six game rules implementation
  engine.rs        # Core engine + session orchestration
  store/           # In-memory store (trait for alt backends)
    mod.rs
    memory.rs
  api/             # HTTP server wiring (axum)
    mod.rs
    server.rs
examples/
  server.rs        # Example executable (demo server)
docs/              # Extended docs (rules, API, integration)
```

---

Extended documents:

- [Detailed Rules](docs/rules.md)
- [API Guide](docs/api.md)
- [Integration Notes](docs/integration.md)

---

Future ideas: persistence backends, matchmaking service, WebSocket streaming, multi-deal match structure.

## Infrastructure Philosophy

This repository intentionally ships only:

- A minimal HTTP example (`examples/server.rs`)
- A single multi-stage `Dockerfile`
- An optional ergonomic `Makefile` (build/test/run/docker shortcuts)

Rationale: keep the core game engine small, dependency-light, and easy to embed.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities privately – process described in [SECURITY.md](SECURITY.md).

## Funding / Support

If you find this useful, see [FUNDING.md](FUNDING.md).

## License

MIT – see [LICENSE.md](LICENSE.md).
