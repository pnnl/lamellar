use lamellar::array::prelude::*;

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let len = num_pes * 4;

    let array = UnsafeArray::<usize>::new(world.team(), len, Distribution::Block).block();
    unsafe {
        array
            .dist_iter_mut()
            .enumerate()
            .for_each(move |(i, elem)| *elem = i)
            .block();
    }
    world.barrier();

    let atomic_array = array.into_atomic().block();
    atomic_array
        .dist_iter()
        .for_each(move |elem| {
            elem.fetch_add(my_pe);
        })
        .block();
    world.barrier();

    // into_read_only() consumes atomic_array and returns the new handle - bind it.
    let read_only_array = atomic_array.into_read_only().block();

    if my_pe == 0 {
        read_only_array
            .onesided_iter()
            .into_iter()
            .for_each(|elem| print!("{elem} "));
        println!();
    }
}
