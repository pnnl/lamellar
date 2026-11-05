//! Celerity - A test crate for benchmarking lamellar-runtime functions using iai and criterion.
//! 
//! This crate provides comprehensive benchmarks for the lamellar-runtime API
//! using the iai benchmarking framework for instruction-level analysis and criterion
//! for statistical benchmarking.

pub use lamellar;

/// Simple test function to verify the library structure
pub fn test_function() -> usize {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        assert_eq!(test_function(), 42);
    }
}
