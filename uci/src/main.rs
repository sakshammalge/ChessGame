//! The program a chess GUI starts and talks to over UCI.
//!
//! For now it only prints its name. Phase 2 adds the UCI command loop.

fn main() {
    println!("{} by {}", engine::full_name(), engine::AUTHOR);
}
