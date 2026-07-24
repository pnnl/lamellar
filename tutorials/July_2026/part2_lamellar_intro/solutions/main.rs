use lamellar::active_messaging::prelude::*;

#[AmData(Debug, Clone)]
struct HelloWorld {
    original_pe: usize,
}

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

    let request = world.spawn_am_all(HelloWorld {
        original_pe: my_pe,
    });
    request.block(); // wait for all PEs to finish executing the AM before exiting
}
