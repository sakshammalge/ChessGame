//! Shows how much faster a release build is than a debug build.
//!
//! Run it both ways and compare:
//!
//! ```text
//! cargo run -p tools --bin speed-demo
//! cargo run -p tools --bin speed-demo --release
//! ```

use std::hint::black_box;
use std::time::Instant;

use tools::count_bits_slow;

fn main() {
    const N: u64 = 20_000_000;

    let start = Instant::now();
    let mut total: u64 = 0;
    // A simple pseudo-random sequence (xorshift) so the numbers vary.
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..N {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        // `black_box` stops the optimizer from computing the answer at
        // compile time, so we measure real work.
        total += count_bits_slow(black_box(x)) as u64;
    }
    let elapsed = start.elapsed();

    let mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    println!("{} ({mode} build)", engine::full_name());
    println!("counted {total} bits in {N} numbers in {elapsed:.2?}");
}
