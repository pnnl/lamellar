//Two common enums in rust are Option and Result
// Option is used to represent the possibility of a value being present or absent
// Result is used to represent the possibility of an operation failing
// These are both used extensively for error handling in Rust

// they take the form of:
// enum Option<T> {
//     Some(T),
//     None,
// }
//
// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }
//



// It is common to return an Option from a function that may produce an empty value
fn max_real(nums: &[Complex]) -> Option<&Complex> {
    if nums.is_empty() {
        return None;
    }
    let mut max = &nums[0];
    for num in nums.iter() {
        if num.re > max.re {
            max = num;
        }
    }
    Some(max)
    // in reality we would probably do something like this:
    // nums.iter().max_by_key(|num| num.re)
}

// It is common to return a Result from a function that may fail
fn sum_if_all_negative(nums: &[Complex]) -> Result<Complex, &'static str> {
    for num in nums.iter() {
        if num.re > 0.0 || num.im > 0.0 {
            return Err("All numbers must be negative");
        }
    }
    let mut sum = Complex { re: 0.0, im: 0.0 };
    for num in nums.iter() {
        sum = sum + *num;
    }
    Ok(sum)
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

fn process_nums_unwrap(nums: &[Complex]) {
    let max = max_real(nums).unwrap(); // this will panic if max_real returns None
    println!("Max real: {:?}", max);
    let sum = sum_if_all_negative(nums).unwrap(); // this will panic if sum_if_all_negative returns Err
    println!("Sum: {:?}", sum);
}

fn process_nums_expect(nums: &[Complex]) {
    let max = max_real(nums).expect("expected at least one number");
    println!("Max real: {:?}", max);
    let sum = sum_if_all_negative(nums).expect("Expected all numbers to be positive");
    println!("Sum: {:?}", sum);
}

fn process_nums_match(nums: &[Complex]) {
    match max_real(nums) {
        Some(max) => println!("Max real: {:?}", max),
        None => println!("No numbers"),
    }
    match sum_if_all_negative(nums) {
        Ok(sum) => println!("Sum: {:?}", sum),
        Err(e) => println!("Error: {}", e),
    }
}

fn process_nums_short_circuit(nums: &[Complex]) -> Result<(), &'static str> {
    let max = max_real(nums).ok_or("No numbers")?;
    println!("Max real: {:?}", max);
    let sum = sum_if_all_negative(nums)?;
    println!("Sum: {:?}", sum);
    Ok(())
}

fn main(){
    let nums_0 = vec![
        Complex { re: 1.0, im: 2.0 },
        Complex { re: 3.0, im: 4.0 },
        Complex { re: 5.0, im: 6.0 },
    ];
   
    let nums_1 = vec![
        Complex { re: -1.0, im: -2.0 },
        Complex { re: -3.0, im: -4.0 },
        Complex { re: -5.0, im: -6.0 },
    ];
    let nums_2 = vec![];
    

    // run and then comment out each of the following lines to see the different error handling strategies
    process_nums_unwrap(&nums_0);
    process_nums_unwrap(&nums_1);
    process_nums_unwrap(&nums_2); // this will panic

    process_nums_expect(&nums_0);
    process_nums_expect(&nums_1);
    process_nums_expect(&nums_2); // this will panic

    process_nums_match(&nums_0);
    process_nums_match(&nums_1);
    process_nums_match(&nums_2);

    process_nums_short_circuit(&nums_0);
    match process_nums_short_circuit(&nums_1){
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
    let _ = process_nums_short_circuit(&nums_2);


   
}