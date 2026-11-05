/// ------------Lamellar Test: array put/get on a sub-team -------------------
/// Creates an array on the team of the odd PEs (1, 3, ...) and runs the raw RDMA calls on it. Inside the team the PEs are
/// numbered 0, 1, ... (their team PEs); the array API (`put_pe`, `blocking_get_pe`, the element index to PE mapping) is in terms of those, while the
/// allocation the data lives in is addressed by world PE, so this fails when a team PE reaches the transport untranslated.
///
/// Every team PE holds PER (= 4) elements; the value at (team_pe, offset) starts as `team_pe * 100 + offset`.
///
/// usage: array_subteam_rdma_test <UnsafeArray|AtomicArray|GlobalLockArray|LocalLockArray|ReadOnlyArray> <Block|Cyclic>
/// Needs at least 4 PEs. Members print one line when all checks pass. ReadOnlyArray only runs the reads.
/// -----------------------------------------------------------------------------------------------
use lamellar::array::prelude::*;
use lamellar::{LamellarEnv, StridedArch};

const PER: usize = 4;

// The raw calls under one name per array type (their inherent methods have the same names, so these are `r_`-prefixed).
trait Raw: Sized {
    const WRITABLE: bool;
    // creates the array on the team and sets this PE's elements to `init(team_pe, offset)`
    fn make(team: &std::sync::Arc<LamellarTeam>, len: usize, dist: Distribution, team_pe: usize, init: &dyn Fn(usize, usize) -> i64) -> Self;
    fn local(&self) -> Vec<i64>;
    fn sync(&self);
    fn r_get(&self, g: usize) -> i64;
    fn r_get_buffer(&self, g: usize, n: usize) -> Vec<i64>;
    fn r_get_pe(&self, pe: usize, off: usize) -> i64;
    fn r_put_pe(&self, pe: usize, off: usize, v: i64);
    fn r_put_pe_buffer(&self, pe: usize, off: usize, v: Vec<i64>);
    fn r_put(&self, g: usize, v: i64);
    fn r_put_buffer(&self, g: usize, v: Vec<i64>);
    fn r_put_all(&self, off: usize, v: i64);
}

macro_rules! raw_common {
    () => {
        fn sync(&self) {
            self.barrier();
        }
        fn r_get(&self, g: usize) -> i64 {
            unsafe { self.blocking_get(g) }
        }
        fn r_get_buffer(&self, g: usize, n: usize) -> Vec<i64> {
            unsafe { self.blocking_get_buffer(g, n) }
        }
        fn r_get_pe(&self, pe: usize, off: usize) -> i64 {
            unsafe { self.blocking_get_pe(pe, off) }
        }
    };
}

macro_rules! raw_writes {
    () => {
        const WRITABLE: bool = true;
        fn r_put_pe(&self, pe: usize, off: usize, v: i64) {
            unsafe { self.put_pe(pe, off, v).block() }
        }
        fn r_put_pe_buffer(&self, pe: usize, off: usize, v: Vec<i64>) {
            unsafe { self.put_pe_buffer(pe, off, v).block() }
        }
        fn r_put(&self, g: usize, v: i64) {
            unsafe { self.put(g, v).block() }
        }
        fn r_put_buffer(&self, g: usize, v: Vec<i64>) {
            unsafe { self.put_buffer(g, v).block() }
        }
        fn r_put_all(&self, off: usize, v: i64) {
            unsafe { self.put_all(off, v).block() }
        }
    };
}

impl Raw for UnsafeArray<i64> {
    raw_common!();
    raw_writes!();
    fn make(team: &std::sync::Arc<LamellarTeam>, len: usize, dist: Distribution, team_pe: usize, init: &dyn Fn(usize, usize) -> i64) -> Self {
        let array = UnsafeArray::<i64>::new(team.clone(), len, dist).block();
        unsafe {
            for (off, e) in array.local_as_mut_slice().iter_mut().enumerate() {
                *e = init(team_pe, off);
            }
        }
        array.barrier();
        array
    }
    fn local(&self) -> Vec<i64> {
        unsafe { self.local_as_slice().to_vec() }
    }
}

