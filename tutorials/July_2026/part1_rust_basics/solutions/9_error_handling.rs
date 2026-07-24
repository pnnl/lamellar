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

fn process_nums_match(nums: &[Complex]) {
    match max_real(nums) {
        Some(max) => println!("Max real: {max:?}"),
        None => println!("No numbers"),
    }
    match sum_if_all_negative(nums) {
        Ok(sum) => println!("Sum: {sum:?}"),
        Err(e) => println!("Error: {e}"),
    }
}

fn process_nums_short_circuit(nums: &[Complex]) -> Result<(), &'static str> {
    let max = max_real(nums).ok_or("No numbers")?;
    println!("Max real: {max:?}");
    let sum = sum_if_all_negative(nums)?;
    println!("Sum: {sum:?}");
    Ok(())
}

fn main() {
    let nums_0 = vec![
        Complex { re: 1.0, im: 2.0 },
        Complex { re: 3.0, im: 4.0 },
    ];
    let nums_2: Vec<Complex> = vec![];

    // avoid unwrap/expect on inputs that might be empty; match or `?` handle it safely
    process_nums_match(&nums_0);
    process_nums_match(&nums_2);

    match process_nums_short_circuit(&nums_0) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {e}"),
    }
    let _ = process_nums_short_circuit(&nums_2);
}
