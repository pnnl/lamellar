/// ------------Lamellar Example: checking RDMA gets into ordinary (unregistered) buffers -------------
/// Fetches known data from a peer's memory region into heap `Vec`s at sizes around the staging limits
/// (`LAMELLAR_RDMA_STAGING_MIN` / `_MAX`, by default 512 B and 1 MiB) and verifies every byte, through
/// `get_buffer` and `get_into_buffer`, blocking and spawned, one at a time and many at once.
///
/// Each PE prints `get_staging_check PE <n>: <checks> checks, <failures> failures` and the example exits
/// with status 1 if any check failed. Needs at least 2 PEs.
///----------------------------------------------------------------
use lamellar::memregion::prelude::*;

const REGION_LEN: usize = 4 << 20;

fn pattern(pe: usize, i: usize) -> u8 {
    (i.wrapping_mul(31).wrapping_add(pe * 17 + 7) & 0xff) as u8
}

fn check(
    got: &[u8],
    pe: usize,
    off: usize,
    size: usize,
    what: &str,
    checks: &mut usize,
    failures: &mut usize,
) {
    *checks += 1;
    if got.len() != size {
        eprintln!("[{what}] size {size} offset {off} from PE {pe}: got {} bytes", got.len());
        *failures += 1;
        return;
    }
    if let Some(bad) = (0..size).find(|&i| got[i] != pattern(pe, off + i)) {
        eprintln!(
            "[{what}] size {size} offset {off} from PE {pe}: byte {bad} is {} but should be {}",
            got[bad],
            pattern(pe, off + bad)
        );
        *failures += 1;
    }
}

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    if num_pes < 2 {
        if my_pe == 0 {
            println!("get_staging_check needs at least 2 PEs");
        }
        return;
    }

    // Every PE fills its own segment of a shared region with a pattern that depends on the PE and the index.
    let region = world.alloc_shared_mem_region::<u8>(REGION_LEN).block();
    let src = world.alloc_one_sided_mem_region::<u8>(REGION_LEN);
    unsafe {
        for (i, e) in src.as_mut_slice().iter_mut().enumerate() {
            *e = pattern(my_pe, i);
        }
        region.put_buffer(my_pe, 0, src.clone()).block();
    }
    world.barrier();

    let peer = (my_pe + 1) % num_pes;
    // Sizes straddle the default staging limits (512 B and 1 MiB) and a few power-of-two class boundaries.
    let sizes: [usize; 31] = [
        1,
        7,
        63,
        64,
        65,
        255,
        256,
        511,
        512,
        513,
        1000,
        1024,
        1025,
        2047,
        4095,
        4096,
        4097,
        8191,
        8192,
        65535,
        65536,
        65537,
        262144,
        (1 << 20) - 1,
        1 << 20,
        (1 << 20) + 1,
        (1 << 20) + 4096,
        2 << 20,
        (2 << 20) + 17,
        3 << 20,
        REGION_LEN,
    ];
    let mut checks = 0usize;
    let mut failures = 0usize;

    for &size in &sizes {
        // Different offsets so the source and destination alignments vary.
        for off in [0usize, 1, 4093, REGION_LEN - size] {
            if off + size > REGION_LEN {
                continue;
            }
            let buf = unsafe { region.get_buffer(peer, off, size).block() };
            check(&buf, peer, off, size, "get_buffer", &mut checks, &mut failures);

            let buf = unsafe { region.get_buffer(peer, off, size).spawn() }.block();
            check(&buf, peer, off, size, "get_buffer spawned", &mut checks, &mut failures);

            // The destination starts out as a recognisable value so a get that wrote nothing is caught.
            // `split_off(0)` hands the whole buffer to the get and leaves `lb` as the empty handle we unwrap afterwards
            let mut lb = LamellarBuffer::from_vec(&world, vec![0xAAu8; size]);
            unsafe { region.get_into_buffer(peer, off, lb.split_off(0)).block() };
            match lb.try_unwrap() {
                Ok(v) => check(&v, peer, off, size, "get_into_buffer", &mut checks, &mut failures),
                Err(_) => {
                    eprintln!("[get_into_buffer] size {size} offset {off}: buffer still shared");
                    failures += 1;
                }
            }

            let mut lb = LamellarBuffer::from_vec(&world, vec![0xAAu8; size]);
            unsafe { region.get_into_buffer(peer, off, lb.split_off(0)).spawn() }.block();
            match lb.try_unwrap() {
                Ok(v) => check(&v, peer, off, size, "get_into_buffer spawned", &mut checks, &mut failures),
                Err(_) => {
                    eprintln!("[get_into_buffer spawned] size {size} offset {off}: buffer still shared");
                    failures += 1;
                }
            }
        }
    }

    // Many gets in flight at once: concurrent operations must never share a staging block.
    for (size, count) in [(4096usize, 256usize), (65536, 48), (1 << 20, 4)] {
        let tasks: Vec<_> = (0..count)
            .map(|k| unsafe { region.get_buffer(peer, k * size, size).spawn() })
            .collect();
        for (k, task) in tasks.into_iter().enumerate() {
            let buf = task.block();
            check(&buf, peer, k * size, size, "get_buffer concurrent", &mut checks, &mut failures);
        }
    }

    world.barrier();
    println!("get_staging_check PE {my_pe}: {checks} checks, {failures} failures");
    if failures > 0 {
        std::process::exit(1);
    }
}
