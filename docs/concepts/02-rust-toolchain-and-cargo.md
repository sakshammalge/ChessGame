# The Rust Toolchain and Cargo

Used in: Phase 0 (setup). Everything later builds on this.

This note explains the tools we installed, how our repo is laid out as a Cargo
workspace, how tests work, and why debug and release builds behave so
differently. It assumes you know C, C++ or Python, and compares with them
where that helps.

## 1. The problem

A C++ project needs a compiler (gcc, clang), a build system (Make, CMake), a
way to get libraries (apt, vcpkg, Conan, or copying files around), a test
framework (GoogleTest, Catch2), a formatter (clang-format) and a linter
(clang-tidy). Each is a separate tool with its own config, and two people's
machines rarely match. Python has a similar story with pip, venv, pytest,
black and flake8.

For our engine that would mean: lots of setup before the first line of chess
code, "works on my machine" bugs, and speed numbers that aren't comparable
because people built with different flags.

## 2. The idea in plain words

Rust ships **one official set of tools** that all work together:

| Tool | What it does | C/C++ or Python equivalent |
|---|---|---|
| `rustup` | Installs and switches Rust versions | Installing gcc by hand, `pyenv` |
| `rustc` | The compiler | `gcc`, `clang` |
| `cargo` | Build tool, package manager, test runner | CMake + vcpkg + ctest, or pip + pytest |
| `rustfmt` (`cargo fmt`) | Formats code in one standard style | clang-format, black |
| `clippy` (`cargo clippy`) | Linter with hundreds of checks | clang-tidy, flake8/pylint |

You almost never call `rustc` directly. You call `cargo`, and it calls `rustc`
with the right flags.

Some words you'll see everywhere:

- **Crate**: one Rust package, which is the unit the compiler builds. A crate
  is either a **library** (code others use, starting at `src/lib.rs`) or a
  **binary** (a program with a `main`, starting at `src/main.rs`). Think of a
  library crate as a `.a`/`.so` and a binary crate as an executable.
- **`Cargo.toml`**: the file that describes a crate: its name, version,
  dependencies and settings. Like `CMakeLists.txt` or `pyproject.toml`, but
  much shorter.
- **Workspace**: several crates in one repo that build together, share one
  `target/` folder (where the build output goes) and one `Cargo.lock` (the
  exact versions of every dependency).
- **Module**: a namespace inside a crate. Each file is a module. In C you'd
  have `board.h` + `board.c`; in Rust you write `mod board;` in `lib.rs` and
  put the code in `board.rs`. There are no header files: the compiler reads
  the whole crate at once.

## 3. Worked example: our workspace

### 3.1 Installing Rust

On your own machine, install `rustup` from https://rustup.rs:

```text
# Linux / macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows: download and run rustup-init.exe from https://rustup.rs
# (it will ask you to install the Visual Studio C++ build tools first)
```

Then open a new terminal and check:

```text
$ rustc --version
rustc 1.97.0 (...)
$ cargo --version
cargo 1.97.0 (...)
```

You don't need to pick a version yourself. Our repo has a
`rust-toolchain.toml` file:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

The first time you run `cargo` inside the repo, `rustup` reads this file and
installs the stable compiler plus the formatter and linter if they're missing.
Everyone, including CI, uses the same tools.

For your editor, install **rust-analyzer** (the VS Code extension is called
"rust-analyzer"). It gives you autocomplete, go-to-definition, inline type
hints and errors as you type. It makes learning Rust much easier, because you
can hover over anything to see its type.

### 3.2 The layout

```
ChessGame/
├── Cargo.toml            # the workspace: lists the member crates
├── Cargo.lock            # exact dependency versions (generated, committed)
├── rust-toolchain.toml   # which Rust version to use
├── engine/               # library crate: the chess brain, no I/O
│   ├── Cargo.toml
│   └── src/lib.rs
├── uci/                  # binary crate: the program a GUI runs
│   ├── Cargo.toml
│   └── src/main.rs
├── tools/                # library + several small programs for us
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs        # code shared by the tools
│       └── bin/
│           └── speed-demo.rs
└── target/               # build output (ignored by git)
```

How the crates depend on each other:

```
   uci (binary: chessgame)      tools (perft, benchmarks, ...)
            \                       /
             \                     /
              v                   v
                engine (library)
```

`engine` depends on nothing. That's deliberate: the engine core must not know
whether it's being driven by a GUI over UCI, by a test tool, or (in phase 10)
by a web page. Keeping I/O out of it is what lets one engine serve all three.

### 3.3 The workspace `Cargo.toml`

```toml
[workspace]
members = ["engine", "uci", "tools"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
...

[workspace.dependencies]
engine = { path = "engine" }
```

- `members` lists the crates. `cargo build` at the top builds all of them.
- `resolver = "3"` picks the newest rules Cargo uses to choose dependency
  versions. You can ignore it.
