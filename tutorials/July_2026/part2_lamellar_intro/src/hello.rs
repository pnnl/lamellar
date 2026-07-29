// Part 2, Section 0: minimal setup/orientation demo — no AMs, no bugs.
// Just prints each PE's my_pe()/num_pes() so you can see the effect of
// --pes and --lamellae before anything else in the tutorial.
use lamellar::active_messaging::prelude::*;

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    println!(
        "Hello from PE {:?} of {:?}",
        world.my_pe(),
        world.num_pes()
    );
    world.barrier();
}
