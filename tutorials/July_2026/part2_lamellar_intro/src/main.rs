// Part 2, Section 2: Active Messages basics.
// Live-code this file with the class; it has an intentional bug (see below).
use lamellar::active_messaging::prelude::*;

// The data we ship along with the active message.
#[AmData(Debug, Clone)]
struct HelloWorld {
    original_pe: usize,
}

// #[am] registers this LamellarAM impl so it can be executed remotely.
#[am]
impl LamellarAM for HelloWorld {
    async fn exec(self) {
        println!(
            "Hello World on PE {:?} of {:?}, received from PE {:?}",
            lamellar::current_pe,
            lamellar::num_pes,
            self.original_pe,
        );
    }
}

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    world.barrier();

    // Send a Hello World Active Message to all PEs.
    let request = world.spawn_am_all(HelloWorld {
        original_pe: my_pe,
    });

    // BUG: forgot to wait for the request before the world drops -
    // comment out the next line and see prints arrive incomplete/out of order.
    // request.block();
} // world drop performs an implicit barrier, but does NOT wait for outstanding AMs
