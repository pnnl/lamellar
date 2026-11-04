/// ------------Lamellar Test: array put/get on a sub-team -------------------
/// Creates an UnsafeArray on the team of the odd PEs (1, 3, ...) and runs the raw RDMA calls on it. Inside the team the PEs are
/// ranked 0, 1, ...; the array API (`put_pe`, `blocking_get_pe`, the element index to PE mapping) is in terms of that rank, while the
/// allocation the data lives in is addressed by world PE, so this fails when a team rank reaches the transport untranslated.
///
/// Every team PE holds PER (= 4) elements; the value at (rank, offset) starts as `rank * 100 + offset`.
///
/// usage: array_subteam_rdma_test <Block|Cyclic>      Needs at least 4 PEs. Members print one line when all checks pass.
/// -----------------------------------------------------------------------------------------------
use lamellar::array::prelude::*;
use lamellar::{LamellarEnv, StridedArch};

const PER: usize = 4;

#[lamellar::main]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cyclic = match args[1].as_str() {
        "Block" => false,
        "Cyclic" => true,
        _ => panic!("usage: array_subteam_rdma_test <Block|Cyclic>"),
    };
    let dist = if cyclic { Distribution::Cyclic } else { Distribution::Block };

    let world = lamellar::LamellarWorldBuilder::new().build();
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    assert!(num_pes >= 4, "needs at least 4 PEs");

    // the odd PEs: 1, 3, ...
    let team = world.create_team_from_arch(StridedArch::new(1, 2, num_pes / 2));
    let rank = team.as_ref().and_then(|t| t.team_pe_id().ok());
    let mut success = true;

    if let (Some(team), Some(rank)) = (team.as_ref(), rank) {
        let size = team.num_pes();
        let len = PER * size;
        let next = (rank + 1) % size;
        let prev = (rank + size - 1) % size;
        // (team rank, offset in that rank's local data) of global element g
        let loc = |g: usize| if cyclic { (g % size, g / size) } else { (g / PER, g % PER) };
        // global element held at (rank, offset)
        let glob = |r: usize, off: usize| if cyclic { r + off * size } else { r * PER + off };
        let init = |r: usize, off: usize| (r * 100 + off) as i64;

        macro_rules! check {
            ($name:expr, $got:expr, $want:expr) => {
                let (got, want) = ($got, $want);
                if got != want {
                    eprintln!("{} wrong on rank {} (world pe {}): got {:?} expected {:?}", $name, rank, my_pe, got, want);
                    success = false;
                }
            };
        }

        let array = UnsafeArray::<i64>::new(team.clone(), len, dist).block();
        unsafe {
            for (off, e) in array.local_as_mut_slice().iter_mut().enumerate() {
                *e = init(rank, off);
            }
        }
        array.barrier();
        eprintln!("[progress] rank {} (world pe {}): array filled", rank, my_pe);

        // reads by global index, by PE and offset, and the whole array in one call
        unsafe {
            for g in 0..len {
                let (pe, off) = loc(g);
                check!(format!("blocking_get({})", g), array.blocking_get(g), init(pe, off));
            }
            let all = array.blocking_get_buffer(0, len);
            let want: Vec<i64> = (0..len).map(|g| { let (pe, off) = loc(g); init(pe, off) }).collect();
            check!("blocking_get_buffer", all, want);
            for pe in 0..size {
                for off in 0..PER {
                    check!(format!("blocking_get_pe({}, {})", pe, off), array.blocking_get_pe(pe, off), init(pe, off));
                }
            }
        }
        eprintln!("[progress] rank {} (world pe {}): reads done", rank, my_pe);
        array.barrier();

        // writes to the next rank by PE and offset; each rank has exactly one writer
        let w = |r: usize, off: usize| (1000 + r * 10 + off) as i64;
        unsafe {
            array.put_pe(next, 0, w(rank, 0)).block();
            array.put_pe(next, 1, w(rank, 1)).block();
            array.put_pe_buffer(next, 2, vec![w(rank, 2), w(rank, 3)]).block();
        }
        array.barrier();
        for off in 0..PER {
            check!(format!("put_pe at offset {}", off), unsafe { array.local_as_slice()[off] }, w(prev, off));
        }
        eprintln!("[progress] rank {} (world pe {}): put_pe done", rank, my_pe);
        array.barrier();

        // writes to the next rank by global index
        let x = |r: usize, off: usize| (5000 + r * 10 + off) as i64;
        unsafe {
            for off in 0..PER {
                array.put(glob(next, off), x(rank, off)).block();
            }
        }
        array.barrier();
        for off in 0..PER {
            check!(format!("put at offset {}", off), unsafe { array.local_as_slice()[off] }, x(prev, off));
        }
        eprintln!("[progress] rank {} (world pe {}): put done", rank, my_pe);
        array.barrier();

        // one rank writes the whole array with one call
        let z = |g: usize| (9000 + g) as i64;
        if rank == 0 {
            unsafe { array.put_buffer(0, (0..len).map(z).collect::<Vec<i64>>()).block() };
        }
        array.barrier();
        for off in 0..PER {
            check!(format!("put_buffer at offset {}", off), unsafe { array.local_as_slice()[off] }, z(glob(rank, off)));
        }
        eprintln!("[progress] rank {} (world pe {}): put_buffer done", rank, my_pe);
        array.barrier();

        // one rank writes the same value at one offset on every PE of the team
        if rank == 0 {
            unsafe { array.put_all(PER - 1, 7777).block() };
        }
        array.barrier();
        check!("put_all", unsafe { array.local_as_slice()[PER - 1] }, 7777i64);
        eprintln!("[progress] rank {} (world pe {}): put_all done", rank, my_pe);
        array.barrier();
        if success {
            println!("rank {} (world pe {}) ok", rank, my_pe);
        }
    }

    world.barrier();
    if !success {
        eprintln!("failed");
        std::process::exit(1);
    }
}