- `[workspace.package]` holds settings shared by all members. Each member
  writes `version.workspace = true` to inherit them, so we bump the version in
  one place.
- `edition = "2024"`: Rust editions are like C++ standards (`-std=c++20`).
  New editions can change a few language rules, but crates of different
  editions still work together. 2024 is the newest.
- `[workspace.dependencies]` declares that "engine" lives in the `engine/`
  folder. A member then just writes `engine = { workspace = true }`.

A member's `Cargo.toml`, e.g. `uci/Cargo.toml`:

```toml
[package]
name = "uci"
version.workspace = true
edition.workspace = true

[[bin]]
name = "chessgame"
path = "src/main.rs"

[dependencies]
engine = { workspace = true }
```

The `[[bin]]` section renames the executable to `chessgame` (the crate is
still called `uci`). The double brackets mean "one entry in a list of
binaries": a crate can have several.

### 3.4 Our first Rust code, line by line

`engine/src/lib.rs`:

```rust
//! The chess engine core.

/// The engine's name, as reported to chess GUIs over UCI.
pub const NAME: &str = "ChessGame";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn full_name() -> String {
    format!("{NAME} {VERSION}")
}
```

- `//!` is a doc comment for the whole file (module), and `///` is a doc
  comment for the item below it. `cargo doc --open` turns them into a
  website. Plain `//` comments are like C's.
- `pub` means "visible outside this module". Everything is **private by
  default**, the opposite of C where every non-`static` function is visible.
- `const NAME: &str = ...`: a compile-time constant. The type comes after the
  name with a colon (`NAME: &str`), the reverse of C's `const char* NAME`.
- `&str` is a borrowed, read-only string slice: a pointer plus a length,
  pointing at text that lives somewhere else (here, baked into the binary).
  Close to C's `const char*`, but it knows its length and is always valid
  UTF-8.
- `String` is an owned, growable string on the heap, like `std::string`.
  `full_name` builds a new string, so it returns `String`, not `&str`.
- `fn full_name() -> String`: a function with no arguments returning a
  `String`. The return type goes after `->`.
- The function body has no `return` and no semicolon on the last line. In
  Rust almost everything is an expression, and a block's value is its last
  expression. `return x;` also works, but the short form is idiomatic.
- `format!(...)` and `env!(...)` end in `!`, which means they are **macros**,
  not functions. `format!` is like Python's f-strings: `{NAME}` inserts the
  value of `NAME`. `env!("CARGO_PKG_VERSION")` reads an environment variable
  **at compile time**; Cargo sets it to the version from `Cargo.toml`.

`uci/src/main.rs`:

```rust
fn main() {
    println!("{} by {}", engine::full_name(), engine::AUTHOR);
}
```

`engine::full_name()` calls a function from the `engine` crate. `::` is the
path separator, like C++'s `std::`. We didn't need an `#include`: listing
`engine` under `[dependencies]` is enough.

`tools/src/lib.rs`, a loop with mutable variables:

```rust
pub fn count_bits_slow(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        count += (x & 1) as u32;
        x >>= 1;
    }
    count
}
```

- `let` declares a variable. Variables are **immutable by default**; `mut`
  makes them changeable. Even the parameter `x` needs `mut` because we shift
  it. This default catches accidental changes.
- `u64`, `u32`: unsigned integers with an exact size, like C's `uint64_t` and
  `uint32_t`. Rust also has `i8` ... `i128`, `usize` (pointer-sized, used for
  indexing), `f32`/`f64`, and `bool`.
- `count` has no written type. The compiler **infers** it: it's added to a
  `u32` and returned as `u32`, so it must be `u32`.
- `as u32` is an explicit cast. Rust never converts between number types
  silently, unlike C. `x & 1` is a `u64`, so it must be cast to add it to a
  `u32`.
- Integer overflow is checked in debug builds: if `count` overflowed, the
  program would stop with an error instead of silently wrapping around as C
  does. (In release builds it wraps, for speed.)

### 3.5 Tests

Tests live right next to the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_bits() {
        assert_eq!(count_bits_slow(0b1011), 3);
        assert_eq!(count_bits_slow(0x0000_0000_0000_FF00), 8); // white pawns at the start
    }
}
```

- `#[...]` is an **attribute**, extra information for the compiler.
  `#[cfg(test)]` means "only compile this module when testing", so tests add
  nothing to the real program. `#[test]` marks a function as a test.
- `mod tests { ... }` declares a module inline, inside the same file.
- `use super::*;` imports everything from the parent module, so the test can
  call `count_bits_slow` directly. `super` means "the module above".
- `assert_eq!(a, b)` fails the test if `a != b`, and prints both values.
- `0b1011` is binary, `0x...` is hex, and `_` can separate digits for
  readability (`0x0000_0000_0000_FF00`). Handy for bitboards.

Run them:

