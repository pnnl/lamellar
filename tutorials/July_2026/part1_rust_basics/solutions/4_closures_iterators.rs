fn main() {
    let data = vec![1, 2, 3, 4, 5];

    let sum_doubled: i32 = data.iter().map(|x| x * 2).sum();
    println!("sum_doubled: {sum_doubled}");

    let print_all = || {
        // borrow `data` instead of moving it, so `data` is still usable afterward
        for x in &data {
            println!("{x}");
        }
    };
    print_all();
    println!("data again: {data:?}");
}
