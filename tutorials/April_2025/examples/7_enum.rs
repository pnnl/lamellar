
// Enums provided a way to define a type by enumerating its possible values.
// Rust enums are similar to algebraic data types in functional languages, such as Haskell.
enum Cmd{
    Print, // no data associated with this variant
    Create(f64, f64), // two f64 values associated with this variant
    Add(Complex), // a Complex value associated with this variant
}

//We can also define methods for enums
impl Cmd{
    fn execute(&self, c: Complex) -> Complex {
        // Pattern matching on enums is a powerful feature of Rust, and is used in many places.
        // The rust compiler will emit an error if all possible values are not matched
        match self {
            Cmd::Print => {
                println!("Printing complex number: {}", c.print());
                c
            }
            Cmd::Create(re, im) => {
                println!("Creating new complex number: {} + {}i", re.print(), im.print());
                Complex{re: *re, im: *im}
            }
            Cmd::Add(other) => {
                println!("Adding complex numbers: {} + {}i and {} + {}i", c.re.print(), c.im.print(), other.re.print(), other.im.print());
                c + *other
            }
        }
    }
}

// Enums cans implement traits
impl MyPrint for Cmd {
    fn print(&self) -> String {
        match self {
            Cmd::Print => "Print".to_string(),
            Cmd::Create(re, im) => format!("Create({},{})", re.print(), im.print()),
            Cmd::Add(c) => format!("Add({} + {})", c.re.print(), c.im.print()),
        }
    }
}

// Rust enums can also be parameterized with generic types
enum Option<T> {
    Some(T), // This is a variant of the enum MyOption that takes a single generic type T (any type can be used here)
    None, // This would indicate that the enum is empty
}

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

//The Add trait has a single required function "add"
impl std::ops::Add for Complex {
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

    let some_complex = Option::Some(c);
    let some_f32 = Option::Some(3.14);
    let none_str: Option<String> = Option::None;

    match some_complex {
        Option::Some(c) => println!("Some value: {}", c.print()),
        Option::None => println!("No complex number"),
    }

    match none_str {
        Option::Some(c) => println!("Some value: {c:?}"),
        Option::None => println!("No complex number"),
    }

    if let Option::Some(c) = some_f32 {
        println!("Some complex number: {}", c.print());
    }
}