impl Raw for AtomicArray<i64> {
    raw_common!();
    raw_writes!();
    fn make(team: &std::sync::Arc<LamellarTeam>, len: usize, dist: Distribution, team_pe: usize, init: &dyn Fn(usize, usize) -> i64) -> Self {
        let array = AtomicArray::<i64>::new(team.clone(), len, dist).block();
        let data = array.local_data();
        for off in 0..PER {
            data.at(off).store(init(team_pe, off));
        }
        array.barrier();
        array
    }
    fn local(&self) -> Vec<i64> {
        let data = self.local_data();
        (0..PER).map(|off| data.at(off).load()).collect()
    }
}

macro_rules! raw_lock_array {
    ($array:ident) => {
        impl Raw for $array<i64> {
            raw_common!();
            raw_writes!();
            fn make(team: &std::sync::Arc<LamellarTeam>, len: usize, dist: Distribution, team_pe: usize, init: &dyn Fn(usize, usize) -> i64) -> Self {
                let array = $array::<i64>::new(team.clone(), len, dist).block();
                {
                    let mut data = array.write_local_data().block();
                    for (off, e) in data.iter_mut().enumerate() {
                        *e = init(team_pe, off);
                    }
                }
                array.barrier();
                array
            }
            fn local(&self) -> Vec<i64> {
                self.read_local_data().block().iter().cloned().collect()
            }
        }
    };
}
raw_lock_array!(GlobalLockArray);
raw_lock_array!(LocalLockArray);

impl Raw for ReadOnlyArray<i64> {
    raw_common!();
    const WRITABLE: bool = false;
    fn r_put_pe(&self, _: usize, _: usize, _: i64) {
        unreachable!()
    }
    fn r_put_pe_buffer(&self, _: usize, _: usize, _: Vec<i64>) {
        unreachable!()
    }
    fn r_put(&self, _: usize, _: i64) {
        unreachable!()
    }
    fn r_put_buffer(&self, _: usize, _: Vec<i64>) {
        unreachable!()
    }
    fn r_put_all(&self, _: usize, _: i64) {
        unreachable!()
    }
    fn make(team: &std::sync::Arc<LamellarTeam>, len: usize, dist: Distribution, team_pe: usize, init: &dyn Fn(usize, usize) -> i64) -> Self {
        let array = UnsafeArray::<i64>::new(team.clone(), len, dist).block();
        unsafe {
            for (off, e) in array.local_as_mut_slice().iter_mut().enumerate() {
                *e = init(team_pe, off);
            }
        }
        array.barrier();
        array.into_read_only().block()
    }
    fn local(&self) -> Vec<i64> {
        self.local_data().to_vec()
    }
}

