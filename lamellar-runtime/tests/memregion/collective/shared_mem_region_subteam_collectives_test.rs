/// ------------Lamellar Test: SharedMemoryRegion collectives on a sub-team -------------------
/// Allocates a region on the team of the odd PEs (1, 3, ...) and runs the collectives on it. Inside the team the PEs
/// are ranked 0, 1, ...; `root`, `*_at_pe`, `*_from_pe` and the results are all in terms of that rank, while the
/// PEs the runtime talks to are world PEs, so this fails when a team rank is used where a world PE is expected (a
/// put to world PE 0, which is not in the team, for instance).
///
/// Which collectives run natively and which with the put/get algorithms depends on the backend: on rofi_c the
/// reduce-like ones (`sum_at_pe`, `gather_at_pe`, `sum_scatter`) are put/get, the others native.
///
/// usage: shared_mem_region_subteam_collectives_test <len>   (len must be a multiple of the team size)
/// Needs at least 4 PEs. Every member prints one line per operation; non-members print nothing.
/// -----------------------------------------------------------------------------------------------
use lamellar::memregion::prelude::*;
use lamellar::{BroadcastInput, LamellarEnv, ScatterInput, StridedArch};

#[lamellar::main]
fn main() {
    let len = std::env::args().nth(1).expect("len").parse::<usize>().unwrap();
    let world = lamellar::LamellarWorldBuilder::new().build();
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    assert!(num_pes >= 4, "needs at least 4 PEs");

    // the odd PEs: 1, 3, ...
    eprintln!("[progress] world pe {}: creating the team", my_pe);
    let team = world.create_team_from_arch(StridedArch::new(1, 2, num_pes / 2));
    let rank = team.as_ref().and_then(|t| t.team_pe_id().ok());
    eprintln!("[progress] world pe {}: team created, rank {:?}", my_pe, rank);
    let mut success = true;
    let mut out = String::new();

    if let (Some(team), Some(rank)) = (team.as_ref(), rank) {
        let size = team.num_pes();
        assert!(len % size == 0, "len must be a multiple of the team size");
        let n = len * size;
        let root = size - 1; // a rank other than 0, which is also a world PE other than the lowest in the team
        let value = |i: usize, r: usize| ((i % 5 + 2) * (r + 1)) as i32;

        eprintln!("[progress] rank {} (world pe {}): allocating the region", rank, my_pe);
        let region: SharedMemoryRegion<i32> = team.alloc_shared_mem_region::<i32>(n).block();
        eprintln!("[progress] rank {} (world pe {}): region allocated", rank, my_pe);
        unsafe {
            for (i, e) in region.as_mut_slice().iter_mut().enumerate() {
                *e = value(i, rank);
            }
        }
        team.barrier();

        // progress on stderr as it happens, to see where a hang is
        macro_rules! step {
            ($name:expr) => {
                eprintln!("[progress] rank {} (world pe {}): {}", rank, my_pe, $name);
            };
        }
        macro_rules! record {
            ($name:expr, $v:expr) => {
                out.push_str(&format!("{} rank={} (world pe {}) {:?}\n", $name, rank, my_pe, $v));
            };
        }
        let reduce_expected = |f: &dyn Fn(Vec<i32>) -> i32| -> Vec<i32> {
            (0..len).map(|j| f((0..size).map(|r| value(j, r)).collect())).collect()
        };

        unsafe {
            // native or put/get depending on the backend
            step!("sum_all");
            let sum = region.sum_all(0, len).block();
            if sum.as_slice() != reduce_expected(&|v| v.iter().sum()).as_slice() {
                eprintln!("sum_all wrong on rank {}: {:?}", rank, sum);
                success = false;
            }
            record!("sum_all", sum);
            step!("gather_all");
            let gathered = region.gather_all(0, len).block();
            let want: Vec<i32> = (0..size).flat_map(|r| (0..len).map(move |j| value(j, r))).collect();
            if gathered.as_slice() != want.as_slice() {
                eprintln!("gather_all wrong on rank {}: {:?}", rank, gathered);
                success = false;
            }
            record!("gather_all", gathered);
            step!("broadcast_from_pe");
            let bcast = if rank == root {
                BroadcastInput::root(len)
            } else {
                BroadcastInput::not_root(root)
            };
            record!("broadcast_from_pe", region.broadcast_from_pe(bcast, len).block());
            step!("scatter_from_pe");
            let scat = if rank == root {
                ScatterInput::root(0)
            } else {
                ScatterInput::not_root(root)
            };
            let scattered = region.scatter_from_pe(scat, len).block();
            let want: Vec<i32> = (0..len).map(|j| value(rank * len + j, root)).collect();
            if scattered.as_slice() != want.as_slice() {
                eprintln!("scatter_from_pe wrong on rank {}: {:?} expected {:?}", rank, scattered, want);
                success = false;
            }
            record!("scatter_from_pe", scattered);

            // the rooted and reduce-scatter ones (put/get on rofi_c)
            step!("sum_at_pe");
            let at = region.sum_at_pe(0, len, root).block();
            if rank == root && at.as_deref() != Some(reduce_expected(&|v| v.iter().sum()).as_slice()) {
                eprintln!("sum_at_pe wrong on the root: {:?}", at);
                success = false;
            }
            record!("sum_at_pe", at);
            step!("gather_at_pe");
            let at = region.gather_at_pe(0, len, root).block();
            let want: Vec<i32> = (0..size).flat_map(|r| (0..len).map(move |j| value(j, r))).collect();
            if rank == root && at.as_deref() != Some(want.as_slice()) {
                eprintln!("gather_at_pe wrong on the root: {:?}", at);
                success = false;
            }
            record!("gather_at_pe", at);
            step!("sum_scatter");
            let rs = region.sum_scatter(0, len).block();
            let all = reduce_expected(&|v| v.iter().sum());
            let chunk = len / size;
            if rs.as_slice() != &all[rank * chunk..(rank + 1) * chunk] {
                eprintln!("sum_scatter wrong on rank {}: {:?}", rank, rs);
                success = false;
            }
            record!("sum_scatter", rs);
        }
        team.barrier();
    }

    world.barrier();
    print!("{}", out);
    if !success {
        eprintln!("failed");
        std::process::exit(1);
    }
}
