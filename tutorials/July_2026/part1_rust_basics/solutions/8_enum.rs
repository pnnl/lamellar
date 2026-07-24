#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
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

impl MyPrint for f64 {
    fn print(&self) -> String {
        format!("{:.4}", self)
    }
}

enum Cmd {
    Print,
    Create(f64, f64),
    Add(Complex),
}

impl Cmd {
    fn execute(&self, c: Complex) -> Complex {
        match self {
            Cmd::Print => {
                println!("Printing complex number: {}", c.print());
                c
            }
            Cmd::Create(re, im) => {
                println!("Creating new complex number: {} + {}i", re.print(), im.print());
                Complex { re: *re, im: *im }
            }
            Cmd::Add(other) => {
                println!(
                    "Adding complex numbers: {} + {}i and {} + {}i",
                    c.re.print(),
                    c.im.print(),
                    other.re.print(),
                    other.im.print()
                );
                c + *other
            }
        }
    }
}

// Enums can also be parameterized with generic types
#[allow(dead_code)]
enum MyOption<T> {
    Some(T),
    None,
}

fn main() {
    let c = Complex { re: 1.0, im: 2.0 };
    let cmds = vec![
        Cmd::Print,
        Cmd::Create(3.0, 4.0),
        Cmd::Add(Complex { re: 5.0, im: 6.0 }),
    ];
    for cmd in cmds {
        cmd.execute(c);
    }

    let some_complex = MyOption::Some(c);
    match some_complex {
        MyOption::Some(c) => println!("Some value: {}", c.print()),
        MyOption::None => println!("No complex number"),
    }
}
