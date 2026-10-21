# ChessGame

A chess engine written from scratch in Rust, built step by step as a learning
project, with a web UI to come later.

- [Design plan and roadmap](docs/design.md)
- [Devlog](docs/devlog.md): what changed at each step
- [Concepts](docs/concepts/): notes on the ideas used at each step

## Building and running

Install Rust with [rustup](https://rustup.rs). The repo's
`rust-toolchain.toml` picks the right version automatically.

```text
cargo build --release                         # build everything, optimized
cargo run -p uci --release                    # run the engine (chessgame)
cargo test                                    # run all tests
cargo fmt && cargo clippy --all-targets       # format and lint
cargo run -p tools --bin speed-demo --release # debug vs release demo
```

Layout:

- `engine/`: the engine core library (board, movegen, search, eval). No I/O.
- `uci/`: the `chessgame` program that chess GUIs talk to over UCI.
- `tools/`: developer tools (perft, test suites, benchmarks).
- `docs/`: design, devlog and concept notes.
