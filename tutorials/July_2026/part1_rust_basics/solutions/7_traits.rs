use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

// For traits with generic parameters it is possible to have multiple implementations
// of the trait for unique instances of the generic parameter
impl Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        Complex {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }
}

trait MyPrint {
    fn print(&self) -> String;
}

impl MyPrint for Complex {
    fn print(&self) -> String {
        format!("{} + {}i", self.re.print(), self.im.print())
    }
}

// can implement our trait for external types too
impl MyPrint for f64 {
    fn print(&self) -> String {
        format!("{:.4}", self)
    }
}

fn compare_outputs<T: std::fmt::Debug + MyPrint>(obj: T) {
    println!("Debug: {obj:?}");
    println!("MyPrint: {}", obj.print());
}

fn main() {
    let num_0 = Complex::new(0.5, 0.5);
    let num_1 = Complex::new(-0.5, 0.5);
    println!("{num_0:?} + {num_1:?} = {:?}", num_0 + num_1);
    println!("{num_0:?} * {num_1:?} = {:?}", num_0 * num_1);
    println!("{num_0:?} * 2 = {:?}", num_0 * 2.0);
    compare_outputs(num_0);
    compare_outputs(2.0);
}
