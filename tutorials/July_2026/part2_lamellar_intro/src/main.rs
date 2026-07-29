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

    // exec_am_all() is LAZY: it builds the request but does not submit it to
    // the scheduler. Dropping the handle without '.spawn()'/'.block()'/await
    // means the AM never runs at all - the runtime catches this for you
    // (`Cargo.toml` enables the `runtime-warnings-panic` feature) and panics with
    // `[LAMELLAR WARNING] You are dropping a MultiAmHandle that has not been
    // 'await'ed, 'spawn()'ed or 'block()'ed`.
    //
    // BUG: forgot to wait for the request before it drops -
    // uncomment the next line to fix it.
    world.exec_am_all(HelloWorld {
        original_pe: my_pe,
    });
    // .block();

    // spawn_am_all() is EAGER: it submits to the scheduler immediately, so the
    // AM runs on every PE regardless of what you do with the returned handle.
    // Dropping it (as below) just means you never get its result/completion
    // signal directly - but world's Drop impl calls wait_all() for you, so the
    // print below is still guaranteed to land before the process exits.
    world.spawn_am_all(HelloWorld {
        original_pe: my_pe,
    });
} // world drop does barrier() + wait_all() + barrier() - it DOES wait for outstanding AMs
