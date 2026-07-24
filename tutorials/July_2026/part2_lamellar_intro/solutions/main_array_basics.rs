use lamellar::array::prelude::*;

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    let global_len = num_pes * 4; // 4 elements per PE

    let array =
        AtomicArray::<usize>::new(world.team(), global_len, Distribution::Block).block();

    array
        .dist_iter_mut()
        .enumerate()
        .for_each(move |(i, elem)| {
            elem.store(i);
        })
        .block();

    world.barrier();

    // only PE 0 collects + prints the whole array (onesided_iter moves data to PE 0)
    if my_pe == 0 {
        array
            .onesided_iter()
            .into_iter()
            .for_each(|elem| print!("{elem} "));
        println!();
    }
    world.barrier();

    array.print(); // each PE prints only its own local slice
}
