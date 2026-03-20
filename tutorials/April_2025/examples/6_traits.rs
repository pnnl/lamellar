//import modules holding some trait definitions from the standard library
use std::ops::{Add, Mul};

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
}



//The Add trait has a single required function "add"
impl Add for Complex {

    type Output = Self; // this is a trait "associated type" another type of generic within Rust
                        // in this example we are specifying a generic type called "Output" and setting each to "Self" which is data structure this trait is being implemented for

    fn add(self, other: Self) -> Self::Output {
        //The associated type is used to specify the return of the add function
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

// Traits may also have generic parameters assocated with them, in this case the generic parameter is implicit i.e. Mul<rhs=Self>
impl Mul for Complex { // this is syntatical sugar for `impl Mul<rhs=Complex> for Complex`
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

// For traits with generic parameters it is possible to have multiple implemntations of the trait for unique instances of the generic parameter
impl Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        Complex {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }
}

// We can also define our own traits
trait MyPrint {
    fn print(&self) -> String;
}

//can implement for our own types
impl MyPrint for Complex {
    fn print(&self) -> String {
        format!("{} + {}i", self.re.print(), self.im.print())
    }
}
// can implement for external types
impl MyPrint for f64 {
    fn print(&self) -> String {
        //only show four decimal places 
        format!("{:.4}", self)
    }
}

// Generic and Traits commonly are used together in the form of "Trait Bounds"
// in this function we are specifying that the generic type T must implement both the Debug and MyPrint traits
fn compare_outputs<T: std::fmt::Debug + MyPrint>(obj: T){
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
