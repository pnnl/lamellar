// Part 2, Section 3: Distributed Arrays basics.
// Intentional bug: array created but never distributed-iterated correctly (see BUG below).
use lamellar::array::prelude::*;

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    let num_pes = world.num_pes();
    let global_len = num_pes * 4; // 4 elements per PE

    let array =
        AtomicArray::<usize>::new(world.team(), global_len, Distribution::Block).block();

    // each PE increments every element it owns by 1
    array
        .dist_iter_mut()
        .enumerate()
        .for_each(move |(i, elem)| {
            elem.store(i);
        })
        .block();

    world.barrier();

    // BUG: onesided_iter() collects the whole array to PE 0 (data movement!) - fine for
    // small arrays/debugging, but this runs on every PE here, which is wasteful and prints
    // the same full array num_pes times. Guard with `if world.my_pe() == 0` instead.
    array
        .onesided_iter()
        .into_iter()
        .for_each(|elem| print!("{elem} "));
    println!();
    world.barrier();

    array.print(); // show each PE's local slice of the array
}
