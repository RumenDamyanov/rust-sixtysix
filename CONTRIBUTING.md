# Contributing

Thanks for your interest in improving rust-sixtysix!

## Ways to Help

- Report bugs (include reproduction & environment)
- Propose enhancements (outline use case + minimal API surface)
- Improve docs (clarity, examples, spelling)
- Add tests (edge cases, rule enforcement scenarios)

## Development Setup

```bash
git clone https://github.com/rumendamyanov/rust-sixtysix
cd rust-sixtysix
cargo test
```

Run example server:

```bash
cargo run --example server
```

## Coding Guidelines

- Rust 2021 edition.
- Keep public API small & focused.
- Avoid adding external deps without strong justification.
- Write tests for new rule logic or engine behavior.
- Run `cargo fmt` and `cargo clippy` before submitting.

## Commit Style

Conventional-ish, but pragmatic (e.g. `engine: enforce follow suit after close`).

## Pull Requests

1. Open issue (optional for small fixes) describing change.
2. Fork & branch (`feature/short-description`).
3. Add tests (or rationale if not testable).
4. Ensure `cargo test` passes and `cargo clippy` is clean.
5. Submit PR referencing issue.

## Code of Conduct

Participation governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

MIT – see [LICENSE.md](LICENSE.md).
