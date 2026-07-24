#[derive(Debug)]
struct Complex<T> {
    re: T,
    im: T,
}

impl<T> Complex<T> {
    fn new(re: T, im: T) -> Self {
        Complex { re, im }
    }
}

#[derive(Debug)]
struct Point<A, B> {
    x: A,
    y: B,
}

fn gen_complex_and_point<X, Y, Z>(re: Z, im: Z, x: X, y: Y) -> (Complex<Z>, Point<X, Y>) {
    (Complex::new(re, im), Point { x, y })
}

fn main() {
    let data = gen_complex_and_point(0.5, 0.5, 3usize, String::from("pi"));
    println!("data: {data:?}");
}
