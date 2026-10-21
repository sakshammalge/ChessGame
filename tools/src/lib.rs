//! Shared code for the developer tools in `src/bin/`.
//!
//! Each file in `src/bin/` becomes its own program, run with
//! `cargo run -p tools --bin <name>`.

/// Counts the 1 bits in `x` one bit at a time.
///
/// This is deliberately the slow way, so the `speed-demo` tool has some work
/// to do. Real code uses `x.count_ones()`, which compiles to a single CPU
/// instruction (`popcnt`) on modern machines.
pub fn count_bits_slow(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        count += (x & 1) as u32;
        x >>= 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_bits() {
        assert_eq!(count_bits_slow(0), 0);
        assert_eq!(count_bits_slow(0b1011), 3);
        assert_eq!(count_bits_slow(0x0000_0000_0000_FF00), 8); // white pawns at the start
        assert_eq!(count_bits_slow(u64::MAX), 64);
    }

    #[test]
    fn matches_count_ones() {
        for x in [1u64, 7, 0xDEAD_BEEF, 1 << 63, 0x8040_2010_0804_0201] {
            assert_eq!(count_bits_slow(x), x.count_ones());
        }
    }
}
