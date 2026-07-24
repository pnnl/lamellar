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
