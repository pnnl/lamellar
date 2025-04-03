/// A struct representing a complex number.
/// "derive" is a commonly used Rust Macro useful for automatically generating implementations of the specified traits.
/// Traits can be thought of as interfaces in other languages
/// Debug allows our data type to be printed with the debug formatter i.e. println("{:?}")
/// Copy represents a simple inexpensive bitwise copy of the data structure
/// Clone represents a duplication of data strucutre that may or may not be expensive (e.g. reference counted pointers)
/// Not all data structures that implement Clone implement Copy, but all data structures that implement Copy can implment Clone.
/// For this data stucture both implementations are the exact same (bitwise copies).
#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    /// Creates a new complex number with the given real and imaginary parts.
    fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    /// Computes the magnitude of the complex number.
    fn magnitude(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

fn main() {
    let num = Complex::new(0.5, 0.5);
    let magnitude = num.magnitude();

    println!("Num: {num:?} mag: {magnitude}");
}
