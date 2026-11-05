/// ------------Lamellar Test: array collectives on a sub-team -------------------
/// Creates an array on the team of the odd PEs (1, 3, ...) and runs the collectives on it. Inside the team the PEs are
/// numbered 0, 1, ... (their team PEs); `root` and the results are in terms of those, while the PEs the runtime talks to are world PEs,
/// so this fails when a team PE is used where a world PE is expected (the way the region collectives did).
///
/// The array has `len` elements on each team PE (Block distribution); element `g` of the whole array holds `g % 7 + 1`,
/// so the elements differ and a wrong chunk, a wrong order or a wrong PE shows in the result. Which collectives run natively
/// and which with the put/get algorithms depends on the backend (on rofi_c the reduce-like ones are put/get).
///
/// usage: array_subteam_collectives_test <UnsafeArray|AtomicArray|GlobalLockArray> <len>   (len must be a multiple of the team size)
/// Needs at least 4 PEs. Every member prints one line per operation; non-members print nothing.
/// -----------------------------------------------------------------------------------------------
use lamellar::array::prelude::*;
use lamellar::{LamellarEnv, ScatterInput, StridedArch};

macro_rules! array_or_lock {
    (GlobalLockArray, $array: ident, $lock:ident) => {
        $lock
    };
    ($arraytype:ident,$array:ident, $lock:ident) => {
        $array
    };
}

macro_rules! lock_if_needed {
    (GlobalLockArray, $array: ident) => {
        $array.collective_write_local_data().block()
    };
    ($arraytype:ident,$array:ident) => {
        0usize
    };
}

// sets this PE's `len` local elements to `value(first_global_index + j)`
macro_rules! init_local {
    (UnsafeArray, $array:ident, $first:expr, $len:expr, $value:expr) => {{
        let data = unsafe { $array.local_as_mut_slice() };
        for j in 0..$len {
            data[j] = $value($first + j);
        }
    }};
    (AtomicArray, $array:ident, $first:expr, $len:expr, $value:expr) => {{
        let data = $array.local_data();
        for j in 0..$len {
            data.at(j).store($value($first + j));
        }
    }};
    (GlobalLockArray, $array:ident, $first:expr, $len:expr, $value:expr) => {{
        let mut data = $array.write_local_data().block();
        for (j, e) in data.iter_mut().enumerate() {
            *e = $value($first + j);
        }
    }};
}

macro_rules! subteam_test {
    ($array:ident, $len:expr) => {{
        let world = lamellar::LamellarWorldBuilder::new().build();
        let num_pes = world.num_pes();
        let my_pe = world.my_pe();
        assert!(num_pes >= 4, "needs at least 4 PEs");
        let len: usize = $len;
        let mut success = true;
        let mut out = String::new();

        // the odd PEs: 1, 3, ...
        eprintln!("[progress] world pe {}: creating the team", my_pe);
        let team = world.create_team_from_arch(StridedArch::new(1, 2, num_pes / 2));
        let team_pe = team.as_ref().and_then(|t| t.team_pe_id().ok());
        eprintln!("[progress] world pe {}: team created, team_pe {:?}", my_pe, team_pe);

        if let (Some(team), Some(team_pe)) = (team.as_ref(), team_pe) {
            let size = team.num_pes();
            assert!(len % size == 0, "len must be a multiple of the team size");
            let chunk = len / size;
            let root = size - 1; // a team_pe other than 0, which is also a world PE other than the lowest in the team
            let value = |g: usize| (g % 7 + 1) as i32;
            // window `index..index+len` of team PE r holds global elements r * len + index + j
            let at = |r: usize, j: usize| value(r * len + j);

            let array: $array<i32> = $array::<i32>::new(team.clone(), len * size, Distribution::Block)
                .block()
                .into();
            eprintln!(
                "[progress] team_pe {} (world pe {}): array len {} local {} num_pes {} team size {}",
                team_pe,
                my_pe,
                array.len(),
                array.num_elems_local(),
                array.num_pes(),
                size
            );
            init_local!($array, array, team_pe * len, len, value);
            array.wait_all();
            array.barrier();
            eprintln!("[progress] team_pe {} (world pe {}): array filled", team_pe, my_pe);

            macro_rules! step {
                ($name:expr) => {
                    eprintln!("[progress] team_pe {} (world pe {}): {}", team_pe, my_pe, $name);
                };
            }
            macro_rules! record {
                ($name:expr, $v:expr) => {
                    out.push_str(&format!("{} team_pe={} (world pe {}) {:?}\n", $name, team_pe, my_pe, $v));
                };
            }
            macro_rules! check {
                ($name:expr, $got:expr, $want:expr) => {
                    if $got != $want {
                        eprintln!("{} wrong on team_pe {}: got {:?} expected {:?}", $name, team_pe, $got, $want);
                        success = false;
                    }
                };
            }

            let _lock = lock_if_needed!($array, array);

            unsafe {
                // reductions over the team's windows
                let sum_want: Vec<i32> = (0..len).map(|j| (0..size).map(|r| at(r, j)).sum()).collect();
                step!("sum_all");
                let sum = array_or_lock!($array, array, _lock).sum_all(0, len).block();
                check!("sum_all", sum.as_slice(), sum_want.as_slice());
                record!("sum_all", sum.as_slice());

                step!("gather_all");
                let gathered = array_or_lock!($array, array, _lock).gather_all(0, len).block();
                let gather_want: Vec<i32> = (0..size).flat_map(|r| (0..len).map(move |j| at(r, j))).collect();
                check!("gather_all", gathered.as_slice(), gather_want.as_slice());
                record!("gather_all", gathered.as_slice());

                // scatter: team PE r receives chunk r of the root's window
                step!("scatter_from_pe");
                let input = if team_pe == root { ScatterInput::root(0) } else { ScatterInput::not_root(root) };
                let scattered = array_or_lock!($array, array, _lock).scatter_from_pe(input, chunk).block();
                let scatter_want: Vec<i32> = (0..chunk).map(|j| at(root, team_pe * chunk + j)).collect();
                check!("scatter_from_pe", scattered.as_slice(), scatter_want.as_slice());
                record!("scatter_from_pe", scattered.as_slice());

                // the rooted and reduce-scatter ones (put/get on rofi_c)
                step!("sum_at_pe");
                let at_root = array_or_lock!($array, array, _lock).sum_at_pe(0, len, root).block();
                if team_pe == root {
                    check!("sum_at_pe", at_root.as_ref().map(|b| b.as_slice()), Some(sum_want.as_slice()));
                } else {
                    check!("sum_at_pe (non-root)", at_root.is_none(), true);
                }
                record!("sum_at_pe", at_root.as_ref().map(|b| b.as_slice().to_vec()));

                step!("sum_scatter");
                let rs = array_or_lock!($array, array, _lock).sum_scatter(0, len).block();
                check!("sum_scatter", rs.as_slice(), &sum_want[team_pe * chunk..(team_pe + 1) * chunk]);
                record!("sum_scatter", rs.as_slice());
            }
            step!("done");
            drop(_lock);
            array.barrier();
        }

        world.barrier();
        print!("{}", out);
        if !success {
            eprintln!("failed");
            std::process::exit(1);
        }
    }};
}

#[lamellar::main]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let array = args[1].clone();
    let len = args[2].parse::<usize>().unwrap();
    match array.as_str() {
        "UnsafeArray" => subteam_test!(UnsafeArray, len),
        "AtomicArray" => subteam_test!(AtomicArray, len),
        "GlobalLockArray" => subteam_test!(GlobalLockArray, len),
        _ => eprintln!("unsupported array type"),
    }
}