```text
$ cargo test
running 2 tests
test tests::full_name_includes_version ... ok
test tests::version_matches_cargo_toml ... ok
test result: ok. 2 passed; 0 failed; ...
```

`cargo test` builds each crate in test mode and runs every `#[test]`
function. Useful variants:

```text
cargo test -p engine           # only the engine crate
cargo test counts_bits         # only tests whose name contains "counts_bits"
cargo test -- --nocapture      # show println! output from tests
```

### 3.6 Everyday commands

```text
cargo build                     # debug build of everything
cargo build --release           # optimized build
cargo run -p uci                # build and run the chessgame binary
cargo run -p tools --bin speed-demo --release
cargo test                      # run all tests
cargo fmt                       # format all code
cargo clippy --all-targets      # lint all code, including tests
cargo doc --open                # build docs from /// comments and open them
cargo check                     # type-check only, no binary: the fastest feedback
```

`-p <crate>` picks one crate in the workspace. `--bin <name>` picks one
program in a crate that has several.

## 4. How it looks in our code: debug vs release

`cargo build` makes a **debug build**: no optimizations, extra safety checks
(like overflow checks), full debug info. It compiles fast and is easy to debug,
but runs slowly. `cargo build --release` turns on the optimizer. For an engine
that difference is huge, so **we always measure speed in release mode**.

We also tuned the release profile in the workspace `Cargo.toml`:

```toml
[profile.release]
codegen-units = 1
lto = true
panic = "abort"
```

- `codegen-units = 1`: by default Cargo splits a crate into 16 pieces and
  compiles them in parallel, which is faster to build but hides some
  optimizations. One unit lets the optimizer see everything at once.
- `lto = true`: **link-time optimization**. Normally each crate is optimized
  on its own, so a tiny engine function called from `uci` can't be inlined
  across the crate boundary. LTO optimizes the whole program together, like
  `-flto` in gcc/clang.
- `panic = "abort"`: a **panic** is Rust's "something impossible happened"
  error (like an index out of bounds). By default Rust "unwinds" the stack
  like a C++ exception, which needs extra code. Aborting is simpler and
  slightly faster, and an engine has nothing to recover anyway.

These settings make release builds slower to compile, but the engine faster
to run.

The `tools/src/bin/speed-demo.rs` program shows the difference. It counts the
1 bits in 20 million pseudo-random numbers with our slow loop:

```rust
total += count_bits_slow(black_box(x)) as u64;
```

`std::hint::black_box` tells the optimizer "pretend you don't know this
value". Without it, a clever optimizer might work out the answer at compile
time and our measurement would show zero work. Benchmarks need this trick.

## 5. Pitfalls

- **Measuring speed in a debug build.** The most common beginner mistake.
  Always add `--release` before you trust a speed number.
- **Forgetting `--release` makes perft look broken.** In phase 1, deep perft
  in debug mode may take minutes. That's normal.
- **Everything is private by default.** If you get "function is private",
  add `pub`.
- **No implicit number conversions.** `let a: u32 = 5; let b: u64 = a;` is an
  error. Write `a as u64` (or `u64::from(a)`, which only compiles when it can't
  lose information).
- **Integer overflow behaves differently in debug and release.** Debug stops
  with "attempt to add with overflow", release silently wraps. When you want
  wrapping on purpose (we will, for Zobrist hashing), say so with
  `wrapping_add`, `wrapping_mul`, and so on.
- **Committing `target/`.** It's huge and machine-specific, so `.gitignore`
  excludes it. `Cargo.lock` should be committed for programs like ours, so
  everyone builds with the exact same dependency versions.
- **Fighting the formatter.** Don't format by hand. Run `cargo fmt` before
  committing; CI rejects unformatted code.

## 6. Measured effect

`speed-demo`, 20 million numbers, 4-core cloud machine, two runs each:

| Build | Time |
|---|---|
| debug (`cargo run`) | 3.96 s, 3.89 s |
| release (`cargo run --release`) | 0.68 s, 0.72 s |

So the release build is about **5.5 times faster** on this tiny loop. Real
engine code, with lots of small functions to inline and array indexing to
check, often gains 10 to 50 times.

The release binary is also much smaller: the `chessgame` binary is 4.4 MB in
debug and 0.38 MB in release.

## 7. Further reading

- The Rust Book, chapters 1 (getting started), 3 (common concepts) and 11
  (tests): https://doc.rust-lang.org/book/
- The Cargo Book, workspaces and profiles:
  https://doc.rust-lang.org/cargo/reference/workspaces.html and
  https://doc.rust-lang.org/cargo/reference/profiles.html
- Rust for C++ programmers, a short comparison:
  https://github.com/nrc/r4cppp
- Rust by Example, for small runnable snippets:
  https://doc.rust-lang.org/rust-by-example/
- Clippy's list of lints: https://rust-lang.github.io/rust-clippy/
