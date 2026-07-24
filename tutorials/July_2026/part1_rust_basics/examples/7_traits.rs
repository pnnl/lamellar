//import modules holding some trait definitions from the standard library
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

//The Add trait has a single required function "add"
impl Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

// Traits may also have generic parameters associated with them
impl Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

// We can also define our own traits
trait MyPrint {
    fn print(&self) -> String;
}

impl MyPrint for Complex {
    fn print(&self) -> String {
        format!("{} + {}i", self.re.print(), self.im.print())
    }
}

// Generics and Traits commonly are used together in the form of "Trait Bounds"
fn compare_outputs<T: std::fmt::Debug + MyPrint>(obj: T) {
    println!("Debug: {obj:?}");
    println!("MyPrint: {}", obj.print());
}

fn main() {
    let num_0 = Complex::new(0.5, 0.5);
    let num_1 = Complex::new(-0.5, 0.5);
    println!("{num_0:?} + {num_1:?} = {:?}", num_0 + num_1);
    println!("{num_0:?} * {num_1:?} = {:?}", num_0 * num_1);
    println!("{num_0:?} * 2 = {:?}", num_0 * 2.0); // error: no `Mul<f64>` impl for Complex
    compare_outputs(num_0);
    compare_outputs(2.0); // error: f64 does not implement MyPrint
}
