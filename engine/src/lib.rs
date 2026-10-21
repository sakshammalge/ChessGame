//! The chess engine core.
//!
//! This crate holds everything that thinks about chess: the board, move
//! generation, search and evaluation. It does no I/O (no printing, no reading
//! stdin), so the UCI program, the tools and later the web UI can all use it.
//!
//! Right now it only knows its own name. Phase 1 adds the board.

/// The engine's name, as reported to chess GUIs over UCI.
pub const NAME: &str = "ChessGame";

/// The engine's version, read from `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The author line reported to chess GUIs over UCI.
pub const AUTHOR: &str = "Saksham Malge";

/// Returns the full name and version, e.g. `"ChessGame 0.1.0"`.
pub fn full_name() -> String {
    format!("{NAME} {VERSION}")
}

// Unit tests live next to the code they test. `#[cfg(test)]` means this
// module is only compiled for `cargo test`, not for normal builds.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_name_includes_version() {
        assert_eq!(full_name(), format!("ChessGame {VERSION}"));
    }

    #[test]
    fn version_matches_cargo_toml() {
        assert_eq!(VERSION, "0.1.0");
    }
}
