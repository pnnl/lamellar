// Enums provide a way to define a type by enumerating its possible values.
// Rust enums are similar to algebraic data types in functional languages, such as Haskell.
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
    Print,             // no data associated with this variant
    Create(f64, f64),  // two f64 values associated with this variant
    Add(Complex),      // a Complex value associated with this variant
}

impl Cmd {
    fn execute(&self, c: Complex) -> Complex {
        // Pattern matching on enums is a powerful feature of Rust.
        // The compiler enforces that all possible values are matched.
        match self {
            Cmd::Print => {
                println!("Printing complex number: {}", c.print());
                c
            }
            Cmd::Create(re, im) => {
                println!("Creating new complex number: {} + {}i", re.print(), im.print());
                // error: missing arm construction below
            }
        }
    }
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
}
