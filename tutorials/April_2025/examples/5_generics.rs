#[derive(Debug)]
struct Complex<T> {
    // T here captures a Generic type
    re: T, // we can specify that a struct member is of a given generic type
    im: T,
}

impl<T> Complex<T> {
    /// Creates a new complex number with the given real and imaginary parts.
    fn new(re: T, im: T) -> Self {
        //Struct generic types can be used within the implementations of structs
        Complex { re, im }
    }
}

#[derive(Debug)]
struct Point<A,B>{
    x: A,
    y: B
}

// we can also use generics in free standing functions
// Generic labels do not need to match the struct definitions
fn gen_complex_and_point<X,Y,Z>(re: Z, im: Z, x: X, y: Y) -> (Complex<Z>,Point<X,Y>) {
    (Complex::new(re, im),Point{x,y})
}


//We can use multiple generic types


fn main() {
    let data = gen_complex_and_point(0.5, 0.5,3usize,String::from("pi"));
    println!("data: {data:?}");
}
