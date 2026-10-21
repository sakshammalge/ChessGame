# Devlog

One entry per step: what changed, which files, why, and the result. Newest at
the bottom.

## Step 0: Design and planning (2026-10-21)

**What:** Agreed on the overall design and roadmap. No code yet.

**Decisions:**
- Engine in Rust, learning as we go.
- UCI engine first, web UI (WASM + TypeScript) later.
- Aim for maximum strength, built phase by phase, with each optimization
  measured.
- Document every step in this devlog and in `docs/concepts/`.

**Files:**
- `docs/design.md`: design plan and roadmap.
- `docs/devlog.md`: this file.
- `docs/concepts/00-glossary.md`: plain-language definitions of every term in
  the plan.
- `docs/concepts/01-how-chess-engines-work.md`: overview of Elo and the three
  jobs of an engine, with worked examples.
- `README.md`: project description and links to the docs.

**Concepts:** [Glossary](concepts/00-glossary.md),
[How chess engines work](concepts/01-how-chess-engines-work.md)

**Next:** Phase 0, setting up the Rust workspace.

## Step 1: Phase 0, Rust workspace and CI (2026-10-21)

**What:** Set up the Rust toolchain, the Cargo workspace with its three
crates, and GitHub Actions CI. No chess yet: the engine only knows its name,
but every crate builds, has tests where there's something to test, and is
checked on every PR.

**Files:**
- `rust-toolchain.toml`: pins the stable compiler plus `rustfmt` and `clippy`.
- `Cargo.toml`: the workspace (members, shared metadata, release profile with
  LTO, one codegen unit and `panic = "abort"`).
- `Cargo.lock`: exact dependency versions (none yet besides our own crates).
- `engine/`: library crate for the engine core. Exposes `NAME`, `VERSION`,
  `AUTHOR` and `full_name()`, with two unit tests.
- `uci/`: binary crate building the `chessgame` executable. Prints the engine
  name for now; the UCI loop comes in phase 2.
- `tools/`: developer tools. `count_bits_slow()` with tests, and the
  `speed-demo` program comparing debug and release builds.
- `.github/workflows/ci.yml`: formatting check, Clippy with warnings as
  errors, tests in debug and release.
- `.gitignore`: ignores `target/`.
- `README.md`: how to build, run and test.

**Why:**
- Splitting `engine` from `uci` keeps all I/O out of the engine, so the same
  core can later run in the web UI.
- CI catches broken changes before they're merged. Testing in both debug and
  release catches overflow bugs (debug) and optimizer-dependent bugs (release).
- The release profile settings give the fastest runtime code, at the cost of
  slower release compiles.

**Result:** `speed-demo` (20 million numbers): debug 3.9 s, release 0.7 s,
about 5.5 times faster. The `chessgame` binary is 4.4 MB in debug and 0.38 MB
in release. All 4 tests pass; `cargo fmt --check` and Clippy are clean.

**Concepts:** [The Rust toolchain and Cargo](concepts/02-rust-toolchain-and-cargo.md),
[Continuous integration](concepts/03-continuous-integration.md), plus new
glossary entries (rustup, `Cargo.toml`, edition, macro, panic, LTO, rustfmt,
rust-analyzer, GitHub Actions).

**Next:** Phase 1, the rules of chess: squares, pieces, bitboards and FEN.
