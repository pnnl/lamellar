// Two common enums in rust are Option and Result
// Option is used to represent the possibility of a value being present or absent
// Result is used to represent the possibility of an operation failing
//
// enum Option<T> { Some(T), None }
// enum Result<T, E> { Ok(T), Err(E) }

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
}

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

fn process_nums_unwrap(nums: &[Complex]) {
    let max = max_real(nums).unwrap(); // panics if max_real returns None
    println!("Max real: {max:?}");
    let sum = sum_if_all_negative(nums).unwrap(); // panics if sum_if_all_negative returns Err
    println!("Sum: {sum:?}");
}

fn main() {
    let nums_0 = vec![
        Complex { re: 1.0, im: 2.0 },
        Complex { re: 3.0, im: 4.0 },
    ];
    let nums_2: Vec<Complex> = vec![];

    process_nums_unwrap(&nums_0);
    process_nums_unwrap(&nums_2); // this panics: empty slice -> None -> unwrap panic
}
