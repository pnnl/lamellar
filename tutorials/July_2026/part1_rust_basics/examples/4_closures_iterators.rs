// Closures and iterators: the same "chain of adapters, then act on each element"
// pattern used constantly for distributed arrays in part 2, e.g.
//   array.dist_iter_mut().enumerate().for_each(|(i, elem)| ...)

fn main() {
    let data = vec![1, 2, 3, 4, 5];

    let sum_doubled: i32 = data.iter().map(|x| x * 2).sum();
    println!("sum_doubled: {sum_doubled}");

    let print_all = move || {
        // `move` forces this closure to take ownership of `data`
        for x in &data {
            println!("{x}");
        }
    };
    print_all();
    println!("data again: {data:?}"); // error: `data` was moved into `print_all`
}
