fn owned_vec(mut data: Vec<u32>) -> Vec<u32> {
    data.push(10);
    println!("owned vec {data:?} len: {}", data.len());
    data
}

fn borrowed_vec(data: &mut Vec<u32>) {
    data.push(10);
    println!("borrow vec {data:?} len {}", data.len());
}

fn slice(data: &mut [u32]) {
    data.iter_mut().for_each(|elem| *elem *= 2);
    println!("slice {data:?} len {}", data.len());
}

fn main() {
    let mut data = vec![1, 2, 3, 4, 5];
    data = owned_vec(data);
    borrowed_vec(&mut data);
    slice(&mut data);
    println!("data {data:?} len: {}", data.len())
}