fn run<A: Raw>(team: &std::sync::Arc<LamellarTeam>, cyclic: bool, my_pe: usize, team_pe: usize) -> bool {
    let dist = if cyclic { Distribution::Cyclic } else { Distribution::Block };
    let size = team.num_pes();
    let len = PER * size;
    let next = (team_pe + 1) % size;
    let prev = (team_pe + size - 1) % size;
    let mut success = true;
    // (team PE, offset in that PE's local data) of global element g
    let loc = |g: usize| if cyclic { (g % size, g / size) } else { (g / PER, g % PER) };
    // global element held at (team_pe, offset)
    let glob = |r: usize, off: usize| if cyclic { r + off * size } else { r * PER + off };
    let init = |r: usize, off: usize| (r * 100 + off) as i64;

    macro_rules! check {
        ($name:expr, $got:expr, $want:expr) => {
            let (got, want) = ($got, $want);
            if got != want {
                eprintln!("{} wrong on team_pe {} (world pe {}): got {:?} expected {:?}", $name, team_pe, my_pe, got, want);
                success = false;
            }
        };
    }

    let array = A::make(team, len, dist, team_pe, &init);
    eprintln!("[progress] team_pe {} (world pe {}): array made", team_pe, my_pe);

    // reads by global index, by PE and offset, and the whole array in one call
    for g in 0..len {
        let (pe, off) = loc(g);
        check!(format!("blocking_get({})", g), array.r_get(g), init(pe, off));
    }
    let want: Vec<i64> = (0..len).map(|g| { let (pe, off) = loc(g); init(pe, off) }).collect();
    check!("blocking_get_buffer", array.r_get_buffer(0, len), want);
    for pe in 0..size {
        for off in 0..PER {
            check!(format!("blocking_get_pe({}, {})", pe, off), array.r_get_pe(pe, off), init(pe, off));
        }
    }
    eprintln!("[progress] team_pe {} (world pe {}): reads done", team_pe, my_pe);
    array.sync();

    if A::WRITABLE {
        // writes to the next team PE by PE and offset; each PE has exactly one writer
        let w = |r: usize, off: usize| (1000 + r * 10 + off) as i64;
        array.r_put_pe(next, 0, w(team_pe, 0));
        array.r_put_pe(next, 1, w(team_pe, 1));
        array.r_put_pe_buffer(next, 2, vec![w(team_pe, 2), w(team_pe, 3)]);
        array.sync();
        let local = array.local();
        for off in 0..PER {
            check!(format!("put_pe at offset {}", off), local[off], w(prev, off));
        }
        eprintln!("[progress] team_pe {} (world pe {}): put_pe done", team_pe, my_pe);
        array.sync();

        // writes to the next team PE by global index
        let x = |r: usize, off: usize| (5000 + r * 10 + off) as i64;
        for off in 0..PER {
            array.r_put(glob(next, off), x(team_pe, off));
        }
        array.sync();
        let local = array.local();
        for off in 0..PER {
            check!(format!("put at offset {}", off), local[off], x(prev, off));
        }
        eprintln!("[progress] team_pe {} (world pe {}): put done", team_pe, my_pe);
        array.sync();

        // team PE 0 writes the whole array with one call
        let z = |g: usize| (9000 + g) as i64;
        if team_pe == 0 {
            array.r_put_buffer(0, (0..len).map(z).collect());
        }
        array.sync();
        let local = array.local();
        for off in 0..PER {
            check!(format!("put_buffer at offset {}", off), local[off], z(glob(team_pe, off)));
        }
        eprintln!("[progress] team_pe {} (world pe {}): put_buffer done", team_pe, my_pe);
        array.sync();

        // team PE 0 writes the same value at one offset on every PE of the team
        if team_pe == 0 {
            array.r_put_all(PER - 1, 7777);
        }
        array.sync();
        check!("put_all", array.local()[PER - 1], 7777i64);
        eprintln!("[progress] team_pe {} (world pe {}): put_all done", team_pe, my_pe);
        array.sync();
    }
    success
}

#[lamellar::main]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let array_type = args[1].clone();
    let cyclic = match args[2].as_str() {
        "Block" => false,
        "Cyclic" => true,
        _ => panic!("usage: array_subteam_rdma_test <array type> <Block|Cyclic>"),
    };

    let world = lamellar::LamellarWorldBuilder::new().build();
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    assert!(num_pes >= 4, "needs at least 4 PEs");

    // the odd PEs: 1, 3, ...
    let team = world.create_team_from_arch(StridedArch::new(1, 2, num_pes / 2));
    let team_pe = team.as_ref().and_then(|t| t.team_pe_id().ok());
    let mut success = true;

    if let (Some(team), Some(team_pe)) = (team.as_ref(), team_pe) {
        success = match array_type.as_str() {
            "UnsafeArray" => run::<UnsafeArray<i64>>(team, cyclic, my_pe, team_pe),
            "AtomicArray" => run::<AtomicArray<i64>>(team, cyclic, my_pe, team_pe),
            "GlobalLockArray" => run::<GlobalLockArray<i64>>(team, cyclic, my_pe, team_pe),
            "LocalLockArray" => run::<LocalLockArray<i64>>(team, cyclic, my_pe, team_pe),
            "ReadOnlyArray" => run::<ReadOnlyArray<i64>>(team, cyclic, my_pe, team_pe),
            _ => panic!("unsupported array type"),
        };
        if success {
            println!("team_pe {} (world pe {}) ok", team_pe, my_pe);
        }
    }

    world.barrier();
    if !success {
        eprintln!("failed");
        std::process::exit(1);
    }
}
